use eclipsectl::{
    cli,
    config::{self, Settings},
    ipc, operations, paths,
};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

fn call(argv: &[&str]) -> operations::Call {
    let matches = cli::command().try_get_matches_from(argv).unwrap();
    let (name, args) = cli::selected(&matches);
    operations::find(&name)
        .unwrap()
        .call(args, Path::new(r"C:\work"))
        .unwrap()
}

#[test]
fn inventory_and_help_are_complete_and_unambiguous() {
    cli::command().debug_assert();
    let inventory = operations::inventory();
    assert_eq!(inventory.len(), 83);
    assert_eq!(
        inventory
            .iter()
            .map(|op| &op.tool)
            .collect::<HashSet<_>>()
            .len(),
        82
    );
    assert_eq!(
        inventory
            .iter()
            .map(|op| &op.command)
            .collect::<HashSet<_>>()
            .len(),
        83
    );
    assert!(inventory.iter().all(|op| !op.server.is_empty()));
    assert!(include_str!("../docs/COMMANDS.md").contains("eclipsectl launch create-java"));
}

#[test]
fn config_priority_is_per_field_and_defaults_are_stable() {
    let user = Settings {
        eclipse_url: Some("http://user:8765".into()),
        bearer_token: Some("user-token".into()),
        connect_timeout: Some(5),
        ..Settings::default()
    };
    let project = Settings {
        eclipse_url: Some("http://project:8765".into()),
        ..Settings::default()
    };
    let env = HashMap::from([("ECLIPSE_URL".into(), "http://env:8765/".into())]);
    let cli = Settings {
        operation_timeout: Some(900),
        ..Settings::default()
    };
    let resolved = config::resolve(&cli, &env, &project, &user).unwrap();
    assert_eq!(resolved.eclipse_url, "http://env:8765");
    assert_eq!(resolved.bearer_token.as_deref(), Some("user-token"));
    assert_eq!(resolved.connect_timeout, 5);
    assert_eq!(resolved.operation_timeout, 900);
    assert!(!format!("{resolved:?}").contains("user-token"));
    let defaults = config::resolve(
        &Settings::default(),
        &HashMap::new(),
        &Settings::default(),
        &Settings::default(),
    )
    .unwrap();
    assert_eq!(defaults.eclipse_url, "http://localhost:8124");
    assert_eq!(defaults.operation_timeout, 300);
    assert!(
        config::resolve(
            &Settings {
                operation_timeout: Some(0),
                ..Settings::default()
            },
            &HashMap::new(),
            &project,
            &user
        )
        .is_err()
    );
}

#[test]
fn migration_preserves_values_and_never_overwrites() {
    let dir = tempfile::tempdir().unwrap();
    let from = dir.path().join("old.json");
    let to = dir.path().join("new.toml");
    std::fs::write(&from, r#"{"eclipse_url":"http://localhost:8765","bearer_token":"secret","timeout":75,"verbose":false}"#).unwrap();
    config::migrate(&from, &to).unwrap();
    let output = std::fs::read_to_string(&to).unwrap();
    let migrated: Settings = toml::from_str(&output).unwrap();
    assert_eq!(migrated.connect_timeout, Some(75));
    assert_eq!(migrated.bearer_token.as_deref(), Some("secret"));
    assert!(config::migrate(&from, &to).is_err());
    assert_eq!(std::fs::read_to_string(&to).unwrap(), output);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&to).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}

#[test]
fn launch_optional_false_and_empty_environment_remain_distinct() {
    let created = call(&[
        "eclipsectl",
        "launch",
        "create-java",
        "com.example.Main",
        "--type",
        "junit5",
        "--env",
        "URL=a=b",
        "--inherit-env",
        "false",
        "--cwd",
        r"B:\dev\é space",
    ]);
    assert_eq!(created.arguments["appendSystemEnvironment"], false);
    assert_eq!(
        created.arguments["environmentVariables"],
        json!({"URL":"a=b"})
    );
    assert_eq!(created.arguments["workingDirectory"], r"B:\dev\é space");
    let defaults = call(&["eclipsectl", "launch", "create-java", "com.example.Main"]);
    assert!(!defaults.arguments.contains_key("type"));
    let merged = call(&[
        "eclipsectl",
        "launch",
        "env",
        "Main",
        "--env",
        "PROFILE=dev",
    ]);
    assert!(!merged.arguments.contains_key("appendSystemEnvironment"));
    let replaced = call(&[
        "eclipsectl",
        "launch",
        "env",
        "Main",
        "--replace",
        "--clear",
        "--inherit-env",
        "false",
    ]);
    assert_eq!(replaced.arguments["variables"], json!({}));
    assert_eq!(replaced.arguments["updateMode"], "replace");
    assert_eq!(replaced.arguments["appendSystemEnvironment"], false);
    let show = call(&["eclipsectl", "launch", "show", "Main"]);
    assert!(!show.arguments.contains_key("includeEnvironmentValues"));
    let invalid = cli::command()
        .try_get_matches_from([
            "eclipsectl",
            "launch",
            "env",
            "Main",
            "--replace",
            "--remove",
            "SECRET",
        ])
        .unwrap();
    let (name, args) = cli::selected(&invalid);
    assert!(
        operations::find(&name)
            .unwrap()
            .call(args, Path::new("."))
            .is_err()
    );
    assert!(
        cli::command()
            .try_get_matches_from([
                "eclipsectl",
                "launch",
                "create-java",
                "Main",
                "--type",
                "junit3"
            ])
            .is_err()
    );
}

#[test]
fn paths_and_source_are_never_confused() {
    assert_eq!(
        paths::windows_path(r"B:/dev/é space/File.java", Path::new("."), false).unwrap(),
        r"B:\dev\é space\File.java"
    );
    assert_eq!(
        paths::windows_path(r"\\server\share\File.java", Path::new("."), false).unwrap(),
        r"\\server\share\File.java"
    );
    assert_eq!(
        paths::windows_path("src/Main.java", Path::new("."), true).unwrap(),
        "src/Main.java"
    );
    assert!(
        paths::windows_path(
            r"\\wsl.localhost\Ubuntu\home\me\Main.java",
            Path::new("."),
            true
        )
        .is_err()
    );
    let source = "public String chemin() {\n    return \"/mnt/b/é space\";\n}\n";
    let written = call(&[
        "eclipsectl",
        "java",
        "write",
        "src/Main.java",
        "--source",
        source,
    ]);
    assert_eq!(written.arguments["newSource"], source);
    assert_eq!(written.arguments["filePath"], "src/Main.java");
    let refresh = call(&[
        "eclipsectl",
        "refresh",
        "save",
        r"B:\dev\A.java",
        r"C:\é space\B.java",
    ]);
    assert_eq!(
        refresh.arguments["paths"],
        "B:\\dev\\A.java,C:\\é space\\B.java"
    );
}

#[test]
fn debug_arguments_have_explicit_meaning() {
    assert_eq!(
        call(&["eclipsectl", "debug", "thread-select", "main"]).arguments["threadName"],
        "main"
    );
    assert_eq!(
        call(&["eclipsectl", "debug", "stacktrace", "--max-depth", "12"]).arguments["maxDepth"],
        12
    );
    assert_eq!(
        call(&["eclipsectl", "debug", "eval", "x + 1", "--frame", "0"]).arguments["stackFrameIndex"],
        0
    );
    assert!(
        cli::command()
            .try_get_matches_from(["eclipsectl", "debug", "variables", "--frame", "oops"])
            .is_err()
    );
}

#[tokio::test]
async fn ipc_supports_large_fragmented_frames_and_rejects_oversized_lengths() {
    let (mut a, mut b) = tokio::io::duplex(17);
    let payload = json!({"source":"é\n".repeat(100_000)});
    let expected = payload.clone();
    let sender = tokio::spawn(async move {
        ipc::write_frame(&mut a, &payload).await.unwrap();
    });
    let decoded: Value = ipc::read_frame(&mut b).await.unwrap();
    sender.await.unwrap();
    assert_eq!(decoded, expected);
    use tokio::io::AsyncWriteExt;
    let (mut a, mut b) = tokio::io::duplex(8);
    a.write_u32((ipc::MAX_FRAME + 1) as u32).await.unwrap();
    assert!(ipc::read_frame::<_, Value>(&mut b).await.is_err());
}

#[test]
fn launch_wait_preserves_boolean_wire_values_and_omission() {
    for argv in [
        vec!["eclipsectl", "run", "--configuration", "FooTest"],
        vec!["eclipsectl", "debug", "run", "FooTest"],
        vec!["eclipsectl", "coverage", "run", "FooTest"],
    ] {
        for (flag, expected) in [
            (None, None),
            (Some("--wait"), Some(json!(true))),
            (Some("--wait=true"), Some(json!(true))),
            (Some("--wait=false"), Some(json!(false))),
        ] {
            let mut args = argv.clone();
            if let Some(flag) = flag {
                // Exercise the reported order: run --wait --configuration FooTest.
                let index = args.iter().position(|arg| *arg == "run").unwrap() + 1;
                args.insert(index, flag);
            }
            let call = call(&args);
            let wire = operations::find(&call.operation)
                .unwrap()
                .wire_arguments(&call.arguments)
                .unwrap();
            assert_eq!(call.arguments.get("waitForCompletion"), expected.as_ref());
            assert_eq!(wire.get("waitForCompletion"), expected.as_ref());
        }
    }
}

#[test]
fn wire_types_match_plugin_schemas_without_losing_typed_cli_values() {
    let breakpoint = call(&[
        "eclipsectl",
        "debug",
        "breakpoint",
        "add",
        "src/Main.java",
        "12",
    ]);
    assert_eq!(breakpoint.arguments["lineNumber"], 12);
    let wire = operations::find(&breakpoint.operation)
        .unwrap()
        .wire_arguments(&breakpoint.arguments)
        .unwrap();
    assert_eq!(wire["lineNumber"], "12");
    let maven = call(&["eclipsectl", "build", "maven", "example", "clean", "test"]);
    let wire = operations::find(&maven.operation)
        .unwrap()
        .wire_arguments(&maven.arguments)
        .unwrap();
    assert_eq!(wire["goals"], "clean test");
    let launch = call(&[
        "eclipsectl",
        "launch",
        "env",
        "Main",
        "--inherit-env",
        "false",
        "--replace",
        "--clear",
    ]);
    let wire = operations::find(&launch.operation)
        .unwrap()
        .wire_arguments(&launch.arguments)
        .unwrap();
    assert_eq!(wire["appendSystemEnvironment"], false);
    assert_eq!(wire["variables"], json!({}));
}
