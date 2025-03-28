"""Configuration for CLI Eclipse MCP server."""

import argparse
import json
import os
from dataclasses import dataclass
from pathlib import Path
from typing import Optional


@dataclass
class Config:
    """Configuration for Eclipse integration."""

    eclipse_url: str = "http://localhost:8124"
    bearer_token: Optional[str] = None
    timeout: int = 30
    verbose: bool = False


def load_config_file() -> dict:
    """Load configuration from config files.

    Priority:
    1. $CWD/.config/eclipsectl-cli/config.json (project-local)
    2. ~/.config/eclipsectl-cli/config.json (user-global)
    """
    # Try project-local config first
    local_config_path = Path.cwd() / ".config" / "eclipsectl-cli" / "config.json"
    if local_config_path.exists():
        try:
            with open(local_config_path) as f:
                return json.load(f)
        except Exception:
            pass

    # Fall back to user-global config
    global_config_path = Path.home() / ".config" / "eclipsectl-cli" / "config.json"
    if global_config_path.exists():
        try:
            with open(global_config_path) as f:
                return json.load(f)
        except Exception:
            pass

    return {}


def get_config() -> Config:
    """Get configuration from file and environment (no CLI args parsing)."""
    file_config = load_config_file()

    eclipse_url = (
        os.getenv("ECLIPSE_URL")
        or file_config.get("eclipse_url")
        or "http://localhost:8124"
    )

    bearer_token = (
        os.getenv("ECLIPSE_TOKEN")
        or file_config.get("bearer_token")
    )

    timeout = file_config.get("timeout", 30)
    verbose = file_config.get("verbose", False)

    return Config(
        eclipse_url=eclipse_url,
        bearer_token=bearer_token,
        timeout=timeout,
        verbose=verbose
    )


def parse_args() -> Config:
    """Parse command-line arguments and return configuration."""
    # Load from config file
    file_config = load_config_file()

    parser = argparse.ArgumentParser(
        description="CLI Eclipse - MCP server for Eclipse IDE control"
    )

    parser.add_argument(
        "--eclipse-url",
        default=None,
        help="URL of Eclipse eclipsectl-mcp endpoint (default: http://localhost:8124)"
    )

    parser.add_argument(
        "--bearer-token",
        default=None,
        help="Bearer token for eclipsectl-mcp authentication"
    )

    parser.add_argument(
        "--timeout",
        type=int,
        default=30,
        help="HTTP request timeout in seconds (default: 30)"
    )

    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Enable verbose logging"
    )

    args = parser.parse_args()

    # Priority: CLI args > env vars > config file > defaults
    eclipse_url = (
        args.eclipse_url
        or os.getenv("ECLIPSE_URL")
        or file_config.get("eclipse_url")
        or "http://localhost:8124"
    )

    bearer_token = (
        args.bearer_token
        or os.getenv("ECLIPSE_TOKEN")
        or file_config.get("bearer_token")
    )

    return Config(
        eclipse_url=eclipse_url,
        bearer_token=bearer_token,
        timeout=args.timeout,
        verbose=args.verbose
    )


# Default configuration
DEFAULT_CONFIG = Config()
