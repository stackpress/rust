#!/usr/bin/env python3
"""Compile and run the twelve newly supplied crate README examples as doctests."""
import os
from pathlib import Path

from common import ROOT, manifest, run, seed_lock

NEW_READMES = ['crates/lib', 'crates/server/server', 'crates/idea/idea-parser',
               'crates/idea/idea-transformer', 'crates/idea/idea', 'crates/sql/sql',
               'crates/sql/sql-sqlite', 'crates/framework/framework-language',
               'crates/framework/framework-schema', 'crates/framework/framework-sql',
               'crates/framework/framework-view', 'tools/cli']


def main():
    """Attach the real README text to scratch modules so Rustdoc tests the shipped bytes."""
    target = Path(os.environ['CARGO_TARGET_DIR']).resolve()
    consumer = target / 'readme-consumer'
    (consumer / 'src').mkdir(parents=True, exist_ok=True)
    deps = []
    modules = []
    for i, folder in enumerate(NEW_READMES):
        directory = ROOT / folder
        name = manifest(directory / 'Cargo.toml')['package']['name']
        deps.append(f'{name} = {{ path = "{directory}" }}')
        modules.append(f'#[doc = include_str!("{directory}/README.md")]\npub mod readme_{i} {{}}')
    (consumer / 'Cargo.toml').write_text(
        '[workspace]\n[package]\nname = "stackpress-release-readme-consumer"\nversion = "0.0.0"\n'
        'edition = "2024"\npublish = false\n[dependencies]\n' + '\n'.join(deps) + '\n')
    (consumer / 'src/lib.rs').write_text('\n'.join(modules) + '\n')
    env = dict(os.environ, CARGO_TARGET_DIR=str(target / 'readme-build'))
    seed_lock(consumer, env)
    run(['cargo', 'test', '--manifest-path', str(consumer / 'Cargo.toml'), '--doc', '--locked', '--offline'], env=env)


if __name__ == '__main__':
    main()
