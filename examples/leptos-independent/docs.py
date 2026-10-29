#!/usr/bin/env python3
"""Compile every library Markdown Rust fence in an isolated copy of the consumer.

View-specific fences use a separate integration workspace; view is never added to
independent dependencies. These checks run source examples, not a Markdown renderer.
"""
import json
import re
import shutil
import subprocess
import tempfile
import tomllib
from pathlib import Path
from proof import ROOT, HERE, OUT, run

REPORT = OUT / 'docs'
REPORT.mkdir(parents=True, exist_ok=True)
APPROVED = {(p['name'], p['version'], p.get('source'), p.get('checksum'))
            for p in tomllib.loads((ROOT/'Cargo.lock').read_text())['package'] if p.get('source')}
records = []


def fences(path):
    """Retain each exact Rust fence and its source line; no hand-rewritten test surrogate."""
    text = path.read_text()
    for match in re.finditer(r'^```rust[^\n]*\n(.*?)^```\s*$', text, re.M | re.S):
        code = '\n'.join(line[2:] if line.startswith('# ') else line
                         for line in match[1].splitlines())
        yield text[:match.start()].count('\n')+1, code


def scope(path, code):
    """Render-specific APIs compile only in their actual selected target/mode."""
    if any(word in code for word in ['web_sys::', 'wasm_bindgen::', 'hydrate_from', 'mount_to']):
        return 'hydrate'
    if any(word in code for word in ['initialize_ssr', '::render::', '::async_render::', 'stream::{', '::render::{']):
        return 'ssr'
    if 'PreparedPage' in code or 'LeptosRenderer' in code:
        return 'ssr'
    return 'shared'


def audit_lock(directory):
    """Only local consumer entries may differ from approved external dependency tuples."""
    data = tomllib.loads((directory/'Cargo.lock').read_text())
    external = {(p['name'], p['version'], p.get('source'), p.get('checksum'))
                for p in data['package'] if p.get('source')}
    assert external <= APPROVED, external - APPROVED


def check_group(paths, integration=False):
    """Compile fences independently as examples, grouped only for Cargo efficiency."""
    with tempfile.TemporaryDirectory(prefix='view-' if integration else 'independent-', dir=REPORT) as temp:
        directory = Path(temp)
        if integration:
            (directory/'src').mkdir(); (directory/'src/lib.rs').write_text('//! Isolated view documentation imports.\n')
            manifest = '[workspace]\n[package]\nname="view-doc-examples"\nversion="0.0.0"\nedition="2024"\n[features]\nssr=["stackpress-framework-view/leptos-stream","leptos/ssr"]\nhydrate=["stackpress-framework-view/leptos-hydrate","stackpress-framework-view/leptos-async","leptos/hydrate"]\ncsr=["stackpress-framework-view/leptos-csr","leptos/csr"]\n[dependencies]\nleptos={version="=0.8.22",default-features=false}\nstackpress-framework-view={path='+json.dumps(str(ROOT/'crates/framework/framework-view'))+',default-features=false}\nstackpress-leptos={path='+json.dumps(str(ROOT/'crates/leptos'))+',default-features=false}\n[target.\'cfg(target_arch="wasm32")\'.dependencies]\nweb-sys={version="=0.3.106",features=["HtmlElement"]}\n[target.\'cfg(not(target_arch="wasm32"))\'.dev-dependencies]\ntokio={version="=1.54.1",features=["macros","rt-multi-thread"]}\n'
            lock = ROOT/'Cargo.lock'
        else:
            shutil.copytree(HERE/'src', directory/'src')
            manifest = (HERE/'Cargo.toml').read_text().replace('path = "../../crates/leptos"', 'path = '+json.dumps(str(ROOT/'crates/leptos')))
            # This disposable harness owns only extracted fences, not the fixture CLI.
            manifest = manifest.replace('[[example]]\nname = "document"\nrequired-features = ["ssr"]\n', '')
            manifest = manifest.split('[lints.rust]')[0]
            lock = HERE/'Cargo.lock'
        manifest = manifest.replace('edition="2024"', 'edition="2024"\nautoexamples=false') if integration else manifest.replace('edition = "2024"','edition = "2024"\nautoexamples = false')
        (directory/'examples').mkdir()
        grouped = {'ssr':[], 'hydrate':[], 'csr':[]}
        quickstart = None
        for path in paths:
            for line, code in fences(path):
                name = f'doc_{len(records):03}'
                if not re.search(r'\bfn\s+main\s*\(', code):
                    code += '\nfn main() {}\n'
                (directory/'examples'/(name+'.rs')).write_text('#![allow(unused)]\n'+code)
                selected = scope(path, code)
                modes = ['ssr','hydrate','csr'] if selected == 'shared' else [selected]
                for mode in modes:
                    grouped[mode].append(name)
                manifest += '\n[[example]]\nname='+json.dumps(name)+'\npath='+json.dumps('examples/'+name+'.rs')+'\n'
                records.append(dict(file=str(path.relative_to(ROOT)), line=line, example=name,
                                    modes=modes, integration=integration))
                if path == ROOT/'crates/leptos/docs/getting-started.md':
                    quickstart = name
        (directory/'Cargo.toml').write_text(manifest)
        (directory/'Cargo.lock').write_bytes(lock.read_bytes())
        # Resolve only local manifest entries from the seeded lock, offline; reject drift immediately.
        result = subprocess.run(['cargo','metadata','--offline','--format-version','1','--manifest-path',str(directory/'Cargo.toml')],stdout=subprocess.DEVNULL)
        assert result.returncode == 0
        audit_lock(directory)
        for mode, examples in grouped.items():
            if not examples:
                continue
            features = mode if integration else mode+',async,rich,format'+(',stream' if mode=='ssr' else '')
            command = ['cargo','check','--locked','--offline','--manifest-path',str(directory/'Cargo.toml'),'--features',features]
            if mode != 'ssr':
                command += ['--target','wasm32-unknown-unknown']
            command += [arg for name in examples for arg in ['--example',name]]
            run(command)
        if quickstart:
            run(['cargo','run','--locked','--offline','--manifest-path',str(directory/'Cargo.toml'),
                 '--features','ssr,async,rich,format,stream','--example',quickstart])
        audit_lock(directory)


library = sorted((ROOT/'crates/leptos/docs').rglob('*.md'))
view = ROOT/'crates/framework/framework-view/docs'
view_pages = [view/'integration-leptos.md',view/'migration-issue17.md',view/'leptos-concepts.md',
              *sorted((view/'api').glob('leptos-*.md')), *sorted((view/'guides').glob('leptos-*.md'))]
check_group(library)
check_group(view_pages, integration=True)
(REPORT/'summary.json').write_text(json.dumps(dict(fences=records, compiled=len(records),
    independent_has_view_dependency=False),indent=2))
print('PASS',len(records),'literal Markdown Rust examples; native quick start executed')
