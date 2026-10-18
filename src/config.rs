use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub eclipse_url: String,
    pub bearer_token: Option<String>,
    pub connect_timeout: u64,
    pub operation_timeout: u64,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("eclipse_url", &self.eclipse_url)
            .field(
                "bearer_token",
                &self.bearer_token.as_ref().map(|_| "[REDACTED]"),
            )
            .field("connect_timeout", &self.connect_timeout)
            .field("operation_timeout", &self.operation_timeout)
            .finish()
    }
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        let url = reqwest::Url::parse(&self.eclipse_url).context("Invalid Eclipse URL")?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            bail!("Eclipse URL must be an HTTP(S) base URL without credentials, query or fragment");
        }
        if self.connect_timeout == 0
            || self.operation_timeout == 0
            || self.connect_timeout > 86400
            || self.operation_timeout > 86400
        {
            bail!("Timeouts must be between 1 and 86400 seconds");
        }
        if self
            .bearer_token
            .as_ref()
            .is_some_and(|s| s.trim().is_empty() || s.contains(['\r', '\n']))
        {
            bail!("Bearer token must be nonempty and contain no newlines");
        }
        Ok(())
    }

    pub fn connect_duration(&self) -> Duration {
        Duration::from_secs(self.connect_timeout)
    }
    pub fn operation_duration(&self) -> Duration {
        Duration::from_secs(self.operation_timeout)
    }

    pub fn redact(&self, message: impl AsRef<str>) -> String {
        match &self.bearer_token {
            Some(token) if !token.is_empty() => message.as_ref().replace(token, "[REDACTED]"),
            _ => message.as_ref().to_owned(),
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub eclipse_url: Option<String>,
    pub bearer_token: Option<String>,
    pub connect_timeout: Option<u64>,
    pub operation_timeout: Option<u64>,
}

pub fn user_config_path() -> Result<PathBuf> {
    Ok(dirs::config_dir()
        .context("Cannot locate user configuration directory")?
        .join("eclipsectl-cli/config.toml"))
}

fn read_settings(path: &Path) -> Result<Settings> {
    match fs::read_to_string(path) {
        Ok(text) => toml::from_str(text.trim_start_matches('\u{feff}'))
            .map_err(|_| anyhow::anyhow!("Invalid TOML configuration: {}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(e) => Err(e).with_context(|| format!("Cannot read {}", path.display())),
    }
}

fn read_project_settings(cwd: &Path) -> Result<Settings> {
    let preferred = cwd.join(".config/eclipsectl-cli/config.toml");
    if preferred
        .try_exists()
        .with_context(|| format!("Cannot inspect {}", preferred.display()))?
    {
        read_settings(&preferred)
    } else {
        read_settings(&cwd.join(".eclipsectl.toml"))
    }
}

pub fn load(overrides: &Settings, cwd: &Path) -> Result<Config> {
    let user = read_settings(&user_config_path()?)?;
    let project = read_project_settings(cwd)?;
    resolve(overrides, &std::env::vars().collect(), &project, &user)
}

pub fn resolve(
    cli: &Settings,
    env: &HashMap<String, String>,
    project: &Settings,
    user: &Settings,
) -> Result<Config> {
    let env_number = |key: &str| -> Result<Option<u64>> {
        env.get(key)
            .map(|s| {
                s.parse()
                    .with_context(|| format!("{key} must be a positive integer"))
            })
            .transpose()
    };
    let config = Config {
        eclipse_url: cli
            .eclipse_url
            .clone()
            .or_else(|| env.get("ECLIPSE_URL").cloned())
            .or_else(|| project.eclipse_url.clone())
            .or_else(|| user.eclipse_url.clone())
            .unwrap_or_else(|| "http://localhost:8124".into())
            .trim_end_matches('/')
            .into(),
        bearer_token: cli
            .bearer_token
            .clone()
            .or_else(|| env.get("ECLIPSE_TOKEN").cloned())
            .or_else(|| project.bearer_token.clone())
            .or_else(|| user.bearer_token.clone()),
        connect_timeout: cli
            .connect_timeout
            .or(if cli.connect_timeout.is_none() {
                env_number("ECLIPSE_CONNECT_TIMEOUT")?
            } else {
                None
            })
            .or(project.connect_timeout)
            .or(user.connect_timeout)
            .unwrap_or(30),
        operation_timeout: cli
            .operation_timeout
            .or(if cli.operation_timeout.is_none() {
                env_number("ECLIPSE_OPERATION_TIMEOUT")?
            } else {
                None
            })
            .or(project.operation_timeout)
            .or(user.operation_timeout)
            .unwrap_or(300),
    };
    config.validate()?;
    Ok(config)
}

pub fn migrate(from: &Path, to: &Path) -> Result<()> {
    let input: serde_json::Value =
        serde_json::from_str(fs::read_to_string(from)?.trim_start_matches('\u{feff}'))
            .map_err(|_| anyhow::anyhow!("Invalid JSON configuration"))?;
    let obj = input
        .as_object()
        .context("JSON configuration must be an object")?;
    for key in obj.keys() {
        if !["eclipse_url", "bearer_token", "timeout", "verbose"].contains(&key.as_str()) {
            bail!("Unknown legacy configuration field: {key}");
        }
    }
    let string = |key: &str| -> Result<Option<String>> {
        obj.get(key)
            .filter(|v| !v.is_null())
            .map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .with_context(|| format!("{key} must be a string"))
            })
            .transpose()
    };
    let settings = Settings {
        eclipse_url: string("eclipse_url")?,
        bearer_token: string("bearer_token")?,
        connect_timeout: obj
            .get("timeout")
            .map(|v| v.as_u64().context("timeout must be a positive integer"))
            .transpose()?,
        operation_timeout: None,
    };
    resolve(
        &settings,
        &HashMap::new(),
        &Settings::default(),
        &Settings::default(),
    )?;
    let output = toml::to_string_pretty(&settings)?;
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(to)
        .context("Cannot create destination (it must not already exist)")?;
    file.write_all(b"# Migrated by eclipsectl. Prefer ECLIPSE_TOKEN for secrets.\n")?;
    file.write_all(output.as_bytes())?;
    file.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_config_prefers_nested_toml_and_reloads_changes() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join(".config/eclipsectl-cli/config.toml");
        fs::create_dir_all(nested.parent().unwrap()).unwrap();
        fs::write(
            dir.path().join(".eclipsectl.toml"),
            "eclipse_url = 'http://localhost:8124'",
        )
        .unwrap();
        fs::write(&nested, "eclipse_url = 'http://localhost:8765'").unwrap();
        assert_eq!(
            read_project_settings(dir.path())
                .unwrap()
                .eclipse_url
                .as_deref(),
            Some("http://localhost:8765")
        );
        fs::write(&nested, "eclipse_url = 'http://localhost:8766'").unwrap();
        assert_eq!(
            read_project_settings(dir.path())
                .unwrap()
                .eclipse_url
                .as_deref(),
            Some("http://localhost:8766")
        );
    }

    #[test]
    fn project_config_uses_legacy_filename_only_when_nested_file_is_absent() {
        let dir = tempfile::tempdir().unwrap();
        assert!(
            read_project_settings(dir.path())
                .unwrap()
                .eclipse_url
                .is_none()
        );
        fs::write(
            dir.path().join(".eclipsectl.toml"),
            "eclipse_url = 'http://localhost:8765'",
        )
        .unwrap();
        assert_eq!(
            read_project_settings(dir.path())
                .unwrap()
                .eclipse_url
                .as_deref(),
            Some("http://localhost:8765")
        );
        let nested = dir.path().join(".config/eclipsectl-cli/config.toml");
        fs::create_dir_all(nested.parent().unwrap()).unwrap();
        fs::write(&nested, "connect_timeout = 5").unwrap();
        let project = read_project_settings(dir.path()).unwrap();
        assert!(project.eclipse_url.is_none());
        assert_eq!(project.connect_timeout, Some(5));
        fs::write(&nested, "invalid TOML").unwrap();
        assert!(read_project_settings(dir.path()).is_err());
    }
}
