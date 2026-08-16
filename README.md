# CLI Eclipse

**Command-line interface for controlling Eclipse IDE** via the eclipsectl-mcp plugin.

## Overview

**eclipsectl-cli** provides a **CLI tool** (`eclipse` command) to control Eclipse IDE from bash/shell, perfect for **Claude Code Skills** and automation scripts.

```
┌─── Shell / Claude Skills ──┐     ┌─── Windows ──────────┐
│  eclipse project list      │     │  Eclipse IDE         │
│  eclipse test all          │ ──► │  eclipsectl-mcp (:8765)    │
│  eclipse editor current    │     │    - 74+ MCP tools   │
└────────────────────────────┘     └──────────────────────┘
```

### Two modes available

1. **CLI mode** ⭐ **RECOMMENDED** - Direct bash commands
   - Use: `eclipse project list`, `eclipse test all`, etc.
   - Perfect for: Claude Code Skills, shell scripts, automation
   - See: [CLI_USAGE.md](CLI_USAGE.md)

2. **MCP Server mode** - For MCP integration (legacy)
   - Use: Add to `.mcp.json` for MCP protocol
   - See Troubleshooting section below for connection issues

## Features

### Available (via eclipsectl-mcp)
- ✅ **File Operations**: Create, edit, replace, delete files and directories
- ✅ **Code Analysis**: Get JavaDoc, source code, compilation errors
- ✅ **Project Management**: List projects, get layout, read properties
- ✅ **Tests**: Run JUnit tests (all, package, class, method level)
- ✅ **Build**: Execute Maven builds
- ✅ **Editor**: Get current file, selection, format code
- ✅ **Console**: Access Eclipse console output
- ✅ **Workspace**: Refresh files and projects

- ✅ **Workspace**: Refresh files, get workspace info
- ✅ **Refactoring**: Rename types/packages, move types, extract methods
- ✅ **Launch Configs**: Create and inspect Java Application configurations, manage their environment, and run/debug them
- ✅ **Debug**: Breakpoints, step through, inspect variables, evaluate expressions
- ✅ **Coverage**: Run with EclEmma code coverage, get reports
- ✅ **Code Generation**: Generate getters/setters, toString, equals/hashCode

> **Tip**: After editing files externally (e.g., from Claude Code), use `eclipse refresh save <paths>` to sync Eclipse and trigger save actions (format on save, organize imports).

See [ANALYSIS.md](docs/ANALYSIS.md) for architecture details.

## Quick Start

### 1. Install eclipsectl-mcp plugin in Eclipse (Windows)

**Option A: Eclipse Marketplace**
1. Open Eclipse
2. Help → Eclipse Marketplace
3. Search "eclipsectl-mcp"
4. Install and restart Eclipse

**Option B: Update Site**
1. Help → Install New Software
2. Add site: `https://gradusnikov.github.io/eclipse-chatgpt-plugin/`
3. Install and restart Eclipse

### 2. Configure eclipsectl-mcp HTTP Server

1. Window → Preferences → eclipsectl-mcp
2. Enable "HTTP MCP Server"
3. Set hostname: `localhost`
4. Set port: `8124`
5. Generate Bearer token (save it!)
6. Apply and restart Eclipse

### 3. Install eclipsectl-cli (WSL)

From a GitHub Release wheel (recommended):

```bash
pipx install ./eclipsectl_cli-0.1.0-py3-none-any.whl
```

From a source checkout for development:

```bash
git clone https://github.com/jrame/eclipsectl-cli.git
cd eclipsectl-cli
python3 -m venv .venv
source .venv/bin/activate
bash install.sh
```

### 4. Configure eclipsectl-cli

Create `~/.config/eclipsectl-cli/config.json`:

```json
{
  "eclipse_url": "http://localhost:8124",
  "bearer_token": "YOUR_BEARER_TOKEN_HERE"
}
```

Or use environment variables:
```bash
export ECLIPSE_URL="http://localhost:8124"
export ECLIPSE_TOKEN="YOUR_BEARER_TOKEN"
```

### 5. Test the connection

```bash
# From WSL terminal
eclipse project list
```

Create and inspect a Java launch configuration:

```bash
eclipse debug config create-java com.example.Main \
  --type java --project example --env PROFILE=dev
eclipse debug config show Main
eclipse debug run Main
```

### 6. Add to Claude Code (optional)

Edit `~/.claude/config.json` or project `.mcp.json`:

```json
{
  "mcpServers": {
    "eclipse": {
      "command": "eclipsectl-cli"
    }
  }
}
```

## Usage

Once configured, use Eclipse commands from Claude Code:

```
@eclipse list all projects in workspace
@eclipse create a new Java class User in package com.example.model
@eclipse run all tests in the project
@eclipse get compilation errors
@eclipse format the current file
@eclipse refresh workspace
```

## Path Conversion

eclipsectl-cli automatically converts paths between WSL and Windows:

| WSL Path | Windows Path |
|----------|--------------|
| `/mnt/c/projects/MyApp/src/Main.java` | `C:\projects\MyApp\src\Main.java` |
| `/mnt/d/workspace/Test.java` | `D:\workspace\Test.java` |

**Important**: Your Java projects should be on `/mnt/c/` (or `/mnt/d/`) so both WSL and Windows can access them.

## Architecture

### Components

1. **Claude Code** (WSL) - AI assistant interface
2. **eclipsectl-cli** (WSL) - MCP wrapper with path conversion
3. **eclipsectl-mcp Plugin** (Windows) - HTTP MCP server in Eclipse
4. **Eclipse IDE** (Windows) - Java development environment

### MCP Tools Exposed

See [ECLIPSECTL_MCP_TOOLS.md](docs/ECLIPSECTL_MCP_TOOLS.md) for complete reference.

**Summary**:
- `eclipse-coder`: 5 file operation tools
- `eclipse-ide`: 18 Java/IDE integration tools
- `duck-duck-search`, `webpage-reader`, `time`, `memory`: Utility tools

## Development

### Setup

```bash
git clone https://github.com/jrame/eclipsectl-cli
cd eclipsectl-cli
python -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
```

### Run tests

```bash
pytest
# or
pytest -m unit  # unit tests only
```

### Project structure

```
eclipsectl-cli/
├── src/eclipsectl_cli/
│   ├── __init__.py
│   ├── cli.py              # Entry point
│   ├── server.py           # MCP server
│   ├── config.py           # Configuration
│   ├── eclipse_client.py   # HTTP client for eclipsectl-mcp
│   ├── tools.py            # MCP tool definitions
│   └── path_converter.py   # WSL↔Windows path conversion
├── docs/
│   ├── ANALYSIS.md         # Complete analysis of options
│   └── ECLIPSECTL_MCP_TOOLS.md   # eclipsectl-mcp tools reference
├── tests/
└── pyproject.toml
```

## Troubleshooting

### Cannot connect to eclipsectl-mcp

**Error**: "Cannot connect to Eclipse eclipsectl-mcp at http://localhost:8124"

**Solutions**:
1. Verify Eclipse is running on Windows
2. Verify eclipsectl-mcp plugin is installed
3. Verify HTTP MCP Server is enabled (Preferences → eclipsectl-mcp)
4. Check port 8124 is not blocked by firewall
5. Test connection from WSL:
   ```bash
   curl -H "Authorization: Bearer YOUR_TOKEN" http://localhost:8124/mcp
   ```

### Bearer token not configured

**Error**: "No bearer token configured"

**Solutions**:
1. Set `ECLIPSE_TOKEN` env var
2. Add `bearer_token` to `~/.config/eclipsectl-cli/config.json`
3. Verify the token was copied correctly from Eclipse

### Windows 11 networking

On Windows 11 22H2+, enable mirrored mode in `.wslconfig`:
```ini
[wsl2]
networkingMode=mirrored
```

For older Windows versions, use Windows IP instead of localhost:
```bash
# Get Windows IP from WSL
ip route show | grep -i default | awk '{ print $3}'

# Example config with Windows IP
{
  "eclipse_url": "http://169.254.x.x:8124",
  "bearer_token": "YOUR_TOKEN"
}
```

### Path conversion issues

**Problem**: Files not found in Eclipse

**Solution**: Ensure projects are on Windows drives accessible from WSL:
- ✅ `/mnt/c/projects/MyApp` → OK
- ✅ `/mnt/d/workspace/Test` → OK
- ❌ `/home/user/local-project` → NOT accessible from Eclipse

## Future Extensions

See [ANALYSIS.md](docs/ANALYSIS.md) for detailed options.

**Possible additions**:
- **Advanced refactoring**: Add JavaLens-MCP as secondary server
- **Launch configurations**: Develop custom plugin extension
- **Concurrency**: Support multiple JavaLens instances

## Build a release wheel

```bash
pip install build
python -m build
```

The wheel in `dist/` contains both the Python package and the `eclipse` Bash
command. Test it in an isolated environment with:

```bash
pipx install --force ./dist/eclipsectl_cli-0.1.0-py3-none-any.whl
eclipse --help
```

Attach the `.whl` file to the corresponding GitHub Release. End users only
need that single asset and `pipx`.

## License

MIT

## Credits

Built on top of:
- [eclipsectl-mcp (gradusnikov/eclipse-chatgpt-plugin)](https://github.com/gradusnikov/eclipse-chatgpt-plugin)
- [Model Context Protocol](https://modelcontextprotocol.io/)
