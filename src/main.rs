use anyhow::Result;
use eclipsectl::{
    cli, config, daemon,
    ipc::{Action, Response},
    operations,
};
use serde_json::{Value, json};
use std::{path::PathBuf, time::Duration};

#[tokio::main]
async fn main() {
    let raw: Vec<_> = std::env::args_os().collect();
    let json_output = raw.iter().any(|a| a == "--json");
    let matches = match cli::command().try_get_matches_from(raw) {
        Ok(matches) => matches,
        Err(error) => {
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) {
                let _ = error.print();
                std::process::exit(0);
            }
            if json_output {
                emit(
                    &Response::error("invalid_arguments", error.to_string()),
                    true,
                );
            } else {
                let _ = error.print();
            }
            std::process::exit(2);
        }
    };
    let filter = if matches.get_flag("quiet") {
        "off"
    } else if matches.get_flag("verbose") {
        "eclipsectl=debug"
    } else {
        "eclipsectl=warn"
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(false)
        .without_time()
        .init();
    let response = match execute(&matches).await {
        Ok(response) => response,
        Err(error) => {
            // Config validation/migration diagnostics never include file contents or token values.
            let mut message = format!("{error:#}");
            for token in [
                std::env::var("ECLIPSE_TOKEN").ok(),
                matches.get_one::<String>("bearer-token").cloned(),
            ]
            .into_iter()
            .flatten()
            {
                if !token.is_empty() {
                    message = message.replace(&token, "[REDACTED]");
                }
            }
            Response::error("execution_failed", message)
        }
    };
    emit(&response, matches.get_flag("json"));
    std::process::exit(if response.ok {
        0
    } else if response
        .error
        .as_ref()
        .is_some_and(|e| e.code == "invalid_arguments")
    {
        2
    } else {
        1
    });
}

async fn execute(matches: &clap::ArgMatches) -> Result<Response> {
    let (name, args) = cli::selected(matches);
    match name.as_str() {
        "daemon serve" => {
            daemon::serve_at(&daemon::runtime_dir()?, Duration::from_secs(300)).await?;
            Ok(Response::success(json!({"stopped":true})))
        }
        "daemon status" => daemon::status().await,
        "daemon stop" => daemon::stop().await,
        "daemon restart" => daemon::restart().await,
        "config migrate" => {
            config::migrate(
                args.get_one::<PathBuf>("from").expect("required"),
                args.get_one::<PathBuf>("to").expect("required"),
            )?;
            Ok(Response::success(json!({"migrated":true})))
        }
        _ => {
            let cwd = std::env::current_dir()?;
            let config = config::load(&cli::overrides(matches), &cwd)?;
            let timeout =
                Duration::from_secs(config.connect_timeout + config.operation_timeout * 2 + 10);
            if name == "doctor" {
                daemon::request(Action::Doctor { config }, timeout).await
            } else {
                let call = match operations::find(&name)?.call(args, &cwd) {
                    Ok(call) => call,
                    Err(error) => {
                        return Ok(Response::error("invalid_arguments", error.to_string()));
                    }
                };
                daemon::request(Action::Call { config, call }, timeout).await
            }
        }
    }
}

fn print_content(value: &Value, stderr: bool) {
    let text = if let Some(content) = value.get("content").and_then(Value::as_array) {
        let text: Vec<_> = content
            .iter()
            .filter_map(|c| c.get("text").and_then(Value::as_str))
            .collect();
        if text.is_empty() {
            serde_json::to_string_pretty(value).unwrap_or_default()
        } else {
            text.join("\n")
        }
    } else {
        serde_json::to_string_pretty(value).unwrap_or_default()
    };
    if stderr {
        eprintln!("{text}");
    } else {
        println!("{text}");
    }
}

fn emit(response: &Response, json_output: bool) {
    if json_output {
        println!(
            "{}",
            serde_json::to_string(response).expect("serializable response")
        );
    } else if let Some(error) = &response.error {
        eprintln!("Error [{}]: {}", error.code, error.message);
        if let Some(details) = &error.details {
            print_content(details, true);
        }
    } else if let Some(result) = &response.result {
        print_content(result, false);
    }
}
