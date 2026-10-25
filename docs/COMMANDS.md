# Command inventory

Generated help and this inventory use the same registry in `src/operations.json`. All commands accept `--json`, `--verbose`, `--quiet` and connection overrides. Repeat options marked `...`; boolean options take `true` or `false` unless marked as flags.

| Command and arguments | MCP server | MCP tool |
|---|---|---|
| `eclipsectl debug processes` | `eclipse-debug` | `listProcesses` |
| `eclipsectl code format [--source <source>] [--project <project>]` | `eclipse-ide` | `formatCode` |
| `eclipsectl problems panel [--severity <error/warning/all>] [--max <max>] [--skip-unknown-location <true/false>]` | `eclipse` | `problems` |
| `eclipsectl workspace info` | `eclipse-ide` | `getWorkspaceInfo` |
| `eclipsectl workspace wait --idle [--timeout <seconds>] [--poll-interval <ms>]` | `eclipse-ide` | `getWorkspaceState` (polled) |
| `eclipsectl project list` | `eclipse-ide` | `listProjects` |
| `eclipsectl project layout <project>` | `eclipse-ide` | `getProjectLayout` |
| `eclipsectl project properties <project>` | `eclipse-ide` | `getProjectProperties` |
| `eclipsectl project read <project> <path>` | `eclipse-ide` | `readProjectResource` |
| `eclipsectl project import <path> [--project <project>]` | `eclipse-ide` | `importProject` |
| `eclipsectl project mkdir <project> <path>` | `eclipse-ide` | `createDirectories` |
| `eclipsectl code javadoc <fqn>` | `eclipse-ide` | `getJavaDoc` |
| `eclipsectl code source <class>` | `eclipse-ide` | `getSource` |
| `eclipsectl code hierarchy <class> <method> [--signature <signature>] [--max-depth <max-depth>]` | `eclipse-ide` | `getMethodCallHierarchy` |
| `eclipsectl build maven <project> <goals>... [--profile <profile>...] [--timeout <timeout>]` | `eclipse-ide` | `runMavenBuild` |
| `eclipsectl build eclipse [--project <name>]... [--build-kind incremental\|full] [--include-dependents true\|false] [--timeout <1..3600>] [--max-problems <1..500>]` | `eclipse-ide` | `buildProjects` |
| `eclipsectl build errors [--project <project>] [--severity <severity>] [--max <max>]` | `eclipse-ide` | `getCompilationErrors` |
| `eclipsectl build wait [--timeout <seconds>] [--poll-interval <ms>]` | `eclipse-ide` | `getWorkspaceState` (polled) |
| `eclipsectl test all [--project <project>] [--timeout <seconds>]` | `eclipse-ide` | `runAllTests` |
| `eclipsectl test package <project> <package> [--timeout <seconds>]` | `eclipse-ide` | `runPackageTests` |
| `eclipsectl test class <project> <class> [--timeout <seconds>]` | `eclipse-ide` | `runClassTests` |
| `eclipsectl test method <project> <class> <method> [--timeout <seconds>]` | `eclipse-ide` | `runTestMethod` |
| `eclipsectl editor current` | `eclipse-ide` | `getCurrentlyOpenedFile` |
| `eclipsectl editor selection` | `eclipse-ide` | `getEditorSelection` |
| `eclipsectl editor list` | `eclipse-ide` | `getOpenEditors` |
| `eclipsectl editor open <file> [--line <line>] [--project <project>]` | `eclipse` | `openFile` |
| `eclipsectl console output [--name <name>] [--max-lines <max-lines>] [--include-all-consoles <true/false>]` | `eclipse-ide` | `getConsoleOutput` |
| `eclipsectl search text <pattern> [--pattern-file <pattern-file>...]` | `eclipse-ide` | `fileSearch` |
| `eclipsectl search regex <pattern> [--pattern-file <pattern-file>...]` | `eclipse-ide` | `fileSearchRegExp` |
| `eclipsectl search files [--pattern-file <pattern-file>...] [--max <max>]` | `eclipse-ide` | `findFiles` |
| `eclipsectl search replace <text> <replacement> [--pattern-file <pattern-file>...]` | `eclipse-ide` | `searchAndReplace` |
| `eclipsectl search type <name>` | `eclipse` | `open_type` |
| `eclipsectl search references <element-type> <element-name> [--project <project>] [--scope <scope>] [--include-subtypes]` | `eclipse` | `findReferences` |
| `eclipsectl search hierarchy <class>` | `eclipse` | `getTypeHierarchy` |
| `eclipsectl search call-hierarchy <class> <method> [--signature <signature>]` | `eclipse` | `call_hierarchy` |
| `eclipsectl search implementations <class> [--method <method>]` | `eclipse` | `getImplementations` |
| `eclipsectl problems list [--severity <severity>] [--max <max>] [--project <project>]` | `eclipse-ide` | `getCompilationErrors` |
| `eclipsectl problems quickfixes <file> [--project <project>] [--line <line>] [--problem-id <problem-id>]` | `eclipse` | `getQuickFixes` |
| `eclipsectl problems apply-fix <file> <line> <index> [--project <project>]` | `eclipse` | `applyQuickFix` |
| `eclipsectl problems markers <file> [--project <project>]` | `eclipse` | `getMarkers` |
| `eclipsectl maven list` | `eclipse-ide` | `listMavenProjects` |
| `eclipsectl maven effective-pom <project>` | `eclipse-ide` | `getEffectivePom` |
| `eclipsectl maven dependencies <project>` | `eclipse-ide` | `getProjectDependencies` |
| `eclipsectl maven find-tests <project>` | `eclipse-ide` | `findTestClasses` |
| `eclipsectl refactor rename-type <file> <name> [--project <project>]` | `eclipse-refactor` | `refactorRenameJavaType` |
| `eclipsectl refactor move-type <file> <package> [--project <project>]` | `eclipse-refactor` | `refactorMoveJavaType` |
| `eclipsectl refactor rename-file <file> <name> [--project <project>]` | `eclipse-refactor` | `renameFile` |
| `eclipsectl refactor rename-package <package> <name> [--project <project>]` | `eclipse-refactor` | `refactorRenamePackage` |
| `eclipsectl refactor move-resource <source> <target> [--project <project>]` | `eclipse-refactor` | `moveResource` |
| `eclipsectl refactor extract-method <file> <start-line> <end-line> <name> [--project <project>] [--visibility <visibility>]` | `eclipse-refactor` | `refactorExtractMethod` |
| `eclipsectl refactor rename-method <file> <offset> <name> [--project <project>] [--preview <true|false>]` | `eclipse-refactor` | `refactorRenameMethod` |
| `eclipsectl refactor rename-field <file> <offset> <name> [--project <project>] [--preview <true|false>]` | `eclipse-refactor` | `refactorRenameField` |
| `eclipsectl refactor rename-variable <file-or-class> <old-name> <name> [--line <line>] [--method <method>] [--project <project>] [--preview <true|false>]` | `eclipse-refactor` | `refactorRenameVariable` |
| `eclipsectl refactor change-signature <file> <offset> [--name <name>] [--return-type <return-type>] [--visibility <visibility>] [--parameters <parameters>] [--project <project>] [--preview <true|false>]` | `eclipse-refactor` | `refactorChangeSignature` |
| `eclipsectl refactor extract-variable <file> <offset> <length> <name> [--replace-all <replace-all>] [--final <final>] [--project <project>] [--preview <true|false>]` | `eclipse-refactor` | `refactorExtractVariable` |
| `eclipsectl refactor extract-constant <file> <offset> <length> <name> [--visibility <visibility>] [--replace-all <replace-all>] [--project <project>] [--preview <true|false>]` | `eclipse-refactor` | `refactorExtractConstant` |
| `eclipsectl refactor inline-method <file> <offset> [--all <all>] [--delete-source <delete-source>] [--project <project>] [--preview <true|false>]` | `eclipse-refactor` | `refactorInlineMethod` |
| `eclipsectl refactor inline-variable <file> <offset> [--project <project>] [--preview <true|false>]` | `eclipse-refactor` | `refactorInlineVariable` |
| `eclipsectl refactor get-preview <preview-id> [--offset <offset>]` | `eclipse-refactor` | `refactorGetPreview` |
| `eclipsectl refactor apply-preview <preview-id>` | `eclipse-refactor` | `refactorApplyPreview` |
| `eclipsectl codegen getters-setters <file> [--project <project>] [--class <class>] [--field <field>...] [--getters <getters>] [--setters <setters>] [--insertion-point <insertion-point>]` | `eclipse-codegen` | `generateGettersSetters` |
| `eclipsectl codegen tostring <file> [--project <project>] [--class <class>] [--field <field>...] [--tostring <tostring>] [--equals <equals>] [--hashcode <hashcode>]` | `eclipse-codegen` | `generateToStringEqualsHashCode` |
| `eclipsectl codegen implement <file> [--project <project>] [--class <class>] [--method <method>...]` | `eclipse-codegen` | `implementOverrideMethods` |
| `eclipsectl source organize-imports <file> [--project <project>]` | `eclipse-source` | `organizeImports` |
| `eclipsectl source organize-package <package> [--project <project>]` | `eclipse-source` | `organizeImportsInPackage` |
| `eclipsectl source format <file> [--project <project>] [--lines <lines>]` | `eclipse-source` | `formatFile` |
| `eclipsectl java read <file> [--project <project>] [--class <class>]` | `eclipse-source` | `readJava` |
| `eclipsectl java write <file> [--project <project>] [--class <class>] [--source <source>]` | `eclipse-source` | `writeJava` |
| `eclipsectl java remove <file> [--project <project>] [--class <class>] [--members <members>]` | `eclipse-source` | `removeJava` |
| `eclipsectl java remove-lines <file> <lines> [--project <project>]` | `eclipse-source` | `removeLines` |
| `eclipsectl java reset-read-gate [--file <file>] [--project <project>]` | `eclipse-source` | `resetReadGate` |
| `eclipsectl run [--configuration <configuration>] [--wait]` | `eclipse` | `run` |
| `eclipsectl refresh workspace <paths>...` | `eclipse` | `refresh` |
| `eclipsectl refresh save <paths>...` | `eclipse` | `refreshAndSave` |
| `eclipsectl debug breakpoint add <file> <line> [--condition <condition>] [--project <project>] [--hit-count <hit-count>]` | `eclipse-debug` | `addBreakpoint` |
| `eclipsectl debug breakpoint remove <file> <line> [--project <project>]` | `eclipse-debug` | `removeBreakpoint` |
| `eclipsectl debug breakpoint enable <file> <line> [--project <project>]` | `eclipse-debug` | `enableBreakpoint` |
| `eclipsectl debug breakpoint disable <file> <line> [--project <project>]` | `eclipse-debug` | `disableBreakpoint` |
| `eclipsectl debug breakpoint list` | `eclipse-debug` | `listBreakpoints` |
| `eclipsectl debug threads [--session <session>]` | `eclipse-debug` | `listThreads` |
| `eclipsectl debug thread-select <name> [--session <session>]` | `eclipse-debug` | `selectThread` |
| `eclipsectl debug run <configuration> [--wait]` | `eclipse-debug` | `runDebug` |
| `eclipsectl launch list` | `eclipse-debug` | `listRunConfigurations` |
| `eclipsectl launch create-java <class> [--name <name>] [--project <project>] [--type <type>] [--args <args>] [--vm-args <vm-args>] [--cwd <cwd>] [--env <env>...] [--inherit-env <inherit-env>] [--name-conflict <name-conflict>]` | `eclipse-debug` | `createJavaLaunchConfiguration` |
| `eclipsectl launch show <configuration> [--include-environment-values]` | `eclipse-debug` | `getLaunchConfiguration` |
| `eclipsectl launch env <configuration> [--env <env>...] [--remove <remove>...] [--mode <mode>] [--inherit-env <inherit-env>]` | `eclipse-debug` | `updateLaunchEnvironment` |
| `eclipsectl debug step-into` | `eclipse-debug` | `stepInto` |
| `eclipsectl debug step-over` | `eclipse-debug` | `stepOver` |
| `eclipsectl debug step-return` | `eclipse-debug` | `stepReturn` |
| `eclipsectl debug resume` | `eclipse-debug` | `resume` |
| `eclipsectl debug suspend` | `eclipse-debug` | `suspend` |
| `eclipsectl debug terminate` | `eclipse-debug` | `terminate` |
| `eclipsectl debug status` | `eclipse-debug` | `getDebugStatus` |
| `eclipsectl debug wait --suspended [--timeout <seconds>] [--poll-interval <ms>]` | `eclipse-debug` | `getDebugStatus` (polled) |
| `eclipsectl debug stacktrace [--max-depth <max-depth>] [--package-filter <package-filter>] [--skip-framework <true/false>]` | `eclipse-debug` | `getStackTrace` |
| `eclipsectl debug variables [--frame <frame>]` | `eclipse-debug` | `getVariables` |
| `eclipsectl debug eval <expression> [--frame <frame>]` | `eclipse-debug` | `evaluateExpression` |
| `eclipsectl coverage run <configuration> [--wait]` | `eclipse-coverage` | `runCoverage` |
| `eclipsectl coverage report` | `eclipse-coverage` | `getCoverageReport` |
| `eclipsectl coverage class <class>` | `eclipse-coverage` | `getClassCoverage` |

`launch env` additionally accepts `--clear` (send an explicit empty map, requires replace mode) and `--replace` (shortcut for `--mode replace`).

`refactor rename-variable` requires `--line` (1-based) and/or `--method` with the
existing variable name. The alternative `<file> <name> --offset <offset>` and
legacy `<file> <offset> <name>` forms select a UTF-16 offset directly. Offset
selection is exclusive with named selectors. All forms accept `--project` and
`--preview true|false`. Named selection requires the updated plugin and refuses
ambiguous files or variable bindings. This tool sends offsets and lines as JSON
integers.

Former Python `refresh_file` and `refresh_workspace` wrapper handlers returned placeholder messages. Use `refresh workspace <paths...>` for actual Eclipse refresh. The `formatCode` and `problems` route aliases are replaced by `formatFile` and `getCompilationErrors`, matching the former client.

CLI values are typed internally. The registry explicitly serializes legacy plugin number/boolean parameters as strings where its schemas require them, joins Maven goals with spaces, and joins file patterns with commas. `coverage report` has no project filter because the plugin accepts no arguments.

Exception: `--wait` always sends a JSON boolean (`waitForCompletion`) for run, debug and coverage. Older plugin schemas advertise a string, but their Java methods require a Boolean and reject string values with `argument type mismatch`.
