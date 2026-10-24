//! Opt-in, read-only checks against an actual Eclipse plugin.
//! Set ECLIPSE_URL/ECLIPSE_TOKEN and run cargo test --test live -- --ignored.
use eclipsectl::{
    config::{self, Settings},
    mcp::Pool,
    operations::{self, Call},
};
use rmcp::{
    ServiceExt,
    transport::{
        StreamableHttpClientTransport, streamable_http_client::StreamableHttpClientTransportConfig,
    },
};
use serde_json::{Map, json};
use std::collections::BTreeSet;

#[tokio::test]
#[ignore = "requires a running Eclipse plugin and credentials"]
async fn plugin_schemas_cover_every_registered_operation() {
    let config = config::load(&Settings::default(), &std::env::current_dir().unwrap()).unwrap();
    let servers: BTreeSet<_> = operations::inventory()
        .iter()
        .map(|op| op.server.as_str())
        .collect();
    let mut problems = Vec::new();
    for server in servers {
        let mut transport = StreamableHttpClientTransportConfig::with_uri(format!(
            "{}/mcp/{server}",
            config.eclipse_url
        ));
        if let Some(token) = &config.bearer_token {
            transport = transport.auth_header(token);
        }
        transport.reinit_on_expired_session = false;
        let http = reqwest::Client::builder()
            .connect_timeout(config.connect_duration())
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        let session = tokio::time::timeout(
            config.connect_duration(),
            ().serve(StreamableHttpClientTransport::with_client(http, transport)),
        )
        .await
        .expect("initialization timeout")
        .expect("initialization failed");
        let tools = tokio::time::timeout(config.operation_duration(), session.list_all_tools())
            .await
            .unwrap()
            .unwrap();
        for op in operations::inventory()
            .iter()
            .filter(|op| op.server == server)
        {
            let Some(tool) = tools.iter().find(|tool| tool.name == op.tool) else {
                problems.push(format!("{}: missing tool {}", op.command, op.tool));
                continue;
            };
            let properties = tool
                .input_schema
                .get("properties")
                .and_then(|v| v.as_object());
            for arg in &op.args {
                if !properties.is_some_and(|props| props.contains_key(&arg.key)) {
                    problems.push(format!(
                        "{}: unknown wire argument {}; plugin schema: {:?}",
                        op.command, arg.key, tool.input_schema
                    ));
                }
                if let Some(property) = properties.and_then(|props| props.get(&arg.key)) {
                    let expected = if arg.wire.is_some() {
                        "string"
                    } else {
                        match arg.kind.as_str() {
                            "integer" => "integer",
                            "boolean" | "flag" => "boolean",
                            "env" => "object",
                            "strings" if arg.key == "path" || arg.key == "paths" => "string",
                            "strings" => "array",
                            _ => "string",
                        }
                    };
                    // Older plugins advertise a string for this Java Boolean parameter.
                    // Reflection accepts only JSON booleans; the read-only launch test
                    // below checks the actual invocation instead of trusting this schema.
                    let legacy_wait_schema = arg.key == "waitForCompletion"
                        && matches!(op.tool.as_str(), "run" | "runDebug" | "runCoverage")
                        && expected == "boolean";
                    if let Some(actual) = property.get("type").and_then(|v| v.as_str())
                        && actual != expected
                        && !(legacy_wait_schema && actual == "string")
                    {
                        problems.push(format!(
                            "{} / {}: registry type {expected}, plugin type {actual}, schema {:?}",
                            op.command, arg.key, property
                        ));
                    }
                }
            }
        }
        let _ = session.cancel().await;
    }
    assert!(
        problems.is_empty(),
        "Live schema incompatibilities:\n{}",
        problems.join("\n")
    );
}

#[tokio::test]
#[ignore = "requires a running Eclipse plugin and credentials"]
async fn live_read_only_smoke() {
    let config = config::load(&Settings::default(), &std::env::current_dir().unwrap()).unwrap();
    let pool = Pool::default();
    let health = pool.doctor(&config).await;
    assert!(health.ok, "{health:?}");
    for operation in [
        "project list",
        "workspace info",
        "launch list",
        "debug status",
    ] {
        let response = pool
            .call(
                &config,
                &Call {
                    operation: operation.into(),
                    arguments: Map::new(),
                },
            )
            .await;
        assert!(response.ok, "{operation}: {response:?}");
    }
    let response = pool
        .call(
            &config,
            &Call {
                operation: "search text".into(),
                arguments: json!({"containingText":"eclipsectl-rust-read-only-smoke-no-match","fileNamePatterns":["*.java","*.xml"]})
                    .as_object()
                    .unwrap()
                    .clone(),
            },
        )
        .await;
    assert!(response.ok, "{response:?}");
    pool.close().await;
}

#[tokio::test]
#[ignore = "requires a running Eclipse plugin and credentials"]
async fn launch_wait_is_accepted_when_listing_configurations() {
    let config = config::load(&Settings::default(), &std::env::current_dir().unwrap()).unwrap();
    let pool = Pool::default();
    // No configuration means list only; this never starts a launch.
    let matches = eclipsectl::cli::command()
        .try_get_matches_from(["eclipsectl", "run", "--wait"])
        .unwrap();
    let (_, args) = eclipsectl::cli::selected(&matches);
    let call = operations::find("run")
        .unwrap()
        .call(args, &std::env::current_dir().unwrap())
        .unwrap();
    let response = pool.call(&config, &call).await;
    pool.close().await;
    assert!(response.ok, "{response:?}");
}
