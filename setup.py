#!/usr/bin/env python3
"""Setup script to install bash CLI wrapper.

NOTE: cmdclass hooks below only work for regular installs (pip install .)
For editable installs (pip install -e .), use install.sh instead.
This is a PEP 660 limitation - editable installs don't trigger cmdclass.
"""

from setuptools import setup
from setuptools.command.install import install
from setuptools.command.develop import develop
import os
import shutil
import sys


def install_bash_cli():
    """Copy bash CLI to bin directory.

    This function is called by PostInstallCommand for normal installs.
    For editable installs, this is NOT called - use install.sh instead.
    """
    source = os.path.join(os.path.dirname(__file__), 'bin', 'eclipse')

    # Get the bin directory where scripts are installed
    bin_dir = os.path.join(sys.prefix, 'bin')

    dest = os.path.join(bin_dir, 'eclipse')

    print(f"Installing bash CLI: {source} -> {dest}")

    if os.path.exists(source):
        shutil.copy2(source, dest)
        os.chmod(dest, 0o755)
        print(f"✅ Bash CLI installed to {dest}")
    else:
        print(f"⚠️  Warning: bash CLI not found at {source}")


class PostInstallCommand(install):
    """Post-installation for installation mode."""
    def run(self):
        install.run(self)
        install_bash_cli()


class PostDevelopCommand(develop):
    """Post-installation for development mode."""
    def run(self):
        develop.run(self)
        install_bash_cli()


setup(
    cmdclass={
        'install': PostInstallCommand,
        'develop': PostDevelopCommand,
    },
)
