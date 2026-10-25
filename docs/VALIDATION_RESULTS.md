# Validation results — 2026-10-16

## Variable rename by line and method — 2026-10-25 (Windows)

`refactor rename-variable` now accepts an old/new name pair with `--line`
(1-based), `--method`, or both, plus an explicit `--offset` alternative. The
legacy positional offset form is preserved. The matching plugin changes are in
`B:\dev\eclipse-workspace2\eclipsectl-mcp-plugin`; named target resolution uses
JDT bindings under the existing workspace lock and frozen-preview lifecycle.

Formatting, Clippy with warnings denied and the release build passed. The Rust
suites passed, including selector validation, legacy syntax, numeric
wire values and mocked MCP routing. The plugin's targeted Maven/Tycho verification
passed **32 PDE tests**, including real variable/parameter renames, CRLF/UTF-16,
repeated references, overloaded methods, separate scopes, comments/strings,
duplicate source files, preview/apply and HTTP schema/error checks. Catalogue
generation and generator fixtures passed. Tests use disposable projects in an
isolated Eclipse workbench; the plugin has not been installed in the user's
running Eclipse, and Linux/WSL was not tested for this change.

## Additional Java refactorings — 2026-10-25 (Windows)

Eight CLI operations and their matching tools were added in the sibling plugin:
rename method/field/local variable, change signature, extract variable/constant,
and inline method/local variable.

Formatting, Clippy with warnings denied, the full Rust test suite and the release
build passed. The subsequently added process test for refactoring help/invalid
arguments also passed independently. Contracts cover all eight wire mappings,
positions, optional booleans, signature JSON and invalid requests. Mock transport
tests verify routing and reuse of the refactoring MCP session.

The plugin's targeted Maven/Tycho build passed **15 PDE tests** in an isolated
Eclipse workbench and disposable Java projects. These exercise real JDT changes,
cross-file callers, overloads, shadowed variables, parameter reorder/add/remove,
return type changes, expression extraction, inline operations and rejection of
invalid selections or unsaved caller buffers. The generated tool catalogue and
its generator fixtures also passed validation.

Plugin command (from its repository):

```powershell
.\mvnw.cmd -o -B -pl plugins/plugin.eclipsectl.mcp.dependencies,plugins/plugin.eclipsectl.mcp.main,tests/plugin.eclipsectl.mcp.main.tests -am verify '-Dtest=JavaRefactoringServiceTest' '-DfailIfNoTests=false'
```

The plugin was not installed in the user's Eclipse. CLI HTTP calls to an installed
updated plugin and Linux/WSL validation were not run for this change.

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
