use crate::{
    config::Config,
    ipc::Response,
    operations::{Call, find},
};
use anyhow::{Context, Result};
use rmcp::{
    RoleClient, ServiceExt,
    model::{CallToolRequestParams, CallToolResponse},
    service::RunningService,
    transport::{
        StreamableHttpClientTransport, streamable_http_client::StreamableHttpClientTransportConfig,
    },
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

type Session = RunningService<RoleClient, ()>;
type Slot = Arc<Mutex<Option<Session>>>;

#[derive(Default)]
pub struct Pool {
    sessions: Mutex<HashMap<[u8; 32], Slot>>,
}

impl Pool {
    async fn slot(&self, config: &Config, server: &str) -> Slot {
        let mut hash = Sha256::new();
        for part in [
            config.eclipse_url.as_bytes(),
            config.bearer_token.as_deref().unwrap_or("").as_bytes(),
            server.as_bytes(),
        ] {
            hash.update((part.len() as u64).to_be_bytes());
            hash.update(part);
        }
        // Connection timeout is part of transport configuration, operation timeout is per call.
        hash.update(config.connect_timeout.to_be_bytes());
        self.sessions
            .lock()
            .await
            .entry(hash.finalize().into())
            .or_insert_with(|| Arc::new(Mutex::new(None)))
            .clone()
    }

    async fn connect(config: &Config, server: &str) -> Result<Session> {
        config.validate()?;
        let client = reqwest::Client::builder()
            .connect_timeout(config.connect_duration())
            .retry(reqwest::retry::never())
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        let mut transport_config = StreamableHttpClientTransportConfig::with_uri(format!(
            "{}/mcp/{server}",
            config.eclipse_url.trim_end_matches('/')
        ))
        .max_concurrent_requests(1);
        if let Some(token) = &config.bearer_token {
            transport_config = transport_config.auth_header(token);
        }
        // Never replay calls after an expired session: writes may already have executed.
        transport_config.reinit_on_expired_session = false;
        transport_config.max_sse_event_size = crate::ipc::MAX_FRAME;
        let transport = StreamableHttpClientTransport::with_client(client, transport_config);
        tokio::time::timeout(config.connect_duration(), ().serve(transport))
            .await
            .context("MCP initialization timed out")?
            .context("MCP initialization failed")
    }

    pub async fn call(&self, config: &Config, call: &Call) -> Response {
        let op = match find(&call.operation) {
            Ok(op) => op,
            Err(_) => return Response::error("invalid_operation", "Unknown operation"),
        };
        if let Err(e) = config
            .validate()
            .and_then(|()| op.validate(&call.arguments))
        {
            return Response::error("invalid_request", e.to_string());
        }
        let slot = self.slot(config, &op.server).await;
        // Waiting for a session is bounded separately; the operation gets its complete budget after acquiring it.
        let mut session = match tokio::time::timeout(config.operation_duration(), slot.lock()).await
        {
            Ok(guard) => guard,
            Err(_) => {
                return Response::error("queue_timeout", "Timed out waiting for the MCP session");
            }
        };
        if session.is_none() {
            match Self::connect(config, &op.server).await {
                Ok(connected) => *session = Some(connected),
                Err(e) => {
                    return Response::error("connection_failed", config.redact(format!("{e:#}")));
                }
            }
        }
        let arguments = match op.wire_arguments(&call.arguments) {
            Ok(arguments) => arguments,
            Err(_) => {
                return Response::error(
                    "invalid_arguments",
                    "Cannot serialize operation arguments",
                );
            }
        };
        let params = CallToolRequestParams::new(op.tool.clone()).with_arguments(arguments);
        // call_tool_once deliberately avoids automatic input-required retries.
        let outcome = tokio::time::timeout(
            config.operation_duration(),
            session.as_ref().expect("connected").call_tool_once(params),
        )
        .await;
        match outcome {
            Ok(Ok(CallToolResponse::Complete(result))) => {
                let value = match serde_json::to_value(result) {
                    Ok(v) => v,
                    Err(_) => {
                        return Response::error(
                            "serialization_failed",
                            "Cannot encode MCP response",
                        );
                    }
                };
                if value.get("isError").and_then(Value::as_bool) == Some(true) {
                    let mut response =
                        Response::error("mcp_tool_error", "Eclipse reported a tool error");
                    response.error.as_mut().expect("error").details = Some(value);
                    response
                } else {
                    Response::success(value)
                }
            }
            Ok(Ok(other)) => {
                let mut response = Response::error(
                    "unsupported_mcp_result",
                    "Eclipse requested additional input or an asynchronous task; this CLI does not replay the call",
                );
                response.error.as_mut().expect("error").details = match other {
                    CallToolResponse::InputRequired(value) => serde_json::to_value(value).ok(),
                    CallToolResponse::Task(value) => serde_json::to_value(value).ok(),
                    _ => None,
                };
                response
            }
            Ok(Err(e)) => {
                if let Some(old) = session.take() {
                    let _ =
                        tokio::time::timeout(std::time::Duration::from_secs(2), old.cancel()).await;
                }
                Response::error(
                    "mcp_failed",
                    config.redact(format!(
                        "MCP call failed; execution may have occurred and was not retried: {e}"
                    )),
                )
            }
            Err(_) => {
                if let Some(old) = session.take() {
                    let _ =
                        tokio::time::timeout(std::time::Duration::from_secs(2), old.cancel()).await;
                }
                Response::error(
                    "operation_timeout",
                    "MCP operation timed out; execution may have occurred and was not retried",
                )
            }
        }
    }

    pub async fn doctor(&self, config: &Config) -> Response {
        if let Err(e) = config.validate() {
            return Response::error("invalid_config", e.to_string());
        }
        let slot = self.slot(config, "eclipse-ide").await;
        let mut session = match tokio::time::timeout(config.operation_duration(), slot.lock()).await
        {
            Ok(guard) => guard,
            Err(_) => return Response::error("queue_timeout", "MCP session is busy"),
        };
        if session.is_none() {
            match Self::connect(config, "eclipse-ide").await {
                Ok(connected) => *session = Some(connected),
                Err(e) => return Response::error("connection_failed", config.redact(format!("{e:#}. Check Eclipse, its HTTP bind address, port, token and firewall. WSL NAT requires the Windows host IP; mirrored networking can use localhost."))),
            }
        }
        match tokio::time::timeout(
            config.operation_duration(),
            session.as_ref().expect("connected").list_all_tools(),
        )
        .await
        {
            Ok(Ok(tools)) => Response::success(
                json!({"configuration":"ok", "daemon":"ok", "mcp":"ok", "eclipse_url":config.eclipse_url, "authentication_configured":config.bearer_token.is_some(), "tools":tools.len(), "wsl":crate::paths::is_wsl()}),
            ),
            _ => {
                *session = None;
                Response::error(
                    "health_check_failed",
                    "MCP tools/list failed; check the Eclipse plugin",
                )
            }
        }
    }

    pub async fn close(&self) {
        let slots = std::mem::take(&mut *self.sessions.lock().await);
        for (_, slot) in slots {
            if let Some(session) = slot.lock().await.take() {
                let _ =
                    tokio::time::timeout(std::time::Duration::from_secs(2), session.cancel()).await;
            }
        }
    }
}
