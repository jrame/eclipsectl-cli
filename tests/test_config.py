"""Tests for configuration module."""

import pytest
from eclipsectl_cli.config import Config, DEFAULT_CONFIG


def test_default_config():
    """Test default configuration values."""
    config = DEFAULT_CONFIG
    assert config.eclipse_url == "http://localhost:8124"
    assert config.timeout == 30
    assert config.verbose is False
    assert config.bearer_token is None


def test_custom_config():
    """Test custom configuration."""
    config = Config(
        eclipse_url="http://localhost:9090",
        bearer_token="test-token-123",
        timeout=60,
        verbose=True
    )
    assert config.eclipse_url == "http://localhost:9090"
    assert config.bearer_token == "test-token-123"
    assert config.timeout == 60
    assert config.verbose is True
