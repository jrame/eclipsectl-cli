# eclipsectl CLI usage

The binary is `eclipsectl` (`eclipsectl.exe` on Windows). Every command has `--help`; [the full inventory](docs/COMMANDS.md) lists all bindings.

See [catalogue coverage and optional parameters](docs/CATALOG_COVERAGE.md) for
`debug processes`, Java snippet formatting with `code format`, `problems panel`,
and the additional debug, console, hierarchy, Maven and test options.

Project configuration is `.config/eclipsectl-cli/config.toml` relative to the current working directory on both Windows and WSL. If absent, `.eclipsectl.toml` is accepted as a fallback. Every command rereads configuration automatically; no daemon restart is required. CLI options and environment variables override file settings. See [README.md](README.md#configuration) for user configuration locations and precedence.

## Common options

`--json`, `--verbose`/`-v`, `--quiet`/`-q`, `--eclipse-url`, `--bearer-token`, `--connect-timeout`, `--operation-timeout` can appear before or after subcommands. Prefer `ECLIPSE_TOKEN` rather than putting tokens in shell history. Help and version queries never start the daemon or contact Eclipse.

Required operands are positional. Optional context uses named options such as `--project`, `--class`, `--line`, `--frame`. Repeat collection options: `--pattern-file '*.java' --pattern-file '*.xml'`. Boolean settings take `true`/`false`; switches such as `--wait` need no value.

For option values beginning with a hyphen, use the equals form, for example `--vm-args=-Xmx1g` or `--args='--profile dev'`. Use `--` before positional Maven flags: `eclipsectl build maven example -- clean test -DskipTests`.

```sh
eclipsectl problems list --severity WARNING --max 25
eclipsectl refactor extract-method src/Main.java 10 20 extracted --project example --visibility private
eclipsectl codegen getters-setters src/Main.java --project example --field name --getters true --setters false
eclipsectl debug variables --frame 0
eclipsectl debug stacktrace --max-depth 20
```

`debug thread-select` takes a **thread name**, matching the plugin. Stack inspection uses `--max-depth`, replacing the old ignored thread identifier.

## Build Eclipse projects

```sh
eclipsectl build eclipse --project example --project example-tests --build-kind full --include-dependents false --timeout 600 --max-problems 50 --json
eclipsectl build eclipse
```

`build eclipse` invokes `eclipse-ide/buildProjects`, compiles saved resources and
waits for completion. Omit `--project` to build all open projects; repeat it for a
collection of names. Commas and blank names are rejected. Dependencies are always
built; transitive dependents are included unless `--include-dependents false`.
Omitted options retain plugin defaults: incremental build, dependents included,
120 seconds and 100 problem details. `--build-kind` accepts `incremental` or `full`;
`--max-problems` accepts 1..500. Totals remain complete when details are limited.
The report includes per-project error/warning totals and problem locations.
Compilation errors, cancellation and incomplete results are MCP errors (exit 1),
with the complete MCP error result preserved in `error.details` under `--json`.

`--timeout` is the plugin's deadline (1..3600 seconds), including scheduling wait;
expiry requests build cancellation. For this command only, the client MCP budget
is the greater of `--operation-timeout` (including its configured/environment value)
and the plugin deadline plus 10 seconds, leaving time to receive the plugin's
incomplete-result report. Without `--timeout`, this minimum is 130 seconds.
Session queueing and connection retain their separate configured budgets; the IPC
deadline includes the extended MCP budget. Other commands retain their timeouts.
`build wait` observes existing auto/manual builds and never starts a build.
Requires a companion plugin exposing `buildProjects`.

## Wait for Eclipse state

```sh
eclipsectl workspace wait --idle --timeout 60
eclipsectl build wait --timeout 120
eclipsectl build errors --project example
eclipsectl debug run 'Main dev'
eclipsectl debug wait --suspended --timeout 30
eclipsectl debug variables --frame 0
```

Waits poll Eclipse in the daemon without starting a build or suspending the debugger.
`workspace wait --idle` waits for running/waiting workspace jobs and pending builds;
unrelated sleeping maintenance jobs are excluded. `build wait` waits for Eclipse
auto/manual builds, including delayed builds. Both require idle observations spanning
at least 500 ms, restarting that window whenever a busy state is observed. They return
immediately after that window if no work is pending, and do not imply compilation success.
They cannot guarantee that another build will not be scheduled after the wait returns.

`debug wait --suspended` polls the active session reported by `debug status`, waiting
through `none` and `running` states. It does not change the selected thread: select a
specific suspended thread with `debug thread-select` before reading its variables if needed.
All polling holds the corresponding MCP session lock to preserve context.

`--timeout` is the total polling budget in seconds (1..86400), defaulting to the
configured `--operation-timeout` (300 seconds by default). Session queueing and connection
have their own bounded budgets. `--poll-interval` is in milliseconds (50..60000, default 250).
The polling deadline also bounds a slow state read, and each state read respects
`--operation-timeout`. On success, `--json` preserves
the full final MCP response in `result`. A deadline returns exit 1 and error code
`wait_timeout`, with the last observed MCP response in `error.details` when available.
MCP failures stop the wait immediately without retrying the failed call. Unknown state
formats return `wait_state_unavailable`.

Workspace/build waits require the plugin's `eclipse-ide/getWorkspaceState` tool
(`Workspace status: idle|busy`, `Build status: idle|busy`). Rebuild/update the companion
eclipsectl-mcp plugin before using these waits; older plugins fail explicitly.
`debug wait` supports the existing `Debug status: ...` text response. Build waits do
not track external build processes; Maven launches are separate from Eclipse auto/manual builds.

## Java editing and stdin

Supply `--source` / `--members`, or pipe UTF-8 input. Without an option or redirected stdin, the command fails instead of waiting at the terminal.

```sh
eclipsectl java read src/Main.java --project example
eclipsectl java write src/Main.java --project example <<'JAVA'
public String hello() {
    return "hello";
}
JAVA
eclipsectl java remove src/Main.java --members oldMethod --project example
eclipsectl java remove-lines src/Main.java 10-20 --project example
eclipsectl java reset-read-gate --file src/Main.java --project example
```

PowerShell supports pipelines and multiline strings:

```powershell
@'
public String hello() {
    return "hello";
}
'@ | eclipsectl java write src/Main.java --project example
```

The plugin's read gate is preserved. Read a class before editing it as required by Eclipse. If the session is lost or expires, read again before retrying an edit.

## Java refactorings

The additional refactorings require a plugin exposing `refactorRenameMethod`,
`refactorRenameField`, `refactorRenameVariable`, `refactorChangeSignature`,
`refactorExtractVariable`, `refactorExtractConstant`, `refactorInlineMethod` and
`refactorInlineVariable` on `eclipse-refactor`. Rebuild and restart the updated
plugin before using them.

Positions are **zero-based UTF-16 offsets in the saved source**, including line
endings; expression lengths also count UTF-16 code units. Select the identifier
of the declaration or reference to disambiguate overloads, nested classes and
shadowed variables. Save pending editor changes first. Recompute positions after
each refactoring. Eclipse validates the change and updates references through JDT.

```sh
eclipsectl refactor rename-method src/Main.java 120 calculate --project example
eclipsectl refactor rename-field src/Main.java 64 total --project example
eclipsectl refactor rename-variable src/Main.java 180 amount --project example
eclipsectl refactor change-signature src/Main.java 120 --name calculate --return-type long --visibility public --project example
eclipsectl refactor extract-variable src/Main.java 200 5 subtotal --final true --project example
eclipsectl refactor extract-constant src/Main.java 240 2 DEFAULT_LIMIT --visibility private --project example
eclipsectl refactor inline-method src/Main.java 300 --all true --delete-source true --project example
eclipsectl refactor inline-variable src/Main.java 180 --project example
```

The positions above are illustrative. `--replace-all true|false` controls matching
occurrences for extraction (default `false`). Inline method defaults to all calls
when selecting a declaration and a single call when selecting an invocation.
`--all false` requires an invocation. The declaration is retained unless
`--delete-source true --all true` is explicit. Inline variable replaces references
to the local variable with its initializer and removes its declaration.

Change signature requires at least one of `--name`, `--return-type`, `--visibility`
or `--parameters`. The last option accepts a JSON array describing the final
parameter order. `oldIndex` is the zero-based original parameter index, or `-1`
for a new parameter. Each entry requires `name` and `type`. New parameters require
`defaultValue`, a Java expression inserted in existing calls. Omitted original
parameters are removed; `[]` removes all parameters; omitting `--parameters`
preserves them. JDT rejects invalid changes, including removed parameters still
used by the body.

```sh
eclipsectl refactor change-signature src/Main.java 120 --parameters '[{"oldIndex":1,"name":"value","type":"int"},{"oldIndex":0,"name":"label","type":"String"},{"oldIndex":-1,"name":"enabled","type":"boolean","defaultValue":"true"}]' --project example
```

### Rename a variable by name

Select a local variable or parameter using its existing name and a 1-based line,
an enclosing method name, or both:

```sh
eclipsectl refactor rename-variable FooTest str strSuper --line 8
eclipsectl refactor rename-variable FooTest str strSuper --method testFoo
eclipsectl refactor rename-variable FooTest str strSuper --method testFoo --line 8 --preview true
```

The first argument can be a Java file path or a unique simple class name (with
or without `.java`). Add `--project` or use an explicit path if multiple files
match. JDT resolves the actual local variable or parameter and updates its
references. Comments and strings are ignored. Repeated uses of the same variable
on the selected line are accepted; different variables with the same name are
refused. Use `--line` to narrow overloaded methods or separate variable scopes.
Save pending editor changes first. These selectors require a rebuilt/restarted
plugin exposing `oldName`, `lineNumber` and `methodName` on `refactorRenameVariable`.

For precise selection, the explicit offset form is available; the original
positional offset form remains supported:

```sh
eclipsectl refactor rename-variable src/FooTest.java strSuper --offset 180 --project example
eclipsectl refactor rename-variable src/FooTest.java 180 strSuper --project example
```

`--offset` cannot be combined with an old name, `--line` or `--method`. For this
tool, offsets and line numbers are JSON integers on the MCP wire.

### Preview and apply

All eight position-based refactorings above accept `--preview true|false`.
Omitting it, or using `--preview false`, applies directly as before. With
`--preview true`, Eclipse freezes the change and returns a diff and `previewId`
without applying it. This requires `refactorGetPreview` and `refactorApplyPreview`
on the updated `eclipse-refactor` plugin.

```sh
eclipsectl refactor rename-method src/Main.java 120 calculate --project example --preview true --json
eclipsectl refactor get-preview <preview-id> --json
eclipsectl refactor get-preview <preview-id> --offset 100000 --json
eclipsectl refactor apply-preview <preview-id> --json
```

Replace the ID with the returned `previewId`. Read **all report pages in order**,
starting at offset 0 (the default), then use the exact next offset returned by
each page until `[End of preview.]`. The `100000` above is illustrative: each
page contains at most 100000 characters and the plugin may adjust its boundary.
These offsets address report characters, not positions in a Java file. The
initial diff alone does not satisfy the plugin's full-report read requirement.
`get-preview` reads the saved report without recalculating the change.

The ID expires after **15 minutes** and is for **one use**; an application attempt
consumes it after the complete-report check, including an attempt refused for
source conflicts. Eclipse refuses an incomplete preview or affected sources
that changed or have unsaved editor changes before writing files. Save sources
before creating the preview and keep them unchanged until application. On an
expired, used or conflicted preview, inspect the error and create/read a new
preview as needed. The plugin preserves Eclipse refactoring undo.

All three commands use the existing serialized `eclipse-refactor` MCP session.
`--json` preserves the complete MCP result, including diffs, page instructions
and tool-error details. Failed applications are never replayed automatically;
after a transport failure the change may already have executed, so inspect
Eclipse before taking further action.

## Launch environments

```sh
eclipsectl launch create-java com.example.Main --name 'Main dev' --project example --type java --env PROFILE=dev
eclipsectl launch show 'Main dev'
eclipsectl launch env 'Main dev' --env PROFILE=test --remove OLD_VAR
eclipsectl launch env 'Main dev' --replace --clear --inherit-env false
eclipsectl run --configuration 'Main dev' --wait
eclipsectl debug run 'Main dev' --wait
eclipsectl coverage run 'Main dev' --wait
```

Types: `java`, `junit4`, `junit5`, `testng`, `auto`; omission delegates selection to Eclipse. Names conflict by default; `--name-conflict generate` requests a unique name.

Environment edits default to merge. `--replace` or `--mode replace` replaces the map; `--clear` sends an explicit empty map and requires replace mode. `--remove` cannot combine with replace mode. Omitting `--inherit-env` during an update preserves that setting; explicit `false` is retained. `launch show` masks values unless `--include-environment-values` is requested.

## Migration

| Previous interface | Rust interface |
|---|---|
| old executable | `eclipsectl` |
| `debug configs` | `launch list` |
| `debug config create-java/show/update-environment` | `launch create-java/show/env` |
| `format` / `editor format` / `source format` | `source format` |
| `remove lines` | `java remove-lines` |
| `gate reset` | `java reset-read-gate` |
| optional positional context | named options |
| JSON configuration | `config migrate --from ... --to ...` |
| MCP stdio server | removed; use the CLI |

New capabilities include `project mkdir`, `search replace`, JSON output and `doctor`. No old executable aliases are installed. Stop the old Python daemon before uninstalling it; Rust uses separate runtime storage.
