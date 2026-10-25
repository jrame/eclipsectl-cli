use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use eclipsectl::{cli, config::Config, mcp::Pool, operations};
use serde_json::{Value, json};
use std::{path::Path, process::Command, sync::Arc};
use tokio::sync::Mutex;

fn call(args: &[&str]) -> anyhow::Result<operations::Call> {
    let mut argv = vec!["eclipsectl", "refactor"];
    argv.extend_from_slice(args);
    let matches = cli::command().try_get_matches_from(argv)?;
    let (name, args) = cli::selected(&matches);
    operations::find(&name)?.call(args, Path::new(r"B:\work"))
}

const REFACTORINGS: &[(&str, &[&str])] = &[
    ("rename-method", &["Example.java", "0", "calculate"]),
    ("rename-field", &["Example.java", "0", "total"]),
    ("rename-variable", &["Example.java", "0", "value"]),
    (
        "change-signature",
        &["Example.java", "0", "--name", "calculate"],
    ),
    ("extract-variable", &["Example.java", "0", "2", "value"]),
    ("extract-constant", &["Example.java", "0", "2", "VALUE"]),
    ("inline-method", &["Example.java", "0"]),
    ("inline-variable", &["Example.java", "0"]),
];

#[test]
fn preview_is_optional_and_remains_a_native_boolean_for_all_eight_refactorings() {
    for (name, operands) in REFACTORINGS {
        let mut args = vec![*name];
        args.extend_from_slice(operands);
        assert!(!call(&args).unwrap().arguments.contains_key("preview"));
        for enabled in ["true", "false"] {
            let mut explicit = args.clone();
            explicit.extend(["--preview", enabled]);
            let request = call(&explicit).unwrap();
            let op = operations::find(&request.operation).unwrap();
            assert_eq!(op.server, "eclipse-refactor");
            let wire = op.wire_arguments(&request.arguments).unwrap();
            assert_eq!(wire["preview"], enabled == "true");
            assert_eq!(
                wire["offset"],
                if *name == "rename-variable" {
                    json!(0)
                } else {
                    json!("0")
                }
            );
        }
        for suffix in [vec!["--preview"], vec!["--preview", "yes"]] {
            let mut invalid = args.clone();
            invalid.extend(suffix);
            assert!(call(&invalid).is_err());
        }
        let mut raw = call(&args).unwrap().arguments;
        raw.insert("preview".into(), json!("true"));
        assert!(
            operations::find(&format!("refactor {name}"))
                .unwrap()
                .validate(&raw)
                .is_err()
        );
    }
}

#[test]
fn report_offsets_and_ids_follow_the_plugin_schema() {
    for (name, tool) in [
        ("get-preview", "refactorGetPreview"),
        ("apply-preview", "refactorApplyPreview"),
    ] {
        let request = call(&[name, "/mnt/b/opaque-id"]).unwrap();
        let op = operations::find(&request.operation).unwrap();
        assert_eq!(op.tool, tool);
        assert_eq!(op.server, "eclipse-refactor");
        assert_eq!(
            Value::Object(op.wire_arguments(&request.arguments).unwrap()),
            json!({"previewId":"/mnt/b/opaque-id"})
        );
        assert!(call(&[name]).is_err());
        assert!(
            op.validate(json!({"previewId":7}).as_object().unwrap())
                .is_err()
        );
        assert!(op.validate(json!({}).as_object().unwrap()).is_err());
    }
    for offset in ["0", "99999", "2147483647"] {
        let request = call(&["get-preview", "id", "--offset", offset]).unwrap();
        assert_eq!(
            operations::find(&request.operation)
                .unwrap()
                .wire_arguments(&request.arguments)
                .unwrap()["offset"],
            offset
        );
    }
    for offset in ["-1", "2147483648", "1.5", "x"] {
        assert!(call(&["get-preview", "id", "--offset", offset]).is_err());
    }
    let op = operations::find("refactor get-preview").unwrap();
    for offset in [json!(-1), json!("0"), json!(2147483648u64)] {
        assert!(
            op.validate(
                json!({"previewId":"id","offset":offset})
                    .as_object()
                    .unwrap()
            )
            .is_err()
        );
    }
    assert!(call(&["apply-preview", "id", "--offset", "0"]).is_err());
}

#[test]
fn preview_help_is_available_without_starting_a_daemon() {
    let temp = tempfile::tempdir().unwrap();
    for name in REFACTORINGS
        .iter()
        .map(|(name, _)| *name)
        .chain(["get-preview", "apply-preview"])
    {
        let output = Command::new(env!("CARGO_BIN_EXE_eclipsectl"))
            .args(["refactor", name, "--help"])
            .env("ECLIPSECTL_RUNTIME_DIR", temp.path())
            .env("ECLIPSE_URL", "invalid-url")
            .output()
            .unwrap();
        assert!(output.status.success(), "{name}: {:?}", output.stderr);
        let help = String::from_utf8(output.stdout).unwrap();
        if name.ends_with("-preview") {
            assert!(help.contains("<preview-id>"));
        } else {
            assert!(help.contains("--preview <preview>"));
            assert!(help.contains("true, false"));
        }
    }
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 0);
}

#[derive(Default)]
struct MockState {
    requests: Mutex<Vec<(HeaderMap, Value)>>,
}

fn tool_result(tool: &str, arguments: &Value) -> Value {
    if tool == "refactorApplyPreview" && arguments["previewId"] != "ok" {
        return json!({"isError":true,"content":[{"type":"text","text":format!("Refused: {}", arguments["previewId"])}],"structuredContent":{"unchanged":true}});
    }
    json!({"content":[{"type":"text","text":"Full diff\n-é\n+é😀\n[End of preview.]"}],"structuredContent":{"previewId":"ok","arguments":arguments},"isError":false})
}

async fn handler(
    State(state): State<Arc<MockState>>,
    headers: HeaderMap,
    Json(request): Json<Value>,
) -> Response {
    state.requests.lock().await.push((headers, request.clone()));
    let id = &request["id"];
    let result = match request["method"].as_str().unwrap() {
        "initialize" => return ([("mcp-session-id", "preview-session")], Json(json!({"jsonrpc":"2.0","id":id,"result":{"protocolVersion":"2025-03-26","capabilities":{"tools":{}},"serverInfo":{"name":"preview-mock","version":"1"}}}))).into_response(),
        "notifications/initialized" | "notifications/cancelled" => return StatusCode::ACCEPTED.into_response(),
        "tools/call" => {
            if request["params"]["arguments"]["previewId"] == "expired-session" {
                return StatusCode::NOT_FOUND.into_response();
            }
            tool_result(request["params"]["name"].as_str().unwrap(), &request["params"]["arguments"])
        }
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };
    Json(json!({"jsonrpc":"2.0","id":id,"result":result})).into_response()
}

#[tokio::test]
async fn preview_cycle_preserves_results_session_wire_types_and_errors_without_replay() {
    let state = Arc::new(MockState::default());
    // Only the intended server route exists: other routing fails this test.
    let app = Router::new()
        .route("/mcp/eclipse-refactor", post(handler))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let config = Config {
        eclipse_url: format!("http://{}", listener.local_addr().unwrap()),
        bearer_token: Some("mock-token".into()),
        connect_timeout: 3,
        operation_timeout: 5,
    };
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let pool = Pool::default();
    let mut expected_calls = 0;
    for (name, operands) in REFACTORINGS {
        let mut args = vec![*name];
        args.extend_from_slice(operands);
        args.extend(["--preview", "true"]);
        let request = call(&args).unwrap();
        let op = operations::find(&request.operation).unwrap();
        let wire = Value::Object(op.wire_arguments(&request.arguments).unwrap());
        let response = pool.call(&config, &request).await;
        assert!(response.ok);
        assert_eq!(response.result.unwrap(), tool_result(&op.tool, &wire));
        expected_calls += 1;
    }
    for args in [
        vec![
            "rename-variable",
            "FooTest",
            "str",
            "strSuper",
            "--line",
            "8",
            "--preview",
            "true",
        ],
        vec![
            "rename-variable",
            "FooTest",
            "str",
            "strSuper",
            "--method",
            "testFoo",
            "--line",
            "8",
        ],
        vec!["get-preview", "ok"],
        vec!["get-preview", "ok", "--offset", "99999"],
        vec!["apply-preview", "ok"],
    ] {
        let request = call(&args).unwrap();
        let op = operations::find(&request.operation).unwrap();
        let wire = Value::Object(op.wire_arguments(&request.arguments).unwrap());
        let response = pool.call(&config, &request).await;
        assert!(response.ok);
        assert_eq!(response.result.unwrap(), tool_result(&op.tool, &wire));
        expected_calls += 1;
    }
    for id in [
        "incomplete",
        "changed-source",
        "unsaved-source",
        "expired",
        "already-used",
    ] {
        let response = pool
            .call(&config, &call(&["apply-preview", id]).unwrap())
            .await;
        assert!(!response.ok);
        let error = response.error.unwrap();
        assert_eq!(error.code, "mcp_tool_error");
        assert_eq!(
            error.details.unwrap(),
            tool_result("refactorApplyPreview", &json!({"previewId":id}))
        );
        expected_calls += 1;
    }
    let requests = state.requests.lock().await;
    assert_eq!(
        requests
            .iter()
            .filter(|(_, r)| r["method"] == "initialize")
            .count(),
        1
    );
    let calls: Vec<_> = requests
        .iter()
        .filter(|(_, r)| r["method"] == "tools/call")
        .collect();
    assert_eq!(calls.len(), expected_calls);
    assert!(
        calls
            .iter()
            .all(|(h, _)| h.get("mcp-session-id").unwrap() == "preview-session")
    );
    drop(requests);
    let response = pool
        .call(
            &config,
            &call(&["apply-preview", "expired-session"]).unwrap(),
        )
        .await;
    assert!(!response.ok);
    assert_eq!(response.error.unwrap().code, "mcp_failed");
    let requests = state.requests.lock().await;
    assert_eq!(
        requests
            .iter()
            .filter(|(_, r)| r["method"] == "tools/call")
            .count(),
        expected_calls + 1
    );
    assert_eq!(
        requests
            .iter()
            .filter(|(_, r)| r["method"] == "initialize")
            .count(),
        1
    );
    drop(requests);
    pool.close().await;
    task.abort();
}
