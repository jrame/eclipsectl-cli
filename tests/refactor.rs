use eclipsectl::{cli, operations};
use serde_json::{Value, json};
use std::path::Path;

fn call(args: &[&str]) -> anyhow::Result<operations::Call> {
    let mut argv = vec!["eclipsectl", "refactor"];
    argv.extend_from_slice(args);
    let matches = cli::command().try_get_matches_from(argv)?;
    let (name, arguments) = cli::selected(&matches);
    operations::find(&name)?.call(arguments, Path::new(r"B:\work"))
}

#[test]
fn refactorings_preserve_positions_paths_and_plugin_wire_types() {
    for (command, tool, operands, extra) in [
        (
            "rename-method",
            "refactorRenameMethod",
            vec!["calculate"],
            json!({"newName":"calculate"}),
        ),
        (
            "rename-field",
            "refactorRenameField",
            vec!["total"],
            json!({"newName":"total"}),
        ),
        (
            "rename-variable",
            "refactorRenameVariable",
            vec!["amount"],
            json!({"newName":"amount"}),
        ),
        (
            "change-signature",
            "refactorChangeSignature",
            vec!["--return-type", "java.util.List<String>"],
            json!({"returnType":"java.util.List<String>"}),
        ),
        (
            "extract-variable",
            "refactorExtractVariable",
            vec!["5", "subtotal", "--replace-all", "false", "--final", "true"],
            json!({"length":"5","variableName":"subtotal","replaceAll":false,"declareFinal":true}),
        ),
        (
            "extract-constant",
            "refactorExtractConstant",
            vec!["2", "LIMIT", "--visibility", "package"],
            json!({"length":"2","constantName":"LIMIT","visibility":"package"}),
        ),
        (
            "inline-method",
            "refactorInlineMethod",
            vec!["--all", "true", "--delete-source", "true"],
            json!({"inlineAll":true,"deleteSource":true}),
        ),
        (
            "inline-variable",
            "refactorInlineVariable",
            vec![],
            json!({}),
        ),
    ] {
        let mut args = vec![command, r"B:/dev/é space/Example.java", "0"];
        args.extend(operands);
        args.extend(["--project", "example"]);
        let call = call(&args).unwrap();
        assert_eq!(call.arguments["offset"], 0);
        let operation = operations::find(&call.operation).unwrap();
        assert_eq!(operation.server, "eclipse-refactor");
        assert_eq!(operation.tool, tool);
        let mut expected =
            json!({"filePath":r"B:\dev\é space\Example.java","offset":"0","projectName":"example"});
        expected
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        if command == "rename-variable" {
            expected["offset"] = json!(0);
        }
        assert_eq!(
            Value::Object(operation.wire_arguments(&call.arguments).unwrap()),
            expected
        );
        assert!(
            include_str!("../docs/COMMANDS.md").contains(&format!("eclipsectl refactor {command}"))
        );
    }
    let omitted = call(&["inline-method", "src/Example.java", "42"]).unwrap();
    assert_eq!(
        omitted.arguments,
        json!({"filePath":"src/Example.java","offset":42})
            .as_object()
            .unwrap()
            .clone()
    );
    let explicit = call(&[
        "inline-method",
        "src/Example.java",
        "42",
        "--all",
        "false",
        "--delete-source",
        "false",
    ])
    .unwrap();
    assert_eq!(explicit.arguments["inlineAll"], false);
    assert_eq!(explicit.arguments["deleteSource"], false);
}

#[test]
fn variable_rename_accepts_named_selectors_and_explicit_or_legacy_offsets() {
    let op = operations::find("refactor rename-variable").unwrap();
    for (selectors, expected) in [
        (vec!["--line", "8"], json!({"lineNumber":8})),
        (vec!["--line=8"], json!({"lineNumber":8})),
        (vec!["--method", "testFoo"], json!({"methodName":"testFoo"})),
        (
            vec!["--method", "testFoo", "--line", "8"],
            json!({"lineNumber":8,"methodName":"testFoo"}),
        ),
    ] {
        let mut args = vec!["rename-variable", "FooTest", "str", "strSuper"];
        args.extend(selectors);
        args.extend(["--project", "example", "--preview", "true"]);
        let request = call(&args).unwrap();
        let mut expected = expected.as_object().unwrap().clone();
        expected.extend(json!({"filePath":"FooTest","oldName":"str","newName":"strSuper","projectName":"example","preview":true}).as_object().unwrap().clone());
        assert_eq!(request.arguments, expected);
        assert_eq!(op.wire_arguments(&request.arguments).unwrap(), expected);
    }
    let explicit = call(&[
        "rename-variable",
        "src/FooTest.java",
        "strSuper",
        "--offset",
        "180",
    ])
    .unwrap();
    let legacy = call(&["rename-variable", "src/FooTest.java", "180", "strSuper"]).unwrap();
    assert_eq!(explicit.arguments, legacy.arguments);
    assert_eq!(explicit.arguments["offset"], 180);
    assert!(!explicit.arguments.contains_key("oldName"));
}

#[test]
fn invalid_named_variable_selections_are_rejected_in_cli_and_ipc() {
    for suffix in [
        vec![],
        vec!["--line", "0"],
        vec!["--line", "-1"],
        vec!["--line", "2147483648"],
        vec!["--line", "x"],
        vec!["--method", " "],
        vec!["--line", "8", "--offset", "10"],
        vec!["--offset", "10"],
    ] {
        let mut args = vec!["rename-variable", "FooTest", "str", "strSuper"];
        args.extend(suffix);
        assert!(call(&args).is_err(), "{args:?}");
    }
    for args in [
        vec!["rename-variable", "FooTest", "strSuper", "--line", "8"],
        vec![
            "rename-variable",
            "FooTest",
            "strSuper",
            "--offset",
            "2147483648",
        ],
        vec!["rename-variable", "FooTest", "", "strSuper", "--line", "8"],
    ] {
        assert!(call(&args).is_err(), "{args:?}");
    }
    let op = operations::find("refactor rename-variable").unwrap();
    for extra in [
        json!({}),
        json!({"oldName":"str"}),
        json!({"lineNumber":8}),
        json!({"oldName":"str","lineNumber":0}),
        json!({"oldName":"str","lineNumber":"8"}),
        json!({"oldName":"str","methodName":""}),
        json!({"offset":1,"oldName":"str","lineNumber":8}),
        json!({"offset":1,"methodName":"testFoo"}),
        json!({"offset":2147483648u64}),
    ] {
        let mut arguments = json!({"filePath":"FooTest","newName":"strSuper"})
            .as_object()
            .unwrap()
            .clone();
        arguments.extend(extra.as_object().unwrap().clone());
        assert!(op.validate(&arguments).is_err(), "{arguments:?}");
    }
}

#[test]
fn signature_parameters_keep_json_expressions_and_empty_lists() {
    let parameters = r#"[{"oldIndex":1,"name":"text","type":"java.util.List<String>"},{"oldIndex":-1,"name":"path","type":"String","defaultValue":"\"/mnt/b/é space\""}]"#;
    for parameters in [parameters, "[]"] {
        let call = call(&[
            "change-signature",
            "src/Example.java",
            "12",
            "--parameters",
            parameters,
        ])
        .unwrap();
        assert_eq!(call.arguments["parameters"], parameters);
        let wire = operations::find(&call.operation)
            .unwrap()
            .wire_arguments(&call.arguments)
            .unwrap();
        assert_eq!(wire["parameters"], parameters);
    }
}

#[test]
fn invalid_refactorings_are_rejected_before_transport() {
    for args in [
        vec!["rename-method", "Example.java", "-1", "name"],
        vec!["rename-field", "Example.java", "2147483648", "name"],
        vec!["extract-variable", "Example.java", "0", "0", "name"],
        vec![
            "extract-constant",
            "Example.java",
            "2147483647",
            "1",
            "NAME",
        ],
        vec![
            "extract-constant",
            "Example.java",
            "0",
            "2",
            "NAME",
            "--visibility",
            "invalid",
        ],
        vec![
            "inline-method",
            "Example.java",
            "0",
            "--delete-source",
            "true",
        ],
        vec![
            "inline-method",
            "Example.java",
            "0",
            "--all",
            "false",
            "--delete-source",
            "true",
        ],
        vec!["change-signature", "Example.java", "0"],
        vec!["change-signature", "Example.java", "0", "--name", " "],
        vec!["change-signature", "Example.java", "0", "--return-type", ""],
    ] {
        assert!(call(&args).is_err(), "{args:?}");
    }
    for parameters in [
        "broken",
        "{}",
        "null",
        "[0]",
        "[{}]",
        r#"[{"oldIndex":-2,"name":"x","type":"int"}]"#,
        r#"[{"oldIndex":2147483648,"name":"x","type":"int"}]"#,
        r#"[{"oldIndex":0,"name":"x","type":"int"},{"oldIndex":0,"name":"y","type":"int"}]"#,
        r#"[{"oldIndex":-1,"name":"x","type":"int"}]"#,
        r#"[{"oldIndex":-1,"name":"x","type":"int","defaultValue":null}]"#,
        r#"[{"oldIndex":0,"name":"","type":"int"}]"#,
        r#"[{"oldIndex":0,"name":"x","type":"int","typo":true}]"#,
    ] {
        assert!(
            call(&[
                "change-signature",
                "Example.java",
                "0",
                "--parameters",
                parameters
            ])
            .is_err(),
            "{parameters}"
        );
    }
    // Raw IPC requests are validated independently of clap.
    let operation = operations::find("refactor extract-variable").unwrap();
    for offset in [json!(-1), json!("0"), json!(2147483648u64)] {
        assert!(operation.validate(json!({"filePath":"Example.java","offset":offset,"length":1,"variableName":"x"}).as_object().unwrap()).is_err());
    }
}
