#!/usr/bin/env python3
"""Normal-edge ownership and mode proofs, including real feature-gated import failures."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib
from proof import ROOT, HERE, OUT, MANIFEST, TARGET

GRAPH = OUT / 'graphs'
GRAPH.mkdir(parents=True, exist_ok=True)
APPROVED = {(p['name'], p['version'], p.get('source'), p.get('checksum'))
            for p in tomllib.loads((ROOT/'Cargo.lock').read_text())['package'] if p.get('source')}
FORBIDDEN = {'stackpress-framework-view', 'stackpress-server', 'stackpress-server-http',
             'stackpress', 'stackpress-lib', 'stackpress-framework-language',
             'stackpress-framework-preferences', 'stackpress-sql', 'stackpress-framework-sql',
             'stackpress-framework-schema', 'stackpress-framework-admin',
             'stackpress-idea', 'stackpress-idea-parser', 'stackpress-idea-transformer', 'tauri'}
BROWSER_HOSTS = {'stackpress-server', 'stackpress-server-http', 'stackpress-framework-sql',
                 'stackpress-sql', 'sqlx', 'tokio', 'libc', 'tauri',
                 'stackpress-idea', 'stackpress-idea-parser', 'stackpress-idea-transformer'}
records = []


def external(path):
    """Compare identity/source/checksum tuples, never infer equality from package names."""
    return {(p['name'], p['version'], p.get('source'), p.get('checksum'))
            for p in tomllib.loads(path.read_text())['package'] if p.get('source')}


def tree(package, features, browser=False):
    """Browser normal graphs exclude proc-macro edges because those execute on the host."""
    independent = package == 'leptos-independent'
    command = ['cargo', 'tree', '--manifest-path', MANIFEST if independent else str(ROOT/'Cargo.toml'),
               '-p', package, '--locked', '--offline', '--no-default-features', '--prefix', 'none',
               '--format', '{p}', '--edges', 'normal,no-proc-macro' if browser else 'normal']
    if features:
        command += ['--features', features]
    if browser:
        command += ['--target', 'wasm32-unknown-unknown']
    output = subprocess.check_output(command, text=True)
    names = {line.split()[0] for line in output.splitlines()}
    tag = package + '-' + (features or 'default').replace(',', '-')
    (GRAPH/(tag+'.txt')).write_text(output)
    if package in ['leptos-independent', 'stackpress-leptos']:
        assert not names & FORBIDDEN, (tag, names & FORBIDDEN)
    if package == 'stackpress-framework-view' and 'leptos' not in features:
        assert not names & {'leptos', 'stackpress-leptos'}, tag
    if package == 'stackpress-framework-view' and features in ['', 'leptos', 'leptos-async']:
        assert 'stackpress-server' not in names and 'stackpress-lib' not in names, tag
    if browser:
        assert not names & BROWSER_HOSTS, (tag, names & BROWSER_HOSTS)
    if package == 'stackpress' and features.startswith('leptos'):
        assert 'stackpress-framework-view' not in names, tag
    records.append(dict(package=package, features=features or 'default', browser_runtime_only=browser,
                        packages=len(names)))


for package, selections in [
    ('leptos-independent', ['', 'ssr', 'ssr,async', 'ssr,async,rich,format,stream',
                            'hydrate', 'hydrate,async,rich,format', 'csr', 'csr,async,rich,format']),
    ('stackpress-leptos', ['', 'async,rich,format', 'ssr', 'ssr,rich,format,stream',
                          'hydrate', 'hydrate,async,rich,format', 'csr', 'csr,async,rich,format']),
    ('stackpress-framework-view', ['', 'preferences,flash', 'native-file', 'web-preferences,web-flash',
                                  'leptos', 'leptos-async', 'leptos-ssr', 'leptos-stream',
                                  'leptos,preferences', 'leptos-hydrate,leptos-async,preferences,flash',
                                  'leptos-csr,leptos-async,preferences,flash']),
    ('stackpress-framework-admin', ['', 'ssr,format', 'hydrate,format', 'csr,format']),
    ('stackpress', ['', 'leptos', 'leptos-ssr,leptos-rich,leptos-format',
                    'leptos-hydrate,leptos-async', 'leptos-csr,leptos-async',
                    'framework-view-preferences,framework-view-flash', 'framework-view-leptos',
                    'framework-view-leptos-stream', 'framework-view-leptos-hydrate', 'framework-view-leptos-csr']),
]:
    for features in selections:
        tree(package, features, 'hydrate' in features or 'csr' in features)

# Native and WASM entry points must reject each mixed pair and native SSR on WASM.
negative = []
for package, prefixes in [('leptos-independent', ''), ('stackpress-leptos', ''),
                          ('stackpress-framework-view', 'leptos-'), ('stackpress', 'leptos-')]:
    for modes in [('ssr', 'hydrate'), ('ssr', 'csr'), ('hydrate', 'csr'), ('ssr',)]:
        features = ','.join(prefixes + mode for mode in modes)
        command = ['cargo', 'check', '--manifest-path', MANIFEST if package == 'leptos-independent' else str(ROOT/'Cargo.toml'),
                   '-p', package, '--locked', '--offline', '--no-default-features', '--features', features]
        if len(modes) == 1:
            command += ['--target', 'wasm32-unknown-unknown']
        result = subprocess.run(command, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        tag = package + '-invalid-' + '-'.join(modes)
        (GRAPH/(tag+'.log')).write_text(result.stdout)
        assert result.returncode != 0 and ('select exactly one' in result.stdout or
               'select one render mode' in result.stdout or 'ssr is a native mode' in result.stdout), (tag, result.stdout[-4000:])
        negative.append(tag)

# Actual external consumers prove import absence. The independent consumer itself
# never acquires a view/facade dependency, including dev dependencies.
imports = []
cases = [
    ('view-contracts', 'stackpress-framework-view', '', 'pub fn check(){let _=stackpress_framework_view::PageContract::new(1,"independent");}', None),
    ('view-no-integration', 'stackpress-framework-view', '', 'use stackpress_framework_view::leptos;', 'leptos'),
    ('view-prefs-no-integration', 'stackpress-framework-view', 'preferences,flash', 'use stackpress_framework_view::leptos;', 'leptos'),
    ('view-integration-no-ssr', 'stackpress-framework-view', 'leptos', 'use stackpress_framework_view::leptos::render;', 'render'),
    ('facade-leptos-no-view', 'stackpress', 'leptos-ssr', 'use stackpress::framework_view;', 'framework_view'),
]
for tag, package, features, source, missing in cases:
    with tempfile.TemporaryDirectory(prefix=tag+'-', dir=GRAPH) as temporary:
        directory = Path(temporary); (directory/'src').mkdir()
        location = ROOT if package == 'stackpress' else ROOT/'crates/framework/framework-view'
        selected = ','.join(json.dumps(f) for f in features.split(',') if f)
        (directory/'Cargo.toml').write_text('[workspace]\n[package]\nname="import-proof"\nversion="0.0.0"\nedition="2024"\n[dependencies]\n'+package+'={path='+json.dumps(str(location))+',default-features=false,features=['+selected+']}\n')
        (directory/'Cargo.lock').write_bytes((ROOT/'Cargo.lock').read_bytes())
        (directory/'src/lib.rs').write_text(source)
        command = ['cargo', 'check', '--offline', '--manifest-path', str(directory/'Cargo.toml')]
        result = subprocess.run(command, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        (GRAPH/(tag+'.log')).write_text(result.stdout)
        assert external(directory/'Cargo.lock') <= APPROVED, tag
        if missing:
            assert result.returncode != 0 and 'unresolved import' in result.stdout and missing in result.stdout, (tag, result.stdout)
        else:
            assert result.returncode == 0, result.stdout
        imports.append(dict(case=tag, expected_failure=bool(missing)))

assert external(HERE/'Cargo.lock') <= APPROVED
metadata = json.loads(subprocess.check_output(['cargo','metadata','--offline','--locked','--format-version','1','--no-deps','--manifest-path',MANIFEST],text=True))
assert len(metadata['workspace_members']) == 1
normal = {d['name'] for d in metadata['packages'][0]['dependencies'] if d['kind'] is None and d['target'] is None}
assert normal == {'leptos', 'stackpress-leptos'}
(GRAPH/'summary.json').write_text(json.dumps(dict(graphs=records, rejected_modes=negative,
    import_cases=imports, browser_edges='normal,no-proc-macro', fixture_external_tuples=len(external(HERE/'Cargo.lock'))), indent=2))
print(f'PASS {len(records)} normal graphs, {len(negative)} invalid mode builds, {len(imports)} isolated import gates')
