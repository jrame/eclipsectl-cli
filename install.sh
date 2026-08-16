#!/bin/bash
# Installation script for eclipsectl-cli

set -euo pipefail

echo "📦 Installing eclipsectl-cli..."

# Activate virtualenv if exists
if [ -f .venv/bin/activate ]; then
    source .venv/bin/activate
    echo "✅ Using virtualenv: .venv"
fi

# The wheel/editable metadata installs both the Python entry point and the
# Bash wrapper declared by tool.setuptools.script-files.
python -m pip install -e .

BIN_DIR=$(python -c "import sys; print(sys.prefix + '/bin')")
test -x "$BIN_DIR/eclipse"
echo "✅ Bash CLI installed to $BIN_DIR/eclipse"

echo ""
echo "✅ Installation complete!"
echo ""
echo "Installed commands:"
echo "  eclipse          - Ultra-fast bash CLI (~40ms)"
echo "  eclipsectl-cli   - MCP server"
echo ""
echo "Test it:"
echo "  eclipse project list"
echo "  eclipse daemon status"
echo ""
echo "Performance:"
echo "  First call:  ~4s (starts daemon)"
echo "  Next calls:  ~40ms (via daemon)"
