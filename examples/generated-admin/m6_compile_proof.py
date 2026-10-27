#!/usr/bin/env python3
"""Actual bootstrap/remove/rename compilation with parent-controlled subprocesses."""
import hashlib
import json
from pathlib import Path
import subprocess
import shutil
import tempfile

from m6_verify import ROOT, LOGS, TARGET, run


def hashes(output):
    """Compare the complete writer manifest and source set, not just expected leaf files."""
    return {str(path.relative_to(output)): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in output.rglob('*') if path.is_file()}


def main():
    """Prove the obsolete consumer actually fails after real marked-root replacement."""
    run('compile-tool-build', ['cargo','build','-p','generated-admin-tool','--features','format','--locked'])
    # Workspace tests may rebuild their own default tool. Freeze this proof's feature selection.
    immutable_tool = LOGS/'admin-tool-proof'
    shutil.copy2(TARGET/'debug/generated-admin-tool', immutable_tool)
    with tempfile.TemporaryDirectory(dir=ROOT / 'examples/generated-admin/tool') as inputs, tempfile.TemporaryDirectory(dir=LOGS) as temp:
        source = Path(inputs) / 'schema.json'
        consumer = Path(temp)
        output = consumer / 'src/generated'
        (consumer / 'src').mkdir()
        base = '''plugin "framework-schema" {} plugin "framework-sql" {} plugin "framework-admin" {} model Profile { id String @id name String @field.string @filter.string @span.string @list.string @view.string }'''
        idea = Path(inputs) / 'schema.idea'
        idea.write_text(base)
        manifest = f'''[package]
name = "admin-m6-compile-proof"
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
stackpress-framework-sql = {{ path = "{ROOT}/crates/framework/framework-sql", features = ["server"] }}
stackpress-lib = {{ path = "{ROOT}/crates/lib" }}
stackpress-server = {{ path = "{ROOT}/crates/server" }}
stackpress-sql = {{ path = "{ROOT}/crates/sql/sql" }}
'''
        (consumer / 'Cargo.toml').write_text(manifest)
        (consumer / 'src/lib.rs').write_text('''#[path="generated/contracts/mod.rs"] pub mod contracts;
#[path="generated/ui/mod.rs"] pub mod ui;
pub fn consumer() { let _ = ui::profile::descriptors(); let _ = ui::pages::profile::spec(); }
''')
        native = """
pub mod generated { pub use crate::contracts; #[path="stores/mod.rs"] pub mod stores; }
#[path="generated/ui/pages_host.rs"] pub mod pages_host;
pub mod host {
    use stackpress_lib::{Request,Response,Invocation,Result,Flow};
    pub async fn page(_:&mut Request,_:&mut Response,_:Invocation<'_>)->Result<Flow>{Ok(Flow::Continue)}
    pub async fn api(_:&mut Request,_:&mut Response,_:Invocation<'_>)->Result<Flow>{Ok(Flow::Continue)}
    pub mod transfer {
        use super::*;
        pub async fn export(_:&mut Request,_:&mut Response,_:Invocation<'_>)->Result<Flow>{Ok(Flow::Continue)}
        pub async fn import(_:&mut Request,_:&mut Response,_:Invocation<'_>)->Result<Flow>{Ok(Flow::Continue)}
    }
}
"""
        # Route stubs only establish the generated signature boundary. Separate native tests
        # execute the actual host and generated operations against SQLite.
        def library(source):
            (consumer/'src/lib.rs').write_text(source if 'pub mod generated {' in source else source + native)
        library((consumer/'src/lib.rs').read_text())
        tool = str(immutable_tool)
        generate = [tool, '--input', str(idea), '--output', str(output)]
        run('m6-compile-bootstrap-generation', generate)
        initial = hashes(output)
        run('m6-compile-stable-check', generate + ['--check'])
        run('m6-compile-stable-regeneration', generate)
        assert initial == hashes(output)
        check = ['cargo', 'check', '--offline', '--manifest-path', str(consumer / 'Cargo.toml')]
        run('m6-compile-initial-consumer', check)
        # Change both source model identity and its selected field, using an existing app policy.
        idea.write_text(base.replace('Profile', 'Project').replace('name String', 'title String'))
        run('m6-compile-renamed-generation', generate)
        assert not (output / 'ui/profile.rs').exists()
        assert (output / 'ui/project.rs').exists()
        try:
            run('m6-compile-stale-consumer', check)
        except SystemExit as error:
            assert error.code == 101
            assert 'could not find `profile` in `ui`' in (LOGS / 'm6-compile-stale-consumer.log').read_text()
        else:
            raise AssertionError('stale consumer compiled after model removal')
        lib = consumer / 'src/lib.rs'
        library(lib.read_text().replace('ui::profile', 'ui::project').replace('pages::profile', 'pages::project'))
        run('m6-compile-updated-consumer', check)
        # JSON quote/backslash/newline/closing-tag labels remain data in actual Rust compilation.
        config = {
            'plugin': {'framework-schema': {}, 'framework-sql': {}, 'framework-admin': {}},
            'model': {'Profile': {'name': 'Profile', 'mutable': True, 'attributes': {}, 'columns': [
                {'name': 'id', 'type': 'String', 'required': True, 'multiple': False, 'attributes': {'id': True}},
                {'name': 'name', 'type': 'String', 'required': True, 'multiple': False,
                 'attributes': {'label': ['</script> "quote" \\ \n 🦀'], 'field.string': [{'placeholder': '"; panic!("bad"); //'}], 'list.string': True}},
            ]}},
        }
        # Use the parser's documented normalized JSON keys (including type and mutable).
        source.write_text(json.dumps(config))
        run('m6-compile-hostile-json-generation', [tool, '--input', str(source), '--output', str(output)])
        library(lib.read_text().replace('ui::project', 'ui::profile').replace('pages::project', 'pages::profile'))
        run('m6-compile-hostile-json-consumer', check)
        # The new formatted import receives the same hostile literal through typed props.
        config['model']['Profile']['columns'][1]['attributes'].pop('field.string')
        config['model']['Profile']['columns'][1]['attributes']['field.number'] = [{'label': '</script> "quote" \\ \n 🦀 {Role, SchemaError'}]
        source.write_text(json.dumps(config))
        (consumer/'Cargo.toml').write_text(manifest.replace('framework-admin" }','framework-admin", features = ["format"] }'))
        run('m6-compile-hostile-format-generation', [tool, '--input', str(source), '--output', str(output)])
        run('m6-compile-hostile-format-consumer', check)
        # Omitted field roles and complete removal remain real, warning-clean consumers.
        idea.write_text('plugin "framework-schema" {} plugin "framework-sql" {} plugin "framework-admin" {} model Profile { id String @id name String @label("RoleContext,Role,MODEL {Role, SchemaError") }')
        run('m6-compile-omitted-generation',generate)
        run('m6-compile-omitted-consumer',check)
        assert 'RoleContext,Role,MODEL {Role, SchemaError' in (output/'ui/pages.rs').read_text()
        assert 'RoleContext,Role,MODEL {Role, SchemaError' in (output/'ui/profile.rs').read_text()
        idea.write_text('plugin "framework-schema" {} plugin "framework-sql" {} plugin "framework-admin" {} model Other { id String @id }')
        run('m6-compile-empty-generation',generate)
        library('\n'.join(['#[path="generated/contracts/mod.rs"] pub mod contracts;', '#[path="generated/ui/mod.rs"] pub mod ui;', 'pub fn consumer(){let _=ui::pages::MODELS;let _=ui::pages::spec("profile");}']))
        run('m6-compile-empty-consumer',check)
        run('m6-compile-empty-clippy',['cargo','clippy','--offline','--manifest-path',str(consumer/'Cargo.toml'),'--','-D','warnings'])
        # Compile the full opt-in revision-2 fixture, including every new formatted family.
        formatted = [tool, '--input', str(ROOT/'examples/generated-admin/schema-format.idea'), '--output', str(output)]
        run('m6-compile-format-generation', formatted)
        format_hashes = hashes(output)
        run('m6-compile-format-stable-check', formatted+['--check'])
        run('m6-compile-format-regeneration', formatted)
        assert format_hashes == hashes(output)
        (consumer/'Cargo.toml').write_text(manifest.replace('framework-admin" }','framework-admin", features = ["format"] }'))
        library('\n'.join(['#[path="generated/contracts/mod.rs"] pub mod contracts;', '#[path="generated/ui/mod.rs"] pub mod ui;', 'pub fn consumer(){let _=ui::task::descriptors();let _=ui::pages::task::spec();}']))
        run('m6-compile-format-consumer', check)
        (LOGS / 'm6-compile-proof.json').write_text(json.dumps({'initialHashes': initial, 'staleExit': 101, 'renamedConsumer': True, 'hostileJsonConsumer': True, 'hostileFormatConsumer': True, 'omittedRolesConsumer': True, 'removedAllConsumer': True, 'formattedConsumer': True, 'formatHashes': format_hashes}, indent=2))


if __name__ == '__main__':
    main()
