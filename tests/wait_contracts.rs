use eclipsectl::{cli, operations};
use serde_json::json;
use std::path::Path;

#[test]
fn wait_flags_are_required_and_local_options_never_reach_mcp() {
    for argv in [
        vec!["eclipsectl", "workspace", "wait", "--idle"],
        vec!["eclipsectl", "build", "wait"],
        vec!["eclipsectl", "debug", "wait", "--suspended"],
    ] {
        let mut argv = argv;
        argv.extend(["--timeout", "12", "--poll-interval", "50"]);
        let matches = cli::command().try_get_matches_from(argv).unwrap();
        let (name, args) = cli::selected(&matches);
        let op = operations::find(&name).unwrap();
        let call = op.call(args, Path::new(r"C:\work")).unwrap();
        assert!(op.wait_for.is_some());
        assert_eq!(call.arguments["timeout"], 12);
        assert_eq!(call.arguments["pollInterval"], 50);
        assert_eq!(
            op.wire_arguments(&call.arguments).unwrap(),
            json!({}).as_object().unwrap().clone()
        );
    }
    for argv in [
        vec!["eclipsectl", "workspace", "wait"],
        vec!["eclipsectl", "workspace", "wait", "--idle=false"],
        vec!["eclipsectl", "debug", "wait"],
        vec!["eclipsectl", "debug", "wait", "--suspended=false"],
        vec!["eclipsectl", "build", "wait", "--timeout", "0"],
        vec!["eclipsectl", "build", "wait", "--timeout", "86401"],
        vec!["eclipsectl", "build", "wait", "--poll-interval", "49"],
        vec!["eclipsectl", "build", "wait", "--poll-interval", "60001"],
    ] {
        assert!(cli::command().try_get_matches_from(argv).is_err());
    }
}

#[test]
fn daemon_validates_wait_options_independently_of_clap() {
    let op = operations::find("debug wait").unwrap();
    for args in [
        json!({}),
        json!({"suspended":false}),
        json!({"suspended":true,"timeout":0}),
        json!({"suspended":true,"timeout":86401}),
        json!({"suspended":true,"pollInterval":0}),
        json!({"suspended":true,"pollInterval":60001}),
    ] {
        assert!(op.validate(args.as_object().unwrap()).is_err());
    }
}
