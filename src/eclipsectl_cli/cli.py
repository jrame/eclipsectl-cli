"""CLI entry point for Eclipse MCP server."""

import asyncio
import sys

from .server import main


def cli_entry_point():
    """CLI entry point for the server."""
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nServer terminated by user", file=sys.stderr)
        return 1
    except Exception as e:
        print(f"Server error: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc(file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(cli_entry_point())
