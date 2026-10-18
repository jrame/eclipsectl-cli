use crate::{
    ipc::{self, Action, Request, Response},
    mcp::Pool,
};
use anyhow::{Context, Result, bail};
use fs2::FileExt;
use serde_json::json;
#[cfg(windows)]
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Notify,
    task::JoinSet,
};

pub trait Stream: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> Stream for T {}
type Connection = Box<dyn Stream>;

pub fn runtime_dir() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("ECLIPSECTL_RUNTIME_DIR") {
        return Ok(path.into());
    }
    #[cfg(unix)]
    if let Some(path) = std::env::var_os("XDG_RUNTIME_DIR") {
        return Ok(PathBuf::from(path).join("eclipsectl-cli"));
    }
    Ok(dirs::cache_dir()
        .context("Cannot locate runtime directory")?
        .join("eclipsectl-cli/run"))
}

pub fn prepare_runtime(path: &Path) -> Result<()> {
    if fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink()) {
        bail!("Runtime directory must not be a symlink");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(path)?;
        let meta = fs::metadata(path)?;
        // SAFETY: geteuid has no preconditions.
        if meta.uid() != unsafe { libc::geteuid() } {
            bail!("Runtime directory is owned by another user");
        }
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    #[cfg(windows)]
    fs::create_dir_all(path)?;
    Ok(())
}

fn lock_file(path: &Path) -> Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }
    Ok(options.open(path)?)
}

#[cfg(windows)]
fn endpoint(path: &Path) -> Result<String> {
    let sid = windows::current_user_sid()?;
    let hash = Sha256::digest(format!("{sid}:{}", path.display()).as_bytes());
    Ok(format!(r"\\.\pipe\eclipsectl-{:x}", hash))
}

#[cfg(unix)]
fn endpoint(path: &Path) -> Result<PathBuf> {
    Ok(path.join("daemon.sock"))
}

async fn connect(path: &Path) -> Result<Connection> {
    #[cfg(unix)]
    {
        Ok(Box::new(
            tokio::net::UnixStream::connect(endpoint(path)?).await?,
        ))
    }
    #[cfg(windows)]
    {
        let name = endpoint(path)?;
        for _ in 0..100 {
            match tokio::net::windows::named_pipe::ClientOptions::new().open(&name) {
                Ok(client) => return Ok(Box::new(client)),
                Err(e) if e.raw_os_error() == Some(231) => {
                    tokio::time::sleep(Duration::from_millis(10)).await
                }
                Err(e) => return Err(e.into()),
            }
        }
        bail!("Daemon pipe is busy")
    }
}

pub async fn exchange_at(path: &Path, action: Action, timeout: Duration) -> Result<Response> {
    exchange_request_at(
        path,
        Request {
            version: ipc::VERSION,
            action,
        },
        timeout,
    )
    .await
}

pub async fn exchange_request_at(
    path: &Path,
    request: Request,
    timeout: Duration,
) -> Result<Response> {
    tokio::time::timeout(timeout, async {
        let mut stream = connect(path).await?;
        ipc::write_frame(&mut stream, &request).await?;
        ipc::read_frame(&mut stream).await
    })
    .await
    .context("Daemon communication timed out; an operation may still be running")?
}

fn status_is_ready(response: &Response) -> Result<()> {
    if !response.ok {
        bail!("Daemon rejected handshake; run eclipsectl daemon restart");
    }
    let value = response
        .result
        .as_ref()
        .context("Invalid daemon handshake")?;
    if value["protocol"] != ipc::VERSION || value["version"] != env!("CARGO_PKG_VERSION") {
        bail!("Daemon version differs from this CLI; run eclipsectl daemon restart");
    }
    Ok(())
}

pub async fn ensure_at(path: &Path) -> Result<()> {
    prepare_runtime(path)?;
    if let Ok(status) = exchange_at(path, Action::Status, Duration::from_secs(2)).await {
        return status_is_ready(&status);
    }
    let lock = lock_file(&path.join("startup.lock"))?;
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        match lock.try_lock_exclusive() {
            Ok(()) => break,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if let Ok(status) = exchange_at(path, Action::Status, Duration::from_secs(1)).await
                {
                    return status_is_ready(&status);
                }
                if Instant::now() >= deadline {
                    bail!("Timed out waiting for daemon startup lock");
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(e) => return Err(e.into()),
        }
    }
    if let Ok(status) = exchange_at(path, Action::Status, Duration::from_secs(2)).await {
        return status_is_ready(&status);
    }
    // No credentials are passed in the process command line or daemon metadata.
    #[cfg(windows)]
    let mut child = windows::spawn_daemon(path)?;
    #[cfg(unix)]
    let mut child = {
        use std::os::unix::process::CommandExt;
        use std::process::Stdio;
        let mut command = std::process::Command::new(std::env::current_exe()?);
        command
            .args(["daemon", "serve"])
            .env("ECLIPSECTL_RUNTIME_DIR", path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        // SAFETY: setsid is async-signal-safe and touches no Rust runtime state.
        unsafe {
            command.pre_exec(|| {
                if libc::setsid() == -1 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(())
                }
            });
        }
        command.spawn().context("Cannot start eclipsectl daemon")?
    };
    loop {
        if let Ok(status) = exchange_at(path, Action::Status, Duration::from_secs(1)).await {
            status_is_ready(&status)?;
            // Reap an exiting child without keeping the CLI alive for the daemon lifetime.
            let _ = child.try_wait();
            return Ok(());
        }
        if let Some(code) = child.try_wait()? {
            bail!(
                "Daemon exited with {code}; run eclipsectl daemon serve --verbose for diagnostics"
            );
        }
        if Instant::now() >= deadline {
            bail!(
                "Daemon startup timed out; run eclipsectl daemon serve --verbose for diagnostics"
            );
        }
        tokio::time::sleep(Duration::from_millis(40)).await;
    }
}

pub async fn request(action: Action, timeout: Duration) -> Result<Response> {
    let path = runtime_dir()?;
    ensure_at(&path).await?;
    // A failed exchange is never retried: it may already have executed a write.
    exchange_at(&path, action, timeout).await
}

pub async fn status() -> Result<Response> {
    let path = runtime_dir()?;
    match exchange_at(&path, Action::Status, Duration::from_secs(2)).await {
        Ok(status) => Ok(status),
        Err(_) => Ok(Response::success(json!({"running": false}))),
    }
}

pub async fn stop() -> Result<Response> {
    let path = runtime_dir()?;
    if let Ok(reply) = exchange_at(&path, Action::Shutdown, Duration::from_secs(3)).await {
        return Ok(reply);
    }
    Ok(Response::success(json!({"running":false})))
}

pub async fn restart() -> Result<Response> {
    let _ = stop().await?;
    let path = runtime_dir()?;
    let deadline = Instant::now() + Duration::from_secs(15);
    // The daemon holds this lock until all active operations have drained.
    prepare_runtime(&path)?;
    loop {
        let lock = lock_file(&path.join("daemon.lock"))?;
        if lock.try_lock_exclusive().is_ok() {
            drop(lock);
            break;
        }
        if Instant::now() >= deadline {
            bail!("Daemon is draining active operations; restart again after they finish");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    ensure_at(&path).await?;
    status().await
}

struct Listener {
    #[cfg(unix)]
    inner: tokio::net::UnixListener,
    #[cfg(windows)]
    inner: tokio::net::windows::named_pipe::NamedPipeServer,
    #[cfg(windows)]
    name: String,
}

impl Listener {
    fn bind(path: &Path) -> Result<Self> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let socket = endpoint(path)?;
            if socket.exists() {
                fs::remove_file(&socket)?;
            }
            let inner = tokio::net::UnixListener::bind(&socket)?;
            fs::set_permissions(&socket, fs::Permissions::from_mode(0o600))?;
            Ok(Self { inner })
        }
        #[cfg(windows)]
        {
            let name = endpoint(path)?;
            Ok(Self {
                inner: windows::create_pipe(&name, true)?,
                name,
            })
        }
    }

    async fn accept(&mut self) -> Result<Connection> {
        #[cfg(unix)]
        {
            Ok(Box::new(self.inner.accept().await?.0))
        }
        #[cfg(windows)]
        {
            self.inner.connect().await?;
            let next = windows::create_pipe(&self.name, false)?;
            Ok(Box::new(std::mem::replace(&mut self.inner, next)))
        }
    }
}

pub async fn serve_at(path: &Path, idle_timeout: Duration) -> Result<()> {
    prepare_runtime(path)?;
    let lock = lock_file(&path.join("daemon.lock"))?;
    lock.try_lock_exclusive()
        .context("A daemon is already running")?;
    let mut listener = Listener::bind(path)?;
    let pool = Arc::new(Pool::default());
    let stopping = Arc::new(AtomicBool::new(false));
    let notify = Arc::new(Notify::new());
    let mut tasks = JoinSet::new();
    let mut last_activity = Instant::now();
    let mut tick = tokio::time::interval(Duration::from_millis(100));
    tracing::info!("daemon ready");
    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let stream = accepted?;
                let pool = pool.clone(); let stopping = stopping.clone(); let notify = notify.clone();
                last_activity = Instant::now();
                tasks.spawn(async move {
                    if handle(stream, pool, stopping, notify).await.is_err() {
                        // Request data and MCP errors may contain secrets; never log them here.
                        tracing::debug!("IPC client disconnected");
                    }
                });
            },
            Some(_) = tasks.join_next(), if !tasks.is_empty() => { last_activity = Instant::now(); },
            _ = tick.tick() => { if tasks.is_empty() && last_activity.elapsed() >= idle_timeout { break; } },
            _ = notify.notified() => { if stopping.load(Ordering::Acquire) { break; } },
            _ = shutdown_signal() => { break; },
        }
    }
    drop(listener);
    while tasks.join_next().await.is_some() {}
    pool.close().await;
    #[cfg(unix)]
    {
        let _ = fs::remove_file(endpoint(path)?);
    }
    drop(lock);
    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        if let Ok(mut signal) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            tokio::select! { _ = signal.recv() => {}, _ = tokio::signal::ctrl_c() => {} }
        } else {
            let _ = tokio::signal::ctrl_c().await;
        }
    }
    #[cfg(windows)]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

async fn handle(
    mut stream: Connection,
    pool: Arc<Pool>,
    stopping: Arc<AtomicBool>,
    notify: Arc<Notify>,
) -> Result<()> {
    let request: Request =
        tokio::time::timeout(Duration::from_secs(10), ipc::read_frame(&mut stream)).await??;
    let reply = if request.version != ipc::VERSION {
        Response::error(
            "protocol_mismatch",
            "CLI and daemon protocol versions differ; restart the daemon",
        )
    } else {
        match request.action {
            Action::Status => Response::success(
                json!({"running":true, "pid":std::process::id(), "protocol":ipc::VERSION, "version":env!("CARGO_PKG_VERSION")}),
            ),
            Action::Shutdown => {
                stopping.store(true, Ordering::Release);
                Response::success(json!({"stopping":true}))
            }
            Action::Call { config, call } => pool.call(&config, &call).await,
            Action::Doctor { config } => pool.doctor(&config).await,
        }
    };
    let result = tokio::time::timeout(
        Duration::from_secs(30),
        ipc::write_frame(&mut stream, &reply),
    )
    .await?;
    if stopping.load(Ordering::Acquire) {
        notify.notify_one();
    }
    result
}

#[cfg(windows)]
mod windows {
    use super::*;
    use windows_sys::Win32::{
        Foundation::{CloseHandle, LocalFree},
        Security::{
            Authorization::{
                ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
            },
            GetTokenInformation, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER, TokenUser,
        },
        System::Threading::{
            CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, CREATE_UNICODE_ENVIRONMENT, CreateProcessW,
            GetCurrentProcess, GetExitCodeProcess, OpenProcessToken, PROCESS_INFORMATION,
            STARTF_USESTDHANDLES, STARTUPINFOW,
        },
    };

    pub struct Child(windows_sys::Win32::Foundation::HANDLE);
    impl Child {
        pub fn try_wait(&mut self) -> Result<Option<u32>> {
            let mut code = 0;
            // SAFETY: self owns a live process handle until Drop.
            if unsafe { GetExitCodeProcess(self.0, &mut code) } == 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            Ok((code != 259).then_some(code)) // STILL_ACTIVE
        }
    }
    impl Drop for Child {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }

    pub fn spawn_daemon(path: &Path) -> Result<Child> {
        use std::{ffi::OsString, os::windows::ffi::OsStrExt};
        let exe = std::env::current_exe()?;
        let application: Vec<u16> = exe.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut line = OsString::from("\"");
        line.push(&exe);
        line.push("\" daemon serve");
        let mut command: Vec<u16> = line.encode_wide().chain(Some(0)).collect();
        let mut environment: std::collections::BTreeMap<OsString, OsString> =
            std::env::vars_os().collect();
        environment.insert(
            OsString::from("ECLIPSECTL_RUNTIME_DIR"),
            path.as_os_str().to_owned(),
        );
        let mut block: Vec<u16> = Vec::new();
        for (key, value) in environment {
            block.extend(key.encode_wide());
            block.push(b'=' as u16);
            block.extend(value.encode_wide());
            block.push(0);
        }
        block.push(0);
        // Command::spawn inherits handles on Windows. Explicitly disable inheritance so a
        // background daemon cannot retain the caller's pipeline or startup file locks.
        // SAFETY: all pointers reference correctly terminated buffers alive for this call.
        unsafe {
            let mut startup: STARTUPINFOW = std::mem::zeroed();
            startup.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
            startup.dwFlags = STARTF_USESTDHANDLES;
            let mut info: PROCESS_INFORMATION = std::mem::zeroed();
            if CreateProcessW(
                application.as_ptr(),
                command.as_mut_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP | CREATE_UNICODE_ENVIRONMENT,
                block.as_ptr().cast(),
                std::ptr::null(),
                &startup,
                &mut info,
            ) == 0
            {
                return Err(std::io::Error::last_os_error())
                    .context("Cannot start eclipsectl daemon");
            }
            CloseHandle(info.hThread);
            Ok(Child(info.hProcess))
        }
    }

    pub fn current_user_sid() -> Result<String> {
        // SAFETY: all buffers outlive the Win32 calls; allocated handles and strings are released.
        unsafe {
            let mut token = std::ptr::null_mut();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            let mut bytes = 0;
            GetTokenInformation(token, TokenUser, std::ptr::null_mut(), 0, &mut bytes);
            let mut buffer = vec![0usize; (bytes as usize).div_ceil(std::mem::size_of::<usize>())];
            let ok = GetTokenInformation(
                token,
                TokenUser,
                buffer.as_mut_ptr().cast(),
                bytes,
                &mut bytes,
            );
            CloseHandle(token);
            if ok == 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            let info = &*buffer.as_ptr().cast::<TOKEN_USER>();
            let mut text = std::ptr::null_mut();
            if ConvertSidToStringSidW(info.User.Sid, &mut text) == 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            let mut len = 0;
            while *text.add(len) != 0 {
                len += 1;
            }
            let sid = String::from_utf16_lossy(std::slice::from_raw_parts(text, len));
            LocalFree(text.cast());
            Ok(sid)
        }
    }

    pub fn create_pipe(
        name: &str,
        first: bool,
    ) -> Result<tokio::net::windows::named_pipe::NamedPipeServer> {
        let sid = current_user_sid()?;
        let sddl: Vec<u16> = format!("D:P(A;;GA;;;{sid})\0").encode_utf16().collect();
        // SAFETY: the security descriptor is valid until create_with_security_attributes_raw returns.
        unsafe {
            let mut descriptor = std::ptr::null_mut();
            if ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                1,
                &mut descriptor,
                std::ptr::null_mut(),
            ) == 0
            {
                return Err(std::io::Error::last_os_error().into());
            }
            let mut attributes = SECURITY_ATTRIBUTES {
                nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: descriptor,
                bInheritHandle: 0,
            };
            let result = tokio::net::windows::named_pipe::ServerOptions::new()
                .first_pipe_instance(first)
                .reject_remote_clients(true)
                .create_with_security_attributes_raw(
                    name,
                    (&mut attributes as *mut SECURITY_ATTRIBUTES).cast(),
                );
            LocalFree(descriptor);
            Ok(result?)
        }
    }
}
