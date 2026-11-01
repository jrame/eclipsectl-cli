//! Snapshot exported from Java declarations by the plugin's GenerateToolCatalog.java.
//! Refresh from docs/mcp-tool-contract.json in eclipsectl-mcp-plugin after API changes.
use eclipsectl::operations::{self, Argument};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::BTreeSet;

#[derive(Deserialize)]
struct Tool {
    server: String,
    tool: String,
    params: Vec<Param>,
}

#[derive(Deserialize)]
struct Param {
    name: String,
    #[serde(rename = "type")]
    schema_type: String,
    #[serde(rename = "javaType")]
    java_type: String,
    required: bool,
}

fn sample(arg: &Argument) -> Value {
    match arg.kind.as_str() {
        "integer" => json!(arg.min.unwrap_or(1)),
        "boolean" | "flag" => json!(false),
        "switch" => json!(true),
        "env" => json!({"EXAMPLE":"value"}),
        "strings" if matches!(arg.key.as_str(), "path" | "paths") => json!("example"),
        "strings" => json!(["example"]),
        _ if arg.key == "parameters" => json!("[]"),
        _ => json!(arg.choices.first().map(String::as_str).unwrap_or("example")),
    }
}

fn json_type(value: &Value) -> &str {
    match value {
        Value::Bool(_) => "boolean",
        Value::Number(_) => "integer",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
        Value::Null => "null",
    }
}

#[test]
fn every_wire_argument_matches_the_java_backed_plugin_contract() {
    let tools: Vec<Tool> = serde_json::from_str(include_str!("plugin-tool-contract.json")).unwrap();
    let mut covered = BTreeSet::new();
    for op in operations::inventory() {
        let tool = tools
            .iter()
            .find(|tool| tool.server == op.server && tool.tool == op.tool)
            .unwrap_or_else(|| panic!("{} missing from plugin contract", op.command));
        covered.insert((&tool.server, &tool.tool));
        let mut args: Map<String, Value> = op
            .args
            .iter()
            .map(|arg| (arg.key.clone(), sample(arg)))
            .collect();
        // Exercise both mutually exclusive variable selector forms.
        let mut variants = Vec::new();
        if op.tool == "refactorRenameVariable" {
            let mut named = args.clone();
            named.remove("offset");
            variants.push(named);
            for key in ["oldName", "lineNumber", "methodName"] {
                args.remove(key);
            }
        }
        variants.push(args);
        for args in variants {
            let wire = op
                .wire_arguments(&args)
                .unwrap_or_else(|error| panic!("{}: {error}", op.command));
            for (name, value) in &wire {
                let param = tool
                    .params
                    .iter()
                    .find(|param| &param.name == name)
                    .unwrap_or_else(|| panic!("{}: unknown plugin argument {name}", op.command));
                assert_eq!(
                    json_type(value),
                    param.schema_type,
                    "{} / {name} (Java {})",
                    op.command,
                    param.java_type
                );
            }
            for param in &tool.params {
                let java_schema = match param.java_type.as_str() {
                    "int" | "Integer" => "integer",
                    "boolean" | "Boolean" => "boolean",
                    "String" => "string",
                    "Object" => param.schema_type.as_str(),
                    java if java.starts_with("Map<") => "object",
                    java if java.starts_with("List<") => "array",
                    java => panic!("Unhandled plugin Java type: {java}"),
                };
                assert_eq!(
                    param.schema_type, java_schema,
                    "{} / {}",
                    op.command, param.name
                );
                assert!(
                    op.args
                        .iter()
                        .any(|arg| !arg.local && arg.key == param.name),
                    "{}: plugin parameter {} has no CLI binding",
                    op.command,
                    param.name
                );
                if param.required {
                    let spec = op.args.iter().find(|arg| arg.key == param.name).unwrap();
                    assert!(
                        spec.required || spec.stdin || spec.default.is_some(),
                        "{}: CLI can omit plugin-required argument {}",
                        op.command,
                        param.name
                    );
                    assert!(
                        wire.contains_key(&param.name),
                        "{}: missing required plugin argument {}",
                        op.command,
                        param.name
                    );
                }
            }
            for arg in op.args.iter().filter(|arg| arg.local) {
                assert!(!wire.contains_key(&arg.key));
            }
        }
    }
    assert_eq!(
        covered.len(),
        tools.len(),
        "All plugin tools must be checked"
    );
}
