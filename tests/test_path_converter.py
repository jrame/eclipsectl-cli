"""Tests for path conversion module."""

import pytest
from eclipsectl_cli.path_converter import PathConverter


class TestWslToWindows:
    """Test WSL to Windows path conversion."""

    def test_simple_c_drive(self):
        """Test conversion of C: drive path."""
        wsl = "/mnt/c/projects/MyApp/src/Main.java"
        expected = "C:\\projects\\MyApp\\src\\Main.java"
        assert PathConverter.wsl_to_windows(wsl) == expected

    def test_simple_d_drive(self):
        """Test conversion of D: drive path."""
        wsl = "/mnt/d/workspace/Test.java"
        expected = "D:\\workspace\\Test.java"
        assert PathConverter.wsl_to_windows(wsl) == expected

    def test_root_mnt(self):
        """Test conversion of /mnt/c (no trailing path)."""
        wsl = "/mnt/c"
        expected = "C:\\"
        assert PathConverter.wsl_to_windows(wsl) == expected

    def test_non_mnt_path_unchanged(self):
        """Test that non-/mnt/ paths are unchanged."""
        wsl = "/home/user/file.txt"
        assert PathConverter.wsl_to_windows(wsl) == wsl

    def test_case_insensitive(self):
        """Test case insensitive drive letter."""
        wsl = "/mnt/C/Projects/Test.java"
        expected = "C:\\Projects\\Test.java"
        assert PathConverter.wsl_to_windows(wsl) == expected


class TestWindowsToWsl:
    """Test Windows to WSL path conversion."""

    def test_simple_c_drive(self):
        """Test conversion of C: drive path."""
        windows = "C:\\projects\\MyApp\\src\\Main.java"
        expected = "/mnt/c/projects/MyApp/src/Main.java"
        assert PathConverter.windows_to_wsl(windows) == expected

    def test_simple_d_drive(self):
        """Test conversion of D: drive path."""
        windows = "D:\\workspace\\Test.java"
        expected = "/mnt/d/workspace/Test.java"
        assert PathConverter.windows_to_wsl(windows) == expected

    def test_root_drive(self):
        """Test conversion of C:\\ (root)."""
        windows = "C:\\"
        expected = "/mnt/c/"
        assert PathConverter.windows_to_wsl(windows) == expected

    def test_relative_path_slashes(self):
        """Test that relative paths just get slash conversion."""
        windows = "relative\\path\\file.txt"
        expected = "relative/path/file.txt"
        assert PathConverter.windows_to_wsl(windows) == expected

    def test_case_insensitive(self):
        """Test case insensitive drive letter."""
        windows = "c:\\Projects\\Test.java"
        expected = "/mnt/c/Projects/Test.java"
        assert PathConverter.windows_to_wsl(windows) == expected


class TestPathDetection:
    """Test path type detection."""

    def test_is_wsl_path(self):
        """Test WSL path detection."""
        assert PathConverter.is_wsl_path("/mnt/c/projects/file.txt") is True
        assert PathConverter.is_wsl_path("/mnt/d/file.txt") is True
        assert PathConverter.is_wsl_path("/home/user/file.txt") is False
        assert PathConverter.is_wsl_path("C:\\projects\\file.txt") is False

    def test_is_windows_path(self):
        """Test Windows path detection."""
        assert PathConverter.is_windows_path("C:\\projects\\file.txt") is True
        assert PathConverter.is_windows_path("D:\\file.txt") is True
        assert PathConverter.is_windows_path("/mnt/c/projects/file.txt") is False
        assert PathConverter.is_windows_path("relative\\path") is False


class TestNormalization:
    """Test path normalization."""

    def test_normalize_wsl_from_windows(self):
        """Test normalizing Windows path to WSL."""
        windows = "C:\\projects\\file.txt"
        expected = "/mnt/c/projects/file.txt"
        assert PathConverter.normalize_wsl(windows) == expected

    def test_normalize_wsl_already_wsl(self):
        """Test normalizing WSL path (should be unchanged)."""
        wsl = "/mnt/c/projects/file.txt"
        assert PathConverter.normalize_wsl(wsl) == wsl

    def test_normalize_windows_from_wsl(self):
        """Test normalizing WSL path to Windows."""
        wsl = "/mnt/c/projects/file.txt"
        expected = "C:\\projects\\file.txt"
        assert PathConverter.normalize_windows(wsl) == expected

    def test_normalize_windows_already_windows(self):
        """Test normalizing Windows path (should be unchanged)."""
        windows = "C:\\projects\\file.txt"
        assert PathConverter.normalize_windows(windows) == windows


class TestRoundTrip:
    """Test round-trip conversions."""

    def test_wsl_to_windows_to_wsl(self):
        """Test WSL → Windows → WSL."""
        original = "/mnt/c/projects/MyApp/src/Main.java"
        windows = PathConverter.wsl_to_windows(original)
        back = PathConverter.windows_to_wsl(windows)
        assert back == original

    def test_windows_to_wsl_to_windows(self):
        """Test Windows → WSL → Windows."""
        original = "C:\\projects\\MyApp\\src\\Main.java"
        wsl = PathConverter.windows_to_wsl(original)
        back = PathConverter.wsl_to_windows(wsl)
        assert back == original
