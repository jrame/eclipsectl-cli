"""Path conversion utilities for WSL ↔ Windows."""

import re
from pathlib import Path, PureWindowsPath, PurePosixPath


class PathConverter:
    """Convert paths between WSL and Windows formats."""

    @staticmethod
    def wsl_to_windows(wsl_path: str) -> str:
        r"""
        Convert WSL path to Windows path.

        Examples:
            /mnt/c/projects/MyApp/src/Main.java → C:\projects\MyApp\src\Main.java
            /mnt/d/workspace/Test.java → D:\workspace\Test.java
            /home/user/local.txt → /home/user/local.txt (unchanged)

        Args:
            wsl_path: Path in WSL format

        Returns:
            Path in Windows format (or unchanged if not a /mnt/ path)
        """
        # Match /mnt/X/... pattern
        match = re.match(r'^/mnt/([a-z])(/.*)?$', wsl_path, re.IGNORECASE)

        if match:
            drive_letter = match.group(1).upper()
            rest_path = match.group(2) or ''

            # Remove leading slash and convert to Windows path
            rest_path = rest_path.lstrip('/')
            windows_path = f"{drive_letter}:\\" + rest_path.replace('/', '\\')

            return windows_path

        # Not a /mnt/ path, return as-is
        return wsl_path

    @staticmethod
    def windows_to_wsl(windows_path: str) -> str:
        r"""
        Convert Windows path to WSL path.

        Examples:
            C:\projects\MyApp\src\Main.java → /mnt/c/projects/MyApp/src/Main.java
            D:\workspace\Test.java → /mnt/d/workspace/Test.java
            relative\path → relative/path (just convert slashes)

        Args:
            windows_path: Path in Windows format

        Returns:
            Path in WSL format
        """
        # Match C:\... or D:\... pattern
        match = re.match(r'^([A-Z]):\\(.*)$', windows_path, re.IGNORECASE)

        if match:
            drive_letter = match.group(1).lower()
            rest_path = match.group(2)

            # Convert to WSL format
            wsl_path = f"/mnt/{drive_letter}/" + rest_path.replace('\\', '/')

            return wsl_path

        # Not an absolute Windows path, just convert slashes
        return windows_path.replace('\\', '/')

    @staticmethod
    def is_wsl_path(path: str) -> bool:
        """Check if path is a WSL path (/mnt/...)."""
        return path.startswith('/mnt/')

    @staticmethod
    def is_windows_path(path: str) -> bool:
        r"""Check if path is an absolute Windows path (C:\...)."""
        return bool(re.match(r'^[A-Z]:\\', path, re.IGNORECASE))

    @staticmethod
    def normalize_wsl(path: str) -> str:
        """
        Normalize a path to WSL format.

        If already WSL format, return as-is.
        If Windows format, convert to WSL.
        """
        if PathConverter.is_windows_path(path):
            return PathConverter.windows_to_wsl(path)
        return path

    @staticmethod
    def normalize_windows(path: str) -> str:
        """
        Normalize a path to Windows format.

        If already Windows format, return as-is.
        If WSL format, convert to Windows.
        """
        if PathConverter.is_wsl_path(path):
            return PathConverter.wsl_to_windows(path)
        return path
