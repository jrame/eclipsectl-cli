"""Tests for Eclipse launch configuration integration."""

from unittest.mock import AsyncMock

import pytest

from eclipsectl_cli.daemon import EclipseDaemon
from eclipsectl_cli.eclipse_client import EclipseClient, TOOL_TO_SERVER
from eclipsectl_cli.tools import TOOLS


@pytest.fixture
def client():
    eclipse_client = EclipseClient("http://localhost:8124", "token")
    eclipse_client._call_mcp_tool = AsyncMock(return_value="OK")
    return eclipse_client


@pytest.mark.asyncio
async def test_launch_configuration_tools_route_to_debug_server(client):
    assert TOOL_TO_SERVER["listRunConfigurations"] == "eclipse-debug"
    assert TOOL_TO_SERVER["createJavaLaunchConfiguration"] == "eclipse-debug"
    assert TOOL_TO_SERVER["getLaunchConfiguration"] == "eclipse-debug"
    assert TOOL_TO_SERVER["updateLaunchEnvironment"] == "eclipse-debug"

    await client.list_run_configurations()
    client._call_mcp_tool.assert_awaited_once_with("listRunConfigurations", {})


@pytest.mark.asyncio
async def test_create_java_launch_configuration_maps_all_arguments(client):
    await client.create_java_launch_configuration(
        "com.example.Main",
        config_name="Main dev",
        project_name="example",
        program_arguments="--profile dev",
        vm_arguments="-Xmx1g",
        working_directory="/mnt/c/work/example",
        environment_variables={"API_URL": "http://localhost"},
        append_system_environment=False,
        name_conflict="generate",
        launch_type="junit5",
    )

    client._call_mcp_tool.assert_awaited_once_with(
        "createJavaLaunchConfiguration",
        {
            "mainClass": "com.example.Main",
            "configName": "Main dev",
            "projectName": "example",
            "programArguments": "--profile dev",
            "vmArguments": "-Xmx1g",
            "workingDirectory": "C:\\work\\example",
            "environmentVariables": {"API_URL": "http://localhost"},
            "appendSystemEnvironment": False,
            "nameConflict": "generate",
            "type": "junit5",
        },
    )


@pytest.mark.asyncio
async def test_create_java_launch_configuration_omits_type_by_default(client):
    await client.create_java_launch_configuration("com.example.Main")

    client._call_mcp_tool.assert_awaited_once_with(
        "createJavaLaunchConfiguration",
        {
            "mainClass": "com.example.Main",
            "appendSystemEnvironment": True,
            "nameConflict": "error",
        },
    )


@pytest.mark.asyncio
async def test_create_java_launch_configuration_rejects_invalid_type(client):
    with pytest.raises(ValueError, match="launch_type"):
        await client.create_java_launch_configuration(
            "com.example.Main", launch_type="junit3"
        )
    client._call_mcp_tool.assert_not_awaited()


@pytest.mark.asyncio
async def test_get_launch_configuration_masks_environment_by_default(client):
    await client.get_launch_configuration("Main dev")

    client._call_mcp_tool.assert_awaited_once_with(
        "getLaunchConfiguration",
        {"configName": "Main dev", "includeEnvironmentValues": False},
    )


@pytest.mark.asyncio
async def test_replace_with_empty_map_removes_all_configured_variables(client):
    await client.update_launch_environment(
        "Main dev", variables={}, update_mode="replace", append_system_environment=False
    )

    client._call_mcp_tool.assert_awaited_once_with(
        "updateLaunchEnvironment",
        {
            "configName": "Main dev",
            "variables": {},
            "updateMode": "replace",
            "appendSystemEnvironment": False,
        },
    )


@pytest.mark.asyncio
async def test_update_omits_system_environment_when_not_requested(client):
    await client.update_launch_environment("Main dev", variables={"PROFILE": "dev"})

    client._call_mcp_tool.assert_awaited_once_with(
        "updateLaunchEnvironment",
        {
            "configName": "Main dev",
            "variables": {"PROFILE": "dev"},
            "updateMode": "merge",
        },
    )


@pytest.mark.asyncio
async def test_replace_rejects_explicit_removals(client):
    with pytest.raises(ValueError, match="remove_variables"):
        await client.update_launch_environment(
            "Main dev", remove_variables=["OLD_TOKEN"], update_mode="replace"
        )
    client._call_mcp_tool.assert_not_awaited()


def test_wrapper_mcp_exposes_launch_configuration_schemas():
    tools = {tool["name"]: tool for tool in TOOLS}

    create_schema = tools["eclipse_create_java_launch_configuration"]["inputSchema"]
    assert create_schema["required"] == ["mainClass"]
    assert create_schema["properties"]["nameConflict"]["enum"] == ["error", "generate"]
    assert create_schema["properties"]["type"]["enum"] == [
        "java", "junit4", "junit5", "testng", "auto"
    ]

    get_schema = tools["eclipse_get_launch_configuration"]["inputSchema"]
    assert get_schema["properties"]["includeEnvironmentValues"]["default"] is False

    update_schema = tools["eclipse_update_launch_environment"]["inputSchema"]
    assert update_schema["properties"]["updateMode"]["enum"] == ["merge", "replace"]
    assert update_schema["properties"]["variables"]["additionalProperties"] == {
        "type": "string"
    }


@pytest.mark.asyncio
async def test_daemon_routes_launch_environment_with_tri_state_inheritance():
    daemon = EclipseDaemon()
    daemon.client = AsyncMock()
    daemon.client.update_launch_environment.return_value = "OK"

    result = await daemon._execute_command("update_launch_environment", {
        "configName": "Main dev",
        "variables": {"PROFILE": "dev"},
        "removeVariables": ["OLD_PROFILE"],
        "updateMode": "merge",
    })

    assert result == "OK"
    daemon.client.update_launch_environment.assert_awaited_once_with(
        "Main dev", {"PROFILE": "dev"}, ["OLD_PROFILE"], "merge", None
    )


@pytest.mark.asyncio
async def test_daemon_routes_launch_configuration_type():
    daemon = EclipseDaemon()
    daemon.client = AsyncMock()
    daemon.client.create_java_launch_configuration.return_value = "OK"

    result = await daemon._execute_command("create_java_launch_configuration", {
        "mainClass": "com.example.MainTest",
        "type": "testng",
    })

    assert result == "OK"
    daemon.client.create_java_launch_configuration.assert_awaited_once_with(
        "com.example.MainTest", None, None, None, None, None, None, True, "error", "testng"
    )
