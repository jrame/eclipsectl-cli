//! Bounded, read-only polling of Eclipse state. The session stays locked throughout.
use crate::ipc::Response;
use anyhow::{Result, bail};
use serde::Deserialize;
use serde_json::Value;
use std::{future::Future, time::Duration};
use tokio::time::{Instant, sleep_until, timeout_at};

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaitCondition {
    WorkspaceIdle,
    BuildIdle,
    Suspended,
}

impl WaitCondition {
    fn label(self) -> &'static str {
        match self {
            Self::WorkspaceIdle => "workspace idle",
            Self::BuildIdle => "build completion",
            Self::Suspended => "debugger suspended",
        }
    }

    fn reached(self, result: &Value) -> Result<bool> {
        let (key, prefix, ready, pending): (_, _, _, &[&str]) = match self {
            Self::WorkspaceIdle => ("workspaceState", "Workspace status: ", "idle", &["busy"]),
            Self::BuildIdle => ("buildState", "Build status: ", "idle", &["busy"]),
            Self::Suspended => (
                "state",
                "Debug status: ",
                "suspended",
                &["none", "running", "terminated"],
            ),
        };
        // Prefer structured state when supplied; older debug plugins return text only.
        let state = result
            .get("structuredContent")
            .and_then(|value| value.get(key))
            .and_then(Value::as_str)
            .or_else(|| {
                result.get("content")?.as_array()?.iter().find_map(|item| {
                    if item.get("type")?.as_str()? != "text" {
                        return None;
                    }
                    item.get("text")?
                        .as_str()?
                        .lines()
                        .find_map(|line| line.strip_prefix(prefix).map(str::trim))
                })
            });
        match state {
            Some(state) if state == ready => Ok(true),
            Some("terminated") if matches!(self, Self::Suspended) => {
                bail!("Debug session terminated before suspension")
            }
            Some(state) if pending.contains(&state) => Ok(false),
            _ => bail!(
                "Eclipse did not return a recognized state for {}; update the eclipsectl-mcp plugin (workspace/build waits require getWorkspaceState)",
                self.label()
            ),
        }
    }
}

/// Poll successful state reads only. Failed calls are returned immediately, never retried.
pub async fn until<F, Fut>(
    condition: WaitCondition,
    budget: Duration,
    interval: Duration,
    mut probe: F,
) -> Response
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Response>,
{
    let deadline = Instant::now() + budget;
    let mut idle_since = None;
    let mut latest = None;
    loop {
        if Instant::now() >= deadline {
            break;
        }
        let response = match timeout_at(deadline, probe()).await {
            Ok(response) => response,
            Err(_) => break,
        };
        if !response.ok {
            return response;
        }
        let state = response
            .result
            .as_ref()
            .map(|value| condition.reached(value));
        match state {
            Some(Ok(true)) => {
                // Builds may schedule follow-up jobs. Require a brief continuous quiet window.
                let since = *idle_since.get_or_insert_with(Instant::now);
                if matches!(condition, WaitCondition::Suspended)
                    || since.elapsed() >= Duration::from_millis(500)
                {
                    return response;
                }
            }
            Some(Ok(false)) => idle_since = None,
            _ => {
                let message = state
                    .and_then(Result::err)
                    .map(|error| error.to_string())
                    .unwrap_or_else(|| "Eclipse returned no state result".into());
                return Response::error("wait_state_unavailable", message);
            }
        }
        latest = response.result;
        sleep_until((Instant::now() + interval).min(deadline)).await;
    }
    let mut response = Response::error(
        "wait_timeout",
        format!(
            "Timed out after {}s waiting for {}",
            budget.as_secs(),
            condition.label()
        ),
    );
    response.error.as_mut().expect("error").details = latest;
    response
}
