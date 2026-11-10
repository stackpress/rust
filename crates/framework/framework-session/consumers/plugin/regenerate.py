#!/usr/bin/env python3
"""Regenerate this application's own model family using the public source emitters.

Record this outer entry point once. Pass --check to fail without repairing output.
"""
import os
from pathlib import Path
import subprocess
import sys

base = Path(__file__).resolve().parent
root = base.parents[4]
# The test-only example is a finite build tool; its generated fixtures are never imported.
subprocess.run([
    'cargo', 'run', '--manifest-path', str(root / 'Cargo.toml'),
    '-p', 'stackpress-framework-api', '--example', 'regenerate_api_models',
    '--locked', '--offline', '--', '--input', str(base / 'schema.idea'),
    '--output', str(base / 'src/models'), *sys.argv[1:],
], env={**os.environ, 'CARGO_TARGET_DIR': str(Path(os.environ['CARGO_TARGET_DIR']) / 'plugin-model-generator')}, check=True)
