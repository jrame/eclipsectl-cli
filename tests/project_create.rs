use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use eclipsectl::{cli, config::Config, mcp::Pool, operations};
use serde_json::{Value, json};
use std::{path::Path, sync::Arc};
use tokio::sync::Mutex;

fn parse(args: &[&str]) -> anyhow::Result<operations::Call> {
    let matches = cli::command().try_get_matches_from(
        ["eclipsectl", "project", "create"]
            .into_iter()
            .chain(args.iter().copied()),
    )?;
    let (name, args) = cli::selected(&matches);
    operations::find(&name)?.call(args, Path::new(r"B:\work"))
}

#[test]
fn java_creation_defaults_paths_and_native_types() {
    let op = operations::find("project create").unwrap();
    let call = parse(&[r"B:\new app", "--java"]).unwrap();
    assert_eq!(
        op.wire_arguments(&call.arguments).unwrap(),
        json!({"projectPath":r"B:\new app"})
            .as_object()
            .unwrap()
            .clone()
    );
    let call = parse(&[
        "B:/new app",
        "--java",
        "--project",
        "Example",
        "--java-version",
        "21",
        "--junit",
        "5",
    ])
    .unwrap();
    assert_eq!(op.wire_arguments(&call.arguments).unwrap(),
        json!({"projectPath":r"B:\new app", "projectName":"Example", "javaVersion":21, "junitVersion":"5"}).as_object().unwrap().clone());
    let import = operations::find("project import").unwrap();
    assert_eq!(import.tool, "importProject");
    assert_eq!(import.args.len(), 2);
}

#[test]
fn creation_rejects_missing_kind_unsupported_options_and_invalid_paths() {
    for args in [
        vec![],
        vec![r"B:\app"],
        vec!["--java"],
        vec![r"B:\app", "--java", "--junit", "4"],
        vec![r"B:\app", "--java", "--java-version", "7"],
        vec![r"B:\app", "--java", "--java-version", "21.0"],
        vec![r"B:\app", "--java", "--java-version", "2147483648"],
        vec![r"B:\app", "--java", "--force"],
        vec![r"\\wsl.localhost\Ubuntu\home\app", "--java"],
    ] {
        assert!(parse(&args).is_err(), "{args:?}");
    }
    let op = operations::find("project create").unwrap();
    for args in [
        json!({"projectPath":"B:/app"}),
        json!({"projectPath":"B:/app", "java":false}),
        json!({"projectPath":"B:/app", "java":true, "javaVersion":"21"}),
        json!({"projectPath":"B:/app", "java":true, "junitVersion":5}),
    ] {
        assert!(op.wire_arguments(args.as_object().unwrap()).is_err());
    }
}

#[derive(Clone)]
struct Mock {
    calls: Arc<Mutex<Vec<Value>>>,
    result: Value,
}
async fn handler(
    State(state): State<Mock>,
    Json(request): Json<Value>,
) -> axum::response::Response {
    let result = match request["method"].as_str().unwrap() {
        "initialize" => {
            json!({"protocolVersion":"2025-03-26", "capabilities":{"tools":{}}, "serverInfo":{"name":"create-mock", "version":"1"}})
        }
        "notifications/initialized" => return StatusCode::ACCEPTED.into_response(),
        "tools/call" => {
            state.calls.lock().await.push(request["params"].clone());
            state.result.clone()
        }
        _ => panic!("Unexpected request"),
    };
    Json(json!({"jsonrpc":"2.0", "id":request["id"], "result":result})).into_response()
}

#[tokio::test]
async fn creation_transport_preserves_errors_partial_results_and_does_not_replay() {
    for result in [
        json!({"content":[{"type":"text", "text":"Created and opened Java project 'Example'."}]}),
        json!({"isError":true, "content":[{"type":"text", "text":"Target already exists (nothing overwritten)."}]}),
        json!({"isError":true, "content":[{"type":"text", "text":"Partial creation at B:\\app; files retained for inspection."}]}),
    ] {
        let state = Mock {
            calls: Arc::default(),
            result: result.clone(),
        };
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let config = Config {
            eclipse_url: format!("http://{}", listener.local_addr().unwrap()),
            bearer_token: None,
            connect_timeout: 3,
            operation_timeout: 10,
        };
        let app = Router::new()
            .route("/mcp/eclipse-ide", post(handler))
            .with_state(state.clone());
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let call = parse(&[r"B:\app", "--java", "--java-version", "21", "--junit", "5"]).unwrap();
        let response = Pool::default().call(&config, &call).await;
        let calls = state.calls.lock().await;
        assert_eq!(calls.len(), 1);
        assert_eq!(
            json!({"name":calls[0]["name"], "arguments":calls[0]["arguments"]}),
            json!({"name":"createJavaProject", "arguments":{"projectPath":r"B:\app", "javaVersion":21, "junitVersion":"5"}})
        );
        if result["isError"] == true {
            assert!(!response.ok);
            let error = response.error.unwrap();
            assert_eq!(error.code, "mcp_tool_error");
            assert_eq!(error.details.unwrap(), result);
        } else {
            assert!(response.ok);
            assert_eq!(response.result.unwrap(), result);
        }
        server.abort();
    }
}

#[test]
fn help_and_invalid_arguments_keep_json_exit_contract_without_starting_daemon() {
    let temp = tempfile::tempdir().unwrap();
    let run = |args: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_eclipsectl"))
            .current_dir(temp.path())
            .env("ECLIPSECTL_RUNTIME_DIR", temp.path().join("run"))
            .args(args)
            .output()
            .unwrap()
    };
    let help = run(&["project", "create", "--help"]);
    assert!(help.status.success());
    let text = String::from_utf8(help.stdout).unwrap();
    for option in ["--java", "--project", "--java-version", "--junit"] {
        assert!(text.contains(option));
    }
    let invalid = run(&["project", "create", r"B:\app", "--json"]);
    assert_eq!(invalid.status.code(), Some(2));
    let value: Value = serde_json::from_slice(&invalid.stdout).unwrap();
    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], "invalid_arguments");
    assert!(!temp.path().join("run").exists());
}
