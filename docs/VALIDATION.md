# Validation

## Automated checks

Run `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo test --locked --all-targets` on Windows and Linux. Transport tests run a local MCP HTTP/SSE server and real named pipes/Unix sockets; no Eclipse installation or token is required.

`tests/plugin_contract.rs` checks every operation against `tests/plugin-tool-contract.json`,
exported from the plugin's Java declarations. It verifies the actual serialized types,
Java/schema agreement, required arguments and complete parameter/tool coverage. It runs
in ordinary CI without Eclipse. After changing the plugin API, run
`java scripts/GenerateToolCatalog.java` in the plugin repository and copy its generated
`docs/mcp-tool-contract.json` to this repository's `tests/plugin-tool-contract.json`.
Do not derive the expected types from the CLI registry itself.

The plugin generator rejects schema/Java type mismatches. Its
`ToolParameterContractTest` also checks every builtin method by reflection, including
primitive parameters, nullable values and legacy string conversions. `TypedToolHttpTest`
exercises native and legacy scalars through the real HTTP transport and reflective
executor. Service-only tests and a successful `tools/list` schema comparison cannot
establish that CLI arguments can invoke Java methods.

The plugin's `RefactorServiceTest` exercises file rename/move, Java type rename/move,
package rename and method extraction in disposable projects under the real PDE UI
harness. Exceptions must fail JUnit; a success message alone is insufficient. Assert
saved file contents, removal of original resources, updated callers and a clean
project build. Run it alongside `JavaRefactoringServiceTest` in the plugin repository:

```powershell
.\mvnw.cmd -o -B -pl plugins/plugin.eclipsectl.mcp.dependencies,plugins/plugin.eclipsectl.mcp.main,tests/plugin.eclipsectl.mcp.main.tests -am verify '-Dtest=RefactorServiceTest,JavaRefactoringServiceTest' '-DfailIfNoTests=false'
```

Coverage includes the command inventory, configuration precedence and migration, environment tri-state values, Unicode and multiline source, large/fragmented IPC frames, MCP error results, authentication, session reuse, expiration without replay, queue serialization, operation timeout, graceful draining and idle shutdown. A warm IPC plus mock-MCP benchmark reports p95 and asserts it stays below 100 ms (excluding initial connection).

State-wait tests cover required flags and numeric limits, local-only polling options,
running-to-suspended transitions, workspace quiet-window resets, builds completing while
other workspace jobs remain active, malformed states, MCP errors without retries, slow
reads, total and per-read deadlines, JSON envelopes and the debug session lock. Run
`cargo test --locked --test wait_contracts --test transport --test process` for this scope.
The companion plugin's `WorkspaceStateServiceTest` runs under Tycho's
`integration-test` phase and checks delayed auto/manual build jobs in an Eclipse test workspace.

Eclipse build contracts and simulated MCP checks run with
`cargo test --locked --test eclipse_build`. They cover omitted plugin defaults,
repeated projects encoded as a comma-separated string, ambiguous/blank name rejection,
native boolean/integer types, numeric limits, full/incremental choices, complete MCP
success/error reports without replay, and the build-specific client timeout extension.
`cargo run --locked -- build eclipse --help` checks the interface without Eclipse.

## Live Eclipse smoke test

Use a **disposable** Java project and launch configurations. Configure the real URL/token, then run the same sequence from native PowerShell and native WSL:

1. `eclipsectl doctor --json`, `project list`, `workspace info`, `launch list`.
2. Import the disposable project with `project import <Windows-drive-path>`; inspect layout, properties and a project resource.
3. Run `java read <file> --project <project>`, then add and remove a temporary member through `java write` and `java remove`. Verify the plugin's read gate rejects a write without the required prior read.
4. Exercise `source format`, `source organize-imports`, `refactor rename-type`, `project mkdir` and `refresh save` on disposable resources.
   With the updated plugin, also exercise `refactor rename-method`, `rename-field`,
   `rename-variable`, `change-signature`, `extract-variable`, `extract-constant`,
   `inline-method` and `inline-variable`. Use saved-source UTF-16 offsets and
   recompute them after each edit. Include overloaded methods, shadowed local
   variables, reordered/added/removed signature parameters and a caller in another
   file. Check that JDT updates callers, the disposable project still compiles,
   invalid selections leave source unchanged, and unsaved buffers are rejected.
5. After an edit/refresh, run `workspace wait --idle --timeout 60` and `build wait --timeout 60` before inspecting compilation errors. Also run `build maven <project> clean test` and a known JUnit test through `test class`.
   To explicitly compile saved resources, use `build eclipse --project <disposable-project> --build-kind full --include-dependents false --timeout 60 --max-problems 10 --json`.
   Verify dependency builds and per-project totals; introduce a temporary compile error
   and check exit 1 with the report in `error.details`, then fix it and rebuild.
   `build wait` only observes existing builds. Avoid omitting `--project` in a shared workspace.
6. Create a launch with `launch create-java`, inspect it with `launch show`, update/clear its environment, and run it with `run --configuration <name> --wait`.
7. Add a breakpoint, launch debug, run `debug wait --suspended --timeout 30`, then inspect threads/variables, step and resume. Repeat the wait after a step/resume before inspecting values; terminate the disposable debug session when done.
8. If EclEmma is installed, run coverage and inspect a class report.
9. Repeat from WSL with mounted Windows-drive paths. Verify `wslpath` translates the actual configured mount root. Verify `/home` projects fail clearly.
10. Test both NAT (explicit Windows host IP and reachable plugin binding) and mirrored (`localhost`) networking when available.

Restore or remove only resources created for the smoke test. Do not run workspace-wide edits, builds or `test all` against an arbitrary existing workspace.

## Packaging

Build with `cargo build --locked --release`. Extract the native release archive, run the bundled installer into a temporary directory, verify its checksum and execute the installed `eclipsectl --help` / `--version`. The installed program must run without Python, jq, netcat or a shell wrapper.

Real plugin, Maven/JUnit and EclEmma checks are environment-dependent. Record unavailable capabilities separately from mock-test results; do not claim live checks succeeded when only simulated transport tests ran.

## Java project creation

`cargo test --locked --test project_create` covers required `--java`, optional
arguments, Java integer/JUnit string wire types, Windows path normalization,
unsupported WSL paths, help/JSON exit 2 without a daemon, and simulated MCP
success, conflicts and partial creation errors without replay.

The companion plugin's `ProjectCreationServiceTest` creates only disposable
projects in Tycho's isolated Eclipse workbench. It checks Java nature/builder,
UTF-8, default and explicit JRE selection, layout/properties, real class builds,
JUnit 5 test compilation into `bin-test` and an actual Eclipse JUnit launch.
It also checks occupied files/directories, duplicate names, overlapping locations,
invalid arguments, unavailable execution environments and retained partial state.
Run from the plugin repository:

```powershell
.\mvnw.cmd -o -B -pl plugins/plugin.eclipsectl.mcp.dependencies,plugins/plugin.eclipsectl.mcp.main,tests/plugin.eclipsectl.mcp.main.tests -am verify '-Dtest=ProjectCreationServiceTest,ProjectServiceTest,ToolParameterContractTest' '-DfailIfNoTests=false'
```

For an installed-plugin CLI check, choose a new directory beneath a disposable
parent, run `project create <path> --java --junit 5`, inspect `project properties`
and `project layout`, add a tiny source/test, build with `build eclipse`, and run
`test class`. Check conflicts leave a sentinel file unchanged. Never use an
existing user project for this validation. Distinguish isolated PDE integration
from CLI checks against the user's installed plugin in the results.
