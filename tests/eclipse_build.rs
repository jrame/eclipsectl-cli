use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use eclipsectl::{cli, config::Config, mcp::Pool, operations};
use serde_json::{Value, json};
use std::{path::Path, sync::Arc, time::Duration};
use tokio::sync::Mutex;

fn parse(options: &[&str]) -> anyhow::Result<operations::Call> {
    let matches = cli::command().try_get_matches_from(
        ["eclipsectl", "build", "eclipse"]
            .into_iter()
            .chain(options.iter().copied()),
    )?;
    let (name, args) = cli::selected(&matches);
    operations::find(&name)?.call(args, Path::new(r"C:\work"))
}

#[test]
fn build_contract_preserves_defaults_and_native_types() {
    let op = operations::find("build eclipse").unwrap();
    let defaults = parse(&[]).unwrap();
    assert!(defaults.arguments.is_empty());
    assert!(op.wire_arguments(&defaults.arguments).unwrap().is_empty());
    assert!(op.wait_for.is_none());
    assert!(operations::find("build wait").unwrap().wait_for.is_some());
    let call = parse(&[
        "--project",
        "Application",
        "--project",
        "Shared library",
        "--build-kind",
        "full",
        "--include-dependents",
        "false",
        "--timeout",
        "3600",
        "--max-problems",
        "500",
    ])
    .unwrap();
    assert_eq!(
        op.wire_arguments(&call.arguments).unwrap(),
        json!({
            "projectNames":"Application,Shared library", "buildKind":"full",
            "includeDependents":false, "timeoutSeconds":3600, "maxProblems":500,
        })
        .as_object()
        .unwrap()
        .clone()
    );
    assert_eq!(op.operation_timeout(&call.arguments, 300), 3610);
    assert_eq!(op.operation_timeout(&defaults.arguments, 1), 130);
    assert_eq!(op.operation_timeout(&defaults.arguments, 900), 900);
    assert_eq!(
        operations::find("build wait")
            .unwrap()
            .operation_timeout(&defaults.arguments, 1),
        1
    );
    let call = parse(&[
        "--build-kind",
        "incremental",
        "--include-dependents",
        "true",
        "--timeout",
        "1",
        "--max-problems",
        "1",
    ])
    .unwrap();
    assert_eq!(call.arguments["timeoutSeconds"], 1);
    assert_eq!(call.arguments["maxProblems"], 1);
    assert_eq!(call.arguments["includeDependents"], true);
}

#[test]
fn build_rejects_ambiguous_names_ranges_choices_and_wire_types() {
    for options in [
        vec!["--project", "a,b"],
        vec!["--project", ""],
        vec!["--project", "  "],
        vec!["--timeout", "0"],
        vec!["--timeout", "3601"],
        vec!["--timeout", "1.5"],
        vec!["--max-problems", "0"],
        vec!["--max-problems", "501"],
        vec!["--build-kind", "clean"],
        vec!["--include-dependents", "yes"],
    ] {
        assert!(parse(&options).is_err(), "{options:?}");
    }
    let op = operations::find("build eclipse").unwrap();
    for args in [
        json!({"projectNames":["a,b"]}),
        json!({"projectNames":[]}),
        json!({"projectNames":[" "]}),
        json!({"projectNames":"a,b"}),
        json!({"includeDependents":"false"}),
        json!({"timeoutSeconds":"120"}),
        json!({"timeoutSeconds":3601}),
        json!({"maxProblems":501}),
        json!({"maxProblems":0}),
        json!({"buildKind":"clean"}),
    ] {
        assert!(op.validate(args.as_object().unwrap()).is_err(), "{args}");
    }
}

#[derive(Clone)]
struct MockState {
    requests: Arc<Mutex<Vec<Value>>>,
    result: Value,
    delay: Duration,
}

async fn handler(
    State(state): State<MockState>,
    Json(request): Json<Value>,
) -> axum::response::Response {
    let id = &request["id"];
    let result = match request["method"].as_str().unwrap() {
        "initialize" => {
            json!({"protocolVersion":"2025-03-26", "capabilities":{"tools":{}}, "serverInfo":{"name":"build-mock","version":"1"}})
        }
        "notifications/initialized" | "notifications/cancelled" => {
            return StatusCode::ACCEPTED.into_response();
        }
        "tools/call" => {
            state.requests.lock().await.push(request["params"].clone());
            tokio::time::sleep(state.delay).await;
            state.result.clone()
        }
        _ => panic!("Unexpected MCP request"),
    };
    Json(json!({"jsonrpc":"2.0", "id":id, "result":result})).into_response()
}

#[tokio::test]
async fn build_routes_defaults_native_values_and_preserves_success_and_errors_without_replay() {
    for (options, result, delay) in [
        (
            vec![],
            json!({"content":[{"type":"text","text":"No open projects to build."}]}),
            Duration::ZERO,
        ),
        (
            vec![
                "--project",
                "app",
                "--project",
                "lib",
                "--build-kind",
                "full",
                "--include-dependents",
                "false",
                "--timeout",
                "1",
                "--max-problems",
                "10",
            ],
            json!({"content":[{"type":"text","text":"Build completed: SUCCESS; 0 errors, 2 warnings"}]}),
            Duration::from_millis(1200),
        ),
        (
            vec![],
            json!({"content":[{"type":"text","text":"Build completed: FAILED; app: 1 errors, 0 warnings; Main.java:12"}],"isError":true}),
            Duration::ZERO,
        ),
        (
            vec!["--timeout", "1"],
            json!({"content":[{"type":"text","text":"Cancellation requested; compilation result is incomplete."}],"isError":true}),
            Duration::ZERO,
        ),
    ] {
        let state = MockState {
            requests: Arc::default(),
            result: result.clone(),
            delay,
        };
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let config = Config {
            eclipse_url: format!("http://{}", listener.local_addr().unwrap()),
            bearer_token: None,
            connect_timeout: 3,
            operation_timeout: 1,
        };
        let app = Router::new()
            .route("/mcp/eclipse-ide", post(handler))
            .with_state(state.clone());
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let call = parse(&options).unwrap();
        let response = Pool::default().call(&config, &call).await;
        let requests = state.requests.lock().await;
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0]["name"], "buildProjects");
        assert_eq!(
            requests[0]["arguments"],
            Value::Object(
                operations::find("build eclipse")
                    .unwrap()
                    .wire_arguments(&call.arguments)
                    .unwrap()
            )
        );
        if result["isError"] == true {
            assert!(!response.ok);
            let error = response.error.unwrap();
            assert_eq!(error.code, "mcp_tool_error");
            assert_eq!(error.details.unwrap(), result);
        } else {
            assert!(response.ok, "{response:?}");
            assert_eq!(response.result.unwrap(), result);
        }
        server.abort();
    }
}

#[tokio::test]
async fn build_binary_returns_json_exit_codes_and_help_without_eclipse() {
    for result in [
        json!({"content":[{"type":"text","text":"Build completed: SUCCESS"}]}),
        json!({"content":[{"type":"text","text":"Build completed: FAILED; Main.java:12"}],"isError":true}),
        json!({"content":[{"type":"text","text":"Build timed out; result is incomplete"}],"isError":true}),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let state = MockState {
            requests: Arc::default(),
            result: result.clone(),
            delay: Duration::ZERO,
        };
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let app = Router::new()
            .route("/mcp/eclipse-ide", post(handler))
            .with_state(state.clone());
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let output = tokio::task::spawn_blocking(move || {
            let run = |args: &[&str]| {
                std::process::Command::new(env!("CARGO_BIN_EXE_eclipsectl"))
                    .current_dir(temp.path())
                    .env("ECLIPSECTL_RUNTIME_DIR", temp.path().join("runtime"))
                    .args([
                        "--eclipse-url",
                        &url,
                        "--bearer-token",
                        "mock",
                        "--connect-timeout",
                        "3",
                        "--operation-timeout",
                        "1",
                    ])
                    .args(args)
                    .output()
                    .unwrap()
            };
            let help = run(&["build", "eclipse", "--help"]);
            assert!(help.status.success());
            let help = String::from_utf8(help.stdout).unwrap();
            for option in [
                "--project",
                "--build-kind",
                "--include-dependents",
                "--timeout",
                "--max-problems",
            ] {
                assert!(help.contains(option));
            }
            for args in [vec!["--project", "a,b"], vec!["--timeout", "3601"]] {
                let mut argv = vec!["build", "eclipse", "--json"];
                argv.extend(args);
                let invalid = run(&argv);
                assert_eq!(invalid.status.code(), Some(2));
                let envelope: Value = serde_json::from_slice(&invalid.stdout).unwrap();
                assert_eq!(envelope["error"]["code"], "invalid_arguments");
            }
            // Help and invalid arguments must not have started a daemon.
            assert!(!temp.path().join("runtime").exists());
            let output = run(&["build", "eclipse", "--project", "app", "--json"]);
            let stopped = run(&["daemon", "stop"]);
            assert!(stopped.status.success());
            output
        })
        .await
        .unwrap();
        let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
        if result["isError"] == true {
            assert_eq!(output.status.code(), Some(1));
            assert_eq!(envelope["ok"], false);
            assert_eq!(envelope["error"]["code"], "mcp_tool_error");
            assert_eq!(envelope["error"]["details"], result);
        } else {
            assert_eq!(output.status.code(), Some(0), "{envelope}");
            assert_eq!(envelope["ok"], true);
            assert_eq!(envelope["result"], result);
        }
        assert_eq!(state.requests.lock().await.len(), 1);
        server.abort();
    }
}
