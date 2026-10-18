#!/bin/sh
set -eu
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
INSTALL_DIR=${ECLIPSECTL_INSTALL_DIR:-"$HOME/.local/bin"}
mkdir -p "$INSTALL_DIR"
if [ -f "$SCRIPT_DIR/eclipsectl" ]; then
    install -m 755 "$SCRIPT_DIR/eclipsectl" "$INSTALL_DIR/eclipsectl"
elif [ -f "$SCRIPT_DIR/Cargo.toml" ]; then
    command -v cargo >/dev/null 2>&1 || { echo 'Rust is required to install from source; alternatively download a release archive.' >&2; exit 1; }
    (cd "$SCRIPT_DIR" && cargo build --locked --release)
    install -m 755 "$SCRIPT_DIR/target/release/eclipsectl" "$INSTALL_DIR/eclipsectl"
else
    echo 'Cannot find eclipsectl or Cargo.toml beside this installer.' >&2
    exit 1
fi
echo "Installed: $INSTALL_DIR/eclipsectl"
case ":$PATH:" in *":$INSTALL_DIR:"*) ;; *) echo "Add $INSTALL_DIR to PATH to use eclipsectl." ;; esac
"$INSTALL_DIR/eclipsectl" --version
