use crate::{
    config::Settings,
    operations::{Operation, inventory},
};
use clap::{Arg, ArgAction, ArgMatches, Command, builder::PossibleValuesParser};

fn add_args(mut cmd: Command, op: &Operation) -> Command {
    if op.command == "refactor rename-variable" {
        cmd = cmd.allow_missing_positional(true).after_help(
            "Examples:\n  eclipsectl refactor rename-variable FooTest str strSuper --line 8\n  eclipsectl refactor rename-variable FooTest str strSuper --method testFoo\n  eclipsectl refactor rename-variable FooTest str strSuper --method testFoo --line 8\n  eclipsectl refactor rename-variable src/FooTest.java strSuper --offset 180\nLegacy syntax: eclipsectl refactor rename-variable src/FooTest.java 180 strSuper",
        );
    }
    for spec in &op.args {
        let mut arg =
            Arg::new(spec.name.clone())
                .required(spec.required)
                .help(if spec.help.is_empty() {
                    spec.key.clone()
                } else {
                    spec.help.clone()
                });
        if !spec.positional {
            arg = arg.long(spec.name.clone());
        }
        if spec.kind == "switch" {
            cmd = cmd.arg(arg.action(ArgAction::SetTrue));
            continue;
        }
        arg = arg.value_parser(clap::builder::StringValueParser::new());
        if matches!(spec.kind.as_str(), "strings" | "csv" | "env") {
            arg = arg.action(ArgAction::Append);
            if spec.positional {
                arg = arg.num_args(1..);
            }
        }
        if spec.kind == "flag" {
            arg = arg
                .num_args(0..=1)
                .require_equals(true)
                .default_missing_value("true")
                .value_parser(PossibleValuesParser::new(["true", "false"]));
        } else if spec.kind == "boolean" {
            arg = arg.value_parser(PossibleValuesParser::new(["true", "false"]));
        } else if spec.kind == "integer" {
            use clap::builder::TypedValueParser;
            let min = spec.min.unwrap_or(0);
            let max = spec.max.unwrap_or(u64::MAX);
            arg = arg.value_parser(clap::builder::StringValueParser::new().try_map(move |s| {
                let number = s
                    .parse::<u64>()
                    .map_err(|_| "Expected a nonnegative integer")?;
                if number < min || number > max {
                    return Err("Integer is outside the allowed range");
                }
                Ok::<_, &'static str>(s)
            }));
        } else if !spec.choices.is_empty() {
            arg = arg.value_parser(PossibleValuesParser::new(spec.choices.clone()));
        }
        if let Some(default) = &spec.default {
            arg = arg.default_value(default.clone());
        }
        cmd = cmd.arg(arg);
    }
    if op.command == "launch env" {
        cmd = cmd
            .arg(
                Arg::new("clear")
                    .long("clear")
                    .action(ArgAction::SetTrue)
                    .conflicts_with("env")
                    .help("Send an explicitly empty environment map (replace mode only)"),
            )
            .arg(
                Arg::new("replace")
                    .long("replace")
                    .action(ArgAction::SetTrue)
                    .help("Replace rather than merge the environment"),
            );
    }
    cmd
}

fn branch(prefix: &str, name: &str) -> Command {
    let cmd = Command::new(name.to_owned());
    if let Some(op) = inventory().iter().find(|op| op.command == prefix) {
        return add_args(cmd.about(op.description.clone()), op);
    }
    let mut children = std::collections::BTreeSet::new();
    for op in inventory() {
        let remainder = if prefix.is_empty() {
            Some(op.command.as_str())
        } else {
            op.command.strip_prefix(&format!("{prefix} "))
        };
        if let Some(rest) = remainder {
            children.insert(rest.split(' ').next().expect("command segment"));
        }
    }
    let mut cmd = cmd.subcommand_required(true).arg_required_else_help(true);
    for child in children {
        let path = if prefix.is_empty() {
            child.into()
        } else {
            format!("{prefix} {child}")
        };
        cmd = cmd.subcommand(branch(&path, child));
    }
    cmd
}

pub fn command() -> Command {
    branch("", "eclipsectl")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Control Eclipse IDE from Windows and WSL")
        .arg(
            Arg::new("json")
                .long("json")
                .global(true)
                .action(ArgAction::SetTrue)
                .help("Emit a stable JSON result/error envelope"),
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .global(true)
                .action(ArgAction::SetTrue)
                .conflicts_with("quiet"),
        )
        .arg(
            Arg::new("quiet")
                .long("quiet")
                .short('q')
                .global(true)
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("eclipse-url")
                .long("eclipse-url")
                .global(true)
                .help("Eclipse HTTP base URL"),
        )
        .arg(
            Arg::new("bearer-token")
                .long("bearer-token")
                .global(true)
                .help("Authentication token (prefer ECLIPSE_TOKEN)"),
        )
        .arg(
            Arg::new("connect-timeout")
                .long("connect-timeout")
                .global(true)
                .value_parser(clap::value_parser!(u64).range(1..=86400)),
        )
        .arg(
            Arg::new("operation-timeout")
                .long("operation-timeout")
                .global(true)
                .value_parser(clap::value_parser!(u64).range(1..=86400)),
        )
        .subcommand(
            Command::new("doctor")
                .about("Check configuration, daemon and Eclipse MCP connectivity"),
        )
        .subcommand(
            Command::new("daemon")
                .about("Manage the local persistent MCP daemon")
                .subcommand_required(true)
                .subcommands([
                    Command::new("status"),
                    Command::new("stop"),
                    Command::new("restart"),
                    Command::new("serve").hide(true),
                ]),
        )
        .subcommand(
            Command::new("config").subcommand_required(true).subcommand(
                Command::new("migrate")
                    .about("Convert legacy JSON configuration to TOML, without overwriting")
                    .arg(
                        Arg::new("from")
                            .long("from")
                            .required(true)
                            .value_parser(clap::value_parser!(std::path::PathBuf)),
                    )
                    .arg(
                        Arg::new("to")
                            .long("to")
                            .required(true)
                            .value_parser(clap::value_parser!(std::path::PathBuf)),
                    ),
            ),
        )
}

pub fn overrides(matches: &ArgMatches) -> Settings {
    Settings {
        eclipse_url: matches.get_one::<String>("eclipse-url").cloned(),
        bearer_token: matches.get_one::<String>("bearer-token").cloned(),
        connect_timeout: matches.get_one::<u64>("connect-timeout").copied(),
        operation_timeout: matches.get_one::<u64>("operation-timeout").copied(),
    }
}

pub fn selected(matches: &ArgMatches) -> (String, &ArgMatches) {
    let mut path = Vec::new();
    let mut selected = matches;
    while let Some((name, child)) = selected.subcommand() {
        path.push(name);
        selected = child;
    }
    (path.join(" "), selected)
}
