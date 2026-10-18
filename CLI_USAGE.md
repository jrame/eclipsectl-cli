# eclipsectl CLI usage

The binary is `eclipsectl` (`eclipsectl.exe` on Windows). Every command has `--help`; [the full inventory](docs/COMMANDS.md) lists all bindings.

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
