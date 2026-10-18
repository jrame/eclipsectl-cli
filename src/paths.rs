use anyhow::{Context, Result, bail};
use std::path::Path;

pub fn is_windows(path: &str) -> bool {
    let b = path.as_bytes();
    (b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && matches!(b[2], b'\\' | b'/'))
        || path.starts_with("\\\\")
}

pub fn windows_path(path: &str, cwd: &Path, project_relative: bool) -> Result<String> {
    if is_windows(path) {
        let lower = path.to_ascii_lowercase();
        if lower.starts_with("\\\\wsl$\\") || lower.starts_with("\\\\wsl.localhost\\") {
            bail!("Linux filesystem projects are unsupported; use a Windows drive");
        }
        return Ok(path.replace('/', "\\"));
    }
    if project_relative && !Path::new(path).is_absolute() {
        return Ok(path.replace('\\', "/"));
    }
    let absolute = if Path::new(path).is_absolute() {
        path.into()
    } else {
        cwd.join(path)
    };
    #[cfg(windows)]
    {
        if path.starts_with('/') {
            bail!(
                "Use Windows drive paths with eclipsectl.exe, or run the native WSL binary for Linux paths"
            );
        }
        let text = absolute.to_str().context("Path is not valid Unicode")?;
        if !is_windows(text) {
            bail!("Expected a Windows path; use the native WSL binary for Linux paths");
        }
        Ok(text.replace('/', "\\"))
    }
    #[cfg(unix)]
    {
        if !is_wsl() {
            bail!(
                "Eclipse Windows paths must be Windows absolute paths, or paths on Windows drives under WSL"
            );
        }
        let output = std::process::Command::new("wslpath")
            .arg("-w")
            .arg(&absolute)
            .output()
            .context("Cannot execute wslpath (required for WSL path conversion)")?;
        if !output.status.success() {
            bail!("wslpath could not convert the supplied path");
        }
        let translated = String::from_utf8(output.stdout)?
            .trim_end_matches(['\r', '\n'])
            .to_owned();
        if !is_windows(&translated) || translated.to_lowercase().starts_with("\\\\wsl") {
            bail!("Linux filesystem projects are unsupported; use a Windows drive such as /mnt/c");
        }
        Ok(translated)
    }
}

pub fn is_wsl() -> bool {
    std::env::var_os("WSL_DISTRO_NAME").is_some()
        || std::fs::read_to_string("/proc/sys/kernel/osrelease")
            .is_ok_and(|s| s.to_lowercase().contains("microsoft"))
}
