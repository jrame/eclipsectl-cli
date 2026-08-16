# Eclipse CLI - Guide d'utilisation

**eclipsectl-cli** fournit une interface en ligne de commande pour contrôler Eclipse IDE via le plugin eclipsectl-mcp.

## Installation

Depuis le wheel publié dans une GitHub Release :

```bash
pipx install ./eclipsectl_cli-0.1.0-py3-none-any.whl
```

Cela installe dans un environnement isolé le paquet Python, ses dépendances et
la commande Bash `eclipse` dans votre `PATH`.

Pour le développement depuis les sources :

```bash
python3 -m venv .venv
source .venv/bin/activate
bash install.sh
```

## Configuration

### Fichier de configuration : `~/.config/eclipsectl-cli/config.json`

```json
{
  "eclipse_url": "http://169.254.83.107:8765",
  "bearer_token": "votre-token-ici"
}
```

### Variables d'environnement

```bash
export ECLIPSE_URL="http://169.254.83.107:8765"
export ECLIPSE_TOKEN="votre-token-ici"
```

**Priorité** : Variables d'env > Fichier config > Défauts

## Utilisation

### Vue d'ensemble

```bash
eclipse --help
eclipse --version
```

### 🏠 Commandes Workspace

```bash
# Informations sur le workspace (emplacement, projets, versions Java/Eclipse)
eclipse workspace info
```

### 📂 Commandes Projet

```bash
# Lister tous les projets
eclipse project list

# Structure du projet
eclipse project layout <nom-projet>

# Propriétés du projet
eclipse project properties <nom-projet>

# Importer un projet dans le workspace
eclipse project import <chemin>
```

**Exemples** :
```bash
eclipse project list
eclipse project layout eclipse-mcp
eclipse project properties eclipse-mcp
eclipse project import /mnt/c/projects/MonProjet
```

### 📝 Commandes Code

```bash
# JavaDoc pour symbole
eclipse code javadoc <fichier> <offset>

# Code source du symbole
eclipse code source <fichier> <offset>

# Hiérarchie d'appels
eclipse code hierarchy <fichier> <offset>
```

**Exemples** :
```bash
# Obtenir JavaDoc à l'offset 123 dans Main.java
eclipse code javadoc /mnt/c/projects/MyApp/src/Main.java 123

# Code source du symbole
eclipse code source /mnt/c/projects/MyApp/src/Main.java 456

# Hiérarchie d'appels de la méthode
eclipse code hierarchy /mnt/c/projects/MyApp/src/Main.java 789
```

**Note** : Les chemins WSL (`/mnt/c/...`) sont automatiquement convertis en chemins Windows.

### 🔨 Commandes Build

```bash
# Build Maven
eclipse build maven <projet> <goals...>

# Erreurs de compilation
eclipse build errors [projet]
```

**Exemples** :
```bash
# Build avec Maven
eclipse build maven eclipse-mcp clean install

# Build avec plusieurs goals
eclipse build maven MyApp clean compile test

# Erreurs de compilation (tous les projets)
eclipse build errors

# Erreurs d'un projet spécifique
eclipse build errors eclipse-mcp
```

### ✅ Commandes Test

```bash
# Tous les tests (tous projets Java ou projet spécifique)
eclipse test all [--project <projet>]

# Tests d'un package
eclipse test package <projet> <package>

# Tests d'une classe
eclipse test class <projet> <classe>

# Test d'une méthode spécifique
eclipse test method <projet> <classe> <méthode>
```

**Exemples** :
```bash
# Lancer tous les tests de tous les projets Java
eclipse test all

# Lancer tous les tests d'un projet spécifique
eclipse test all --project eclipse-mcp

# Tests d'un package
eclipse test package eclipse-mcp com.myapp.services

# Tests d'une classe
eclipse test class eclipse-mcp com.myapp.MyTest

# Test spécifique
eclipse test method eclipse-mcp com.myapp.MyTest testSomething
```

### 📄 Commandes Éditeur

```bash
# Fichier actuellement ouvert
eclipse editor current

# Sélection actuelle
eclipse editor selection

# Formater le code
eclipse editor format <fichier>

# Ouvrir un fichier dans Eclipse (optionnellement à une ligne)
eclipse editor open <fichier> [--line N] [--project P]

# Lister les éditeurs ouverts (l'éditeur actif est marqué avec *)
eclipse editor list
```

**Exemples** :
```bash
# Voir quel fichier est ouvert
eclipse editor current

# Voir la sélection actuelle
eclipse editor selection

# Formater un fichier
eclipse editor format /mnt/c/projects/MyApp/src/Main.java

# Ouvrir un fichier à la ligne 42
eclipse editor open src/Main.java --line 42

# Lister tous les éditeurs ouverts
eclipse editor list
```

> **Tip** : Après avoir modifié des fichiers depuis un outil externe (Claude Code, sed, etc.), utilisez `eclipse refresh save <paths>` pour synchroniser Eclipse et appliquer les actions de sauvegarde (format on save, organize imports).

### 💻 Commandes Console

```bash
# Output de la console
eclipse console output [nom-console]
```

**Exemples** :
```bash
# Console par défaut
eclipse console output

# Console spécifique
eclipse console output "Maven Build"
```

## Utilisation dans Claude Code Skills

### Créer un Skill pour Eclipse

Le CLI est parfait pour être utilisé dans les **Skills Claude Code** :

**Exemple de Skill** (`~/.claude/skills/eclipse.md`) :

```markdown
# Eclipse IDE Skill

Use this skill to interact with Eclipse IDE.

## Available Commands

### List projects
\`\`\`bash
eclipse project list
\`\`\`

### Get project structure
\`\`\`bash
eclipse project layout {{project_name}}
\`\`\`

### Get current file
\`\`\`bash
eclipse editor current
\`\`\`

### Run tests
\`\`\`bash
eclipse test class {{project}} {{class}}
\`\`\`

### Build with Maven
\`\`\`bash
eclipse build maven {{project}} clean install
\`\`\`
```

### Utilisation dans la conversation

```
User: Liste les projets Eclipse
Claude: [invoque eclipse project list]

User: Montre la structure de eclipse-mcp
Claude: [invoque eclipse project layout eclipse-mcp]

User: Lance les tests de MyTest
Claude: [invoque eclipse test class eclipse-mcp com.myapp.MyTest]
```

## Exemples complets

### Workflow de développement

```bash
# 1. Voir les projets disponibles
eclipse project list

# 2. Structure du projet
eclipse project layout eclipse-mcp

# 3. Voir le fichier actuel dans Eclipse
eclipse editor current

# 4. Obtenir la JavaDoc d'un symbole
eclipse code javadoc /mnt/c/workspace/eclipse-mcp/src/com/Hello.java 95

# 5. Lancer les tests
eclipse test all

# 6. Voir les erreurs de compilation
eclipse build errors

# 7. Build Maven
eclipse build maven eclipse-mcp clean install

# 8. Voir la console
eclipse console output
```

### Debug et analyse

```bash
# Hiérarchie d'appels pour debug
eclipse code hierarchy Main.java 234

# Code source d'une dépendance
eclipse code source SomeLibrary.java 567

# Formater avant commit
eclipse editor format /mnt/c/projects/MyApp/src/Main.java
```

### Debug et processus

```bash
# Voir les configurations disponibles (run/debug/coverage)
eclipse debug configs

# Créer une configuration Java Application
eclipse debug config create-java com.example.Main \
  --type java \
  --name "Main (dev)" \
  --project example \
  --program-arguments "--profile dev" \
  --vm-arguments "-Xmx1g" \
  --working-directory /mnt/c/work/example \
  --env API_URL=http://localhost:8080 \
  --name-conflict generate

# Inspecter une configuration (valeurs d'environnement masquées par défaut)
eclipse debug config show "Main (dev)"

# Modifier uniquement son environnement
eclipse debug config update-environment "Main (dev)" \
  --variable API_URL=https://example.test \
  --remove-variable OLD_TOKEN \
  --no-append-system-environment

# Lancer une configuration en debug
eclipse debug run MyApp --wait

# Voir tous les processus actifs
eclipse debug processes

# Voir les threads de la session debug
eclipse debug threads

# Ajouter un breakpoint conditionnel
eclipse debug breakpoint add src/Main.java 42 --condition "userId == null"

# Voir l'état du debug
eclipse debug status

# Inspecter les variables
eclipse debug variables
```

### Filtrage des problèmes

```bash
# Lister les erreurs (par défaut)
eclipse problems list

# Lister les warnings en excluant ceux sans location
eclipse problems list warning 100 --skip-unknown

# Voir tous les problèmes sans limite
eclipse problems list all 500
```

## Architecture

```
┌─────────────────────────────┐
│ Bash / Shell / Claude Skill │
│  eclipse project list       │
└──────────────┬──────────────┘
               │ Unix socket (JSON)
               ↓
┌─────────────────────────────┐
│ Daemon Python persistant    │
│  - EclipseClient async      │
│  - Connexions MCP réutilisées│
└──────────────┬──────────────┘
               │ HTTP StreamableHTTP
               ↓
┌─────────────────────────────┐
│ eclipsectl-mcp MCP Server         │
│  http://localhost:8765      │
│  /mcp/eclipse-ide           │
└──────────────┬──────────────┘
               ↓
┌─────────────────────────────┐
│ Eclipse IDE                 │
│  eclipsectl-mcp Plugin            │
└─────────────────────────────┘
```

## Gestion des erreurs

```bash
# Si erreur de connexion
❌ Error: Cannot connect to Eclipse eclipsectl-mcp at http://169.254.83.107:8765
→ Vérifier qu'Eclipse est lancé
→ Vérifier que le plugin eclipsectl-mcp est activé
→ Vérifier que HTTP MCP Server est enabled

# Si erreur de token
❌ Error: No bearer token configured!
→ Configurer ECLIPSE_TOKEN ou config.json

# Si commande non reconnue
eclipse --help  # Voir toutes les commandes
eclipse project --help  # Aide sur les sous-commandes
```

## Logs et Debug

```bash
# Avec logs (stderr contient les logs INFO)
eclipse project list 2>&1

# Sans logs (filtre stderr)
eclipse project list 2>/dev/null

# Logs seulement
eclipse project list 2>&1 >/dev/null | grep INFO
```

## Différences avec MCP

| Aspect | MCP Server | CLI |
|--------|-----------|-----|
| **Usage** | Claude Code via `/mcp` | Shell / Skills bash |
| **Interface** | Conversationnelle | Commandes directes |
| **Protocole** | stdio JSON-RPC | Exit codes & stdout |
| **Configuration** | `.mcp.json` | Variables env / config file |
| **Commande** | `eclipsectl-cli` (serveur) | `eclipse` (CLI) |

**Recommandation** : Utilisez le **CLI** dans les Skills pour plus de simplicité et de contrôle direct.

## Référence complète des commandes

**55+ commandes disponibles** couvrant 79+ outils MCP eclipsectl-mcp

### Workspace (1)
```bash
eclipse workspace info                    # Workspace location, projects, Java/Eclipse versions
```

### Project Commands (5)
```bash
eclipse project list                      # List all projects
eclipse project layout <project>          # Show project structure
eclipse project properties <project>      # Show project properties
eclipse project read <project> <path>     # Read project file content
eclipse project import <path>             # Import project into workspace
```

### Code Analysis (3)
```bash
eclipse code javadoc <file> <offset>      # Get JavaDoc at offset
eclipse code source <file> <offset>       # Get source code at offset
eclipse code hierarchy <file> <offset>    # Get call hierarchy at offset
```

### Build & Tests (6)
```bash
eclipse build maven <project> <goals...>        # Run Maven build
eclipse build errors [project]                  # Get compilation errors
eclipse test all [--project <project>]          # Run all tests (all projects or specific)
eclipse test package <project> <package>        # Run package tests
eclipse test class <project> <class>            # Run class tests
eclipse test method <project> <class> <method>  # Run test method
```

### Editor (5)
```bash
eclipse editor current                    # Get currently opened file
eclipse editor selection                  # Get current selection
eclipse editor format <file>              # Format code file
eclipse editor open <file> [--line N] [--project P]  # Open file in editor
eclipse editor list                       # List open editors (* = active)
```

### Console (1)
```bash
eclipse console output [name]             # Get console output
```

### Search (8)
```bash
eclipse search text <pattern> [filePattern...]        # Search text
eclipse search regex <pattern> [filePattern...]       # Regex search
eclipse search files [filePattern...]                 # Find files
eclipse search type <typeName>                        # Open type
eclipse search references <elemType> <elemName>       # Find references
eclipse search hierarchy <fullyQualifiedClass>        # Type hierarchy
eclipse search call-hierarchy <class> <method> [sig]  # Call hierarchy
eclipse search implementations <class> [method]       # Find implementations
```

### Problems (4)
```bash
eclipse problems list [severity] [max] [--skip-unknown]  # Get problems (skip unknown location with flag)
eclipse problems quickfixes <file> [proj] [line]         # Get quick fixes
eclipse problems apply-fix <file> <line> <idx> [proj]    # Apply quick fix
eclipse problems markers <file> [proj]                    # Get all markers
```

### Maven (4)
```bash
eclipse maven list                        # List all Maven projects
eclipse maven effective-pom <project>     # Get effective POM
eclipse maven dependencies <project>      # Get project dependencies
eclipse maven find-tests <project>        # Find all test classes
```

### Refactor (6)
```bash
eclipse refactor rename-type <file> <newName> [proj]    # Rename Java type
eclipse refactor move-type <file> <package> [proj]      # Move type to package
eclipse refactor rename-package <pkg> <newPkg> [proj]   # Rename package
eclipse refactor extract-method <file> <start> <end> <name> [proj] [vis]  # Extract method
eclipse refactor rename-file <file> <newName> [proj]    # Rename file
eclipse refactor move-resource <src> <target> [proj]    # Move file/folder
```

### Code Generation (3)
```bash
eclipse codegen getters-setters <file> [proj] [class] [fields]  # Generate getters/setters
eclipse codegen tostring <file> [proj] [class] [fields]         # Generate toString/equals/hashCode
eclipse codegen implement <file> [proj] [class] [methods]       # Implement/override methods
```

### Source Operations (3)
```bash
eclipse source organize-imports <file> [proj]    # Organize imports in file
eclipse source organize-package <package> [proj] # Organize imports in package
eclipse source format <file> [proj]              # Format file (project-aware)
```

### Java Member Editing (5)
```bash
# Read Java class structure (required before write/remove)
eclipse read <file> [--project P] [--class C]

# Write a member (method, field, constructor, inner class) into a Java class
eclipse java write <file> <source_code> [--project P] [--class C]

# Remove named members from a Java class (comma-separated)
eclipse java remove <file> <members> [--project P] [--class C]

# Remove line ranges from a Java file
eclipse remove lines <file> <lines> [--project P]

# Reset the readJava gate (for a file or all files)
eclipse gate reset [file] [--project P]
```

**Exemples** :
```bash
# Lire la structure d'une classe Java
eclipse read src/com/example/MyService.java

# Lire une inner class spécifique
eclipse read src/com/example/Outer.java --class Outer.Inner

# Écrire/remplacer une méthode dans une classe
eclipse java write src/MyClass.java 'public void hello() { System.out.println("hi"); }'

# Supprimer des membres par nom
eclipse java remove src/MyClass.java 'getData,setData,oldField'

# Supprimer des lignes (ranges)
eclipse remove lines src/MyClass.java '10-15,42,80-90'

# Reset le gate pour forcer une relecture
eclipse gate reset src/MyClass.java
eclipse gate reset  # reset tous les fichiers
```

### Run (1)
```bash
eclipse run [configuration] [--wait]             # Run Eclipse launch configuration
```

### Debug (15+)
```bash
# Breakpoint Management
eclipse debug breakpoint add <file> <line> [--condition <expr>]  # Add breakpoint
eclipse debug breakpoint remove <file> <line>                     # Remove breakpoint
eclipse debug breakpoint list                                     # List all breakpoints
eclipse debug breakpoint enable <file> <line>                     # Enable breakpoint
eclipse debug breakpoint disable <file> <line>                    # Disable breakpoint

# Debug Session Control
eclipse debug run <config> [--wait]              # Launch debug configuration
eclipse debug configs                            # List run configurations (run/debug/coverage)
eclipse debug config create-java <class> [options]       # Create a Java-based launch configuration
  --type <java|junit4|junit5|testng|auto>        # Configuration type (server default when omitted)
  --name <name>                                  # Explicit configuration name
  --project <project>                            # Project (inferred when omitted)
  --program-arguments <args>                     # Program arguments
  --vm-arguments <args>                          # JVM arguments
  --working-directory <path>                     # WSL paths are converted automatically
  --env <NAME=VALUE>                             # Repeatable environment variable
  --[no-]append-system-environment               # System environment inheritance (default: append)
  --name-conflict <error|generate>               # Duplicate-name behavior (default: error)
eclipse debug config show <config> [--include-environment-values]
                                                    # Values stay masked unless explicitly requested
eclipse debug config update-environment <config> [options]
  --variable <NAME=VALUE>                        # Set/merge a variable; repeatable
  --remove-variable <NAME>                       # Explicit removal; repeatable
  --mode <merge|replace>                         # Default: merge
  --[no-]append-system-environment               # Omit to preserve current inheritance
# In replace mode, omitting --variable sends an empty map and clears configured variables.
# --remove-variable is intentionally rejected together with --mode replace.
eclipse debug step-into                          # Step into method (F5)
eclipse debug step-over                          # Step over line (F6)
eclipse debug step-return                        # Step out of method (F7)
eclipse debug resume                             # Resume execution (F8)
eclipse debug suspend                            # Suspend execution
eclipse debug terminate                          # Terminate debug session
eclipse debug status                             # Get debug session status

# Thread & Process Management
eclipse debug threads                            # List threads in debug session
eclipse debug processes                          # List all active processes (run/debug/coverage)
eclipse debug thread-select <thread>             # Select thread for inspection

# Runtime Inspection
eclipse debug stacktrace [--thread <id>]         # Get stack trace
eclipse debug variables [--frame <id>]           # Get variables in scope
eclipse debug eval <expression> [--frame <id>]   # Evaluate expression
```

### Coverage (3)
```bash
eclipse coverage run <config> [--wait]           # Run with code coverage
eclipse coverage report [project]                # Get coverage report
eclipse coverage class <className>               # Get class coverage
```

### Refresh (2)
```bash
eclipse refresh workspace <paths...>             # Refresh workspace paths
eclipse refresh save <paths...>                  # Refresh and save (triggers actions)
```

### Daemon (3)
```bash
eclipse daemon status                    # Check daemon status
eclipse daemon stop                      # Stop daemon
eclipse daemon restart                   # Restart daemon
```

## Performance

- **First call:** ~4s (daemon startup + MCP init)
- **Subsequent calls:** <100ms (reuse connection)
- **Auto-shutdown:** 5 minutes of inactivity

See [DAEMON.md](DAEMON.md) for details.

## Prochaines étapes

1. Créer vos Skills Claude Code avec les commandes Eclipse
2. Intégrer dans vos workflows de développement
3. Combiner avec d'autres outils (git, maven, etc.)
