# CLI Eclipse

## Vue d'ensemble

**eclipsectl-cli** fournit une **interface CLI** (`eclipse` command) pour contrôler Eclipse IDE (Windows) depuis WSL/Linux via le plugin **eclipsectl-mcp** (gradusnikov/eclipse-chatgpt-plugin).

**Utilisation recommandée** : CLI pour les Skills Claude Code, pas MCP server.

## Architecture

```
┌─── WSL / Linux ────────────────────┐
│ Claude Code Skills                 │
│   ↓ bash commands                  │
│ eclipse project list               │
│ eclipse test all                   │
│ eclipse editor current             │
│   - Conversion chemins WSL↔Windows │
│   - /mnt/c/... ↔ C:\...           │
│   - Auth Bearer token              │
└────────────────────────────────────┘
          ↓ HTTP StreamableHTTP
┌─── Windows ────────────────────────┐
│ Eclipse IDE                        │
│   ↓                                │
│ eclipsectl-mcp Plugin (:8765)            │
│   - HTTP MCP Server                │
│   - 70 outils MCP disponibles      │
│   - 8 serveurs MCP                 │
│   - eclipse-ide, eclipse,          │
│     eclipse-codegen, eclipse-      │
│     refactor, eclipse-source,      │
│     eclipse-debug, eclipse-coverage│
└────────────────────────────────────┘
```

## Utilisation (CLI - Recommandé)

### Options globales

```bash
eclipse --help              # Aide et liste des commandes
eclipse --verbose <cmd>     # Afficher logs détaillés (debug)
eclipse --quiet <cmd>       # Supprimer tous les logs (sauf erreurs)
```

**Par défaut**, les logs des bibliothèques (httpx, mcp) sont supprimés pour une sortie propre.

### ⚡ Performance : Daemon automatique

Le CLI utilise un **daemon avec connexion persistante** pour éliminer la latence :
- **Premier appel** : ~4s (démarre daemon + init MCP)
- **Appels suivants** : **<0.1s** (réutilise connexion via socket Unix)

Le daemon se lance automatiquement et s'arrête après 5 min d'inactivité.

```bash
# Gestion daemon (optionnel)
eclipse daemon status    # Vérifier si daemon actif
eclipse daemon stop      # Arrêter daemon
eclipse daemon restart   # Redémarrer daemon
```

**Documentation détaillée** : [DAEMON.md](DAEMON.md)

### Commandes disponibles

**71 commandes CLI disponibles** (97% de couverture des 73 outils MCP eclipsectl-mcp)

```bash
# === Projets ===
eclipse project list                       # Liste tous les projets
eclipse project layout <nom>               # Structure d'un projet
eclipse project properties <nom>           # Propriétés du projet
eclipse project read <nom> <path>          # Lire fichier projet
eclipse project import <path>              # Importer un projet

# === Code Analysis ===
eclipse code javadoc <file> <offset>      # JavaDoc à l'offset
eclipse code source <file> <offset>       # Code source à l'offset
eclipse code hierarchy <file> <offset>    # Hiérarchie d'appels

# === Build & Tests ===
eclipse build maven <project> <goals...>  # Build Maven
eclipse build errors [project]            # Erreurs de compilation
eclipse test all                          # Tous les tests
eclipse test package <project> <package>  # Tests d'un package
eclipse test class <project> <class>      # Tests d'une classe
eclipse test method <project> <class> <method>  # Test spécifique

# === Éditeur ===
eclipse editor current                    # Fichier actif
eclipse editor selection                  # Sélection courante
eclipse editor format <file>              # Formater code

# === Console ===
eclipse console output [name]             # Output console

# === Search  ===
eclipse search text <pattern> [filePattern...]   # Recherche texte
eclipse search regex <pattern> [filePattern...]  # Recherche regex
eclipse search files [filePattern...]            # Trouver fichiers
eclipse search type <typeName>                   # Ouvrir type
eclipse search references <elemType> <elemName>  # Trouver références
eclipse search hierarchy <fullyQualifiedClass>   # Hiérarchie de types
eclipse search call-hierarchy <class> <method>   # Hiérarchie d'appels
eclipse search implementations <class>           # Trouver implémentations

# === Problems  ===
eclipse problems list [severity] [max]           # Liste problèmes
eclipse problems quickfixes <file>               # Obtenir quick fixes
eclipse problems apply-fix <file> <line> <idx>   # Appliquer quick fix
eclipse problems markers <file>                  # Obtenir markers

# === Maven  ===
eclipse maven list                               # Lister projets Maven
eclipse maven effective-pom <project>            # Obtenir POM effectif
eclipse maven dependencies <project>             # Dépendances projet
eclipse maven find-tests <project>               # Trouver classes de test

# === Refactor  ===
eclipse refactor rename-type <file> <newName>    # Renommer classe/interface
eclipse refactor move-type <file> <package>      # Déplacer type vers package
eclipse refactor rename-package <pkg> <newPkg>   # Renommer package
eclipse refactor extract-method <file> <start> <end> <name>  # Extraire méthode
eclipse refactor rename-file <file> <newName>    # Renommer fichier
eclipse refactor move-resource <src> <target>    # Déplacer ressource

# === Code Generation  ===
eclipse codegen getters-setters <file>           # Générer getters/setters
eclipse codegen tostring <file>                  # Générer toString/equals/hashCode
eclipse codegen implement <file>                 # Implémenter/overrider méthodes

# === Source Operations  ===
eclipse source organize-imports <file>           # Organiser imports
eclipse source organize-package <package>        # Organiser imports package
eclipse source format <file>                     # Formater fichier

# === Run & Refresh ===
eclipse run [configuration]                      # Exécuter configuration
eclipse refresh workspace <paths...>             # Rafraîchir workspace
eclipse refresh save <paths...>                  # Rafraîchir et sauvegarder

# === Debug  ===
eclipse debug breakpoint add <file> <line> [-c <condition>]  # Ajouter breakpoint
eclipse debug breakpoint remove <file> <line>    # Supprimer breakpoint
eclipse debug breakpoint list                    # Lister breakpoints
eclipse debug breakpoint enable <file> <line>    # Activer breakpoint
eclipse debug breakpoint disable <file> <line>   # Désactiver breakpoint
eclipse debug threads                            # Lister threads
eclipse debug thread-select <threadId>           # Sélectionner thread
eclipse debug run <configuration>             # Lancer debug config
eclipse debug configs                            # Lister debug configs
eclipse debug config create-java <mainClass>     # Créer config Java Application
eclipse debug config show <configuration>        # Inspecter config (environnement masqué)
eclipse debug config update-environment <config> # Modifier l'environnement
eclipse debug step-into                          # Step into
eclipse debug step-over                          # Step over
eclipse debug step-return                        # Step return
eclipse debug resume                             # Reprendre exécution
eclipse debug suspend                            # Suspendre exécution
eclipse debug terminate                          # Terminer session debug
eclipse debug stacktrace [-t <threadId>]         # Stack trace
eclipse debug variables [-f <frameId>]           # Variables du scope
eclipse debug eval <expression> [-f <frameId>]   # Évaluer expression
eclipse debug status                             # Statut debug

# === Coverage  ===
eclipse coverage run <configuration>          # Lancer avec couverture
eclipse coverage report [project]                # Rapport de couverture
eclipse coverage class <project> <class>         # Couverture d'une classe

# === Daemon ===
eclipse daemon status                     # Statut daemon
eclipse daemon stop                       # Arrêter daemon
eclipse daemon restart                    # Redémarrer daemon
```

**Exemples** :
```bash
# Basiques
eclipse project list
eclipse test all
eclipse code javadoc /mnt/c/workspace/User.java 150
eclipse editor current

# Nouveaux (search, problems, refactor, etc.)
eclipse search text "TODO" "*.java"
eclipse search type "ArrayList"
eclipse problems list error 10
eclipse maven list
eclipse refactor rename-type /mnt/c/workspace/User.java UserEntity
eclipse codegen getters-setters /mnt/c/workspace/User.java
eclipse source organize-imports /mnt/c/workspace/User.java
```

### Dans Claude Code Skills

Un Skill complet est fourni dans `.claude/skills/eclipse/SKILL.md`. Pour l'installer :

```bash
mkdir -p ~/.claude/skills/eclipse
cp .claude/skills/eclipse/SKILL.md ~/.claude/skills/eclipse/SKILL.md
```

Puis dans Claude Code :
```bash
/eclipse    # Affiche toutes les commandes disponibles
```

Le Skill contient toutes les commandes organisées par catégorie avec exemples pratiques.

**Documentation complète** : [CLI_USAGE.md](CLI_USAGE.md)

## Prérequis

### Windows (Eclipse IDE)
1. Eclipse IDE installé
2. Plugin eclipsectl-mcp installé (Eclipse Marketplace)
3. HTTP MCP Server activé dans Préférences → eclipsectl-mcp
4. Bearer token généré

### WSL (Claude Code)
1. Python 3.10+
2. Dependencies: `netcat`, `jq` (`apt install netcat jq`)
3. eclipsectl-cli installé (`bash install.sh`)
4. Configuration avec token eclipsectl-mcp

## Configuration

### 1. eclipsectl-mcp (Windows)
- Préférences → eclipsectl-mcp → HTTP MCP Server
- Enable: ✅
- Port: `8124`
- Hostname: `localhost`
- Générer Bearer token → sauvegarder

### 2. eclipsectl-cli (WSL)

**Priorité de configuration** :
1. `$CWD/.config/eclipsectl-cli/config.json` (config locale du projet)
2. `~/.config/eclipsectl-cli/config.json` (config globale utilisateur)
3. Variables d'environnement
4. Valeurs par défaut

**Config globale** `~/.config/eclipsectl-cli/config.json`:
```json
{
  "eclipse_url": "http://localhost:8124",
  "bearer_token": "VOTRE_TOKEN_ICI"
}
```

**Config locale** (optionnel, pour dev) `$PROJECT/.config/eclipsectl-cli/config.json`:
```json
{
  "eclipse_url": "http://169.254.x.x:8765",
  "bearer_token": "TOKEN_DEV",
  "timeout": 30,
  "verbose": false
}
```

**Variables d'environnement** (optionnel):
```bash
export ECLIPSE_URL="http://localhost:8124"
export ECLIPSE_TOKEN="votre_token"
```

### 3. Claude Code

Ajouter à `.mcp.json`:
```json
{
  "mcpServers": {
    "eclipse": {
      "command": "eclipsectl-cli"
    }
  }
}
```

## Outils MCP disponibles

Voir [ECLIPSECTL_MCP_TOOLS.md](docs/ECLIPSECTL_MCP_TOOLS.md) pour référence complète.

### eclipse-ide (25 outils)
- **Code Analysis**: `getJavaDoc`, `getSource`, `formatCode`, `getMethodCallHierarchy`
- **Build & Maven**: `runMavenBuild`, `getEffectivePom`, `listMavenProjects`, `getProjectDependencies`, `getCompilationErrors`
- **Tests**: `runAllTests`, `runPackageTests`, `runClassTests`, `runTestMethod`, `findTestClasses`
- **Projet**: `listProjects`, `getProjectLayout`, `getProjectProperties`, `readProjectResource`, `importProject`
- **Éditeur**: `getCurrentlyOpenedFile`, `getEditorSelection`
- **Console**: `getConsoleOutput`
- **Search**: `fileSearch`, `fileSearchRegExp`, `findFiles`

### eclipse (12 outils)
- **Search & Navigation**: `open_type`, `call_hierarchy`, `findReferences`, `getTypeHierarchy`, `getImplementations`
- **Problems & Fixes**: `problems`, `getQuickFixes`, `applyQuickFix`, `getMarkers`
- **Run**: `run`
- **Refresh**: `refresh`, `refreshAndSave`

### eclipse-codegen (3 outils)
- **Code Generation**: `generateGettersSetters`, `generateToStringEqualsHashCode`, `implementOverrideMethods`

### eclipse-refactor (6 outils)
- **Refactoring**: `refactorRenameJavaType`, `refactorMoveJavaType`, `refactorRenamePackage`, `refactorExtractMethod`, `renameFile`, `moveResource`

### eclipse-source (3 outils)
- **Source Operations**: `organizeImports`, `organizeImportsInPackage`, `formatFile`

### eclipse-debug (22 outils)
- **Breakpoints**: `addBreakpoint`, `removeBreakpoint`, `listBreakpoints`, `enableBreakpoint`, `disableBreakpoint`
- **Threads**: `listThreads`, `selectThread`
- **Launch**: `runDebug`, `listRunConfigurations`, `createJavaLaunchConfiguration`, `getLaunchConfiguration`, `updateLaunchEnvironment`
- **Stepping**: `stepInto`, `stepOver`, `stepReturn`
- **Execution**: `resume`, `suspend`, `terminate`
- **Inspection**: `getStackTrace`, `getVariables`, `evaluateExpression`, `getDebugStatus`

### eclipse-coverage (3 outils)
- **Coverage**: `runCoverage`, `getCoverageReport`, `getClassCoverage`

**Total : 73 outils MCP disponibles (8 serveurs)**

## Fonctionnalités

### ✅ Disponible (73 outils MCP)
- **Code Analysis** : JavaDoc, source code, call hierarchy, method call hierarchy, type hierarchy, implementations
- **Build & Tests** : Maven builds (goals, effective POM, dependencies), JUnit tests (all/package/class/method), compilation errors
- **Refactoring** : rename type/package, move type, extract method
- **Code Generation** : getters/setters, toString/equals/hashCode, implement/override methods
- **Source Operations** : organize imports, format code/file
- **Search** : open type, find references, file search (plain text & regex), find files
- **Problems & Fixes** : get problems/markers, get/apply quick fixes
- **Editor** : current file, selection, console output
- **Project**: list/layout/properties, read resources, Maven projects, import project
- **Refresh**: workspace refresh, refresh & save
- **Debugging** : breakpoints (add/remove/list/enable/disable), threads, stepping (into/over/return), resume/suspend/terminate, stack trace, variables, expression evaluation, debug status
- **Coverage** : launch with coverage, coverage report, class coverage

### ⚠️ Limitations
- **Inline refactoring** : Inline method/variable not supported
- **Multi-file refactorings** : Some advanced refactorings may have limited support

## Conversion chemins

eclipsectl-cli convertit automatiquement les chemins entre WSL et Windows :

| WSL | Windows |
|-----|---------|
| `/mnt/c/projects/MyApp/src/Main.java` | `C:\projects\MyApp\src\Main.java` |
| `/mnt/d/workspace/Test.java` | `D:\workspace\Test.java` |

**Important** : Projets doivent être sur `/mnt/c/` ou `/mnt/d/` (drives Windows accessibles depuis WSL).

## Structure projet

```
src/eclipsectl_cli/
├── __init__.py
├── cli.py              # Point d'entrée
├── server.py           # Serveur MCP
├── config.py           # Configuration
├── eclipse_client.py   # Client HTTP eclipsectl-mcp
├── tools.py            # Définitions outils MCP
└── path_converter.py   # Conversion chemins WSL↔Windows
```

## Développement

### Installation dev
```bash
# IMPORTANT: Use install.sh, NOT pip install -e . directly
bash install.sh

# Or with dev dependencies:
pip install -e ".[dev]"
bash install.sh  # Still needed to copy bin/eclipse
```

**Why install.sh?** Setup.py cmdclass hooks don't work with editable installs (PEP 660). The install.sh script manually copies `bin/eclipse` to the virtualenv bin directory.

### Tests
```bash
pytest
pytest -m unit  # tests unitaires seulement
```

### Lancement local
```bash
python -m eclipsectl_cli
```

## Dépendances
- `mcp>=1.0.0` - Model Context Protocol
- `httpx>=0.27.0` - Client HTTP async
- `requests>=2.31.0` - Client HTTP sync (fallback)

## Documentation

- [ANALYSIS.md](docs/ANALYSIS.md) - Analyse complète des options
- [ECLIPSECTL_MCP_TOOLS.md](docs/ECLIPSECTL_MCP_TOOLS.md) - Référence outils eclipsectl-mcp
- [README.md](README.md) - Documentation principale

## Évolutions futures

Voir [ANALYSIS.md](docs/ANALYSIS.md) section "Évolution future possible".

**Options considérées** :
- JavaLens-MCP (standalone) pour refactorings avancés
- Plugin custom pour launch configurations
- Support concurrence (plusieurs instances)
