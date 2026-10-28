"""Build reusable finite artifacts; plugin regeneration is confined to a disposable workspace."""
from contextlib import contextmanager
import hashlib
import io
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
from .runtime import ROOT, TARGET


def hashes(root):
    """Compare every managed byte, including manifests and all output partitions."""
    return {str(path.relative_to(root)): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in root.rglob('*') if path.is_file()}


@contextmanager
def isolated_workspace(ctx):
    """Overlay current source onto a tracked snapshot, so plugin proofs never touch shared outputs."""
    archive = subprocess.check_output(['git','archive','--format=tar','HEAD'],cwd=ROOT,timeout=60)
    with tempfile.TemporaryDirectory(prefix='workspace-',dir=ctx.logs) as temp:
        root=Path(temp)
        with tarfile.open(fileobj=io.BytesIO(archive)) as source:
            source.extractall(root,filter='data')
        changed=subprocess.check_output(['git','diff','--name-only','HEAD'],cwd=ROOT,text=True,timeout=30).splitlines()
        changed+=subprocess.check_output(['git','ls-files','--others','--exclude-standard'],cwd=ROOT,text=True,timeout=30).splitlines()
        for name in set(changed):
            if name.startswith('.agents/'):
                continue
            source=ROOT/name; destination=root/name
            if source.is_file():
                destination.parent.mkdir(parents=True,exist_ok=True)
                shutil.copy2(source,destination)
            elif destination.is_file():
                destination.unlink()
        yield root


def artifacts(ctx, formatted=False, destination=None, root=ROOT, prefix='build'):
    """One matching SSR/CSR/hydrate source selection, copied outside the disposable target."""
    assert subprocess.check_output(['wasm-bindgen','--version'],text=True,timeout=30).strip()=='wasm-bindgen 0.2.129'
    destination=destination or ctx.logs
    destination.mkdir(parents=True,exist_ok=True)
    features=['--features','format'] if formatted else []
    # Separate workspaces share top-level artifact names in this sole target. A fresh Cargo
    # fingerprint alone can leave another workspace's binary there (including its manifest path).
    # Review-only labels can leave newer shared objects in this one warm target.
    # Clear shared native output before consuming a new generated tree; old source
    # timestamps in an isolated snapshot are not sufficient proof. WASM is cleared below.
    ctx.run(prefix+'-shared-clean',['cargo','clean','-p','generated-admin-shared'],cwd=root)
    ctx.run(prefix+'-web-clean',['cargo','clean','-p','generated-admin-web'],cwd=root)
    ctx.run(prefix+'-web',['cargo','build','-p','generated-admin-web',*features,'--locked'],cwd=root)
    shutil.copy2(TARGET/'debug/generated-admin-web',destination/'admin-web')
    for mode in ['hydrate','csr']:
        ctx.run(prefix+'-'+mode+'-clean',['cargo','clean','--target','wasm32-unknown-unknown','-p','generated-admin-client','-p','generated-admin-shared'],cwd=root)
        ctx.run(prefix+'-'+mode,['cargo','build','-p','generated-admin-client','--target','wasm32-unknown-unknown','--features',mode+(',format' if formatted else ''),'--locked'],cwd=root)
        assets=destination/'assets'/mode
        assets.mkdir(parents=True,exist_ok=True)
        ctx.run(prefix+'-'+mode+'-bindgen',['wasm-bindgen','--target','web','--no-typescript','--out-dir',str(assets),'--out-name','admin',str(TARGET/'wasm32-unknown-unknown/debug/generated_admin_client.wasm')])
        shutil.copy2(ROOT/'examples/generated-admin/bootstrap.js',assets/'start.js')


def generate(ctx,root,formatted,plugin,name,check=False):
    """Select the ordinary app registry; this helper is used only inside the disposable workspace."""
    ctx.run(name+'-tool-clean',['cargo','clean','-p','generated-admin-tool'],cwd=root)
    command=['cargo','run','-p','generated-admin-tool']+(['--features','format'] if formatted else [])+['--locked','--']
    if formatted:
        command+=['--input','examples/generated-admin/schema-format.idea','--output','examples/generated-admin/shared/src/generated-format']
    if plugin: command+=['--plugin']
    if check: command+=['--check']
    ctx.run(name,command,cwd=root)


def build(ctx,suite):
    """Keep all former role/plugin variants; ordinary CRUD/format/transfer builds need no regeneration."""
    if suite not in ['roles','plugin']:
        artifacts(ctx,formatted=suite=='format')
        return
    with isolated_workspace(ctx) as root:
        if suite=='roles':
            for variant,plugin in [('base',False),('plugin',True),('disabled',False)]:
                generate(ctx,root,False,plugin,variant+'-generate')
                artifacts(ctx,destination=ctx.logs/variant,root=root,prefix=variant)
                shutil.copy2(ctx.logs/variant/'admin-web',ctx.logs/(variant+'-web'))
                for mode in ['hydrate','csr']:
                    destination=ctx.logs/'assets'/variant/mode
                    destination.parent.mkdir(parents=True,exist_ok=True)
                    shutil.copytree(ctx.logs/variant/'assets'/mode,destination,dirs_exist_ok=True)
        else:
            managed=root/'examples/generated-admin/shared/src/generated-format'
            original=hashes(managed)
            generate(ctx,root,True,True,'plugin-generate')
            generate(ctx,root,True,True,'plugin-check',check=True)
            artifacts(ctx,True,ctx.logs/'plugin',root,'plugin')
            generate(ctx,root,True,False,'plugin-disable')
            assert original==hashes(managed),'disabling restores the entire root byte-for-byte'
            artifacts(ctx,True,ctx.logs,root,'disabled')
