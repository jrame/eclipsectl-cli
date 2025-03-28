#!/bin/bash
# Installation script for eclipsectl-cli with bash CLI

set -e

echo "📦 Installing eclipsectl-cli..."

# Activate virtualenv if exists
if [ -f .venv/bin/activate ]; then
    source .venv/bin/activate
    echo "✅ Using virtualenv: .venv"
fi

# Install Python package
pip install -e .

# Install bash CLI wrapper (setup.py cmdclass not triggered in editable mode)
echo ""
echo "⚡ Installing bash CLI wrapper..."
BIN_DIR=$(python3 -c "import sys; print(sys.prefix + '/bin')")
cp bin/eclipse "$BIN_DIR/eclipse"
chmod +x "$BIN_DIR/eclipse"
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
