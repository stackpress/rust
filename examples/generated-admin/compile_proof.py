#!/usr/bin/env python3
"""Actual bootstrap/remove/rename compilation with parent-controlled subprocesses."""
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile

from verify import ROOT, LOGS, TARGET, run


def hashes(output):
    """Compare the complete writer manifest and source set, not just expected leaf files."""
    return {str(path.relative_to(output)): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in output.rglob('*') if path.is_file()}


def main():
    """Prove the obsolete consumer actually fails after real marked-root replacement."""
    with tempfile.TemporaryDirectory(dir=ROOT / 'examples/generated-admin/tool') as inputs, tempfile.TemporaryDirectory(dir=LOGS) as temp:
        source = Path(inputs) / 'schema.json'
        consumer = Path(temp)
        output = consumer / 'src/generated'
        (consumer / 'src').mkdir()
        base = '''plugin "framework-schema" {} plugin "framework-admin" {} model Profile { id String @id name String @field.string @filter.string @span.string @list.string @view.string }'''
        idea = Path(inputs) / 'schema.idea'
        idea.write_text(base)
        manifest = f'''[package]
name = "admin-compile-proof"
version = "0.0.0"
edition = "2024"
[workspace]
[dependencies]
stackpress-framework-admin = {{ path = "{ROOT}/crates/framework/framework-admin" }}
stackpress-framework-schema = {{ path = "{ROOT}/crates/framework/framework-schema" }}
stackpress-framework-ui = {{ path = "{ROOT}/crates/framework/framework-ui" }}
leptos = {{ version = "=0.8.22", default-features = false }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
'''
        (consumer / 'Cargo.toml').write_text(manifest)
        (consumer / 'src/lib.rs').write_text('''#[path="generated/contracts/mod.rs"] pub mod contracts;
#[path="generated/ui/mod.rs"] pub mod ui;
pub fn consumer() { let _ = ui::profile::descriptors(); }
''')
        tool = str(TARGET / 'debug/generated-admin-tool')
        generate = [tool, '--input', str(idea), '--output', str(output)]
        run('compile-bootstrap-generation', generate)
        initial = hashes(output)
        run('compile-stable-check', generate + ['--check'])
        run('compile-stable-regeneration', generate)
        assert initial == hashes(output)
        check = ['cargo', 'check', '--offline', '--manifest-path', str(consumer / 'Cargo.toml')]
        run('compile-initial-consumer', check)
        # Change both source model identity and its selected field, using an existing app policy.
        idea.write_text(base.replace('Profile', 'Project').replace('name String', 'title String'))
        run('compile-renamed-generation', generate)
        assert not (output / 'ui/profile.rs').exists()
        assert (output / 'ui/project.rs').exists()
        try:
            run('compile-stale-consumer', check)
        except SystemExit as error:
            assert error.code == 101
            assert 'could not find `profile` in `ui`' in (LOGS / 'compile-stale-consumer.log').read_text()
        else:
            raise AssertionError('stale consumer compiled after model removal')
        lib = consumer / 'src/lib.rs'
        lib.write_text(lib.read_text().replace('ui::profile', 'ui::project'))
        run('compile-updated-consumer', check)
        # JSON quote/backslash/newline/closing-tag labels remain data in actual Rust compilation.
        config = {
            'plugin': {'framework-schema': {}, 'framework-admin': {}},
            'model': {'Profile': {'name': 'Profile', 'mutable': True, 'attributes': {}, 'columns': [
                {'name': 'id', 'type': 'String', 'required': True, 'multiple': False, 'attributes': {'id': True}},
                {'name': 'name', 'type': 'String', 'required': True, 'multiple': False,
                 'attributes': {'label': ['</script> "quote" \\ \n 🦀'], 'field.string': [{'placeholder': '"; panic!("bad"); //'}], 'list.string': True}},
            ]}},
        }
        # Use the parser's documented normalized JSON keys (including type and mutable).
        source.write_text(json.dumps(config))
        run('compile-hostile-json-generation', [tool, '--input', str(source), '--output', str(output)])
        lib.write_text(lib.read_text().replace('ui::project', 'ui::profile'))
        run('compile-hostile-json-consumer', check)
        (LOGS / 'compile-proof.json').write_text(json.dumps({'initialHashes': initial, 'staleExit': 101, 'renamedConsumer': True, 'hostileJsonConsumer': True}, indent=2))


if __name__ == '__main__':
    main()
