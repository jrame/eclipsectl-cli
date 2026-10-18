# Validation results — 2026-10-16

## Completed

| Check | Native Windows | Ubuntu WSL |
|---|---|---|
| Rust toolchain | 1.98.0 | 1.94.1 |
| Formatting | Passed | Passed |
| Clippy, all targets, warnings denied | Passed | Passed |
| Automated tests (excluding opt-in live checks) | 20 passed | 21 passed |
| Live plugin schema audit, all 83 command bindings / 7 endpoints | Passed | Passed |
| Live read-only smoke: doctor, projects, workspace, launch list, debug status, text search | Passed | Passed |
| Optimized release build | Passed | Passed |
| Release archive extraction / installer / help | Passed | Passed (Linux musl archive) |

The automatic tests exercise private native IPC, concurrent process startup, crash recovery, graceful draining, Java read gates, JSON/SSE MCP responses, authentication, large frames, preservation of optional false/empty values, and operations exceeding five seconds. The warm local IPC plus mock-MCP p95 check passed its 100 ms threshold on both systems.

The live audit found and corrected legacy wire-type mismatches (string parameters for numeric/boolean values and Maven goals) and removed the unsupported project filter from `coverage report`.

Windows startup uses CreateProcessW without handle inheritance. Multiprocess tests reproduced the inherited-pipeline-handle failure before the fix and passed after it. The Windows CRT is linked statically; the executable does not require a Visual C++ redistributable.

The delivered Linux archive uses a statically linked x86_64 musl binary, cross-compiled with Zig after the repository disk filled. Its ELF headers were checked for the absence of an external interpreter/shared-library requirements. After space was freed, the archive checksum, extraction, installer, help, version, daemon, real Eclipse doctor and project listing all passed under WSL. The full native WSL test suite above was run on the GNU target before that packaging interruption.

## Local TOML discovery update — 2026-10-17

The preferred project configuration is now `.config/eclipsectl-cli/config.toml` relative to the client's current directory on Windows and WSL. `.eclipsectl.toml` is used only when that file is absent. Invalid preferred files fail explicitly.

All 23 Windows automated tests passed, together with formatting and Clippy. New regressions cover preferred/fallback selection, malformed TOML, rereading changes, and a subprocess mock-MCP check that changes both URL and Bearer identity without restarting the daemon. The Windows and static Linux release binaries were rebuilt. The extracted Linux archive passed installation, help, checksum and a WSL test that changed a temporary local TOML between two loopback test ports while retaining the same daemon.

Live Eclipse checks were not repeated for this configuration-only update; the WSL release check used no private credentials.

## Remaining environment-dependent checks

Actual editing, refactoring, Maven/JUnit execution, debug stepping and EclEmma coverage against a disposable Eclipse project were not run. Their operations have contract/mock coverage and were checked against the installed plugin schemas. Use the procedure in VALIDATION.md to perform these live mutation checks on a disposable project.

WSL connected to the configured explicit Windows address. Mirrored-mode localhost networking was documented but not separately tested.

Release publication is configured in GitHub Actions for `v*` tags; no tag or public release was created in this session. Local archives and SHA-256 files are available under `dist/` after the repository disk space was restored.
