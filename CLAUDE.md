# eclipsectl development

Native Rust CLI for Eclipse IDE. The binary is `eclipsectl` (`eclipsectl.exe` on Windows). It also contains the daemon; there is no local MCP stdio server.

Checks: `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked --all-targets`.

Keep `src/operations.json` and `docs/COMMANDS.md` synchronized. Preserve MCP wire argument types and distinguish absent optional values, false and empty maps. Never replay uncertain operations or log credentials, source or environment values.

See README.md, CLI_USAGE.md and DAEMON.md. Live mutation tests must target a disposable project, never an arbitrary existing Eclipse project.
