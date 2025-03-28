"""Tests for tool definitions."""

import pytest
from eclipsectl_cli.tools import TOOLS


def test_tools_defined():
    """Test that the wrapper exposes its expected tool families without duplicates."""
    assert TOOLS

    tool_names = [tool["name"] for tool in TOOLS]
    assert len(tool_names) == len(set(tool_names))

    # Workspace
    assert "eclipse_refresh_file" in tool_names
    assert "eclipse_refresh_workspace" in tool_names

    # Project Management
    assert "eclipse_list_projects" in tool_names
    assert "eclipse_get_project_layout" in tool_names
    assert "eclipse_get_project_properties" in tool_names

    # Code Analysis
    assert "eclipse_get_javadoc" in tool_names
    assert "eclipse_get_source" in tool_names
    assert "eclipse_get_call_hierarchy" in tool_names

    # Build & Errors
    assert "eclipse_maven_build" in tool_names
    assert "eclipse_get_compilation_errors" in tool_names

    # Tests
    assert "eclipse_run_all_tests" in tool_names
    assert "eclipse_run_package_tests" in tool_names
    assert "eclipse_run_class_tests" in tool_names
    assert "eclipse_run_test_method" in tool_names

    # Editor
    assert "eclipse_get_current_file" in tool_names
    assert "eclipse_get_editor_selection" in tool_names
    assert "eclipse_format_code" in tool_names

    # Console
    assert "eclipse_get_console_output" in tool_names

    # Search
    assert "eclipse_file_search" in tool_names
    assert "eclipse_file_search_regexp" in tool_names
    assert "eclipse_find_files" in tool_names


def test_removed_legacy_editing_tools_stay_unexposed():
    """Legacy editing tools superseded by the current APIs must stay unexposed."""
    tool_names = [tool["name"] for tool in TOOLS]

    # These old tool names are not part of the current wrapper API.
    assert "eclipse_create_file" not in tool_names
    assert "eclipse_insert_into_file" not in tool_names
    assert "eclipse_replace_string" not in tool_names
    assert "eclipse_undo_edit" not in tool_names


def test_tool_structure():
    """Test that each tool has required fields."""
    for tool in TOOLS:
        assert "name" in tool
        assert "description" in tool
        assert "inputSchema" in tool

        # Check input schema structure
        schema = tool["inputSchema"]
        assert schema["type"] == "object"
        assert "properties" in schema
        assert "required" in schema


def test_maven_build_tool():
    """Test Maven build tool definition."""
    maven_tool = next(t for t in TOOLS if t["name"] == "eclipse_maven_build")

    props = maven_tool["inputSchema"]["properties"]
    assert "project" in props
    assert "goals" in props

    # Goals should be an array
    assert props["goals"]["type"] == "array"


def test_javadoc_tool():
    """Test JavaDoc tool definition."""
    javadoc_tool = next(t for t in TOOLS if t["name"] == "eclipse_get_javadoc")

    props = javadoc_tool["inputSchema"]["properties"]
    assert "fqn" in props

    required = javadoc_tool["inputSchema"]["required"]
    assert required == ["fqn"]
