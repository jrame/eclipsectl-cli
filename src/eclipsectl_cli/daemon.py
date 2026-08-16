#!/usr/bin/env python3
"""Eclipse CLI daemon - keeps MCP connection alive for fast CLI responses."""

import asyncio
import json
import logging
import os
import signal
import sys
import time
from pathlib import Path
from typing import Optional, Dict, Any

from .eclipse_client import EclipseClient
from .config import get_config

logger = logging.getLogger(__name__)

# Daemon config
SOCKET_PATH = Path.home() / ".config" / "eclipsectl-cli" / "daemon.sock"
PID_FILE = Path.home() / ".config" / "eclipsectl-cli" / "daemon.pid"
IDLE_TIMEOUT = 300  # 5 minutes of inactivity before auto-shutdown


class EclipseDaemon:
    """Daemon that maintains persistent MCP connection to eclipsectl-mcp."""

    def __init__(self):
        self.client: Optional[EclipseClient] = None
        self.last_activity = time.time()
        self.shutdown_requested = False
        self.server: Optional[asyncio.Server] = None

    async def start(self):
        """Start the daemon."""
        # Setup signal handlers
        signal.signal(signal.SIGTERM, lambda s, f: self._request_shutdown())
        signal.signal(signal.SIGINT, lambda s, f: self._request_shutdown())

        # Initialize Eclipse client
        config = get_config()
        if not config.bearer_token:
            logger.error("No bearer token configured!")
            sys.exit(1)

        self.client = EclipseClient(
            base_url=config.eclipse_url,
            bearer_token=config.bearer_token,
            timeout=config.timeout
        )

        # Connect to eclipsectl-mcp (this takes ~3-4s but only once!)
        logger.info("Connecting to eclipsectl-mcp...")
        try:
            await self.client.connect()
            logger.info("Connected! Ready to serve CLI requests.")
        except Exception as e:
            error_msg = (
                f"\nCannot connect to Eclipse eclipsectl-mcp MCP server at {config.eclipse_url}\n\n"
                f"Error: {type(e).__name__}: {e}\n\n"
                f"Please verify:\n"
                f"  1. Eclipse IDE is running on Windows\n"
                f"  2. eclipsectl-mcp plugin is installed and enabled\n"
                f"  3. HTTP MCP Server is enabled in: Window → Preferences → eclipsectl-mcp → HTTP MCP Server\n"
                f"  4. Port matches your config: {config.eclipse_url}\n"
                f"  5. Bearer token is valid\n\n"
                f"Test connection: curl -v {config.eclipse_url}/mcp/eclipse-ide\n"
            )
            print(error_msg, file=sys.stderr)
            logger.error(error_msg)
            sys.exit(1)

        # Create socket directory
        SOCKET_PATH.parent.mkdir(parents=True, exist_ok=True)

        # Remove stale socket
        if SOCKET_PATH.exists():
            SOCKET_PATH.unlink()

        # Start Unix socket server
        self.server = await asyncio.start_unix_server(
            self._handle_client,
            path=str(SOCKET_PATH)
        )

        # Write PID file
        PID_FILE.write_text(str(os.getpid()))
        logger.info(f"Daemon started, listening on {SOCKET_PATH}")

        # Run until shutdown
        await self._run_with_timeout()

    async def _run_with_timeout(self):
        """Run server and auto-shutdown after idle timeout."""
        while not self.shutdown_requested:
            await asyncio.sleep(10)  # Check every 10s

            # Auto-shutdown if idle too long
            idle_time = time.time() - self.last_activity
            if idle_time > IDLE_TIMEOUT:
                logger.info(f"Idle for {idle_time:.0f}s, shutting down...")
                break

        # Cleanup
        await self._cleanup()

    async def _handle_client(self, reader: asyncio.StreamReader, writer: asyncio.StreamWriter):
        """Handle a CLI client request."""
        self.last_activity = time.time()
        command = None
        args = {}

        try:
            # Read request (JSON)
            data = await reader.read(1024 * 100)  # 100KB max
            if not data:
                return

            request = json.loads(data.decode())
            command = request.get("command")
            args = request.get("args", {})

            logger.debug(f"Request: {command} {args}")

            # Route command
            result = await self._execute_command(command, args)

            # Send response
            response = json.dumps({"success": True, "result": result})
            writer.write(response.encode())
            await writer.drain()

        except KeyError as e:
            msg = f"Missing required parameter {e} for command '{command}'. Got: {list(args.keys())}"
            logger.error(msg)
            error_response = json.dumps({"success": False, "error": msg})
            writer.write(error_response.encode())
            await writer.drain()
        except Exception as e:
            logger.error(f"Request failed: {e}", exc_info=True)
            error_response = json.dumps({"success": False, "error": f"{type(e).__name__}: {e}"})
            writer.write(error_response.encode())
            await writer.drain()

        finally:
            writer.close()
            await writer.wait_closed()

    async def _execute_command(self, command: str, args: Dict[str, Any]) -> str:
        """Execute Eclipse command using persistent client."""
        if not self.client:
            raise RuntimeError("Eclipse client not initialized")

        # Route to appropriate client method
        # === Project ===
        if command == "list_projects":
            return await self.client.list_projects()
        elif command == "get_project_layout":
            return await self.client.get_project_layout(args["project"])
        elif command == "get_project_properties":
            return await self.client.get_project_properties(args["project"])
        elif command == "read_project_resource":
            return await self.client.read_project_resource(args["project"], args["path"])
        elif command == "import_project":
            return await self.client.import_project(args["path"])
        elif command == "get_workspace_info":
            return await self.client.get_workspace_info()

        # === Code Analysis ===
        elif command == "get_javadoc":
            return await self.client.get_javadoc(args["fqn"])
        elif command == "get_source":
            return await self.client.get_source(args["fqn"])
        elif command == "get_call_hierarchy":
            return await self.client.get_call_hierarchy_by_fqn(args["fqn"], args["method"])

        # === Build & Tests ===
        elif command == "run_maven_build":
            return await self.client.run_maven_build(args["project"], args["goals"])
        elif command == "get_compilation_errors":
            return await self.client.get_compilation_errors(args.get("project"))
        elif command == "run_all_tests":
            return await self.client.run_all_tests()
        elif command == "run_package_tests":
            return await self.client.run_package_tests(args["project"], args["package"])
        elif command == "run_class_tests":
            return await self.client.run_class_tests(args["project"], args["class"])
        elif command == "run_test_method":
            return await self.client.run_test_method(args["project"], args["class"], args["method"])

        # === Editor ===
        elif command == "get_current_file":
            return await self.client.get_current_file()
        elif command == "get_editor_selection":
            return await self.client.get_editor_selection()
        elif command == "format_code":
            return await self.client.format_code(args["file"])
        elif command == "open_file":
            return await self.client.open_file(
                args["file"],
                int(args["line"]) if "line" in args else None,
                args.get("project"))
        elif command == "get_open_editors":
            return await self.client.get_open_editors()

        # === Console ===
        elif command == "get_console_output":
            return await self.client.get_console_output(args.get("console_name"))

        # === Search ===
        elif command == "file_search":
            return await self.client.file_search(
                args["containingText"],
                args.get("fileNamePatterns")
            )
        elif command == "file_search_regexp":
            return await self.client.file_search_regexp(
                args["pattern"],
                args.get("fileNamePatterns")
            )
        elif command == "find_files":
            return await self.client.find_files(
                args.get("fileNamePatterns"),
                int(args.get("maxResults", "200"))
            )
        elif command == "open_type":
            return await self.client.open_type(args["name"])
        elif command == "find_references":
            return await self.client.find_references(
                args["elementType"],
                args["elementName"],
                args.get("projectName"),
                args.get("scope", "workspace"),
                args.get("includeSubtypes", False)
            )
        elif command == "get_type_hierarchy":
            return await self.client.get_type_hierarchy(args["fullyQualifiedClassName"])
        elif command == "call_hierarchy":
            return await self.client.call_hierarchy(
                args["fullyQualifiedClassName"],
                args["methodName"],
                args.get("methodSignature")
            )
        elif command == "get_implementations":
            return await self.client.get_implementations(
                args["fullyQualifiedClassName"],
                args.get("methodName")
            )

        # === Problems ===
        elif command == "get_problems":
            return await self.client.get_problems(
                args.get("severity", "error"),
                int(args.get("maxResults", "10"))
            )
        elif command == "get_quick_fixes":
            return await self.client.get_quick_fixes(
                args.get("projectName"),
                args["filePath"],
                int(args["lineNumber"]) if "lineNumber" in args else None,
                args.get("problemId")
            )
        elif command == "apply_quick_fix":
            return await self.client.apply_quick_fix(
                args.get("projectName"),
                args["filePath"],
                int(args["lineNumber"]),
                int(args["fixIndex"])
            )
        elif command == "get_markers":
            return await self.client.get_markers(
                args.get("projectName"),
                args["filePath"]
            )

        # === Maven ===
        elif command == "list_maven_projects":
            return await self.client.list_maven_projects()
        elif command == "get_effective_pom":
            return await self.client.get_effective_pom(args["projectName"])
        elif command == "get_project_dependencies":
            return await self.client.get_project_dependencies(args["projectName"])
        elif command == "find_test_classes":
            return await self.client.find_test_classes(args["projectName"])

        # === Refactor ===
        elif command == "refactor_rename_java_type":
            return await self.client.refactor_rename_java_type(
                args.get("projectName"),
                args["filePath"],
                args["newTypeName"]
            )
        elif command == "refactor_move_java_type":
            return await self.client.refactor_move_java_type(
                args.get("projectName"),
                args["filePath"],
                args["targetPackage"]
            )
        elif command == "refactor_rename_package":
            return await self.client.refactor_rename_package(
                args.get("projectName"),
                args["packageName"],
                args["newPackageName"]
            )
        elif command == "refactor_extract_method":
            return await self.client.refactor_extract_method(
                args.get("projectName"),
                args["filePath"],
                int(args["startLine"]),
                int(args["endLine"]),
                args["methodName"],
                args.get("visibility", "private")
            )
        elif command == "rename_file":
            return await self.client.rename_file(
                args.get("projectName"),
                args["filePath"],
                args["newFileName"]
            )
        elif command == "move_resource":
            return await self.client.move_resource(
                args.get("projectName"),
                args["sourcePath"],
                args["targetPath"]
            )

        # === Code Generation ===
        elif command == "generate_getters_setters":
            return await self.client.generate_getters_setters(
                args.get("projectName"),
                args["filePath"],
                args.get("className"),
                args.get("fieldNames", "").split(",") if args.get("fieldNames") else None,
                args.get("generateGetters", True),
                args.get("generateSetters", True),
                args.get("insertionPoint", "end")
            )
        elif command == "generate_tostring_equals_hashcode":
            return await self.client.generate_tostring_equals_hashcode(
                args.get("projectName"),
                args["filePath"],
                args.get("className"),
                args.get("fieldNames", "").split(",") if args.get("fieldNames") else None,
                args.get("generateToString", True),
                args.get("generateEquals", True),
                args.get("generateHashCode", True)
            )
        elif command == "implement_override_methods":
            return await self.client.implement_override_methods(
                args.get("projectName"),
                args["filePath"],
                args.get("className"),
                args.get("methodNames", "").split(",") if args.get("methodNames") else None
            )

        # === Java Member Editing ===
        elif command == "read_java":
            return await self.client.read_java(
                args.get("projectName"),
                args["filePath"],
                args.get("className")
            )
        elif command == "write_java":
            return await self.client.write_java(
                args.get("projectName"),
                args["filePath"],
                args["newSource"],
                args.get("className")
            )
        elif command == "remove_java":
            return await self.client.remove_java(
                args.get("projectName"),
                args["filePath"],
                args["members"],
                args.get("className")
            )
        elif command == "remove_lines":
            return await self.client.remove_lines(
                args.get("projectName"),
                args["filePath"],
                args["lines"]
            )
        elif command == "reset_read_gate":
            return await self.client.reset_read_gate(
                args.get("projectName"),
                args.get("filePath")
            )

        # === Source Operations ===
        elif command == "organize_imports":
            return await self.client.organize_imports(
                args.get("projectName"),
                args["filePath"]
            )
        elif command == "organize_imports_in_package":
            return await self.client.organize_imports_in_package(
                args.get("projectName"),
                args["packageName"]
            )
        elif command == "format_file":
            return await self.client.format_file(
                args.get("projectName"),
                args["filePath"],
                args.get("lines")
            )

        # === Run ===
        elif command == "run":
            return await self.client.run(args.get("configuration"), args.get("wait", False))

        # === Refresh ===
        elif command == "refresh":
            return await self.client.refresh(args["path"])
        elif command == "refresh_and_save":
            return await self.client.refresh_and_save(args["paths"])

        # === Debug ===
        elif command == "add_breakpoint":
            return await self.client.add_breakpoint(
                args["filePath"], int(args["lineNumber"]), args.get("condition"))
        elif command == "remove_breakpoint":
            return await self.client.remove_breakpoint(args["filePath"], int(args["lineNumber"]))
        elif command == "list_breakpoints":
            return await self.client.list_breakpoints()
        elif command == "enable_breakpoint":
            return await self.client.enable_breakpoint(args["filePath"], int(args["lineNumber"]))
        elif command == "disable_breakpoint":
            return await self.client.disable_breakpoint(args["filePath"], int(args["lineNumber"]))
        elif command == "list_threads":
            return await self.client.list_threads()
        elif command == "select_thread":
            return await self.client.select_thread(args["threadId"])
        elif command == "run_debug":
            return await self.client.run_debug(args["configuration"], args.get("wait", False))
        elif command in ("list_run_configurations", "list_debug_configurations"):
            return await self.client.list_run_configurations()
        elif command == "create_java_launch_configuration":
            return await self.client.create_java_launch_configuration(
                args["mainClass"],
                args.get("configName"),
                args.get("projectName"),
                args.get("programArguments"),
                args.get("vmArguments"),
                args.get("workingDirectory"),
                args.get("environmentVariables"),
                args.get("appendSystemEnvironment", True),
                args.get("nameConflict", "error"),
                args.get("type"),
            )
        elif command == "get_launch_configuration":
            return await self.client.get_launch_configuration(
                args["configName"], args.get("includeEnvironmentValues", False))
        elif command == "update_launch_environment":
            return await self.client.update_launch_environment(
                args["configName"],
                args.get("variables"),
                args.get("removeVariables"),
                args.get("updateMode", "merge"),
                args.get("appendSystemEnvironment"),
            )
        elif command == "step_into":
            return await self.client.step_into()
        elif command == "step_over":
            return await self.client.step_over()
        elif command == "step_return":
            return await self.client.step_return()
        elif command == "debug_resume":
            return await self.client.debug_resume()
        elif command == "debug_suspend":
            return await self.client.debug_suspend()
        elif command == "debug_terminate":
            return await self.client.debug_terminate()
        elif command == "get_stack_trace":
            return await self.client.get_stack_trace(args.get("threadId"))
        elif command == "get_variables":
            return await self.client.get_variables(args.get("frameId"))
        elif command == "evaluate_expression":
            return await self.client.evaluate_expression(args["expression"], args.get("frameId"))
        elif command == "get_debug_status":
            return await self.client.get_debug_status()

        # === Coverage ===
        elif command == "run_coverage":
            return await self.client.run_coverage(args["configuration"], args.get("wait", False))
        elif command == "get_coverage_report":
            return await self.client.get_coverage_report(args.get("projectName"))
        elif command == "get_class_coverage":
            return await self.client.get_class_coverage(args["className"])

        # === Daemon ===
        elif command == "ping":
            return "pong"
        elif command == "shutdown":
            self._request_shutdown()
            return "Daemon shutting down..."
        else:
            raise ValueError(f"Unknown command: {command}")

    def _request_shutdown(self):
        """Request graceful shutdown."""
        self.shutdown_requested = True

    async def _cleanup(self):
        """Cleanup resources."""
        logger.info("Cleaning up...")

        if self.server:
            self.server.close()
            await self.server.wait_closed()

        if self.client:
            await self.client.disconnect()

        if SOCKET_PATH.exists():
            SOCKET_PATH.unlink()

        if PID_FILE.exists():
            PID_FILE.unlink()

        logger.info("Daemon stopped")


def main():
    """Entry point for daemon."""
    # Ensure log directory exists
    log_dir = Path.home() / ".config" / "eclipsectl-cli"
    log_dir.mkdir(parents=True, exist_ok=True)

    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(levelname)s - %(message)s',
        handlers=[
            logging.FileHandler(log_dir / "daemon.log"),
            logging.StreamHandler(sys.stderr)  # Explicit stderr for startup errors
        ]
    )

    try:
        daemon = EclipseDaemon()
        asyncio.run(daemon.start())
    except KeyboardInterrupt:
        logger.info("Daemon interrupted by user")
        sys.exit(0)
    except Exception as e:
        error_msg = f"Fatal error starting daemon: {type(e).__name__}: {e}"
        logger.error(error_msg, exc_info=True)
        print(f"\n❌ {error_msg}\n", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
