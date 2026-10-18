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
    collections::HashSet,
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
