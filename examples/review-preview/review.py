#!/usr/bin/env python3
"""Owned, capped, no-install builds and explicit setup for the debug-only review host."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys

ROOT = Path(__file__).resolve().parents[2]
PLAYWRIGHT = '/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright'
PACKAGE = 'stackpress-example-review-preview'
MOUNTS = dict(gallery='ui-gallery',preferences='ui-preferences',admin='generated-admin',format='ui-format',stream='ui-stream')


def context(directory):
    """Reuse the existing measured watchdog, with exactly one owned target root."""
    directory=directory.expanduser().resolve()
    directory.mkdir(parents=True,exist_ok=True)
    os.environ['ADMIN_PROOF_ROOT']=str(directory)
    os.environ['CARGO_NET_OFFLINE']='true'
    sys.path.insert(0,str(ROOT/'examples/generated-admin'))
    from proofs.runtime import Context
    return Context('review')


def manifest(folder, name):
    """Hash exact allowlisted static bytes; requests can never choose disk paths."""
    entries=[]
    for source in sorted(folder.iterdir()):
        if source.suffix not in ['.js','.wasm','.png']:continue
        mime={'.js':'text/javascript; charset=utf-8','.wasm':'application/wasm','.png':'image/png'}[source.suffix]
        data=source.read_bytes()
        entries.append(dict(route='/'+MOUNTS[name]+'/assets/'+source.name,file=source.name,size=len(data),sha256=hashlib.sha256(data).hexdigest(),mime=mime,cache='revalidate'))
    (folder/'manifest.json').write_text(json.dumps(dict(entries=entries),indent=2)+'\n')


def copy_assets(source, destination, name):
    """Retain original module behavior behind a review-only manual hydration gate."""
    shutil.copytree(source,destination,dirs_exist_ok=True)
    if name=='format':
        original="import init,{start,start_editor} from './proof.js'; await init(); const locale=document.getElementById('app').dataset.editorLocale; window.reviewDispose=locale?start_editor(locale):start(); window.formatReady=true;"
        (destination/'application.js').write_text(original)
    else:
        shutil.copy2(destination/'start.js',destination/'application.js')
    (destination/'start.js').write_text("""// STACKPRESS_REVIEW_PREVIEW: absent from normal example bundles.
if(document.querySelector('meta[name="review-pause"]')?.content==='true'){
 const button=document.createElement('button');button.id='review-start';button.textContent='Start browser controls';
 button.style.cssText='position:fixed;right:1rem;bottom:1rem;z-index:2147483647';
 button.addEventListener('mousedown',event=>event.preventDefault());document.body.prepend(button);
 await new Promise(resolve=>button.addEventListener('click',()=>{button.remove();resolve();},{once:true}));
}
await import('./application.js');
""")
    manifest(destination,name)


def build(ctx):
    """Build matching assets and compiled disabled/plugin variants without tracked regeneration."""
    from proofs.runtime import TARGET
    directory=ctx.logs.parent
    common=dict(CARGO_TARGET_DIR=str(TARGET),PROFILE_UI_NATIVE_TARGET=str(TARGET),PROFILE_UI_TARGET=str(TARGET),
                GALLERY_OUTPUT_DIR=str(directory/'gallery'),PREFERENCES_LOGS=str(directory/'preferences'),UI_STREAM_OUTPUT_DIR=str(directory/'stream'),
                UI_FORMAT_OUTPUT_DIR=str(directory/'format'),ADMIN_PLAYWRIGHT=PLAYWRIGHT)
    ctx.run('gallery-build',['python3','examples/ui-gallery/build.py'],extra=common,timeout=1800)
    ctx.run('preferences-build',['python3','examples/ui-preferences/build.py'],extra=common,timeout=1800)
    ctx.run('stream-build',['python3','examples/ui-stream/build.py'],extra=common,timeout=1800)
    ctx.run('format-build',['python3','examples/ui-format/check.py','build'],extra=common,timeout=3600)
    bundles(ctx)


def bundles(ctx):
    """Regenerate/rebuild only the compiled variants, reusing already verified example assets."""
    from proofs.runtime import TARGET
    from proofs.builds import isolated_workspace, generate
    directory=ctx.logs.parent
    sources=dict(gallery=directory/'gallery/gallery-assets',preferences=directory/'preferences/assets',stream=directory/'stream/assets',format=directory/'format/assets')
    with isolated_workspace(ctx) as snapshot:
        for variant,plugin in [('plugin',True),('disabled',False)]:
            generate(ctx,snapshot,False,plugin,'review-'+variant+'-generate')
            ctx.run('gateway-'+variant,['cargo','build','--offline','--locked','-p',PACKAGE,'--features','review-preview'],cwd=snapshot,timeout=1800,extra={'STACKPRESS_REVIEW_VARIANT':variant})
            bundle=directory/'bundle'/variant;bundle.mkdir(parents=True,exist_ok=True)
            shutil.copy2(TARGET/'debug/slice8-review',bundle/'slice8-review')
            for mode in ['hydrate','csr']:
                ctx.run('admin-'+variant+'-'+mode,['cargo','build','--offline','--locked','-p','generated-admin-client','--target','wasm32-unknown-unknown','--features',mode],cwd=snapshot)
                admin=ctx.logs/'admin-assets'/variant/mode;admin.mkdir(parents=True,exist_ok=True)
                ctx.run('admin-bindgen-'+variant+'-'+mode,['wasm-bindgen','--target','web','--no-typescript','--out-dir',str(admin),'--out-name','admin',str(TARGET/'wasm32-unknown-unknown/debug/generated_admin_client.wasm')])
                shutil.copy2(ROOT/'examples/generated-admin/bootstrap.js',admin/'start.js')
                copy_assets(admin,bundle/'admin'/mode,'admin')
                for name in ['gallery','preferences','format']:
                    source=sources[name]/(mode+'-both' if name=='format' else mode)
                    copy_assets(source,bundle/name/mode,name)
            copy_assets(sources['stream'],bundle/'stream/hydrate','stream')
    print('Prepared '+str(directory/'bundle'),flush=True)


def main():
    """No command starts a listener; serving is exclusively a coordinator devmetrics action."""
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action',choices=['build','bundles','setup','run'])
    parser.add_argument('--directory',type=Path,required=True)
    parser.add_argument('--name',default='command')
    parser.add_argument('--timeout',type=int,default=1800)
    args,command=parser.parse_known_args()
    ctx=context(args.directory)
    if args.action=='build':build(ctx)
    elif args.action=='bundles':bundles(ctx)
    elif args.action=='setup':ctx.run('setup',[str(ctx.logs.parent/'bundle/disabled/slice8-review'),'setup','--directory',str(ctx.logs.parent)],timeout=60)
    else:
        if command and command[0]=='--':command=command[1:]
        if not command:parser.error('run needs a command after --')
        ctx.run(args.name,command,timeout=args.timeout)


if __name__=='__main__':main()
