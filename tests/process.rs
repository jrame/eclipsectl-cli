use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::{io::AsyncWriteExt, process::Command};

struct Sandbox {
    dir: tempfile::TempDir,
}
impl Sandbox {
    fn new() -> Self {
        Self {
            dir: tempfile::tempdir().unwrap(),
        }
    }
    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_eclipsectl"));
        command
            .current_dir(self.dir.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("ECLIPSECTL_RUNTIME_DIR", self.dir.path().join("run"))
            .env_remove("ECLIPSE_URL")
            .env_remove("ECLIPSE_TOKEN")
            .env_remove("ECLIPSE_CONNECT_TIMEOUT")
            .env_remove("ECLIPSE_OPERATION_TIMEOUT");
        #[cfg(windows)]
        command.creation_flags(0x08000000);
        command
    }
    async fn json(&self, args: &[&str]) -> Value {
        let output = self
            .command()
            .args(args)
            .arg("--json")
            .output()
            .await
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_eclipsectl"));
        command
            .current_dir(self.dir.path())
            .env("ECLIPSECTL_RUNTIME_DIR", self.dir.path().join("run"))
            .args(["daemon", "stop"])
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let _ = command.status();
    }
}

#[tokio::test]
async fn help_version_and_invalid_arguments_do_not_start_daemon() {
    let sandbox = Sandbox::new();
    let output = sandbox.command().arg("--help").output().await.unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("eclipsectl"));
    assert!(!sandbox.dir.path().join("run").exists());
    for args in [
        vec!["workspace", "wait", "--help"],
        vec!["build", "wait", "--help"],
        vec!["debug", "wait", "--help"],
    ] {
        let output = sandbox.command().args(args).output().await.unwrap();
        assert!(output.status.success());
    }
    for args in [
        vec!["workspace", "wait", "--json"],
        vec!["debug", "wait", "--json"],
        vec!["build", "wait", "--timeout", "0", "--json"],
    ] {
        let output = sandbox.command().args(args).output().await.unwrap();
        assert_eq!(output.status.code(), Some(2));
        let error: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(error["error"]["code"], "invalid_arguments");
    }
    assert!(!sandbox.dir.path().join("run").exists());
    let output = sandbox
        .command()
        .args(["debug", "variables", "--frame", "invalid", "--json"])
        .output()
        .await
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let error: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(error["error"]["code"], "invalid_arguments");
    assert_eq!(
        sandbox.json(&["daemon", "status"]).await["result"]["running"],
        false
    );
    assert!(!sandbox.dir.path().join("run").exists());
}

#[tokio::test]
async fn explicit_wait_binary_preserves_json_and_reports_deadlines() {
    use axum::{
        Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post,
    };
    use serde_json::json;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    async fn state_read(
        State(count): State<Arc<AtomicUsize>>,
        Json(request): Json<Value>,
    ) -> axum::response::Response {
        let id = request["id"].clone();
        let result = match request["method"].as_str().unwrap_or("") {
            "initialize" => {
                json!({"protocolVersion":"2025-03-26","capabilities":{"tools":{}},"serverInfo":{"name":"wait-mock","version":"1"}})
            }
            "notifications/initialized" | "notifications/cancelled" => {
                return StatusCode::ACCEPTED.into_response();
            }
            "tools/call" => {
                assert_eq!(request["params"]["arguments"], json!({}));
                let text = match request["params"]["name"].as_str() {
                    Some("getDebugStatus") => {
                        if count.fetch_add(1, Ordering::SeqCst) < 2 {
                            "Debug status: running"
                        } else {
                            "Debug status: suspended\nSuspended at: Main.java:12"
                        }
                    }
                    Some("getWorkspaceState") => "Workspace status: busy\nBuild status: busy",
                    _ => panic!("unexpected tool"),
                };
                json!({"content":[{"type":"text","text":text}]})
            }
            _ => return StatusCode::ACCEPTED.into_response(),
        };
        Json(json!({"jsonrpc":"2.0","id":id,"result":result})).into_response()
    }
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let count = Arc::new(AtomicUsize::new(0));
    let app = Router::new()
        .route("/mcp/{server}", post(state_read))
        .with_state(count.clone());
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let sandbox = Sandbox::new();
    // The explicit polling budget exceeds operation-timeout; each state read is quick.
    let output = tokio::time::timeout(
        Duration::from_secs(10),
        sandbox
            .command()
            .args([
                "debug",
                "wait",
                "--suspended",
                "--timeout",
                "4",
                "--poll-interval",
                "600",
                "--operation-timeout",
                "1",
                "--eclipse-url",
                &url,
                "--json",
            ])
            .output(),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["ok"], true);
    assert_eq!(
        result["result"]["content"][0]["text"],
        "Debug status: suspended\nSuspended at: Main.java:12"
    );
    assert_eq!(count.load(Ordering::SeqCst), 3);
    let output = sandbox
        .command()
        .args([
            "build",
            "wait",
            "--timeout",
            "1",
            "--eclipse-url",
            &url,
            "--json",
        ])
        .output()
        .await
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["ok"], false);
    assert_eq!(result["error"]["code"], "wait_timeout");
    assert_eq!(
        result["error"]["details"]["content"][0]["text"],
        "Workspace status: busy\nBuild status: busy"
    );
    server.abort();
}

#[tokio::test]
async fn java_refactoring_help_and_invalid_selections_do_not_start_daemon() {
    let sandbox = Sandbox::new();
    for command in [
        "rename-method",
        "rename-field",
        "rename-variable",
        "change-signature",
        "extract-variable",
        "extract-constant",
        "inline-method",
        "inline-variable",
    ] {
        let output = sandbox
            .command()
            .args(["refactor", command, "--help"])
            .output()
            .await
            .unwrap();
        assert!(output.status.success(), "{command}");
        assert!(String::from_utf8_lossy(&output.stdout).contains("offset"));
    }
    for args in [
        vec![
            "refactor",
            "extract-variable",
            "Example.java",
            "0",
            "0",
            "value",
        ],
        vec![
            "refactor",
            "change-signature",
            "Example.java",
            "0",
            "--parameters",
            "{}",
        ],
        vec![
            "refactor",
            "inline-method",
            "Example.java",
            "0",
            "--delete-source",
            "true",
        ],
    ] {
        let output = sandbox
            .command()
            .args(args)
            .arg("--json")
            .output()
            .await
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        let response: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(response["error"]["code"], "invalid_arguments");
    }
    assert!(!sandbox.dir.path().join("run").exists());
}

#[tokio::test]
async fn simultaneous_autostart_restart_and_crash_recovery_use_one_daemon() {
    let sandbox = Sandbox::new();
    // doctor starts the daemon even though no Eclipse listens at this local port.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let mut children = Vec::new();
    for _ in 0..6 {
        children.push(
            sandbox
                .command()
                .args([
                    "doctor",
                    "--eclipse-url",
                    &url,
                    "--connect-timeout",
                    "1",
                    "--operation-timeout",
                    "1",
                    "--json",
                ])
                .spawn()
                .unwrap(),
        );
    }
    for child in children {
        assert_eq!(
            child.wait_with_output().await.unwrap().status.code(),
            Some(1)
        );
    }
    let before = sandbox.json(&["daemon", "status"]).await;
    assert_eq!(before["result"]["running"], true);
    let pid = before["result"]["pid"].as_u64().unwrap() as u32;
    terminate(pid);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let after = sandbox.json(&["daemon", "restart"]).await;
    assert_eq!(after["result"]["running"], true);
    assert_ne!(after["result"]["pid"], pid);
    let stopped = sandbox.json(&["daemon", "stop"]).await;
    assert_eq!(stopped["result"]["stopping"], true);
}

#[cfg(unix)]
fn terminate(pid: u32) {
    // SAFETY: pid was returned by the isolated daemon started by this test.
    assert_eq!(unsafe { libc::kill(pid as i32, libc::SIGKILL) }, 0);
}
#[cfg(windows)]
fn terminate(pid: u32) {
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess},
    };
    // SAFETY: handle is checked and closed; pid belongs to this test's isolated daemon.
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        assert!(!handle.is_null());
        assert_ne!(TerminateProcess(handle, 1), 0);
        CloseHandle(handle);
    }
}

#[tokio::test]
async fn stdin_source_is_sent_without_shell_or_json_interpolation() {
    use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::post};
    use serde_json::json;
    let app = Router::new().route("/mcp/eclipse-source", post(|Json(request): Json<Value>| async move {
        let result = match request["method"].as_str().unwrap_or("") {
            "initialize" => json!({"protocolVersion":"2025-03-26", "capabilities":{"tools":{}}, "serverInfo":{"name":"stdin-mock","version":"1"}}),
            "notifications/initialized" => return StatusCode::ACCEPTED.into_response(),
            "tools/call" => json!({"content":[],"structuredContent":request["params"]["arguments"]}),
            _ => json!({}),
        };
        Json(json!({"jsonrpc":"2.0","id":request["id"],"result":result})).into_response()
    }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let sandbox = Sandbox::new();
    let source = "public String value() {\r\n  return \"é /mnt/c/$HOME `literal`\";\r\n}\r\n";
    let mut child = sandbox
        .command()
        .args([
            "java",
            "write",
            "src/Main.java",
            "--project",
            "example",
            "--eclipse-url",
            &url,
            "--json",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .await
        .unwrap();
    let output = child.wait_with_output().await.unwrap();
    assert!(
        output.status.success(),
        "{} {}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["result"]["structuredContent"]["newSource"], source);
    server.abort();
}

#[cfg(unix)]
#[test]
fn wslpath_respects_mounts_and_rejects_linux_filesystem_projects() {
    if !eclipsectl::paths::is_wsl() {
        return;
    }
    let cwd = std::env::current_dir().unwrap();
    let windows = eclipsectl::paths::windows_path(cwd.to_str().unwrap(), &cwd, false).unwrap();
    assert!(eclipsectl::paths::is_windows(&windows));
    assert!(eclipsectl::paths::windows_path("/home/example/File.java", &cwd, false).is_err());
}

#[tokio::test]
async fn run_wait_sends_a_boolean_and_waits_for_the_plugin_response() {
    use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::post};
    use serde_json::json;
    use std::sync::Arc;
    use tokio::sync::{Notify, Semaphore};

    let entered = Arc::new(Notify::new());
    let finish = Arc::new(Semaphore::new(0));
    let app = Router::new().route("/mcp/eclipse", post({
        let entered = entered.clone();
        let finish = finish.clone();
        move |Json(request): Json<Value>| {
            let entered = entered.clone();
            let finish = finish.clone();
            async move {
                let result = match request["method"].as_str().unwrap_or("") {
                    "initialize" => json!({"protocolVersion":"2025-03-26", "capabilities":{"tools":{}}, "serverInfo":{"name":"launch-mock","version":"1"}}),
                    "notifications/initialized" => return StatusCode::ACCEPTED.into_response(),
                    "tools/call" => {
                        let args = &request["params"]["arguments"];
                        if request["params"]["name"] != "run"
                            || args["configuration"] != "FooTest"
                            || args["waitForCompletion"] != json!(true)
                        {
                            return Json(json!({"jsonrpc":"2.0","id":request["id"],"result":{"content":[{"type":"text","text":"argument type mismatch"}],"isError":true}})).into_response();
                        }
                        entered.notify_one();
                        let _permit = finish.acquire().await.unwrap();
                        json!({"content":[{"type":"text","text":"Launch 'FooTest' completed"}]})
                    }
                    _ => json!({}),
                };
                Json(json!({"jsonrpc":"2.0","id":request["id"],"result":result})).into_response()
            }
        }
    }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let sandbox = Sandbox::new();
    let child = sandbox
        .command()
        .args([
            "run",
            "--wait",
            "--configuration",
            "FooTest",
            "--eclipse-url",
            &url,
            "--json",
        ])
        .spawn()
        .unwrap();
    let output = tokio::spawn(child.wait_with_output());
    tokio::time::timeout(Duration::from_secs(10), entered.notified())
        .await
        .unwrap();
    assert!(
        !output.is_finished(),
        "CLI returned before launch completion"
    );
    finish.add_permits(1);
    let output = tokio::time::timeout(Duration::from_secs(10), output)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(
        output.status.success(),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        result["result"]["content"][0]["text"],
        "Launch 'FooTest' completed"
    );
    server.abort();
}

#[tokio::test]
async fn local_nested_config_is_reloaded_without_restarting_daemon() {
    use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::post};
    use serde_json::json;
    let sandbox = Sandbox::new();
    let config = sandbox
        .dir
        .path()
        .join(".config/eclipsectl-cli/config.toml");
    std::fs::create_dir_all(config.parent().unwrap()).unwrap();
    // A malformed fallback must be ignored while the preferred file exists.
    std::fs::write(sandbox.dir.path().join(".eclipsectl.toml"), "invalid TOML").unwrap();
    let mut pid = None;
    for token in ["first-token", "second-token"] {
        let app = Router::new().route("/mcp/eclipse-ide", post(move |headers: axum::http::HeaderMap, Json(request): Json<Value>| async move {
            assert_eq!(headers["authorization"], format!("Bearer {token}"));
            let result = match request["method"].as_str().unwrap_or("") {
                "initialize" => json!({"protocolVersion":"2025-03-26", "capabilities":{"tools":{}}, "serverInfo":{"name":"config-mock","version":"1"}}),
                "notifications/initialized" => return StatusCode::ACCEPTED.into_response(),
                "tools/list" => json!({"tools":[]}),
                _ => json!({}),
            };
            Json(json!({"jsonrpc":"2.0","id":request["id"],"result":result})).into_response()
        }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        std::fs::write(&config, format!("eclipse_url = '{url}'\nbearer_token = '{token}'\nconnect_timeout = 2\noperation_timeout = 2\n")).unwrap();
        let doctor = sandbox.json(&["doctor"]).await;
        assert_eq!(doctor["result"]["eclipse_url"], url);
        assert_eq!(doctor["result"]["authentication_configured"], true);
        let current_pid = sandbox.json(&["daemon", "status"]).await["result"]["pid"].clone();
        if let Some(previous) = pid {
            assert_eq!(current_pid, previous);
        }
        pid = Some(current_pid);
        server.abort();
    }
}

#[test]
fn installer_sources_exist_and_binary_name_is_fixed() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(root.join("install.ps1").is_file());
    assert!(root.join("install.sh").is_file());
    assert_eq!(
        Path::new(env!("CARGO_BIN_EXE_eclipsectl"))
            .file_stem()
            .unwrap(),
        "eclipsectl"
    );
}
