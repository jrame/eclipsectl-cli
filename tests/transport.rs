use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response as HttpResponse},
    routing::post,
};
use eclipsectl::{
    config::Config,
    daemon,
    ipc::{Action, Response},
    mcp::Pool,
    operations::Call,
};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{sync::Mutex, task::JoinHandle};

#[derive(Default)]
struct StateData {
    initialize: AtomicUsize,
    calls: Mutex<Vec<(String, String, Value)>>,
    delay: Mutex<Duration>,
    read_files: Mutex<HashSet<String>>,
    replies: Mutex<HashMap<String, VecDeque<Value>>>,
}

struct Mock {
    state: Arc<StateData>,
    config: Config,
    task: JoinHandle<()>,
}
impl Drop for Mock {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Mock {
    async fn start() -> Self {
        let state = Arc::new(StateData::default());
        let app = Router::new()
            .route("/mcp/{server}", post(handler))
            .with_state(state.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        Self {
            state,
            config: Config {
                eclipse_url: format!("http://{addr}"),
                bearer_token: Some("mock-secret".into()),
                connect_timeout: 3,
                operation_timeout: 10,
            },
            task,
        }
    }
}

async fn handler(
    State(state): State<Arc<StateData>>,
    Path(server): Path<String>,
    headers: HeaderMap,
    Json(request): Json<Value>,
) -> HttpResponse {
    if headers.get("authorization").and_then(|v| v.to_str().ok()) != Some("Bearer mock-secret") {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let result = match request["method"].as_str().unwrap_or("") {
        "initialize" => {
            state.initialize.fetch_add(1, Ordering::SeqCst);
            let response = json!({"jsonrpc":"2.0", "id":id, "result":{"protocolVersion":"2025-03-26", "capabilities":{"tools":{}}, "serverInfo":{"name":"eclipse-mock","version":"1"}}});
            return ([("mcp-session-id", "test-session")], Json(response)).into_response();
        }
        "notifications/initialized" | "notifications/cancelled" => {
            return StatusCode::ACCEPTED.into_response();
        }
        "tools/list" => json!({"tools":[]}),
        "tools/call" => {
            let tool = request["params"]["name"].as_str().unwrap_or("").to_owned();
            let args = request["params"]["arguments"].clone();
            state
                .calls
                .lock()
                .await
                .push((server.clone(), tool.clone(), args.clone()));
            let file = args["filePath"].as_str().unwrap_or("").to_owned();
            if tool == "readJava" {
                state.read_files.lock().await.insert(file.clone());
            }
            if tool == "resetReadGate" {
                state.read_files.lock().await.remove(&file);
            }
            if tool == "writeJava" && !state.read_files.lock().await.contains(&file) {
                return Json(json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":"Read required"}],"isError":true}})).into_response();
            }
            let delay = *state.delay.lock().await;
            tokio::time::sleep(delay).await;
            if let Some(replies) = state.replies.lock().await.get_mut(&tool) {
                let result = if replies.len() > 1 {
                    replies.pop_front().unwrap()
                } else {
                    replies.front().unwrap().clone()
                };
                return Json(json!({"jsonrpc":"2.0","id":id,"result":result})).into_response();
            }
            match args.get("containingText").and_then(Value::as_str) {
                Some("expire") => return StatusCode::NOT_FOUND.into_response(),
                Some("error") => {
                    json!({"content":[{"type":"text","text":"readJava required before writeJava"}],"isError":true})
                }
                Some("large") => {
                    json!({"content":[{"type":"text","text":"x".repeat(200_000)}],"structuredContent":{"count":200000}})
                }
                Some("sse") => {
                    let response = json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":"SSE works"}]}});
                    return (
                        [("content-type", "text/event-stream")],
                        format!("event: message\ndata: {response}\n\n"),
                    )
                        .into_response();
                }
                _ => {
                    json!({"content":[{"type":"text","text":"ok"}], "structuredContent":{"server":server,"arguments":args}})
                }
            }
        }
        _ => return Json(
            json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"Method not found"}}),
        )
        .into_response(),
    };
    Json(json!({"jsonrpc":"2.0","id":id,"result":result})).into_response()
}

fn call(operation: &str, args: Value) -> Call {
    Call {
        operation: operation.into(),
        arguments: args.as_object().unwrap().clone(),
    }
}
fn search(text: &str) -> Call {
    call("search text", json!({"containingText":text}))
}

#[tokio::test]
async fn java_refactorings_route_typed_arguments_through_the_mcp_session() {
    let mock = Mock::start().await;
    let pool = Pool::default();
    for (command, tool, arguments, wire) in [
        (
            "rename-method",
            "refactorRenameMethod",
            json!({"newName":"calculate"}),
            json!({"newName":"calculate"}),
        ),
        (
            "rename-field",
            "refactorRenameField",
            json!({"newName":"total"}),
            json!({"newName":"total"}),
        ),
        (
            "rename-variable",
            "refactorRenameVariable",
            json!({"newName":"amount"}),
            json!({"newName":"amount"}),
        ),
        (
            "change-signature",
            "refactorChangeSignature",
            json!({"parameters":"[]"}),
            json!({"parameters":"[]"}),
        ),
        (
            "extract-variable",
            "refactorExtractVariable",
            json!({"length":5,"variableName":"subtotal","replaceAll":false,"declareFinal":true}),
            json!({"length":"5","variableName":"subtotal","replaceAll":false,"declareFinal":true}),
        ),
        (
            "extract-constant",
            "refactorExtractConstant",
            json!({"length":2,"constantName":"LIMIT","visibility":"private"}),
            json!({"length":"2","constantName":"LIMIT","visibility":"private"}),
        ),
        (
            "inline-method",
            "refactorInlineMethod",
            json!({"inlineAll":true,"deleteSource":false}),
            json!({"inlineAll":true,"deleteSource":false}),
        ),
        (
            "inline-variable",
            "refactorInlineVariable",
            json!({}),
            json!({}),
        ),
    ] {
        let mut arguments = arguments.as_object().unwrap().clone();
        arguments.extend(
            json!({"filePath":"src/é Example.java","projectName":"disposable","offset":120})
                .as_object()
                .unwrap()
                .clone(),
        );
        let response = pool
            .call(
                &mock.config,
                &Call {
                    operation: format!("refactor {command}"),
                    arguments,
                },
            )
            .await;
        assert!(response.ok, "{command}: {response:?}");
        let mut expected = wire.as_object().unwrap().clone();
        expected.extend(
            json!({"filePath":"src/é Example.java","projectName":"disposable","offset":"120"})
                .as_object()
                .unwrap()
                .clone(),
        );
        if command == "rename-variable" {
            expected.insert("offset".into(), json!(120));
        }
        let calls = mock.state.calls.lock().await;
        let (server, actual_tool, actual_arguments) = calls.last().unwrap();
        assert_eq!(server, "eclipse-refactor");
        assert_eq!(actual_tool, tool);
        assert_eq!(actual_arguments, &Value::Object(expected));
    }
    assert_eq!(mock.state.initialize.load(Ordering::SeqCst), 1);
    pool.close().await;
}

#[tokio::test]
async fn mcp_routes_reuses_sessions_preserves_results_and_errors() {
    let mock = Mock::start().await;
    let pool = Pool::default();
    let project = pool
        .call(&mock.config, &call("project list", json!({})))
        .await;
    assert!(project.ok, "{project:?}");
    let launch = pool
        .call(
            &mock.config,
            &call(
                "launch create-java",
                json!({"mainClass":"Main", "appendSystemEnvironment":false, "type":"junit5"}),
            ),
        )
        .await;
    assert!(launch.ok, "{launch:?}");
    assert_eq!(
        launch.result.as_ref().unwrap()["structuredContent"]["server"],
        "eclipse-debug"
    );
    assert!(pool.call(&mock.config, &search("large")).await.ok);
    let large = pool
        .call(&mock.config, &search("large"))
        .await
        .result
        .unwrap();
    assert_eq!(large["content"][0]["text"].as_str().unwrap().len(), 200_000);
    assert_eq!(large["structuredContent"]["count"], 200_000);
    assert_eq!(
        pool.call(&mock.config, &search("sse"))
            .await
            .result
            .unwrap()["content"][0]["text"],
        "SSE works"
    );
    let error = pool.call(&mock.config, &search("error")).await;
    assert!(!error.ok);
    assert_eq!(error.error.unwrap().details.unwrap()["isError"], true);
    assert_eq!(mock.state.initialize.load(Ordering::SeqCst), 2);
    let mut wrong = mock.config.clone();
    wrong.bearer_token = Some("wrong-token".into());
    let unauthorized = pool.call(&wrong, &search("test")).await;
    assert!(!unauthorized.ok);
    assert!(!format!("{unauthorized:?}").contains("wrong-token"));
    pool.close().await;
}

#[tokio::test]
async fn expired_session_is_not_replayed_and_next_call_reconnects() {
    let mock = Mock::start().await;
    let pool = Pool::default();
    assert!(!pool.call(&mock.config, &search("expire")).await.ok);
    assert_eq!(mock.state.calls.lock().await.len(), 1);
    assert!(pool.call(&mock.config, &search("next")).await.ok);
    assert_eq!(mock.state.initialize.load(Ordering::SeqCst), 2);
    pool.close().await;
}

#[tokio::test]
async fn session_calls_are_serial_and_timeout_does_not_retry() {
    let mock = Mock::start().await;
    *mock.state.delay.lock().await = Duration::from_millis(100);
    let pool = Arc::new(Pool::default());
    let started = Instant::now();
    let call_a = search("a");
    let call_b = search("b");
    let (a, b) = tokio::join!(
        pool.call(&mock.config, &call_a),
        pool.call(&mock.config, &call_b)
    );
    assert!(a.ok && b.ok);
    assert!(started.elapsed() >= Duration::from_millis(200));
    *mock.state.delay.lock().await = Duration::from_millis(1500);
    let mut short = mock.config.clone();
    short.operation_timeout = 1;
    let error = pool.call(&short, &search("slow")).await;
    assert_eq!(error.error.unwrap().code, "operation_timeout");
    assert_eq!(mock.state.calls.lock().await.len(), 3);
    *mock.state.delay.lock().await = Duration::ZERO;
    assert!(pool.call(&mock.config, &search("reconnect")).await.ok);
    pool.close().await;
}

async fn ready(path: &std::path::Path) {
    let mut last_error = String::new();
    for _ in 0..100 {
        match daemon::exchange_at(path, Action::Status, Duration::from_millis(100)).await {
            Ok(_) => return,
            Err(e) => last_error = format!("{e:#}"),
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("daemon not ready: {last_error}");
}

#[tokio::test]
async fn daemon_handles_large_replies_protocol_versions_and_drains_long_calls() {
    let mock = Mock::start().await;
    let dir = tempfile::tempdir().unwrap();
    let runtime = dir.path().to_owned();
    let task = tokio::spawn(async move {
        daemon::serve_at(&runtime, Duration::from_millis(200))
            .await
            .unwrap();
    });
    ready(dir.path()).await;
    let mismatch = daemon::exchange_request_at(
        dir.path(),
        eclipsectl::ipc::Request {
            version: 999,
            action: Action::Status,
        },
        Duration::from_secs(2),
    )
    .await
    .unwrap();
    assert_eq!(mismatch.error.unwrap().code, "protocol_mismatch");
    assert!(
        daemon::serve_at(dir.path(), Duration::from_secs(1))
            .await
            .is_err()
    );
    let large = daemon::exchange_at(
        dir.path(),
        Action::Call {
            config: mock.config.clone(),
            call: search("large"),
        },
        Duration::from_secs(10),
    )
    .await
    .unwrap();
    assert_eq!(
        large.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap()
            .len(),
        200_000
    );
    *mock.state.delay.lock().await = Duration::from_millis(1200);
    let slow = daemon::exchange_at(
        dir.path(),
        Action::Call {
            config: mock.config.clone(),
            call: search("slow"),
        },
        Duration::from_secs(10),
    );
    let stop = async {
        tokio::time::sleep(Duration::from_millis(400)).await;
        let response = daemon::exchange_at(dir.path(), Action::Shutdown, Duration::from_secs(2))
            .await
            .unwrap();
        assert!(response.ok);
    };
    let (response, _) = tokio::join!(slow, stop);
    assert!(response.unwrap().ok);
    task.await.unwrap();
}

#[tokio::test]
async fn idle_daemon_exits_and_can_restart_in_same_runtime() {
    let dir = tempfile::tempdir().unwrap();
    for _ in 0..2 {
        let runtime = dir.path().to_owned();
        let task = tokio::spawn(async move {
            daemon::serve_at(&runtime, Duration::from_millis(150))
                .await
                .unwrap();
        });
        ready(dir.path()).await;
        tokio::time::timeout(Duration::from_secs(3), task)
            .await
            .unwrap()
            .unwrap();
    }
}

#[tokio::test]
async fn local_overhead_p95_stays_below_100ms() {
    let mock = Mock::start().await;
    let dir = tempfile::tempdir().unwrap();
    let runtime = dir.path().to_owned();
    let task = tokio::spawn(async move {
        daemon::serve_at(&runtime, Duration::from_secs(30))
            .await
            .unwrap();
    });
    ready(dir.path()).await;
    let mut elapsed = Vec::new();
    for i in 0..31 {
        let start = Instant::now();
        let response: Response = daemon::exchange_at(
            dir.path(),
            Action::Call {
                config: mock.config.clone(),
                call: call("project list", json!({})),
            },
            Duration::from_secs(10),
        )
        .await
        .unwrap();
        assert!(response.ok, "{response:?}");
        if i > 0 {
            elapsed.push(start.elapsed());
        }
    }
    elapsed.sort();
    eprintln!("Warm local IPC + mock MCP p95: {:?}", elapsed[28]);
    assert!(
        elapsed[28] < Duration::from_millis(100),
        "p95 {:?}",
        elapsed[28]
    );
    daemon::exchange_at(dir.path(), Action::Shutdown, Duration::from_secs(2))
        .await
        .unwrap();
    task.await.unwrap();
}

#[tokio::test]
async fn java_read_gate_persists_across_calls_and_reset_is_effective() {
    let mock = Mock::start().await;
    let pool = Pool::default();
    let read = call(
        "java read",
        json!({"filePath":"src/Main.java","projectName":"example"}),
    );
    let write = call(
        "java write",
        json!({"filePath":"src/Main.java","projectName":"example","newSource":"public void added() {}"}),
    );
    assert!(!pool.call(&mock.config, &write).await.ok);
    assert!(pool.call(&mock.config, &read).await.ok);
    assert!(pool.call(&mock.config, &write).await.ok);
    assert!(
        pool.call(
            &mock.config,
            &call("java reset-read-gate", json!({"filePath":"src/Main.java"}))
        )
        .await
        .ok
    );
    assert!(!pool.call(&mock.config, &write).await.ok);
    assert_eq!(mock.state.initialize.load(Ordering::SeqCst), 1);
    pool.close().await;
}

#[tokio::test]
async fn operation_can_run_longer_than_legacy_five_second_ipc_limit() {
    let mock = Mock::start().await;
    *mock.state.delay.lock().await = Duration::from_secs(6);
    let dir = tempfile::tempdir().unwrap();
    let runtime = dir.path().to_owned();
    let task = tokio::spawn(async move {
        daemon::serve_at(&runtime, Duration::from_millis(200))
            .await
            .unwrap();
    });
    ready(dir.path()).await;
    let response = daemon::exchange_at(
        dir.path(),
        Action::Call {
            config: mock.config.clone(),
            call: search("long"),
        },
        Duration::from_secs(12),
    )
    .await
    .unwrap();
    assert!(response.ok, "{response:?}");
    daemon::exchange_at(dir.path(), Action::Shutdown, Duration::from_secs(2))
        .await
        .unwrap();
    task.await.unwrap();
}

fn state_result(text: &str) -> Value {
    json!({"content":[{"type":"text","text":text}], "_meta":{"preserved":true}})
}

#[tokio::test]
async fn debug_wait_polls_in_one_session_and_preserves_the_complete_result() {
    let mock = Mock::start().await;
    let final_result = state_result("Debug status: suspended\nSuspended at: Main.java:12");
    mock.state.replies.lock().await.insert(
        "getDebugStatus".into(),
        VecDeque::from([
            state_result("Debug status: none"),
            state_result("Debug status: running\nSession: suspended"),
            final_result.clone(),
        ]),
    );
    let pool = Pool::default();
    let response = pool
        .call(
            &mock.config,
            &call(
                "debug wait",
                json!({"suspended":true,"timeout":2,"pollInterval":50}),
            ),
        )
        .await;
    assert!(response.ok, "{response:?}");
    assert_eq!(response.result.unwrap(), final_result);
    let calls = mock.state.calls.lock().await;
    assert_eq!(calls.len(), 3);
    assert!(
        calls
            .iter()
            .all(|(server, tool, args)| server == "eclipse-debug"
                && tool == "getDebugStatus"
                && args == &json!({}))
    );
    assert_eq!(mock.state.initialize.load(Ordering::SeqCst), 1);
    drop(calls);
    pool.close().await;
}

#[tokio::test]
async fn wait_prefers_structured_state_and_keeps_per_read_timeouts() {
    let mock = Mock::start().await;
    let result = json!({"content":[{"type":"text","text":"Debug status: running"}],"structuredContent":{"state":"suspended"}});
    mock.state
        .replies
        .lock()
        .await
        .insert("getDebugStatus".into(), VecDeque::from([result.clone()]));
    let pool = Pool::default();
    let wait = call("debug wait", json!({"suspended":true,"timeout":5}));
    let response = pool.call(&mock.config, &wait).await;
    assert_eq!(response.result.unwrap(), result);
    *mock.state.delay.lock().await = Duration::from_secs(3);
    let mut config = mock.config.clone();
    config.operation_timeout = 1;
    let start = Instant::now();
    let response = pool.call(&config, &wait).await;
    assert_eq!(response.error.unwrap().code, "operation_timeout");
    assert!(start.elapsed() < Duration::from_millis(2500));
    pool.close().await;
}

#[tokio::test]
async fn workspace_wait_requires_continuous_quiet_and_build_wait_ignores_other_jobs() {
    let mock = Mock::start().await;
    mock.state.replies.lock().await.insert(
        "getWorkspaceState".into(),
        VecDeque::from([
            state_result("Workspace status: busy\nBuild status: busy"),
            state_result("Workspace status: idle\nBuild status: idle"),
            state_result("Workspace status: busy\nBuild status: busy"),
            state_result("Workspace status: idle\nBuild status: idle"),
        ]),
    );
    let pool = Pool::default();
    let start = Instant::now();
    let response = pool
        .call(
            &mock.config,
            &call(
                "workspace wait",
                json!({"idle":true,"timeout":3,"pollInterval":100}),
            ),
        )
        .await;
    assert!(response.ok, "{response:?}");
    assert!(start.elapsed() >= Duration::from_millis(800));
    mock.state.replies.lock().await.insert(
        "getWorkspaceState".into(),
        VecDeque::from([state_result("Workspace status: busy\nBuild status: idle")]),
    );
    let response = pool
        .call(
            &mock.config,
            &call("build wait", json!({"timeout":2,"pollInterval":50})),
        )
        .await;
    assert!(response.ok, "{response:?}");
    assert!(
        mock.state
            .calls
            .lock()
            .await
            .iter()
            .all(|(server, tool, args)| server == "eclipse-ide"
                && tool == "getWorkspaceState"
                && args == &json!({}))
    );
    pool.close().await;
}

#[tokio::test]
async fn wait_timeout_bounds_pending_states_and_slow_mcp_reads() {
    let mock = Mock::start().await;
    mock.state.replies.lock().await.insert(
        "getDebugStatus".into(),
        VecDeque::from([state_result("Debug status: running")]),
    );
    let pool = Pool::default();
    let wait = call(
        "debug wait",
        json!({"suspended":true,"timeout":1,"pollInterval":50}),
    );
    let response = pool.call(&mock.config, &wait).await;
    assert_eq!(response.error.as_ref().unwrap().code, "wait_timeout");
    assert_eq!(
        response.error.unwrap().details.unwrap()["content"][0]["text"],
        "Debug status: running"
    );
    let count = mock.state.calls.lock().await.len();
    *mock.state.delay.lock().await = Duration::from_secs(3);
    let start = Instant::now();
    let response = pool.call(&mock.config, &wait).await;
    assert_eq!(response.error.unwrap().code, "wait_timeout");
    assert!(start.elapsed() < Duration::from_millis(2500));
    assert_eq!(mock.state.calls.lock().await.len(), count + 1);
    pool.close().await;
}

#[tokio::test]
async fn wait_returns_tool_errors_and_unknown_states_without_retrying() {
    let mock = Mock::start().await;
    let pool = Pool::default();
    for (result, expected) in [
        (
            json!({"content":[{"type":"text","text":"not available"}],"isError":true}),
            "mcp_tool_error",
        ),
        (
            state_result("Session: suspended\nUnsupported debug format"),
            "wait_state_unavailable",
        ),
        (
            state_result("Debug status: terminated"),
            "wait_state_unavailable",
        ),
    ] {
        mock.state
            .replies
            .lock()
            .await
            .insert("getDebugStatus".into(), VecDeque::from([result]));
        let count = mock.state.calls.lock().await.len();
        let response = pool
            .call(&mock.config, &call("debug wait", json!({"suspended":true})))
            .await;
        assert_eq!(response.error.unwrap().code, expected);
        assert_eq!(mock.state.calls.lock().await.len(), count + 1);
    }
    mock.state.replies.lock().await.insert(
        "getWorkspaceState".into(),
        VecDeque::from([state_result("# Workspace Information")]),
    );
    let response = pool
        .call(&mock.config, &call("build wait", json!({})))
        .await;
    assert_eq!(response.error.unwrap().code, "wait_state_unavailable");
    pool.close().await;
}

#[tokio::test]
async fn debug_wait_keeps_the_session_locked_until_suspension() {
    let mock = Mock::start().await;
    *mock.state.delay.lock().await = Duration::from_millis(100);
    mock.state.replies.lock().await.insert(
        "getDebugStatus".into(),
        VecDeque::from([
            state_result("Debug status: running"),
            state_result("Debug status: suspended"),
        ]),
    );
    let pool = Arc::new(Pool::default());
    let waiting = {
        let pool = pool.clone();
        let config = mock.config.clone();
        tokio::spawn(async move {
            pool.call(
                &config,
                &call("debug wait", json!({"suspended":true,"pollInterval":50})),
            )
            .await
        })
    };
    tokio::time::timeout(Duration::from_secs(2), async {
        while mock.state.calls.lock().await.is_empty() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let variables = pool
        .call(&mock.config, &call("debug variables", json!({})))
        .await;
    assert!(waiting.await.unwrap().ok);
    assert!(variables.ok);
    assert_eq!(
        mock.state
            .calls
            .lock()
            .await
            .iter()
            .map(|(_, tool, _)| tool.as_str())
            .collect::<Vec<_>>(),
        ["getDebugStatus", "getDebugStatus", "getVariables"]
    );
    pool.close().await;
}
