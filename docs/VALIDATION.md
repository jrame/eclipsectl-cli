# Validation

## Automated checks

Run `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo test --locked --all-targets` on Windows and Linux. Transport tests run a local MCP HTTP/SSE server and real named pipes/Unix sockets; no Eclipse installation or token is required.

Coverage includes the command inventory, configuration precedence and migration, environment tri-state values, Unicode and multiline source, large/fragmented IPC frames, MCP error results, authentication, session reuse, expiration without replay, queue serialization, operation timeout, graceful draining and idle shutdown. A warm IPC plus mock-MCP benchmark reports p95 and asserts it stays below 100 ms (excluding initial connection).

## Live Eclipse smoke test

Use a **disposable** Java project and launch configurations. Configure the real URL/token, then run the same sequence from native PowerShell and native WSL:

1. `eclipsectl doctor --json`, `project list`, `workspace info`, `launch list`.
2. Import the disposable project with `project import <Windows-drive-path>`; inspect layout, properties and a project resource.
3. Run `java read <file> --project <project>`, then add and remove a temporary member through `java write` and `java remove`. Verify the plugin's read gate rejects a write without the required prior read.
4. Exercise `source format`, `source organize-imports`, `refactor rename-type`, `project mkdir` and `refresh save` on disposable resources.
5. Run `build maven <project> clean test`, inspect compilation errors, and run a known JUnit test through `test class`.
6. Create a launch with `launch create-java`, inspect it with `launch show`, update/clear its environment, and run it with `run --configuration <name> --wait`.
7. Add a breakpoint, launch debug, inspect threads/variables, step and resume; terminate the disposable debug session when done.
8. If EclEmma is installed, run coverage and inspect a class report.
9. Repeat from WSL with mounted Windows-drive paths. Verify `wslpath` translates the actual configured mount root. Verify `/home` projects fail clearly.
10. Test both NAT (explicit Windows host IP and reachable plugin binding) and mirrored (`localhost`) networking when available.

Restore or remove only resources created for the smoke test. Do not run workspace-wide edits, builds or `test all` against an arbitrary existing workspace.

## Packaging

Build with `cargo build --locked --release`. Extract the native release archive, run the bundled installer into a temporary directory, verify its checksum and execute the installed `eclipsectl --help` / `--version`. The installed program must run without Python, jq, netcat or a shell wrapper.

Real plugin, Maven/JUnit and EclEmma checks are environment-dependent. Record unavailable capabilities separately from mock-test results; do not claim live checks succeeded when only simulated transport tests ran.
