# Command inventory

Generated help and this inventory use the same registry in `src/operations.json`. All commands accept `--json`, `--verbose`, `--quiet` and connection overrides. Repeat options marked `...`; boolean options take `true` or `false` unless marked as flags.

| Command and arguments | MCP server | MCP tool |
|---|---|---|
| `eclipsectl workspace info` | `eclipse-ide` | `getWorkspaceInfo` |
| `eclipsectl project list` | `eclipse-ide` | `listProjects` |
| `eclipsectl project layout <project>` | `eclipse-ide` | `getProjectLayout` |
| `eclipsectl project properties <project>` | `eclipse-ide` | `getProjectProperties` |
| `eclipsectl project read <project> <path>` | `eclipse-ide` | `readProjectResource` |
| `eclipsectl project import <path>` | `eclipse-ide` | `importProject` |
| `eclipsectl project mkdir <project> <path>` | `eclipse-ide` | `createDirectories` |
| `eclipsectl code javadoc <fqn>` | `eclipse-ide` | `getJavaDoc` |
| `eclipsectl code source <class>` | `eclipse-ide` | `getSource` |
| `eclipsectl code hierarchy <class> <method>` | `eclipse-ide` | `getMethodCallHierarchy` |
| `eclipsectl build maven <project> <goals>...` | `eclipse-ide` | `runMavenBuild` |
| `eclipsectl build errors [--project <project>]` | `eclipse-ide` | `getCompilationErrors` |
| `eclipsectl test all` | `eclipse-ide` | `runAllTests` |
| `eclipsectl test package <project> <package>` | `eclipse-ide` | `runPackageTests` |
| `eclipsectl test class <project> <class>` | `eclipse-ide` | `runClassTests` |
| `eclipsectl test method <project> <class> <method>` | `eclipse-ide` | `runTestMethod` |
| `eclipsectl editor current` | `eclipse-ide` | `getCurrentlyOpenedFile` |
| `eclipsectl editor selection` | `eclipse-ide` | `getEditorSelection` |
| `eclipsectl editor list` | `eclipse-ide` | `getOpenEditors` |
| `eclipsectl editor open <file> [--line <line>] [--project <project>]` | `eclipse` | `openFile` |
| `eclipsectl console output [--name <name>]` | `eclipse-ide` | `getConsoleOutput` |
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
| `eclipsectl debug breakpoint add <file> <line> [--condition <condition>]` | `eclipse-debug` | `addBreakpoint` |
| `eclipsectl debug breakpoint remove <file> <line>` | `eclipse-debug` | `removeBreakpoint` |
| `eclipsectl debug breakpoint enable <file> <line>` | `eclipse-debug` | `enableBreakpoint` |
| `eclipsectl debug breakpoint disable <file> <line>` | `eclipse-debug` | `disableBreakpoint` |
| `eclipsectl debug breakpoint list` | `eclipse-debug` | `listBreakpoints` |
| `eclipsectl debug threads` | `eclipse-debug` | `listThreads` |
| `eclipsectl debug thread-select <name>` | `eclipse-debug` | `selectThread` |
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
| `eclipsectl debug stacktrace [--max-depth <max-depth>]` | `eclipse-debug` | `getStackTrace` |
| `eclipsectl debug variables [--frame <frame>]` | `eclipse-debug` | `getVariables` |
| `eclipsectl debug eval <expression> [--frame <frame>]` | `eclipse-debug` | `evaluateExpression` |
| `eclipsectl coverage run <configuration> [--wait]` | `eclipse-coverage` | `runCoverage` |
| `eclipsectl coverage report` | `eclipse-coverage` | `getCoverageReport` |
| `eclipsectl coverage class <class>` | `eclipse-coverage` | `getClassCoverage` |

`launch env` additionally accepts `--clear` (send an explicit empty map, requires replace mode) and `--replace` (shortcut for `--mode replace`).

Former Python `refresh_file` and `refresh_workspace` wrapper handlers returned placeholder messages. Use `refresh workspace <paths...>` for actual Eclipse refresh. The `formatCode` and `problems` route aliases are replaced by `formatFile` and `getCompilationErrors`, matching the former client.

CLI values are typed internally. The registry explicitly serializes legacy plugin number/boolean parameters as strings where its schemas require them, joins Maven goals with spaces, and joins file patterns with commas. `coverage report` has no project filter because the plugin accepts no arguments.
