# Companion plugin catalogue coverage

The CLI binds all 97 tools in the generated `docs/mcp-tools/*.md` catalogue of
`eclipsectl-mcp-plugin`, checked against commit
`e6078807b69b7eae7a6f953022263506e5dd8133`. The tool identity snapshot in
`tests/catalog_tools.txt` keeps the contract test independent of a neighbouring
plugin checkout. Update it when intentionally adopting a newer catalogue.
See [COMMANDS.md](COMMANDS.md) for every command binding.

## Additional commands

```sh
eclipsectl debug processes
eclipsectl code format --source 'class Example{int value;}' --project example
eclipsectl problems panel --severity warning --max 25 --skip-unknown-location true
```

`debug processes` lists active launches in run, debug and coverage modes.
`code format` returns formatted Java text using workspace or project formatter
settings. It accepts `--source` or UTF-8 standard input and does not write a file.
For example, in PowerShell: `Get-Content -Raw Example.java | eclipsectl code format`.
Use `source format <file>` to format an existing Eclipse resource.

`problems panel` calls `eclipse/problems`, which gathers workspace problem markers
and can skip unknown locations. Its severity values are `error`, `warning`, `all`;
the plugin defaults to errors, 10 details and no location filtering, and caps
details at 500. `problems list` continues to call `getCompilationErrors`, supports
`--project`, and retains its existing uppercase severity values and defaults.

## Additional optional parameters

| Commands | Options and plugin defaults when omitted |
|---|---|
| `debug breakpoint add/remove/enable/disable` | `--project` assists file resolution; omission keeps existing resolution |
| `debug breakpoint add` | `--hit-count` triggers after N hits (positive Java integer); `--condition` remains available |
| `debug stacktrace` | `--package-filter` restricts packages; `--skip-framework true/false` defaults to true; `--max-depth` remains available (default 10) |
| `debug threads`, `debug thread-select` | `--session` selects a debug session; omission uses the focused or first active session |
| `console output` | `--max-lines` defaults to 100; `--include-all-consoles true/false` defaults to false; `--name` remains available |
| `code hierarchy` | `--signature` is a comma-separated list of JDT parameter type signatures to select an overload; `--max-depth` defaults to 3 |
| `project import` | `--project` provides a custom project name; omission preserves the plugin's inferred name |
| `test all` | `--project` limits the run to a project; omission runs workspace tests |
| `test all/class/package/method` | `--timeout` is the plugin wait in seconds, default 60; zero also selects 60 |
| `build maven` | Repeat `--profile` for profiles, sent as one comma-separated string; `--timeout 0` means unlimited and is the plugin default |
| `build errors` | `--severity ERROR/WARNING/ALL` defaults to ALL; `--max` defaults to 100 |

New optional arguments are omitted from MCP requests unless supplied, preserving
the Java service defaults. Boolean options take explicit `true` or `false`.
Numbers and booleans are sent as strings wherever the catalogue schema says
`string`, even when the Java parameter type is numeric or boolean. New numeric
options are limited to the Java integer range to avoid conversion overflow.

The plugin advertises the Maven timeout in seconds, but at the referenced commit
`MavenService.runMavenBuild` passes it to `TimeUnit.MINUTES.toMillis`. The CLI sends
the value unchanged; positive values therefore wait in minutes with that plugin.
Its result reports that the build started rather than a compilation-success
report. No plugin code is changed here. Test timeouts are seconds.

Plugin waits and the global client `--operation-timeout` are separate: increasing
`--timeout` on these Maven/test commands does not extend the MCP request budget.
Set `--operation-timeout` explicitly if a longer client wait is needed.

Validation uses command parsing, exact request argument contracts, catalogue
identity coverage and local simulated transport tests. It does not run builds,
tests, imports, formatting or breakpoint mutations in the user's Eclipse.
