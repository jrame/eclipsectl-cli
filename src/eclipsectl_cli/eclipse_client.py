"""HTTP client for communicating with Eclipse IDE via eclipsectl-mcp MCP."""

import asyncio
from typing import Any, Dict, List, Optional
import logging
import httpx

from mcp.client.session import ClientSession
from mcp.client.streamable_http import streamable_http_client

from .path_converter import PathConverter

logger = logging.getLogger(__name__)

# Static mapping from MCP tool name to server name.
# Built from @McpServer/@Tool annotations in the Eclipse plugin.
TOOL_TO_SERVER: Dict[str, str] = {
    # eclipse-ide (27 tools) — EclipseIntegrationsMcpServer
    "formatCode": "eclipse-ide",
    "getJavaDoc": "eclipse-ide",
    "getSource": "eclipse-ide",
    "getProjectProperties": "eclipse-ide",
    "getProjectLayout": "eclipse-ide",
    "readProjectResource": "eclipse-ide",
    "listProjects": "eclipse-ide",
    "importProject": "eclipse-ide",
    "getCurrentlyOpenedFile": "eclipse-ide",
    "getEditorSelection": "eclipse-ide",
    "getConsoleOutput": "eclipse-ide",
    "runAllTests": "eclipse-ide",
    "runPackageTests": "eclipse-ide",
    "runClassTests": "eclipse-ide",
    "runTestMethod": "eclipse-ide",
    "findTestClasses": "eclipse-ide",
    "runMavenBuild": "eclipse-ide",
    "getEffectivePom": "eclipse-ide",
    "listMavenProjects": "eclipse-ide",
    "getProjectDependencies": "eclipse-ide",
    "fileSearch": "eclipse-ide",
    "fileSearchRegExp": "eclipse-ide",
    "findFiles": "eclipse-ide",
    "getMethodCallHierarchy": "eclipse-ide",
    "getCompilationErrors": "eclipse-ide",
    "getWorkspaceInfo": "eclipse-ide",
    "getOpenEditors": "eclipse-ide",
    "createDirectories": "eclipse-ide",
    "searchAndReplace": "eclipse-ide",
    # eclipse (12 tools) — EclipseMcpServer
    "open_type": "eclipse",
    "call_hierarchy": "eclipse",
    "findReferences": "eclipse",
    "run": "eclipse",
    "problems": "eclipse",
    "getQuickFixes": "eclipse",
    "applyQuickFix": "eclipse",
    "getTypeHierarchy": "eclipse",
    "getImplementations": "eclipse",
    "getMarkers": "eclipse",
    "refresh": "eclipse",
    "refreshAndSave": "eclipse",
    "openFile": "eclipse",
    # eclipse-refactor (6 tools) — EclipseRefactorMcpServer
    "renameFile": "eclipse-refactor",
    "refactorRenameJavaType": "eclipse-refactor",
    "refactorMoveJavaType": "eclipse-refactor",
    "refactorRenamePackage": "eclipse-refactor",
    "moveResource": "eclipse-refactor",
    "refactorExtractMethod": "eclipse-refactor",
    # eclipse-codegen (3 tools) — EclipseCodeGenerationMcpServer
    "generateGettersSetters": "eclipse-codegen",
    "generateToStringEqualsHashCode": "eclipse-codegen",
    "implementOverrideMethods": "eclipse-codegen",
    # eclipse-source (8 tools) — EclipseSourceMcpServer
    "organizeImports": "eclipse-source",
    "organizeImportsInPackage": "eclipse-source",
    "formatFile": "eclipse-source",
    "readJava": "eclipse-source",
    "writeJava": "eclipse-source",
    "removeJava": "eclipse-source",
    "removeLines": "eclipse-source",
    "resetReadGate": "eclipse-source",
    # eclipse-debug (22 tools) — EclipseDebugMcpServer
    "addBreakpoint": "eclipse-debug",
    "removeBreakpoint": "eclipse-debug",
    "listBreakpoints": "eclipse-debug",
    "enableBreakpoint": "eclipse-debug",
    "disableBreakpoint": "eclipse-debug",
    "listThreads": "eclipse-debug",
    "selectThread": "eclipse-debug",
    "runDebug": "eclipse-debug",
    "listRunConfigurations": "eclipse-debug",
    "createJavaLaunchConfiguration": "eclipse-debug",
    "getLaunchConfiguration": "eclipse-debug",
    "updateLaunchEnvironment": "eclipse-debug",
    "stepInto": "eclipse-debug",
    "stepOver": "eclipse-debug",
    "stepReturn": "eclipse-debug",
    "resume": "eclipse-debug",
    "suspend": "eclipse-debug",
    "terminate": "eclipse-debug",
    "getStackTrace": "eclipse-debug",
    "getVariables": "eclipse-debug",
    "evaluateExpression": "eclipse-debug",
    "getDebugStatus": "eclipse-debug",
    # eclipse-coverage (3 tools) — EclipseCoverageMcpServer
    "runCoverage": "eclipse-coverage",
    "getCoverageReport": "eclipse-coverage",
    "getClassCoverage": "eclipse-coverage",
}

# Default server for unknown tools
DEFAULT_SERVER = "eclipse-ide"


class EclipseClient:
    """Client for Eclipse eclipsectl-mcp MCP server."""

    def __init__(self, base_url: str, bearer_token: Optional[str] = None, timeout: int = 30):
        """
        Initialize Eclipse MCP client.

        Args:
            base_url: Base URL of eclipsectl-mcp server (e.g., http://localhost:8765)
            bearer_token: Bearer token for authentication
            timeout: Request timeout in seconds
        """
        self.base_url = base_url.rstrip('/')
        self.bearer_token = bearer_token
        self.timeout = timeout
        # Multi-session: one session per MCP server, lazily connected
        self.sessions: Dict[str, ClientSession] = {}
        self._session_ctxs: Dict[str, Any] = {}
        self._transport_ctxs: Dict[str, Any] = {}

    async def _connect_server(self, server_name: str) -> ClientSession:
        """
        Connect to a specific MCP server lazily.

        Args:
            server_name: MCP server name (e.g., "eclipse-ide", "eclipse-debug")

        Returns:
            The connected ClientSession for this server
        """
        if server_name in self.sessions:
            return self.sessions[server_name]

        # Setup HTTP client with proper headers
        headers = {
            "Accept": "text/event-stream, application/json",
            "Content-Type": "application/json"
        }
        if self.bearer_token:
            headers["Authorization"] = f"Bearer {self.bearer_token}"

        http_client = httpx.AsyncClient(
            headers=headers,
            timeout=httpx.Timeout(self.timeout, read=300.0)
        )

        url = f"{self.base_url}/mcp/{server_name}"
        logger.info(f"Connecting to MCP server '{server_name}' at {url}")

        transport_ctx = streamable_http_client(
            url=url,
            http_client=http_client
        )
        read_stream, write_stream, get_session_id = await transport_ctx.__aenter__()

        session_ctx = ClientSession(read_stream, write_stream)
        session = await session_ctx.__aenter__()

        await session.initialize()
        logger.info(f"Connected to MCP server '{server_name}' successfully")

        self.sessions[server_name] = session
        self._session_ctxs[server_name] = session_ctx
        self._transport_ctxs[server_name] = transport_ctx

        return session

    async def connect(self) -> None:
        """Connect to the default eclipse-ide MCP server (for health check / backward compat)."""
        await self._connect_server("eclipse-ide")

    async def disconnect(self) -> None:
        """Disconnect from all MCP servers."""
        for name in list(self._session_ctxs.keys()):
            try:
                await self._session_ctxs[name].__aexit__(None, None, None)
            except Exception as e:
                logger.warning(f"Error closing session for '{name}': {e}")

        for name in list(self._transport_ctxs.keys()):
            try:
                await self._transport_ctxs[name].__aexit__(None, None, None)
            except Exception as e:
                logger.warning(f"Error closing transport for '{name}': {e}")

        self.sessions.clear()
        self._session_ctxs.clear()
        self._transport_ctxs.clear()
        logger.info("Disconnected from all MCP servers")

    async def _call_mcp_tool(self, tool_name: str, arguments: Dict[str, Any]) -> Any:
        """
        Call an MCP tool, routing to the correct server automatically.

        Args:
            tool_name: Name of the MCP tool to call
            arguments: Tool arguments

        Returns:
            Tool result text

        Raises:
            Exception: If tool call fails
        """
        server_name = TOOL_TO_SERVER.get(tool_name, DEFAULT_SERVER)
        session = await self._connect_server(server_name)

        logger.debug(f"MCP call: {tool_name} on server '{server_name}' with {arguments}")

        result = await session.call_tool(tool_name, arguments)

        logger.debug(f"MCP response: {result}")

        # Extract text content from result
        if hasattr(result, 'content') and result.content:
            # Join all text content
            texts = [c.text for c in result.content if hasattr(c, 'text')]
            return '\n'.join(texts)

        return str(result)

    def ping(self) -> bool:
        """
        Check if eclipsectl-mcp server is reachable.

        Returns:
            True if server is reachable, False otherwise
        """
        try:
            # Run async ping in sync context
            loop = asyncio.get_event_loop()
            if loop.is_running():
                # Already in async context, create task
                return asyncio.create_task(self._async_ping()).result()
            else:
                return loop.run_until_complete(self._async_ping())
        except Exception as e:
            logger.error(f"eclipsectl-mcp ping failed: {e}")
            return False

    async def _async_ping(self) -> bool:
        """Async ping implementation."""
        try:
            session = await self._connect_server("eclipse-ide")
            # Try to list tools as health check
            await session.list_tools()
            return True
        except Exception as e:
            logger.error(f"Async ping failed: {e}")
            return False

    # ===== Workspace Operations =====

    async def refresh_file(self, file_path: str) -> str:
        """
        Refresh a file in Eclipse workspace.
        Note: eclipsectl-mcp doesn't expose refresh as MCP tool, this is a placeholder.
        """
        # Convert WSL to Windows path
        win_path = PathConverter.normalize_windows(file_path)
        logger.info(f"Refresh file: {win_path}")
        return f"File refreshed: {win_path}"

    async def refresh_workspace(self) -> str:
        """
        Refresh entire Eclipse workspace.
        Note: eclipsectl-mcp doesn't expose refresh as MCP tool, this is a placeholder.
        """
        logger.info("Refresh workspace requested")
        return "Workspace refresh requested"

    # ===== Project Management =====

    async def list_projects(self) -> str:
        """List all projects in Eclipse workspace."""
        return await self._call_mcp_tool("listProjects", {})

    async def get_project_layout(self, project: str) -> str:
        """Get project structure."""
        return await self._call_mcp_tool("getProjectLayout", {"projectName": project})

    async def get_project_properties(self, project: str) -> str:
        """Get project properties."""
        return await self._call_mcp_tool("getProjectProperties", {"projectName": project})

    async def read_project_resource(self, project: str, path: str) -> str:
        """Read a file from project (relative path)."""
        return await self._call_mcp_tool("readProjectResource", {
            "projectName": project,
            "resourcePath": path
        })

    async def import_project(self, path: str) -> str:
        """Import a project into the workspace."""
        win_path = PathConverter.normalize_windows(path)
        return await self._call_mcp_tool("importProject", {"projectPath": win_path})

    # ===== Code Analysis =====

    async def get_javadoc(self, fully_qualified_name: str) -> str:
        """Get JavaDoc for a fully qualified class name."""
        return await self._call_mcp_tool("getJavaDoc", {
            "fullyQualifiedName": fully_qualified_name
        })

    async def get_source(self, fully_qualified_class: str) -> str:
        """Get source code for a fully qualified class name."""
        return await self._call_mcp_tool("getSource", {
            "fullyQualifiedClassName": fully_qualified_class
        })

    async def get_call_hierarchy_by_fqn(self, fully_qualified_class: str, method_name: str) -> str:
        """Get method call hierarchy by class name and method."""
        return await self._call_mcp_tool("getMethodCallHierarchy", {
            "fullyQualifiedClassName": fully_qualified_class,
            "methodName": method_name
        })

    # ===== Build & Errors =====

    async def run_maven_build(self, project: str, goals: List[str]) -> str:
        """Run Maven build with given goals."""
        return await self._call_mcp_tool("runMavenBuild", {
            "projectName": project,
            "goals": goals
        })

    async def get_compilation_errors(self, project: Optional[str] = None) -> str:
        """Get compilation errors."""
        args = {}
        if project:
            args["projectName"] = project
        return await self._call_mcp_tool("getCompilationErrors", args)

    # ===== Tests =====

    async def run_all_tests(self) -> str:
        """Run all tests in workspace."""
        return await self._call_mcp_tool("runAllTests", {})

    async def run_package_tests(self, project: str, package: str) -> str:
        """Run tests for a package."""
        return await self._call_mcp_tool("runPackageTests", {
            "projectName": project,
            "packageName": package
        })

    async def run_class_tests(self, project: str, class_name: str) -> str:
        """Run tests for a class."""
        return await self._call_mcp_tool("runClassTests", {
            "projectName": project,
            "className": class_name
        })

    async def run_test_method(self, project: str, class_name: str, method: str) -> str:
        """Run a specific test method."""
        return await self._call_mcp_tool("runTestMethod", {
            "projectName": project,
            "className": class_name,
            "methodName": method
        })

    # ===== Editor Operations =====

    async def get_current_file(self) -> str:
        """Get currently opened file in Eclipse."""
        return await self._call_mcp_tool("getCurrentlyOpenedFile", {})

    async def get_editor_selection(self) -> str:
        """Get current editor selection."""
        return await self._call_mcp_tool("getEditorSelection", {})

    async def format_code(self, file_path: str) -> str:
        """Format code according to Eclipse formatter."""
        win_path = PathConverter.normalize_windows(file_path)
        return await self._call_mcp_tool("formatFile", {
            "filePath": win_path
        })

    # ===== Console =====

    async def get_console_output(self, console_name: Optional[str] = None) -> str:
        """Get Eclipse console output."""
        args = {}
        if console_name:
            args["consoleName"] = console_name
        return await self._call_mcp_tool("getConsoleOutput", args)

    # ===== Refactoring Operations (eclipse-refactor server) =====

    async def rename_file(self, project: Optional[str], file_path: str, new_name: str) -> str:
        """Rename a file in the project."""
        win_path = PathConverter.normalize_windows(file_path)
        args = {"filePath": win_path, "newFileName": new_name}
        if project:
            args["projectName"] = project
        return await self._call_mcp_tool("renameFile", args)

    async def refactor_rename_java_type(self, project: Optional[str], file_path: str, new_type_name: str) -> str:
        """Rename a Java class/interface/enum with all references."""
        win_path = PathConverter.normalize_windows(file_path)
        args = {"filePath": win_path, "newTypeName": new_type_name}
        if project:
            args["projectName"] = project
        return await self._call_mcp_tool("refactorRenameJavaType", args)

    async def refactor_move_java_type(self, project: Optional[str], file_path: str, target_package: str) -> str:
        """Move a Java class/interface/enum to a different package."""
        win_path = PathConverter.normalize_windows(file_path)
        args = {"filePath": win_path, "targetPackage": target_package}
        if project:
            args["projectName"] = project
        return await self._call_mcp_tool("refactorMoveJavaType", args)

    async def refactor_rename_package(self, project: Optional[str], package_name: str, new_package_name: str) -> str:
        """Rename a Java package with all references."""
        args = {"packageName": package_name, "newPackageName": new_package_name}
        if project:
            args["projectName"] = project
        return await self._call_mcp_tool("refactorRenamePackage", args)

    async def move_resource(self, project: Optional[str], source_path: str, target_path: str) -> str:
        """Move a file or folder to a different location."""
        win_source = PathConverter.normalize_windows(source_path)
        win_target = PathConverter.normalize_windows(target_path)
        args = {"sourcePath": win_source, "targetPath": win_target}
        if project:
            args["projectName"] = project
        return await self._call_mcp_tool("moveResource", args)

    async def refactor_extract_method(
        self,
        project: Optional[str],
        file_path: str,
        start_line: int,
        end_line: int,
        method_name: str,
        visibility: Optional[str] = "private"
    ) -> str:
        """Extract code lines into a new method."""
        win_path = PathConverter.normalize_windows(file_path)
        args = {
            "filePath": win_path,
            "startLine": start_line,
            "endLine": end_line,
            "methodName": method_name
        }
        if project:
            args["projectName"] = project
        if visibility:
            args["visibility"] = visibility
        return await self._call_mcp_tool("refactorExtractMethod", args)

    # ===== Code Generation (eclipse-codegen server) =====

    async def generate_getters_setters(
        self,
        project: Optional[str],
        file_path: str,
        class_name: Optional[str] = None,
        field_names: Optional[List[str]] = None,
        generate_getters: bool = True,
        generate_setters: bool = True,
        insertion_point: str = "end"
    ) -> str:
        """Generate getter and/or setter methods for fields."""
        win_path = PathConverter.normalize_windows(file_path)
        args = {
            "filePath": win_path,
            "generateGetters": generate_getters,
            "generateSetters": generate_setters,
            "insertionPoint": insertion_point
        }
        if project:
            args["projectName"] = project
        if class_name:
            args["className"] = class_name
        if field_names:
            args["fieldNames"] = ",".join(field_names)
        return await self._call_mcp_tool("generateGettersSetters", args)

    async def generate_tostring_equals_hashcode(
        self,
        project: Optional[str],
        file_path: str,
        class_name: Optional[str] = None,
        field_names: Optional[List[str]] = None,
        generate_tostring: bool = True,
        generate_equals: bool = True,
        generate_hashcode: bool = True
    ) -> str:
        """Generate toString(), equals(), and/or hashCode() methods."""
        win_path = PathConverter.normalize_windows(file_path)
        args = {
            "filePath": win_path,
            "generateToString": generate_tostring,
            "generateEquals": generate_equals,
            "generateHashCode": generate_hashcode
        }
        if project:
            args["projectName"] = project
        if class_name:
            args["className"] = class_name
        if field_names:
            args["fieldNames"] = ",".join(field_names)
        return await self._call_mcp_tool("generateToStringEqualsHashCode", args)

    async def implement_override_methods(
        self,
        project: Optional[str],
        file_path: str,
        class_name: Optional[str] = None,
        method_names: Optional[List[str]] = None
    ) -> str:
        """Implement or override methods from superclass/interfaces."""
        win_path = PathConverter.normalize_windows(file_path)
        args = {"filePath": win_path}
        if project:
            args["projectName"] = project
        if class_name:
            args["className"] = class_name
        if method_names:
            args["methodNames"] = ",".join(method_names)
        return await self._call_mcp_tool("implementOverrideMethods", args)

    # ===== Source Operations (eclipse-source server) =====

    async def organize_imports(self, project: Optional[str], file_path: str) -> str:
        """Organize imports in a Java file."""
        win_path = PathConverter.normalize_windows(file_path)
        args = {"filePath": win_path}
        if project:
            args["projectName"] = project
        return await self._call_mcp_tool("organizeImports", args)

    async def organize_imports_in_package(self, project: Optional[str], package_name: str) -> str:
        """Organize imports in all files in a package."""
        args = {"packageName": package_name}
        if project:
            args["projectName"] = project
        return await self._call_mcp_tool("organizeImportsInPackage", args)

    async def format_file(self, project: Optional[str], file_path: str, lines: Optional[str] = None) -> str:
        """Format a Java file using Eclipse formatter."""
        win_path = PathConverter.normalize_windows(file_path)
        args = {"filePath": win_path}
        if project:
            args["projectName"] = project
        if lines:
            args["lines"] = lines
        return await self._call_mcp_tool("formatFile", args)

    # ===== Java Member Editing (eclipse-source server) =====

    async def read_java(self, project: Optional[str], file_path: str, class_name: Optional[str] = None) -> str:
        """Read Java class structure (fields, methods, constructors, inner types)."""
        win_path = PathConverter.normalize_windows(file_path)
        args = {"filePath": win_path}
        if project:
            args["projectName"] = project
        if class_name:
            args["className"] = class_name
        return await self._call_mcp_tool("readJava", args)

    async def write_java(self, project: Optional[str], file_path: str, new_source: str, class_name: Optional[str] = None) -> str:
        """Write a code member (method, field, constructor, inner class) into a Java class."""
        win_path = PathConverter.normalize_windows(file_path)
        args = {"filePath": win_path, "newSource": new_source}
        if project:
            args["projectName"] = project
        if class_name:
            args["className"] = class_name
        return await self._call_mcp_tool("writeJava", args)

    async def remove_java(self, project: Optional[str], file_path: str, members: str, class_name: Optional[str] = None) -> str:
        """Remove named members from a Java class."""
        win_path = PathConverter.normalize_windows(file_path)
        args = {"filePath": win_path, "members": members}
        if project:
            args["projectName"] = project
        if class_name:
            args["className"] = class_name
        return await self._call_mcp_tool("removeJava", args)

    async def remove_lines(self, project: Optional[str], file_path: str, lines: str) -> str:
        """Remove line ranges from a Java file."""
        win_path = PathConverter.normalize_windows(file_path)
        args = {"filePath": win_path, "lines": lines}
        if project:
            args["projectName"] = project
        return await self._call_mcp_tool("removeLines", args)

    async def reset_read_gate(self, project: Optional[str] = None, file_path: Optional[str] = None) -> str:
        """Reset the readJava gate for a file or all files."""
        args = {}
        if project:
            args["projectName"] = project
        if file_path:
            win_path = PathConverter.normalize_windows(file_path)
            args["filePath"] = win_path
        return await self._call_mcp_tool("resetReadGate", args)

    # ===== Directory & Replace (eclipse-ide server) =====

    async def create_directories(self, project: str, directory_path: str) -> str:
        """Create directories (and parents) within a project."""
        return await self._call_mcp_tool("createDirectories", {
            "projectName": project,
            "directoryPath": directory_path
        })

    async def search_and_replace(self, containing_text: str, replacement_text: str, file_name_patterns: Optional[str] = None) -> str:
        """Search and replace text across workspace files."""
        args = {"containingText": containing_text, "replacementText": replacement_text}
        if file_name_patterns:
            args["fileNamePatterns"] = file_name_patterns
        return await self._call_mcp_tool("searchAndReplace", args)

    # ===== Eclipse Core Tools (eclipse server) =====

    async def open_type(self, type_name: str) -> str:
        """Search for a type by name using Eclipse's Open Type search."""
        return await self._call_mcp_tool("open_type", {"name": type_name})

    async def call_hierarchy(
        self,
        fully_qualified_class: str,
        method_name: str,
        method_signature: Optional[str] = None
    ) -> str:
        """Get call hierarchy for a method (callers and callees)."""
        args = {
            "fullyQualifiedClassName": fully_qualified_class,
            "methodName": method_name
        }
        if method_signature:
            args["methodSignature"] = method_signature
        return await self._call_mcp_tool("call_hierarchy", args)

    async def find_references(
        self,
        element_type: str,
        element_name: str,
        project_name: Optional[str] = None,
        scope: str = "workspace",
        include_subtypes: bool = False
    ) -> str:
        """Find all references to a Java element."""
        args = {
            "elementType": element_type,
            "elementName": element_name,
            "scope": scope,
            "includeSubtypes": include_subtypes
        }
        if project_name:
            args["projectName"] = project_name
        return await self._call_mcp_tool("findReferences", args)

    async def run(self, configuration: Optional[str] = None, wait: bool = False) -> str:
        """Run an Eclipse launch configuration."""
        args = {}
        if configuration:
            args["configuration"] = configuration
        if wait:
            args["waitForCompletion"] = True
        return await self._call_mcp_tool("run", args)

    async def get_problems(
        self,
        severity: str = "error",
        max_results: int = 10
    ) -> str:
        """Get problems from Eclipse Problems panel."""
        # Note: eclipse-ide server uses 'getCompilationErrors' instead of 'problems'
        # Severity mapping with aliases: error/err -> ERROR, warning/warn -> WARNING, all -> ALL
        severity_map = {
            "error": "ERROR",
            "err": "ERROR",
            "warning": "WARNING",
            "warn": "WARNING",
            "all": "ALL"
        }
        mapped_severity = severity_map.get(severity.lower(), "ERROR")
        return await self._call_mcp_tool("getCompilationErrors", {
            "severity": mapped_severity,
            "maxResults": max_results
        })

    async def get_quick_fixes(
        self,
        project: Optional[str],
        file_path: str,
        line_number: Optional[int] = None,
        problem_id: Optional[str] = None
    ) -> str:
        """Get available Quick Fixes for problems in a file."""
        win_path = PathConverter.normalize_windows(file_path)
        args = {"filePath": win_path}
        if project:
            args["projectName"] = project
        if line_number:
            args["lineNumber"] = str(line_number)
        if problem_id:
            args["problemId"] = problem_id
        return await self._call_mcp_tool("getQuickFixes", args)

    async def apply_quick_fix(
        self,
        project: Optional[str],
        file_path: str,
        line_number: int,
        fix_index: int
    ) -> str:
        """Apply a Quick Fix to resolve a problem."""
        win_path = PathConverter.normalize_windows(file_path)
        args = {
            "filePath": win_path,
            "lineNumber": line_number,
            "fixIndex": fix_index
        }
        if project:
            args["projectName"] = project
        return await self._call_mcp_tool("applyQuickFix", args)

    async def get_type_hierarchy(self, fully_qualified_class: str) -> str:
        """Get type hierarchy (superclasses, subclasses, interfaces)."""
        return await self._call_mcp_tool("getTypeHierarchy", {
            "fullyQualifiedClassName": fully_qualified_class
        })

    async def get_implementations(
        self,
        fully_qualified_class: str,
        method_name: Optional[str] = None
    ) -> str:
        """Find implementations of an interface/abstract class/method."""
        args = {"fullyQualifiedClassName": fully_qualified_class}
        if method_name:
            args["methodName"] = method_name
        return await self._call_mcp_tool("getImplementations", args)

    async def get_markers(self, project: Optional[str], file_path: str) -> str:
        """Get all markers (errors, warnings, infos) for a file."""
        win_path = PathConverter.normalize_windows(file_path)
        args = {"filePath": win_path}
        if project:
            args["projectName"] = project
        return await self._call_mcp_tool("getMarkers", args)

    async def refresh(self, paths: str) -> str:
        """Refresh Eclipse files or directories."""
        # Convert WSL paths to Windows
        path_list = paths.split(",")
        win_paths = [PathConverter.normalize_windows(p.strip()) for p in path_list]
        return await self._call_mcp_tool("refresh", {
            "path": ",".join(win_paths)
        })

    async def refresh_and_save(self, paths: str) -> str:
        """Refresh and save files, triggering save actions."""
        # Convert WSL paths to Windows
        path_list = paths.split(",")
        win_paths = [PathConverter.normalize_windows(p.strip()) for p in path_list]
        return await self._call_mcp_tool("refreshAndSave", {
            "paths": ",".join(win_paths)
        })

    # ===== Search Operations (eclipse-ide server) =====

    async def file_search(
        self,
        containing_text: str,
        file_patterns: Optional[List[str]] = None
    ) -> str:
        """Search for text in workspace files."""
        args = {"containingText": containing_text}
        if file_patterns:
            args["fileNamePatterns"] = file_patterns
        return await self._call_mcp_tool("fileSearch", args)

    async def file_search_regexp(
        self,
        pattern: str,
        file_patterns: Optional[List[str]] = None
    ) -> str:
        """Search workspace files using regex."""
        args = {"pattern": pattern}
        if file_patterns:
            args["fileNamePatterns"] = file_patterns
        return await self._call_mcp_tool("fileSearchRegExp", args)

    async def find_files(
        self,
        file_patterns: Optional[List[str]] = None,
        max_results: int = 200
    ) -> str:
        """Find workspace files matching glob patterns."""
        args = {"maxResults": str(max_results)}
        if file_patterns:
            args["fileNamePatterns"] = file_patterns
        return await self._call_mcp_tool("findFiles", args)

    # ===== Maven Operations (eclipse-ide server) =====

    async def get_effective_pom(self, project: str) -> str:
        """Get effective POM for a Maven project."""
        return await self._call_mcp_tool("getEffectivePom", {"projectName": project})

    async def list_maven_projects(self) -> str:
        """List all Maven projects in workspace."""
        return await self._call_mcp_tool("listMavenProjects", {})

    async def get_project_dependencies(self, project: str) -> str:
        """Get Maven project dependencies."""
        return await self._call_mcp_tool("getProjectDependencies", {"projectName": project})

    async def find_test_classes(self, project: str) -> str:
        """Find all test classes in a project."""
        return await self._call_mcp_tool("findTestClasses", {"projectName": project})

    # ===== Debug Operations (eclipse-debug server) =====

    async def add_breakpoint(self, file_path: str, line_number: int, condition: Optional[str] = None) -> str:
        """Add a breakpoint at a specific line."""
        win_path = PathConverter.normalize_windows(file_path)
        args = {"filePath": win_path, "lineNumber": line_number}
        if condition:
            args["condition"] = condition
        return await self._call_mcp_tool("addBreakpoint", args)

    async def remove_breakpoint(self, file_path: str, line_number: int) -> str:
        """Remove a breakpoint at a specific line."""
        win_path = PathConverter.normalize_windows(file_path)
        return await self._call_mcp_tool("removeBreakpoint", {
            "filePath": win_path, "lineNumber": line_number
        })

    async def list_breakpoints(self) -> str:
        """List all breakpoints."""
        return await self._call_mcp_tool("listBreakpoints", {})

    async def enable_breakpoint(self, file_path: str, line_number: int) -> str:
        """Enable a breakpoint."""
        win_path = PathConverter.normalize_windows(file_path)
        return await self._call_mcp_tool("enableBreakpoint", {
            "filePath": win_path, "lineNumber": line_number
        })

    async def disable_breakpoint(self, file_path: str, line_number: int) -> str:
        """Disable a breakpoint."""
        win_path = PathConverter.normalize_windows(file_path)
        return await self._call_mcp_tool("disableBreakpoint", {
            "filePath": win_path, "lineNumber": line_number
        })

    async def list_threads(self) -> str:
        """List all threads in the debug session."""
        return await self._call_mcp_tool("listThreads", {})

    async def select_thread(self, thread_id: str) -> str:
        """Select a thread for inspection."""
        return await self._call_mcp_tool("selectThread", {"threadName": thread_id})

    async def run_debug(self, configuration: str, wait: bool = False) -> str:
        """Launch a debug configuration."""
        args = {"configName": configuration}
        if wait:
            args["waitForCompletion"] = True
        return await self._call_mcp_tool("runDebug", args)

    async def list_run_configurations(self) -> str:
        """List available launch configurations."""
        return await self._call_mcp_tool("listRunConfigurations", {})

    async def list_debug_configurations(self) -> str:
        """Backward-compatible alias for listing launch configurations."""
        return await self.list_run_configurations()

    async def create_java_launch_configuration(
        self,
        main_class: str,
        config_name: Optional[str] = None,
        project_name: Optional[str] = None,
        program_arguments: Optional[str] = None,
        vm_arguments: Optional[str] = None,
        working_directory: Optional[str] = None,
        environment_variables: Optional[Dict[str, str]] = None,
        append_system_environment: bool = True,
        name_conflict: str = "error",
        launch_type: Optional[str] = None,
    ) -> str:
        """Create an Eclipse Java-based launch configuration."""
        if name_conflict not in {"error", "generate"}:
            raise ValueError("name_conflict must be 'error' or 'generate'")
        if launch_type is not None and launch_type not in {
            "java", "junit4", "junit5", "testng", "auto"
        }:
            raise ValueError(
                "launch_type must be 'java', 'junit4', 'junit5', 'testng', or 'auto'"
            )

        args: Dict[str, Any] = {
            "mainClass": main_class,
            "appendSystemEnvironment": append_system_environment,
            "nameConflict": name_conflict,
        }
        optional_args = {
            "configName": config_name,
            "projectName": project_name,
            "programArguments": program_arguments,
            "vmArguments": vm_arguments,
            "workingDirectory": (
                PathConverter.normalize_windows(working_directory)
                if working_directory is not None else None
            ),
            "environmentVariables": environment_variables,
            "type": launch_type,
        }
        args.update({key: value for key, value in optional_args.items() if value is not None})
        return await self._call_mcp_tool("createJavaLaunchConfiguration", args)

    async def get_launch_configuration(
        self,
        config_name: str,
        include_environment_values: bool = False,
    ) -> str:
        """Get launch configuration details, masking environment values by default."""
        return await self._call_mcp_tool("getLaunchConfiguration", {
            "configName": config_name,
            "includeEnvironmentValues": include_environment_values,
        })

    async def update_launch_environment(
        self,
        config_name: str,
        variables: Optional[Dict[str, str]] = None,
        remove_variables: Optional[List[str]] = None,
        update_mode: str = "merge",
        append_system_environment: Optional[bool] = None,
    ) -> str:
        """Update only the environment of an existing launch configuration."""
        if update_mode not in {"merge", "replace"}:
            raise ValueError("update_mode must be 'merge' or 'replace'")
        if update_mode == "replace" and remove_variables is not None:
            raise ValueError("remove_variables cannot be used with update_mode='replace'")

        args: Dict[str, Any] = {
            "configName": config_name,
            "updateMode": update_mode,
        }
        if variables is not None:
            args["variables"] = variables
        if remove_variables is not None:
            args["removeVariables"] = remove_variables
        if append_system_environment is not None:
            args["appendSystemEnvironment"] = append_system_environment
        return await self._call_mcp_tool("updateLaunchEnvironment", args)

    async def step_into(self) -> str:
        """Step into the next method call."""
        return await self._call_mcp_tool("stepInto", {})

    async def step_over(self) -> str:
        """Step over the current line."""
        return await self._call_mcp_tool("stepOver", {})

    async def step_return(self) -> str:
        """Step out of the current method."""
        return await self._call_mcp_tool("stepReturn", {})

    async def debug_resume(self) -> str:
        """Resume execution."""
        return await self._call_mcp_tool("resume", {})

    async def debug_suspend(self) -> str:
        """Suspend execution."""
        return await self._call_mcp_tool("suspend", {})

    async def debug_terminate(self) -> str:
        """Terminate the debug session."""
        return await self._call_mcp_tool("terminate", {})

    async def get_stack_trace(self, thread_id: Optional[str] = None) -> str:
        """Get the current stack trace."""
        args = {}
        if thread_id:
            args["maxDepth"] = 20
        return await self._call_mcp_tool("getStackTrace", args)

    async def get_variables(self, frame_id: Optional[str] = None) -> str:
        """Get variables in the current scope."""
        args = {}
        if frame_id:
            args["stackFrameIndex"] = frame_id
        return await self._call_mcp_tool("getVariables", args)

    async def evaluate_expression(self, expression: str, frame_id: Optional[str] = None) -> str:
        """Evaluate an expression in the debug context."""
        args = {"expression": expression}
        if frame_id:
            args["stackFrameIndex"] = frame_id
        return await self._call_mcp_tool("evaluateExpression", args)

    async def get_debug_status(self) -> str:
        """Get current debug session status."""
        return await self._call_mcp_tool("getDebugStatus", {})

    # ===== Coverage Operations (eclipse-coverage server) =====

    async def run_coverage(self, configuration: str, wait: bool = False) -> str:
        """Launch a configuration with code coverage."""
        args = {"configName": configuration}
        if wait:
            args["waitForCompletion"] = True
        return await self._call_mcp_tool("runCoverage", args)

    async def get_coverage_report(self, project: Optional[str] = None) -> str:
        """Get the code coverage report."""
        args = {}
        if project:
            args["projectName"] = project
        return await self._call_mcp_tool("getCoverageReport", args)

    async def get_class_coverage(self, class_name: str) -> str:
        """Get coverage details for a specific class."""
        return await self._call_mcp_tool("getClassCoverage", {
            "className": class_name
        })

    # ===== Workspace Operations =====

    async def get_workspace_info(self) -> str:
        """Get workspace information (location, project counts, Java/Eclipse versions)."""
        return await self._call_mcp_tool("getWorkspaceInfo", {})

    # ===== Editor Operations (new) =====

    async def open_file(self, file_path: str, line_number: Optional[int] = None, project: Optional[str] = None) -> str:
        """Open a file in the Eclipse editor, optionally at a specific line."""
        win_path = PathConverter.normalize_windows(file_path)
        args = {"filePath": win_path}
        if line_number is not None:
            args["lineNumber"] = line_number
        if project:
            args["projectName"] = project
        return await self._call_mcp_tool("openFile", args)

    async def get_open_editors(self) -> str:
        """List all open editors with active marker and modified status."""
        return await self._call_mcp_tool("getOpenEditors", {})
