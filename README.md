# eclipsectl

Native Rust command-line control of Eclipse IDE through the **eclipsectl-mcp** plugin. The same code builds `eclipsectl.exe` for Windows and `eclipsectl` for Linux/WSL. No Python, Bash, jq or netcat is required to run the CLI.

The executable contains both the CLI and its automatically started daemon. Windows uses a named pipe restricted to the current user; WSL uses a private Unix socket. Each environment has its own configuration and daemon. The Eclipse plugin continues to speak MCP Streamable HTTP. This project no longer exposes an MCP stdio server.

## Install

Download the Windows x64 ZIP or Linux x64 tar.gz from [Releases](https://github.com/jrame/eclipsectl-cli/releases), verify its SHA-256 checksum, extract it and run the included installer. Source installation requires Rust 1.88 or newer and a native C/C++ build toolchain (MSVC on Windows, a C compiler and CMake on Linux).

```powershell
# Windows: extracted archive or source checkout
.\install.ps1
eclipsectl --help
```

```sh
# WSL: extracted archive or source checkout
sh install.sh
eclipsectl --help
```

Installers use `$HOME/.local/bin` (Windows: `$env:USERPROFILE\.local\bin`). Add it to PATH if needed. They do not edit shell profiles or system settings. `cargo install --locked --path .` is also supported.

The Windows runtime is linked statically, so a Visual C++ redistributable is not required. A Linux archive suffixed `musl` contains a static executable; it uses the same CLI and installer.

## Configuration

Enable the HTTP MCP server in Eclipse's eclipsectl-mcp preferences and generate a Bearer token. The default URL is `http://localhost:8124`; configure the actual address and port explicitly if different.

Create `.config/eclipsectl-cli/config.toml` in the directory from which you invoke the CLI, or a user configuration:

| Environment | User configuration |
|---|---|
| Windows | `%APPDATA%\eclipsectl-cli\config.toml` |
| WSL/Linux | `$XDG_CONFIG_HOME/eclipsectl-cli/config.toml`, default `~/.config/eclipsectl-cli/config.toml` |

```toml
eclipse_url = "http://localhost:8124"
connect_timeout = 30
operation_timeout = 300
# Prefer ECLIPSE_TOKEN to storing bearer_token here.
```

```powershell
$env:ECLIPSE_TOKEN = "YOUR_TOKEN"
eclipsectl doctor
```

```sh
export ECLIPSE_TOKEN='YOUR_TOKEN'
eclipsectl doctor
```

Precedence applies **per field**: CLI options, environment, project TOML, user TOML, defaults. Environment variables: `ECLIPSE_URL`, `ECLIPSE_TOKEN`, `ECLIPSE_CONNECT_TIMEOUT`, `ECLIPSE_OPERATION_TIMEOUT`. Timeouts are seconds, from 1 to 86400. Invalid configuration fails explicitly.

The project file is resolved from the current working directory, including on Windows. If `.config/eclipsectl-cli/config.toml` is absent, `.eclipsectl.toml` in that same directory is accepted as a fallback. When both exist, only the nested file is used. Configuration is reread on every command and sent to the daemon; editing the TOML requires no reload command or daemon restart.

Migrate old JSON explicitly; the destination must not exist:

```sh
eclipsectl config migrate --from .config/eclipsectl-cli/config.json --to .config/eclipsectl-cli/config.toml
```

The old `timeout` becomes `connect_timeout`; operation timeout defaults to 300 seconds. The old `verbose` setting is replaced by the CLI option. The config directory remains named `eclipsectl-cli`; the executable is always `eclipsectl`.

## Commands

```sh
eclipsectl project list
eclipsectl workspace info --json
eclipsectl build maven example clean test
eclipsectl test class example com.example.MainTest
eclipsectl editor open 'B:\dev\example\src\Main.java' --line 12
eclipsectl search text TODO --pattern-file '*.java'
eclipsectl source format src/Main.java --project example
eclipsectl launch create-java com.example.Main --project example --name 'Main dev' --env PROFILE=dev
eclipsectl launch env 'Main dev' --env PROFILE=test --inherit-env false
eclipsectl debug run 'Main dev' --wait
eclipsectl daemon status
```

All **83 command bindings / 82 distinct MCP tools / 7 endpoints** are listed in [the command inventory](docs/COMMANDS.md). Use `eclipsectl <family> <command> --help` for details. See [CLI_USAGE.md](CLI_USAGE.md), [DAEMON.md](DAEMON.md), and [the real Eclipse smoke test procedure](docs/VALIDATION.md).

Human output contains Eclipse text. `--json` preserves the complete MCP result, including non-text content and `structuredContent`:

```json
{"ok":true,"result":{"content":[{"type":"text","text":"..."}]}}
```

Failures use `{"ok":false,"error":{"code":"...","message":"...","details":{}}}`; `details` appears only when available. Exit codes: 0 success, 1 execution/configuration failure, 2 invalid CLI arguments. Diagnostics go to stderr. `--verbose` enables application diagnostics and `--quiet` suppresses logs; errors remain visible. The application never logs tokens, source or launch environment values. Explicit command results can contain requested source code or environment values.

## Windows and WSL paths

Pass Windows drive paths, UNC paths, or paths on mounted Windows drives in WSL. WSL uses `wslpath`, respecting custom mount roots. Project-relative paths stay relative; filesystem-relative paths resolve in the client before IPC. Expressions, source, search patterns, JVM arguments and environment values are never translated.

Projects under `/home` and `\\wsl.localhost\...` are outside this version's scope. Store Eclipse projects on a Windows drive. The upstream refresh API uses comma-separated paths, so refresh rejects paths containing commas.

**WSL networking:** mirrored mode can access Eclipse Windows at `localhost`. NAT mode requires the Windows host address, an accessible plugin bind address and a firewall permitting its port. The Windows host address is visible through `ip route show default`. A plugin listening only on Windows loopback cannot be reached through the NAT host address. The CLI does not rewrite URLs or change network settings. See [Microsoft's documentation](https://learn.microsoft.com/windows/wsl/networking).

## Development

```sh
cargo build --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --locked --release
```

The library separates CLI, registry, paths, configuration, IPC, MCP sessions and daemon. `src/operations.json` is the shared command contract; keep it synchronized with `docs/COMMANDS.md`. Commit `Cargo.lock`.

CI checks Windows/Linux. Tags named `v*` build and publish native x64 archives with SHA-256 checksums. Real Eclipse validation requires a configured plugin and is separate from mock-based CI.
