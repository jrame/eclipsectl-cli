# Validation results — 2026-10-16

## Java project creation — 2026-10-31 (Windows)

Implemented `project create <path> --java` / `eclipse-ide/createJavaProject`,
including optional project name, installed Java execution environment and JUnit 5.
The CLI and plugin catalogues now cover 98 tools (101 CLI operation bindings).

Passed native Windows checks: `cargo build --locked`, command help,
`cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`,
`cargo test --locked --all-targets` (63 passed, 4 opt-in live checks ignored),
and `cargo build --locked --release`. The creation tests use a simulated HTTP
MCP peer for wire types, success/error preservation and no replay, and a real CLI
subprocess for help, JSON argument errors and no daemon startup.

The plugin catalogue generator and its fixtures passed. Tycho built the plugin
and ran **19 passing tests**, including 8 `ProjectCreationServiceTest` cases,
9 existing import/facade tests and 2 parameter-contract tests across all tools:

```powershell
.\mvnw.cmd -o -B -pl plugins/plugin.eclipsectl.mcp.dependencies,plugins/plugin.eclipsectl.mcp.main,tests/plugin.eclipsectl.mcp.main.tests -am verify '-Dtest=ProjectCreationServiceTest,ProjectServiceTest,ToolParameterContractTest' '-DfailIfNoTests=false'
```

These are actual Eclipse workbench/JDT integration tests in Tycho's isolated
workspace, not mocked project creation. They verified open/registered projects,
Java nature/builder, explicit UTF-8, properties/layout, default JRE and explicit
Java 21 execution environment, real class compilation, JUnit 5 compilation into
`bin-test`, and an Eclipse JUnit run with one successful test. Other cases cover
occupied files/directories, duplicate names, project-location overlaps, default
workspace locations, concurrent reservations, unavailable Java execution
environments, invalid arguments and retained/reported partial creation.

The test runtime explicitly includes `org.eclipse.jdt.junit5.runtime`: JDT's
JUnit UI dependency alone does not bring that runner into Tycho's workspace.
Creation preflights its presence when JUnit 5 is requested. The one-time online
build downloaded this missing test dependency; subsequent runs passed offline.

The complete p2 repository build also passed with `mvnw.cmd -o -B verify -DskipTests`
after those tests. Installable output is in the plugin repository at
`repository/target/repository/`, with archive
`repository/target/eclipsectl-repository-1.0.1-SNAPSHOT.zip`.

The user's installed Eclipse plugin was not replaced and no user project was
modified. A complete CLI HTTP mutation against that installed plugin and new
Linux/WSL execution were not performed. Those are distinct from the passing
isolated Eclipse integration and mock transport checks above. Earlier sections
below describe previous changes and their then-current tool counts/checks.

## CLI/Java argument-type audit — 2026-10-26 (Windows)

The `problems list --max 10 --severity error` failure exposed a blind spot: CLI
tests expected numeric strings, mocked MCP handlers accepted those strings, and
the plugin's advertised schemas also incorrectly said `string` for typed Java
parameters. Schema-only audits and direct service tests did not exercise that
boundary. Earlier successful schema audits are not evidence of invocable types.

After the initial compilation-problem fix, the audit found and corrected **33
additional CLI argument mappings**, the corresponding **33 plugin schema
types**, and **13 debug parameters incorrectly advertised as required**.
The scope includes breakpoint lines/hit counts, debug frames and flags, Java
refactoring positions, quick fixes, code generation and inspection options.
Legitimate Java string adapters retain string encoding.

The ordinary Rust suite now checks every operation against an exported contract
from all **97 builtin Java tools**, including required arguments and both
variable-selector forms. The plugin generator rejects Java/schema mismatches;
reflection tests check native values, legacy scalars, nullable arguments and
primitive parameters. A real HTTP test exercises the production MCP factory,
JSON transport and reflective executor, including invalid values.

Formatting, Clippy, all **58 ordinary Rust tests**, the release build, generator
fixtures, reflection contract tests and the new HTTP regression test passed.
Read-only calls against the user's existing Eclipse passed for `problems list`,
`build errors`, `problems panel --skip-unknown-location false` and
`console output --include-all-consoles false`, with explicit numeric limits.
The corrected plugin update site was built but not installed into that Eclipse.
Linux/WSL and real workspace mutations were not exercised in this audit.

The audit also found that the six legacy `RefactorServiceTest` methods caught
exceptions and only printed them. Their earlier green result is **not** counted
as functional proof. Those tests have now been rewritten: exceptions propagate
to JUnit, UI callbacks run on the real workbench display, and assertions check
saved contents, removal of original files, updated cross-package callers and
clean project builds. Method extraction now selects the intended assignments
and checks the generated method's parameters, visibility, return type and call.
Each test uses a uniquely named disposable project and closes its own editors.

Targeted Maven/Tycho verification passed **38 PDE tests**: the six rewritten
tests and the 32 newer `JavaRefactoringServiceTest` tests, with no failures,
errors or skips. Two temporary negative controls each made Maven exit 1 and
failed exactly the expected test: a service exception for a missing source file,
and a fake success response without renaming the file. The corrected test source
was restored after each control, its hash verified, and all six tests passed
again in a final run. These checks ran on Windows in an isolated
Eclipse workbench; they do not establish complete refactoring coverage or
validation against the user's running Eclipse or Linux/WSL.

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
