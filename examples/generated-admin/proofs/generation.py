"""Actual bootstrap/remove/rename compilation with parent-controlled subprocesses."""
import hashlib
import json
from pathlib import Path
import subprocess
import shutil
import tempfile
from .runtime import ROOT, TARGET


def hashes(output):
    """Compare the complete writer manifest and source set, not just expected leaf files."""
    return {str(path.relative_to(output)): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in output.rglob('*') if path.is_file()}


def compile_proof(ctx):
    """Prove the obsolete consumer actually fails after real marked-root replacement."""
    LOGS = ctx.logs
    run = ctx.run
    run('compile-tool-clean', ['cargo','clean','-p','generated-admin-tool'])
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
stackpress-framework-sql = {{ path = "{ROOT}/crates/framework/framework-sql", features = ["server"] }}
stackpress-lib = {{ path = "{ROOT}/crates/lib" }}
stackpress-server = {{ path = "{ROOT}/crates/server" }}
stackpress-sql = {{ path = "{ROOT}/crates/sql/sql" }}
'''
        (consumer / 'Cargo.toml').write_text(manifest)
        (consumer / 'src/lib.rs').write_text('''#[path="generated/contracts/mod.rs"] pub mod contracts;
#[path="generated/ui/mod.rs"] pub mod ui;
#[path="generated/metadata/mod.rs"] pub mod metadata;
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
        run('compile-bootstrap-generation', generate)
        initial = hashes(output)
        run('compile-stable-check', generate + ['--check'])
        run('compile-stable-regeneration', generate)
        assert initial == hashes(output)
        check = ['cargo', 'check', '--offline', '--manifest-path', str(consumer / 'Cargo.toml')]
        run('compile-initial-consumer', check)
        # A stale metadata partition must fail against the selected revision-3 library.
        metadata = output/'metadata/schema.rs'
        current = metadata.read_text()
        assert 'CONTRACT_REVISION: u32 = 3;' in current
        metadata.write_text(current.replace('CONTRACT_REVISION: u32 = 3;', 'CONTRACT_REVISION: u32 = 2;'))
        try:
            run('compile-mixed-revision', check)
        except SystemExit as error:
            assert error.code == 101
            assert 'E0080' in (LOGS/'compile-mixed-revision.log').read_text()
        else:
            raise AssertionError('mixed generated revisions compiled')
        finally:
            metadata.write_text(current)
        run('compile-restored-revision', check)
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
        library(lib.read_text().replace('ui::profile', 'ui::project').replace('pages::profile', 'pages::project'))
        run('compile-updated-consumer', check)
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
        run('compile-hostile-json-generation', [tool, '--input', str(source), '--output', str(output)])
        library(lib.read_text().replace('ui::project', 'ui::profile').replace('pages::project', 'pages::profile'))
        run('compile-hostile-json-consumer', check)
        # The new formatted import receives the same hostile literal through typed props.
        config['model']['Profile']['columns'][1]['attributes'].pop('field.string')
        config['model']['Profile']['columns'][1]['attributes']['field.number'] = [{'label': '</script> "quote" \\ \n 🦀 {Role, SchemaError'}]
        source.write_text(json.dumps(config))
        (consumer/'Cargo.toml').write_text(manifest.replace('framework-admin" }','framework-admin", features = ["format"] }'))
        run('compile-hostile-format-generation', [tool, '--input', str(source), '--output', str(output)])
        run('compile-hostile-format-consumer', check)
        # Omitted field roles and complete removal remain real, warning-clean consumers.
        idea.write_text('plugin "framework-schema" {} plugin "framework-sql" {} plugin "framework-admin" {} model Profile { id String @id name String @label("RoleContext,Role,MODEL {Role, SchemaError") }')
        run('compile-omitted-generation',generate)
        run('compile-omitted-consumer',check)
        assert 'RoleContext,Role,MODEL {Role, SchemaError' in (output/'ui/pages.rs').read_text()
        assert 'RoleContext,Role,MODEL {Role, SchemaError' in (output/'ui/profile.rs').read_text()
        idea.write_text('plugin "framework-schema" {} plugin "framework-sql" {} plugin "framework-admin" {} model Other { id String @id }')
        run('compile-empty-generation',generate)
        library('\n'.join(['#[path="generated/contracts/mod.rs"] pub mod contracts;', '#[path="generated/ui/mod.rs"] pub mod ui;', 'pub fn consumer(){let _=ui::pages::MODELS;let _=ui::pages::spec("profile");}']))
        run('compile-empty-consumer',check)
        run('compile-empty-clippy',['cargo','clippy','--offline','--manifest-path',str(consumer/'Cargo.toml'),'--','-D','warnings'])
        # Compile the full opt-in revision-3 fixture, including every new formatted family.
        formatted = [tool, '--input', str(ROOT/'examples/generated-admin/schema-format.idea'), '--output', str(output)]
        run('compile-format-generation', formatted)
        format_hashes = hashes(output)
        run('compile-format-stable-check', formatted+['--check'])
        run('compile-format-regeneration', formatted)
        assert format_hashes == hashes(output)
        (consumer/'Cargo.toml').write_text(manifest.replace('framework-admin" }','framework-admin", features = ["format"] }'))
        library('\n'.join(['#[path="generated/contracts/mod.rs"] pub mod contracts;', '#[path="generated/ui/mod.rs"] pub mod ui;', 'pub fn consumer(){let _=ui::task::descriptors();let _=ui::pages::task::spec();}']))
        run('compile-format-consumer', check)
        (LOGS / 'compile-proof.json').write_text(json.dumps({'initialHashes': initial, 'staleExit': 101, 'mixedRevisionExit': 101, 'renamedConsumer': True, 'hostileJsonConsumer': True, 'hostileFormatConsumer': True, 'omittedRolesConsumer': True, 'removedAllConsumer': True, 'formattedConsumer': True, 'formatHashes': format_hashes}, indent=2))
