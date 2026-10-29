"""Assemble explicit debug review assets; ordinary/release frontends stay separate."""
import hashlib
import json
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[4]
MARKER = b'STACKPRESS_INSTALLED_REVIEW'
CATALOG_START = """// STACKPRESS_INSTALLED_REVIEW: local fixture drafts, no HTTP or IPC adapter.
import init, { start } from './admin.js';
await init();
const dispose = start();
// Keep the original output nodes intact for existing exact-state assertions.
// Only this packaged review adds a collapsed mirror outside the Leptos root.
const debug = document.createElement('details');
debug.id = 'admin-review-debug';
const summary = document.createElement('summary');
summary.textContent = 'Debug state (review only)';
const state = document.createElement('pre');
const update = () => {
  state.textContent = ['proof-draft','project-draft','task-draft']
    .map(id => id + ': ' + document.getElementById(id).textContent).join('\\n');
};
debug.append(summary,state); document.querySelector('header').append(debug);
const observer = new MutationObserver(update);
observer.observe(document.getElementById('app'),{childList:true,subtree:true,characterData:true});
update();
window.adminDispose = () => { observer.disconnect(); dispose(); };
window.adminReady = true;
"""


def ordinary(source, destination):
    """Replace only an owned staging folder, refusing stale review bytes at the source."""
    assert_clean(source)
    if destination.is_symlink():
        raise ValueError('Refusing symlink staging folder')
    if destination.exists():
        shutil.rmtree(destination)
    shutil.copytree(source, destination)
    shutil.copy2(ROOT/'examples/profile-ui/native/native-bootstrap.js', destination/'start.js')


def assert_clean(folder):
    """Check every shipped byte, including binary WASM and nested bindgen snippets."""
    if folder.is_symlink() or not folder.is_dir():
        raise ValueError('Expected a regular frontend directory')
    review_names={'admin-review','admin-review.html','review-controls.js','installed-review.json','review-schema.json'}
    for path in folder.rglob('*'):
        if path.is_symlink():
            raise ValueError('Symlink frontend asset refused')
        if path.name in review_names or (path.is_file() and MARKER in path.read_bytes()):
            raise ValueError('Review asset in ordinary frontend: '+str(path))


def prepare(source, admin, destination, variant, profile='debug'):
    """Copy exact matching CSR bytes and add only a labelled local role-catalog page."""
    if profile != 'debug' or variant not in ('disabled', 'plugin'):
        raise ValueError('Installed review requires debug and a compiled variant')
    ordinary(source, destination)
    catalog = destination/'admin-review'; catalog.mkdir()
    # Exclude the web bootstrap: it installs the HTTP mutation/CSV transport.
    for name in ('admin.js', 'admin_bg.wasm'):
        shutil.copy2(admin/name, catalog/name)
    if (admin/'snippets').exists():
        shutil.copytree(admin/'snippets', catalog/'snippets')
    (catalog/'start.js').write_text(CATALOG_START)
    styles = ROOT/'crates/leptos/src/components'
    css = ''.join((styles/name).read_text() for name in ('styles.css','core.css','advanced.css','breadth.css'))
    css += (ROOT/'examples/generated-admin/shared/src/roles.css').read_text()
    (catalog/'catalog.css').write_text(css+'''\nbody{margin:0}header{padding:1rem;font:16px system-ui}
/* Preserve proof state in the DOM while keeping raw dumps out of the walkthrough. */
#proof-draft,#project-draft,#task-draft{display:none}
#admin-review-debug pre{white-space:pre-wrap;overflow-wrap:anywhere}
''')
    # No admin-seed means generated-admin-client mounts Admin, its local role catalog.
    (destination/'admin-review.html').write_text('''<!doctype html><html lang="en"><head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; connect-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; object-src 'none'; base-uri 'none'; form-action 'none'; frame-src 'none'">
<title>Generated admin review</title><link rel="stylesheet" href="admin-review/catalog.css"></head><body>
<!-- STACKPRESS_INSTALLED_REVIEW -->
<header><a href="index.html">Return to Profile UI</a><h1>Generated admin review</h1>
<p>Debug review only. Local role-catalog drafts; not installed admin CRUD. No admin mutation or IPC service.</p>
<p>Compiled variant: '''+variant+'''</p></header><div id="app"></div>
<script type="module" src="admin-review/start.js"></script></body></html>''')
    shutil.copy2(ROOT/'examples/profile-ui/native/review-controls.js', destination/'review-controls.js')
    # Keep the normal native acquisition sequence; the review panel is outside its root.
    index = destination/'index.html'
    index.write_text(index.read_text().replace('</body>', '<script type="module" src="review-controls.js"></script></body>'))
    files = {str(path.relative_to(destination)): hashlib.sha256(path.read_bytes()).hexdigest()
             for path in destination.rglob('*') if path.is_file()}
    (destination/'installed-review.json').write_text(json.dumps(dict(feature='review-preview',profile=profile,variant=variant,files=files),indent=2)+'\n')


def verify(folder, variant):
    """Refuse mismatched or modified staging before a coordinator mobile build."""
    if folder.is_symlink():raise ValueError('Symlink review folder refused')
    report = json.loads((folder/'installed-review.json').read_text())
    if (report['feature'], report['profile'], report['variant']) != ('review-preview','debug',variant):
        raise ValueError('Installed review selection mismatch')
    actual = {str(path.relative_to(folder)): hashlib.sha256(path.read_bytes()).hexdigest()
              for path in folder.rglob('*') if path.is_file() and path.name != 'installed-review.json'}
    if actual != report['files'] or any(path.is_symlink() for path in folder.rglob('*')):
        raise ValueError('Installed review assets changed; rebuild assets')
    return report
