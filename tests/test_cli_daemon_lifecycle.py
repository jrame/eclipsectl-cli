"""Regression tests for the Bash CLI daemon lifecycle helpers."""

import os
import shutil
import socket
import subprocess
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
CLI_SCRIPT = REPOSITORY_ROOT / "bin" / "eclipse"


def _run_sourced_cli(
    command: str, home: Path, cli_script: Path = CLI_SCRIPT
) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env["HOME"] = str(home)
    env.pop("ECLIPSECTL_PYTHON", None)
    return subprocess.run(
        ["bash", "-c", f'source "$1"; {command}', "bash", str(cli_script)],
        cwd=REPOSITORY_ROOT,
        env=env,
        check=False,
        capture_output=True,
        text=True,
    )


def _create_socket_file(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    unix_socket = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    try:
        unix_socket.bind(str(path))
    finally:
        unix_socket.close()


def test_remove_stale_daemon_files_removes_socket_and_pid(tmp_path):
    config_dir = tmp_path / ".config" / "eclipsectl-cli"
    socket_path = config_dir / "daemon.sock"
    pid_path = config_dir / "daemon.pid"
    _create_socket_file(socket_path)
    pid_path.write_text("999999999")

    result = _run_sourced_cli("remove_stale_daemon_files", tmp_path)

    assert result.returncode == 0, result.stderr
    assert not socket_path.exists()
    assert not pid_path.exists()


def test_started_daemon_readiness_requires_matching_live_pid(tmp_path):
    config_dir = tmp_path / ".config" / "eclipsectl-cli"
    socket_path = config_dir / "daemon.sock"
    _create_socket_file(socket_path)

    result = _run_sourced_cli(
        'printf "%s" "$$" > "$PID_FILE"; '
        'is_started_daemon_ready "$$"; '
        '! is_started_daemon_ready "$(( $$ + 1 ))"',
        tmp_path,
    )

    assert result.returncode == 0, result.stderr


def test_sourcing_cli_does_not_execute_main(tmp_path):
    result = _run_sourced_cli(":", tmp_path)

    assert result.returncode == 0, result.stderr
    assert result.stdout == ""
    assert result.stderr == ""


def test_packaged_cli_uses_python_from_its_installation_environment(tmp_path):
    environment_bin = tmp_path / "venv" / "bin"
    environment_bin.mkdir(parents=True)
    installed_cli = environment_bin / "eclipse"
    installed_python = environment_bin / "python3"
    shutil.copy2(CLI_SCRIPT, installed_cli)
    installed_python.write_text("#!/bin/sh\nexit 0\n")
    installed_python.chmod(0o755)

    exposed_bin = tmp_path / "exposed-bin"
    exposed_bin.mkdir()
    exposed_cli = exposed_bin / "eclipse"
    exposed_cli.symlink_to(installed_cli)

    result = _run_sourced_cli(
        'printf "%s" "$ECLIPSECTL_PYTHON"', tmp_path / "home", exposed_cli
    )

    assert result.returncode == 0, result.stderr
    assert Path(result.stdout) == installed_python
