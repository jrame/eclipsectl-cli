"""MCP server for Eclipse IDE control via eclipsectl-mcp."""

import asyncio
import logging
import sys
from typing import Any, Dict

from mcp.server import Server
from mcp.server.stdio import stdio_server
from mcp.types import Tool, TextContent

from .config import parse_args
from .eclipse_client import EclipseClient
from .tools import TOOLS

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    stream=sys.stderr
)
logger = logging.getLogger(__name__)


class EclipseServer:
    """MCP server for Eclipse IDE operations via eclipsectl-mcp plugin."""

    def __init__(self, config):
        self.config = config
        self.eclipse_client: EclipseClient | None = None
        self.server = Server("eclipse")

        # Register handlers
        self.server.list_tools()(self.handle_list_tools)
        self.server.call_tool()(self.handle_call_tool)

        if config.verbose:
            logger.setLevel(logging.DEBUG)

    async def handle_list_tools(self) -> list[Tool]:
        """List available Eclipse tools."""
        logger.debug("Listing tools")

        tools = []
        for tool_def in TOOLS:
            tools.append(Tool(
                name=tool_def["name"],
                description=tool_def["description"],
                inputSchema=tool_def["inputSchema"]
            ))

        return tools

    async def handle_call_tool(self, name: str, arguments: Dict[str, Any]) -> list[TextContent]:
        """Handle tool execution."""
        logger.info(f"Calling tool: {name} with args: {arguments}")

        # Initialize Eclipse client if not already done
        if not self.eclipse_client:
            if not self.config.bearer_token:
                error_msg = (
                    "No bearer token configured. Please set ECLIPSE_TOKEN env var, "
                    "use --bearer-token flag, or add 'bearer_token' to ~/.config/eclipsectl-cli/config.json"
                )
                logger.error(error_msg)
                return [TextContent(type="text", text=f"Error: {error_msg}")]

            self.eclipse_client = EclipseClient(
                base_url=self.config.eclipse_url,
                bearer_token=self.config.bearer_token,
                timeout=self.config.timeout
            )

        # Connect to eclipsectl-mcp if not connected
        try:
            await self.eclipse_client.connect()
        except Exception as e:
            error_msg = (
                f"Cannot connect to Eclipse eclipsectl-mcp at {self.config.eclipse_url}. "
                f"Error: {e}\n"
                "Is Eclipse running with eclipsectl-mcp plugin and HTTP MCP Server enabled?"
            )
            logger.error(error_msg)
            return [TextContent(type="text", text=f"Error: {error_msg}")]

        try:
            # Route to appropriate handler
            result = await self._route_tool_call(name, arguments)
            return [TextContent(type="text", text=str(result))]

        except Exception as e:
            logger.error(f"Tool execution failed: {e}", exc_info=True)
            return [TextContent(type="text", text=f"Error: {str(e)}")]

    async def _route_tool_call(self, name: str, args: Dict[str, Any]) -> str:
        """Route tool call to appropriate Eclipse client method."""

        # ===== Workspace =====
        if name == "eclipse_refresh_file":
            return await self.eclipse_client.refresh_file(args["file"])

        elif name == "eclipse_refresh_workspace":
            return await self.eclipse_client.refresh_workspace()

        # ===== Project Management =====
        elif name == "eclipse_list_projects":
            return await self.eclipse_client.list_projects()

        elif name == "eclipse_get_project_layout":
            return await self.eclipse_client.get_project_layout(args["project"])

        elif name == "eclipse_get_project_properties":
            return await self.eclipse_client.get_project_properties(args["project"])

        # ===== Code Analysis =====
        elif name == "eclipse_get_javadoc":
            return await self.eclipse_client.get_javadoc(args["fqn"])

        elif name == "eclipse_get_source":
            return await self.eclipse_client.get_source(args["fqn"])

        elif name == "eclipse_get_call_hierarchy":
            return await self.eclipse_client.get_call_hierarchy_by_fqn(args["fqn"], args["method"])

        # ===== Build & Errors =====
        elif name == "eclipse_maven_build":
            return await self.eclipse_client.run_maven_build(args["project"], args["goals"])

        elif name == "eclipse_get_compilation_errors":
            project = args.get("project")
            return await self.eclipse_client.get_compilation_errors(project)

        # ===== Tests =====
        elif name == "eclipse_run_all_tests":
            return await self.eclipse_client.run_all_tests()

        elif name == "eclipse_run_package_tests":
            return await self.eclipse_client.run_package_tests(args["project"], args["package"])

        elif name == "eclipse_run_class_tests":
            return await self.eclipse_client.run_class_tests(args["project"], args["class"])

        elif name == "eclipse_run_test_method":
            return await self.eclipse_client.run_test_method(
                args["project"], args["class"], args["method"]
            )

        # ===== Editor Operations =====
        elif name == "eclipse_get_current_file":
            return await self.eclipse_client.get_current_file()

        elif name == "eclipse_get_editor_selection":
            return await self.eclipse_client.get_editor_selection()

        elif name == "eclipse_format_code":
            return await self.eclipse_client.format_code(args["file"])

        # ===== Console =====
        elif name == "eclipse_get_console_output":
            console_name = args.get("consoleName")
            return await self.eclipse_client.get_console_output(console_name)

        # ===== Search & Navigation =====
        elif name == "eclipse_file_search":
            return await self.eclipse_client.file_search(
                args["containingText"], args.get("fileNamePatterns"))
        elif name == "eclipse_file_search_regexp":
            return await self.eclipse_client.file_search_regexp(
                args["pattern"], args.get("fileNamePatterns"))
        elif name == "eclipse_find_files":
            return await self.eclipse_client.find_files(
                args.get("fileNamePatterns"), int(args.get("maxResults", 200)))
        elif name == "eclipse_open_type":
            return await self.eclipse_client.open_type(args["name"])
        elif name == "eclipse_find_references":
            return await self.eclipse_client.find_references(
                args["elementType"], args["elementName"],
                args.get("projectName"), args.get("scope", "workspace"),
                args.get("includeSubtypes", False))
        elif name == "eclipse_get_type_hierarchy":
            return await self.eclipse_client.get_type_hierarchy(args["fullyQualifiedClassName"])
        elif name == "eclipse_call_hierarchy":
            return await self.eclipse_client.call_hierarchy(
                args["fullyQualifiedClassName"], args["methodName"],
                args.get("methodSignature"))
        elif name == "eclipse_get_implementations":
            return await self.eclipse_client.get_implementations(
                args["fullyQualifiedClassName"], args.get("methodName"))

        # ===== Problems & Quick Fixes =====
        elif name == "eclipse_get_problems":
            return await self.eclipse_client.get_problems(
                args.get("severity", "error"), int(args.get("maxResults", 10)))
        elif name == "eclipse_get_quick_fixes":
            return await self.eclipse_client.get_quick_fixes(
                args.get("projectName"), args["filePath"],
                int(args["lineNumber"]) if "lineNumber" in args else None,
                args.get("problemId"))
        elif name == "eclipse_apply_quick_fix":
            return await self.eclipse_client.apply_quick_fix(
                args.get("projectName"), args["filePath"],
                int(args["lineNumber"]), int(args["fixIndex"]))
        elif name == "eclipse_get_markers":
            return await self.eclipse_client.get_markers(
                args.get("projectName"), args["filePath"])

        # ===== Maven =====
        elif name == "eclipse_list_maven_projects":
            return await self.eclipse_client.list_maven_projects()
        elif name == "eclipse_get_effective_pom":
            return await self.eclipse_client.get_effective_pom(args["projectName"])
        elif name == "eclipse_get_project_dependencies":
            return await self.eclipse_client.get_project_dependencies(args["projectName"])
        elif name == "eclipse_find_test_classes":
            return await self.eclipse_client.find_test_classes(args["projectName"])

        # ===== Refactoring =====
        elif name == "eclipse_rename_file":
            return await self.eclipse_client.rename_file(
                args.get("projectName"), args["filePath"], args["newFileName"])
        elif name == "eclipse_refactor_rename_java_type":
            return await self.eclipse_client.refactor_rename_java_type(
                args.get("projectName"), args["filePath"], args["newTypeName"])
        elif name == "eclipse_refactor_move_java_type":
            return await self.eclipse_client.refactor_move_java_type(
                args.get("projectName"), args["filePath"], args["targetPackage"])
        elif name == "eclipse_refactor_rename_package":
            return await self.eclipse_client.refactor_rename_package(
                args.get("projectName"), args["packageName"], args["newPackageName"])
        elif name == "eclipse_move_resource":
            return await self.eclipse_client.move_resource(
                args.get("projectName"), args["sourcePath"], args["targetPath"])
        elif name == "eclipse_refactor_extract_method":
            return await self.eclipse_client.refactor_extract_method(
                args.get("projectName"), args["filePath"],
                int(args["startLine"]), int(args["endLine"]),
                args["methodName"], args.get("visibility", "private"))

        # ===== Code Generation =====
        elif name == "eclipse_generate_getters_setters":
            return await self.eclipse_client.generate_getters_setters(
                args.get("projectName"), args["filePath"],
                args.get("className"),
                args.get("fieldNames", "").split(",") if args.get("fieldNames") else None,
                args.get("generateGetters", True), args.get("generateSetters", True),
                args.get("insertionPoint", "end"))
        elif name == "eclipse_generate_tostring_equals_hashcode":
            return await self.eclipse_client.generate_tostring_equals_hashcode(
                args.get("projectName"), args["filePath"],
                args.get("className"),
                args.get("fieldNames", "").split(",") if args.get("fieldNames") else None)
        elif name == "eclipse_implement_override_methods":
            return await self.eclipse_client.implement_override_methods(
                args.get("projectName"), args["filePath"],
                args.get("className"),
                args.get("methodNames", "").split(",") if args.get("methodNames") else None)

        # ===== Source Operations =====
        elif name == "eclipse_organize_imports":
            return await self.eclipse_client.organize_imports(
                args.get("projectName"), args["filePath"])
        elif name == "eclipse_organize_imports_in_package":
            return await self.eclipse_client.organize_imports_in_package(
                args.get("projectName"), args["packageName"])
        elif name == "eclipse_format_file":
            return await self.eclipse_client.format_file(
                args.get("projectName"), args["filePath"], args.get("lines"))

        # ===== Run =====
        elif name == "eclipse_run":
            return await self.eclipse_client.run(args.get("configuration"))

        # ===== Refresh =====
        elif name == "eclipse_refresh":
            return await self.eclipse_client.refresh(args["path"])
        elif name == "eclipse_refresh_and_save":
            return await self.eclipse_client.refresh_and_save(args["paths"])

        # ===== Debug =====
        elif name == "eclipse_add_breakpoint":
            return await self.eclipse_client.add_breakpoint(
                args["filePath"], int(args["lineNumber"]), args.get("condition"))
        elif name == "eclipse_remove_breakpoint":
            return await self.eclipse_client.remove_breakpoint(
                args["filePath"], int(args["lineNumber"]))
        elif name == "eclipse_list_breakpoints":
            return await self.eclipse_client.list_breakpoints()
        elif name == "eclipse_enable_breakpoint":
            return await self.eclipse_client.enable_breakpoint(
                args["filePath"], int(args["lineNumber"]))
        elif name == "eclipse_disable_breakpoint":
            return await self.eclipse_client.disable_breakpoint(
                args["filePath"], int(args["lineNumber"]))
        elif name == "eclipse_list_threads":
            return await self.eclipse_client.list_threads()
        elif name == "eclipse_select_thread":
            return await self.eclipse_client.select_thread(args["threadId"])
        elif name == "eclipse_run_debug":
            return await self.eclipse_client.run_debug(args["configuration"])
        elif name == "eclipse_list_debug_configurations":
            return await self.eclipse_client.list_run_configurations()
        elif name == "eclipse_create_java_launch_configuration":
            return await self.eclipse_client.create_java_launch_configuration(
                args["mainClass"],
                args.get("configName"),
                args.get("projectName"),
                args.get("programArguments"),
                args.get("vmArguments"),
                args.get("workingDirectory"),
                args.get("environmentVariables"),
                args.get("appendSystemEnvironment", True),
                args.get("nameConflict", "error"),
            )
        elif name == "eclipse_get_launch_configuration":
            return await self.eclipse_client.get_launch_configuration(
                args["configName"], args.get("includeEnvironmentValues", False))
        elif name == "eclipse_update_launch_environment":
            return await self.eclipse_client.update_launch_environment(
                args["configName"],
                args.get("variables"),
                args.get("removeVariables"),
                args.get("updateMode", "merge"),
                args.get("appendSystemEnvironment"),
            )
        elif name == "eclipse_step_into":
            return await self.eclipse_client.step_into()
        elif name == "eclipse_step_over":
            return await self.eclipse_client.step_over()
        elif name == "eclipse_step_return":
            return await self.eclipse_client.step_return()
        elif name == "eclipse_debug_resume":
            return await self.eclipse_client.debug_resume()
        elif name == "eclipse_debug_suspend":
            return await self.eclipse_client.debug_suspend()
        elif name == "eclipse_debug_terminate":
            return await self.eclipse_client.debug_terminate()
        elif name == "eclipse_get_stack_trace":
            return await self.eclipse_client.get_stack_trace(args.get("threadId"))
        elif name == "eclipse_get_variables":
            return await self.eclipse_client.get_variables(args.get("frameId"))
        elif name == "eclipse_evaluate_expression":
            return await self.eclipse_client.evaluate_expression(
                args["expression"], args.get("frameId"))
        elif name == "eclipse_get_debug_status":
            return await self.eclipse_client.get_debug_status()

        # ===== Coverage =====
        elif name == "eclipse_run_coverage":
            return await self.eclipse_client.run_coverage(args["configuration"])
        elif name == "eclipse_get_coverage_report":
            return await self.eclipse_client.get_coverage_report(args.get("projectName"))
        elif name == "eclipse_get_class_coverage":
            return await self.eclipse_client.get_class_coverage(args["className"])

        # ===== Project Resource =====
        elif name == "eclipse_read_project_resource":
            return await self.eclipse_client.read_project_resource(args["project"], args["path"])
        elif name == "eclipse_import_project":
            return await self.eclipse_client.import_project(args["path"])

        # ===== Directory & Replace =====
        elif name == "eclipse_create_directories":
            return await self.eclipse_client.create_directories(
                args["projectName"], args["directoryPath"])
        elif name == "eclipse_search_and_replace":
            return await self.eclipse_client.search_and_replace(
                args["containingText"], args["replacementText"], args.get("fileNamePatterns"))

        else:
            return f"Unknown tool: {name}"

    async def run(self):
        """Run the MCP server."""
        logger.info(f"Starting Eclipse MCP server")
        logger.info(f"Eclipse URL: {self.config.eclipse_url}")
        logger.info(f"Bearer token: {'configured' if self.config.bearer_token else 'NOT SET'}")

        async with stdio_server() as (read_stream, write_stream):
            await self.server.run(
                read_stream,
                write_stream,
                self.server.create_initialization_options()
            )


async def main():
    """Main entry point."""
    config = parse_args()
    server = EclipseServer(config)
    await server.run()


if __name__ == "__main__":
    asyncio.run(main())
