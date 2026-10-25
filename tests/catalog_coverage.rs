use eclipsectl::{cli, operations};
use serde_json::{Value, json};
use std::path::Path;

#[test]
fn all_97_catalogue_tools_have_cli_bindings() {
    let tools: std::collections::HashSet<_> = include_str!("catalog_tools.txt").lines().collect();
    assert_eq!(tools.len(), 97);
    let actual: std::collections::HashSet<_> = operations::inventory()
        .iter()
        .map(|op| format!("{}/{}", op.server, op.tool))
        .collect();
    assert_eq!(actual, tools.into_iter().map(str::to_owned).collect());
}

fn wire(args: &[&str]) -> Value {
    let matches = cli::command()
        .try_get_matches_from(std::iter::once("eclipsectl").chain(args.iter().copied()))
        .unwrap();
    let (name, matches) = cli::selected(&matches);
    let operation = operations::find(&name).unwrap();
    let call = operation.call(matches, Path::new(r"B:\work")).unwrap();
    Value::Object(operation.wire_arguments(&call.arguments).unwrap())
}

#[test]
fn additional_tools_have_distinct_bindings_and_preserve_source() {
    for (command, server, tool) in [
        ("debug processes", "eclipse-debug", "listProcesses"),
        ("code format", "eclipse-ide", "formatCode"),
        ("problems panel", "eclipse", "problems"),
    ] {
        let operation = operations::find(command).unwrap();
        assert_eq!((&*operation.server, &*operation.tool), (server, tool));
        assert!(include_str!("../docs/COMMANDS.md").contains(&format!("eclipsectl {command}")));
    }
    assert_eq!(wire(&["debug", "processes"]), json!({}));
    let source = "class É {\r\nString path = \"/home/example\";\r\n}\r\n";
    assert_eq!(
        wire(&["code", "format", "--source", source, "--project", "app"]),
        json!({"code": source, "projectName": "app"})
    );
    assert_eq!(
        wire(&[
            "problems",
            "panel",
            "--severity",
            "warning",
            "--max",
            "25",
            "--skip-unknown-location",
            "false"
        ]),
        json!({"severity":"warning", "maxResults":"25", "skipUnknownLocation":"false"})
    );
    assert_eq!(wire(&["problems", "panel"]), json!({}));
    // Existing compilation-problem command keeps its binding and defaults.
    assert_eq!(
        operations::find("problems list").unwrap().tool,
        "getCompilationErrors"
    );
    assert_eq!(
        wire(&["problems", "list"]),
        json!({"severity":"ERROR", "maxResults":"10"})
    );
}

#[test]
fn breakpoint_project_and_hit_count_are_optional_and_wire_encoded() {
    for action in ["add", "remove", "enable", "disable"] {
        assert_eq!(
            wire(&["debug", "breakpoint", action, "src/Main.java", "12"]),
            json!({"filePath":"src/Main.java", "lineNumber":"12"})
        );
        assert_eq!(
            wire(&[
                "debug",
                "breakpoint",
                action,
                "src/Main.java",
                "12",
                "--project",
                "app"
            ]),
            json!({"filePath":"src/Main.java", "lineNumber":"12", "projectName":"app"})
        );
    }
    assert_eq!(
        wire(&[
            "debug",
            "breakpoint",
            "add",
            "src/Main.java",
            "12",
            "--hit-count",
            "3",
            "--condition",
            "count > 1"
        ]),
        json!({"filePath":"src/Main.java", "lineNumber":"12", "hitCount":"3", "condition":"count > 1"})
    );
}

#[test]
fn inspection_parameters_preserve_false_and_plugin_defaults() {
    assert_eq!(wire(&["debug", "threads"]), json!({}));
    assert_eq!(
        wire(&["debug", "threads", "--session", "Main dev"]),
        json!({"sessionName":"Main dev"})
    );
    assert_eq!(
        wire(&["debug", "thread-select", "main", "--session", "Main dev"]),
        json!({"threadName":"main", "sessionName":"Main dev"})
    );
    assert_eq!(wire(&["debug", "stacktrace"]), json!({}));
    assert_eq!(
        wire(&[
            "debug",
            "stacktrace",
            "--package-filter",
            "com.example",
            "--skip-framework",
            "false",
            "--max-depth",
            "20"
        ]),
        json!({"maxDepth":"20", "packageFilter":"com.example", "skipFramework":"false"})
    );
    assert_eq!(wire(&["console", "output"]), json!({}));
    assert_eq!(
        wire(&[
            "console",
            "output",
            "--name",
            "Maven Console",
            "--max-lines",
            "250",
            "--include-all-consoles",
            "true"
        ]),
        json!({"consoleName":"Maven Console", "maxLines":"250", "includeAllConsoles":"true"})
    );
    assert_eq!(
        wire(&["console", "output", "--include-all-consoles", "false"]),
        json!({"includeAllConsoles":"false"})
    );
    assert_eq!(
        wire(&[
            "code",
            "hierarchy",
            "com.example.Main",
            "main",
            "--signature",
            "[QString;",
            "--max-depth",
            "5"
        ]),
        json!({"fullyQualifiedClassName":"com.example.Main", "methodName":"main", "methodSignature":"[QString;", "maxDepth":"5"})
    );
}

#[test]
fn project_import_tests_and_maven_keep_existing_operands() {
    assert_eq!(
        wire(&["project", "import", "B:/dev/app", "--project", "renamed"]),
        json!({"projectPath":r"B:\dev\app", "projectName":"renamed"})
    );
    assert_eq!(wire(&["test", "all"]), json!({}));
    assert_eq!(
        wire(&["test", "all", "--project", "app", "--timeout", "120"]),
        json!({"projectName":"app", "timeout":"120"})
    );
    for (args, expected) in [
        (
            vec!["test", "class", "app", "com.example.MainTest"],
            json!({"projectName":"app", "className":"com.example.MainTest"}),
        ),
        (
            vec!["test", "package", "app", "com.example"],
            json!({"projectName":"app", "packageName":"com.example"}),
        ),
        (
            vec!["test", "method", "app", "com.example.MainTest", "works"],
            json!({"projectName":"app", "className":"com.example.MainTest", "methodName":"works"}),
        ),
    ] {
        assert_eq!(wire(&args), expected);
        let mut args = args;
        args.extend(["--timeout", "0"]);
        let mut timed = expected;
        timed["timeout"] = json!("0");
        assert_eq!(wire(&args), timed);
    }
    assert_eq!(
        wire(&["build", "maven", "app", "clean", "test"]),
        json!({"projectName":"app", "goals":"clean test"})
    );
    assert_eq!(
        wire(&[
            "build",
            "maven",
            "app",
            "clean",
            "test",
            "--profile",
            "dev",
            "--profile",
            "integration",
            "--timeout",
            "0"
        ]),
        json!({"projectName":"app", "goals":"clean test", "profiles":"dev,integration", "timeout":"0"})
    );
    assert_eq!(wire(&["build", "errors"]), json!({}));
    assert_eq!(
        wire(&["build", "errors", "--severity", "WARNING", "--max", "30"]),
        json!({"severity":"WARNING", "maxResults":"30"})
    );
}

#[test]
fn invalid_new_option_values_fail_before_connecting() {
    for args in [
        vec![
            "debug",
            "breakpoint",
            "add",
            "src/Main.java",
            "1",
            "--hit-count",
            "0",
        ],
        vec![
            "debug",
            "breakpoint",
            "add",
            "src/Main.java",
            "1",
            "--hit-count",
            "2147483648",
        ],
        vec!["test", "all", "--timeout", "2147483648"],
        vec!["build", "maven", "app", "test", "--timeout", "-1"],
        vec!["console", "output", "--max-lines", "many"],
        vec!["console", "output", "--include-all-consoles", "yes"],
        vec!["debug", "stacktrace", "--skip-framework", "yes"],
        vec!["problems", "panel", "--severity", "fatal"],
    ] {
        assert!(
            cli::command()
                .try_get_matches_from(std::iter::once("eclipsectl").chain(args))
                .is_err()
        );
    }
}
