use crate::paths::windows_path;
use crate::wait::WaitCondition;
use anyhow::{Context, Result, bail};
use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{
    io::{IsTerminal, Read},
    path::Path,
    sync::OnceLock,
};

#[derive(Debug, Deserialize)]
pub struct Operation {
    pub command: String,
    pub tool: String,
    pub server: String,
    pub description: String,
    pub args: Vec<Argument>,
    #[serde(default)]
    pub wait_for: Option<WaitCondition>,
}

#[derive(Debug, Deserialize)]
pub struct Argument {
    pub key: String,
    pub name: String,
    pub kind: String,
    pub wire: Option<String>,
    #[serde(default)]
    pub positional: bool,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub path: bool,
    #[serde(default)]
    pub project_relative: bool,
    #[serde(default)]
    pub stdin: bool,
    pub default: Option<String>,
    #[serde(default)]
    pub choices: Vec<String>,
    #[serde(default)]
    pub help: String,
    #[serde(default)]
    pub local: bool,
    pub min: Option<u64>,
    pub max: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Call {
    pub operation: String,
    pub arguments: Map<String, Value>,
}

pub fn inventory() -> &'static [Operation] {
    static INVENTORY: OnceLock<Vec<Operation>> = OnceLock::new();
    INVENTORY.get_or_init(|| {
        serde_json::from_str(include_str!("operations.json"))
            .expect("valid compiled operation inventory")
    })
}

pub fn find(command: &str) -> Result<&'static Operation> {
    inventory()
        .iter()
        .find(|o| o.command == command)
        .context("Unknown operation")
}

impl Operation {
    /// Allow buildProjects to return its timeout/cancellation report before the client expires.
    /// Queue and connection budgets remain independent of this per-invocation budget.
    pub fn operation_timeout(&self, arguments: &Map<String, Value>, configured: u64) -> u64 {
        if self.server == "eclipse-ide" && self.tool == "buildProjects" {
            configured.max(
                arguments
                    .get("timeoutSeconds")
                    .and_then(Value::as_u64)
                    .unwrap_or(120)
                    .saturating_add(10),
            )
        } else {
            configured
        }
    }

    /// Convert validated CLI values to the actual Eclipse plugin wire types.
    pub fn wire_arguments(&self, arguments: &Map<String, Value>) -> Result<Map<String, Value>> {
        self.validate(arguments)?;
        let mut output = arguments.clone();
        for arg in &self.args {
            if arg.local {
                output.remove(&arg.key);
                continue;
            }
            if let (Some(wire), Some(value)) = (&arg.wire, arguments.get(&arg.key)) {
                let encoded = match wire.as_str() {
                    "string" if value.is_boolean() || value.is_number() => value.to_string(),
                    "space" | "csv" => {
                        let items: Vec<_> = value
                            .as_array()
                            .context("Expected an argument list")?
                            .iter()
                            .map(|v| v.as_str().context("Expected a string argument"))
                            .collect::<Result<_>>()?;
                        items.join(if wire == "space" { " " } else { "," })
                    }
                    _ => bail!("Invalid compiled wire encoding"),
                };
                output.insert(arg.key.clone(), Value::from(encoded));
            }
        }
        Ok(output)
    }

    pub fn call(&self, matches: &ArgMatches, cwd: &Path) -> Result<Call> {
        let mut arguments = Map::new();
        for arg in &self.args {
            if arg.kind == "switch" {
                arguments.insert(arg.key.clone(), Value::from(matches.get_flag(&arg.name)));
                continue;
            }
            let values: Vec<String> = matches
                .get_many::<String>(&arg.name)
                .map(|v| v.cloned().collect())
                .unwrap_or_default();
            let mut value = if values.is_empty() {
                None
            } else {
                Some(match arg.kind.as_str() {
                    "integer" => Value::from(values[0].parse::<u64>()?),
                    "boolean" | "flag" => Value::from(values[0].parse::<bool>()?),
                    "strings" => Value::from(values.clone()),
                    "csv" => Value::from(values.join(",")),
                    "env" => {
                        let mut map = Map::new();
                        for entry in &values {
                            let (key, val) = entry
                                .split_once('=')
                                .context("Environment entries must use NAME=VALUE")?;
                            if key.is_empty() {
                                bail!("Environment variable names must not be empty");
                            }
                            map.insert(key.into(), Value::from(val));
                        }
                        Value::Object(map)
                    }
                    _ => Value::from(values[0].clone()),
                })
            };
            if arg.stdin && value.is_none() {
                if std::io::stdin().is_terminal() {
                    bail!("Supply --{} or pipe UTF-8 input on stdin", arg.name);
                }
                let mut input = String::new();
                std::io::stdin()
                    .take(crate::ipc::MAX_FRAME as u64 + 1)
                    .read_to_string(&mut input)?;
                if input.len() > crate::ipc::MAX_FRAME {
                    bail!("Input exceeds 64 MiB limit");
                }
                value = Some(Value::from(if arg.key == "members" {
                    input.lines().collect::<Vec<_>>().join(",")
                } else {
                    input
                }));
            }
            if arg.path
                && let Some(v) = &mut value
            {
                if let Some(array) = v.as_array_mut() {
                    for entry in array {
                        *entry = Value::from(windows_path(
                            entry.as_str().context("Expected path")?,
                            cwd,
                            arg.project_relative,
                        )?);
                    }
                } else {
                    *v = Value::from(windows_path(
                        v.as_str().context("Expected path")?,
                        cwd,
                        arg.project_relative,
                    )?);
                }
            }
            if let Some(value) = value {
                arguments.insert(arg.key.clone(), value);
            }
        }
        // Preserve the original <file> <offset> <name> syntax while accepting
        // <file-or-class> <old-name> <name> --line/--method and <file> <name> --offset.
        if self.command == "refactor rename-variable"
            && !["offset", "lineNumber", "methodName"]
                .iter()
                .any(|key| arguments.contains_key(*key))
            && let Some(offset) = arguments
                .get("oldName")
                .and_then(Value::as_str)
                .and_then(|value| value.parse::<u64>().ok())
        {
            arguments.remove("oldName");
            arguments.insert("offset".into(), Value::from(offset));
        }
        let clear = matches
            .try_get_one::<bool>("clear")
            .ok()
            .flatten()
            .copied()
            .unwrap_or(false);
        let replace = matches
            .try_get_one::<bool>("replace")
            .ok()
            .flatten()
            .copied()
            .unwrap_or(false);
        if clear {
            arguments.insert("variables".into(), Value::Object(Map::new()));
        }
        if replace {
            arguments.insert("updateMode".into(), Value::from("replace"));
        }
        if clear && arguments.get("updateMode").and_then(Value::as_str) != Some("replace") {
            bail!("--clear requires --replace or --mode replace");
        }
        // The Eclipse refresh API accepts comma-separated paths, not a JSON array.
        for key in ["path", "paths"] {
            if let Some(Value::Array(paths)) = arguments.get(key) {
                let mut output = Vec::new();
                for path in paths {
                    let path = path.as_str().context("Expected path")?;
                    if path.contains(',') {
                        bail!("The Eclipse refresh API cannot represent paths containing commas");
                    }
                    output.push(path.to_owned());
                }
                arguments.insert(key.into(), Value::from(output.join(",")));
            }
        }
        self.validate(&arguments)?;
        Ok(Call {
            operation: self.command.clone(),
            arguments,
        })
    }

    pub fn validate(&self, args: &Map<String, Value>) -> Result<()> {
        for key in args.keys() {
            if !self.args.iter().any(|a| a.key == *key) {
                bail!("Unknown argument {key}");
            }
        }
        for arg in &self.args {
            let value = args.get(&arg.key);
            if (arg.required || arg.stdin) && value.is_none() {
                bail!("Missing argument {}", arg.name);
            }
            if let Some(v) = value {
                let valid = match arg.kind.as_str() {
                    "integer" => v.as_u64().is_some(),
                    "boolean" | "flag" => v.is_boolean(),
                    "switch" => v == &Value::Bool(true),
                    "env" => v
                        .as_object()
                        .is_some_and(|m| m.values().all(Value::is_string)),
                    "strings" if matches!(arg.key.as_str(), "path" | "paths") => v.is_string(),
                    "strings" => v.as_array().is_some_and(|a| a.iter().all(Value::is_string)),
                    _ => v.is_string(),
                };
                if !valid {
                    bail!("Invalid value type for {}", arg.name);
                }
                if let Some(number) = v.as_u64()
                    && (arg.min.is_some_and(|min| number < min)
                        || arg.max.is_some_and(|max| number > max))
                {
                    bail!("Value out of range for {}", arg.name);
                }
                if !arg.choices.is_empty()
                    && !arg.choices.iter().any(|s| Some(s.as_str()) == v.as_str())
                {
                    bail!("Invalid value for {}", arg.name);
                }
            }
        }
        if self.tool == "buildProjects"
            && let Some(names) = args.get("projectNames").and_then(Value::as_array)
            && (names.is_empty()
                || names.iter().any(|name| {
                    name.as_str()
                        .is_none_or(|name| name.trim().is_empty() || name.contains(','))
                }))
        {
            bail!("Project names must be nonblank and cannot contain commas; repeat --project");
        }
        if self.tool == "updateLaunchEnvironment"
            && args.get("updateMode").and_then(Value::as_str) == Some("replace")
            && args.contains_key("removeVariables")
        {
            bail!("--remove cannot be combined with replace mode");
        }
        if self.tool == "refactorExtractMethod"
            && args.get("startLine").and_then(Value::as_u64)
                > args.get("endLine").and_then(Value::as_u64)
        {
            bail!("start-line must not exceed end-line");
        }
        if self.server == "eclipse-refactor" && self.args.iter().any(|arg| arg.key == "offset") {
            for key in ["offset", "length"] {
                if let Some(value) = args.get(key).and_then(Value::as_u64)
                    && (value > i32::MAX as u64 || (key == "length" && value == 0))
                {
                    bail!("{key} must fit a Java int; length must be positive");
                }
            }
            if let (Some(offset), Some(length)) = (
                args.get("offset").and_then(Value::as_u64),
                args.get("length").and_then(Value::as_u64),
            ) && offset + length > i32::MAX as u64
            {
                bail!("Selection end must fit a Java int");
            }
        }
        if self.tool == "refactorRenameVariable" {
            if args.contains_key("offset") {
                if ["oldName", "lineNumber", "methodName"]
                    .iter()
                    .any(|key| args.contains_key(*key))
                {
                    bail!("--offset cannot be combined with an old name, --line or --method");
                }
            } else {
                if !args
                    .get("oldName")
                    .and_then(Value::as_str)
                    .is_some_and(|s| !s.trim().is_empty())
                {
                    bail!("Supply an old name with --line/--method, or use --offset");
                }
                if !args.contains_key("lineNumber") && !args.contains_key("methodName") {
                    bail!("Select the variable with --line and/or --method, or use --offset");
                }
            }
            for key in ["newName", "methodName"] {
                if args
                    .get(key)
                    .and_then(Value::as_str)
                    .is_some_and(|s| s.trim().is_empty())
                {
                    bail!("{key} must not be empty");
                }
            }
        }
        if self.tool == "refactorChangeSignature" {
            if !["newName", "returnType", "visibility", "parameters"]
                .iter()
                .any(|key| args.contains_key(*key))
            {
                bail!("Specify --name, --return-type, --visibility or --parameters");
            }
            for key in ["newName", "returnType"] {
                if args
                    .get(key)
                    .and_then(Value::as_str)
                    .is_some_and(|s| s.trim().is_empty())
                {
                    bail!("{key} must not be empty");
                }
            }
            if let Some(parameters) = args.get("parameters").and_then(Value::as_str) {
                validate_parameters(parameters)?;
            }
        }
        if self.tool == "refactorInlineMethod"
            && args.get("deleteSource").and_then(Value::as_bool) == Some(true)
            && args.get("inlineAll").and_then(Value::as_bool) != Some(true)
        {
            bail!("--delete-source true requires --all true");
        }
        Ok(())
    }
}

fn validate_parameters(input: &str) -> Result<()> {
    let value: Value = serde_json::from_str(input).context("parameters must be a JSON array")?;
    let parameters = value
        .as_array()
        .context("parameters must be a JSON array")?;
    let mut indices = std::collections::HashSet::new();
    for parameter in parameters {
        let fields = parameter
            .as_object()
            .context("Each parameter must be an object")?;
        if fields
            .keys()
            .any(|key| !["oldIndex", "name", "type", "defaultValue"].contains(&key.as_str()))
        {
            bail!("Unknown parameter field");
        }
        let index = fields
            .get("oldIndex")
            .and_then(Value::as_i64)
            .filter(|index| (-1..=i32::MAX as i64).contains(index))
            .context("Each parameter needs an oldIndex (-1 for a new parameter)")?;
        if index >= 0 && !indices.insert(index) {
            bail!("Duplicate parameter oldIndex");
        }
        for key in ["name", "type"] {
            if !fields
                .get(key)
                .and_then(Value::as_str)
                .is_some_and(|s| !s.trim().is_empty())
            {
                bail!("Each parameter needs a nonempty {key}");
            }
        }
        if let Some(default) = fields.get("defaultValue")
            && !default.is_string()
        {
            bail!("defaultValue must be a Java expression string");
        }
        if index == -1
            && !fields
                .get("defaultValue")
                .and_then(Value::as_str)
                .is_some_and(|s| !s.trim().is_empty())
        {
            bail!("New parameters need a defaultValue Java expression for existing callers");
        }
    }
    Ok(())
}
