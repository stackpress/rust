#!/usr/bin/env python3
"""Listener-free Part B assets, isolation checks and native verification; no mobile launch."""
import argparse
import json
from pathlib import Path
import shutil
import sys
import review

NATIVE='stackpress-example-profile-ui-native'
CLIENT='stackpress-example-profile-ui-client'
sys.path.insert(0,str(review.ROOT/'examples/profile-ui/native/proofs'))
import review_assets


def assets(ctx):
    """Compile actual matching generated CSR variants without a Tauri options listener."""
    from proofs.builds import isolated_workspace
    from proofs.runtime import TARGET
    root=ctx.logs.parent
    ctx.run('installed-profile-csr',['python3','examples/profile-ui/build-client.py','csr'],
            extra={'PROFILE_UI_TARGET':str(TARGET)},timeout=1200)
    with isolated_workspace(ctx) as snapshot:
        for variant,plugin in [('plugin',True),('disabled',False)]:
            provenance=review.review_schema(ctx,snapshot,plugin,'installed-'+variant+'-generate')
            ctx.run('installed-'+variant+'-wasm-clean',['cargo','clean','--target','wasm32-unknown-unknown',
                    '-p','generated-admin-client','-p','generated-admin-shared'],cwd=snapshot)
            ctx.run('installed-'+variant+'-csr',['cargo','build','--offline','--locked','-p','generated-admin-client',
                    '--target','wasm32-unknown-unknown','--features','csr'],cwd=snapshot,timeout=1200)
            admin=ctx.logs/'installed-admin'/variant
            if admin.exists():shutil.rmtree(admin)
            admin.mkdir(parents=True)
            ctx.run('installed-'+variant+'-bindgen',['wasm-bindgen','--target','web','--no-typescript',
                    '--out-dir',str(admin),'--out-name','admin',str(TARGET/'wasm32-unknown-unknown/debug/generated_admin_client.wasm')])
            destination=root/'ios-assets'/variant
            review_assets.prepare(review.ROOT/'examples/profile-ui/dist/csr',admin,destination,variant)
            # Record provenance inside the hash-checked folder, then refresh its inventory.
            (destination/'review-schema.json').write_text(json.dumps(provenance,indent=2)+'\n')
            inventory=json.loads((destination/'installed-review.json').read_text())
            import hashlib
            inventory['files']['review-schema.json']=hashlib.sha256((destination/'review-schema.json').read_bytes()).hexdigest()
            (destination/'installed-review.json').write_text(json.dumps(inventory,indent=2)+'\n')
            review_assets.verify(destination,variant)
    review_assets.ordinary(review.ROOT/'examples/profile-ui/dist/csr',ctx.logs/'normal-debug-frontend')


def tests(ctx, name):
    """Exercise launcher ownership/error paths without simctl, Xcode, apps or sockets."""
    ctx.run(name+'-python',['python3','-m','unittest','discover','-s',
            'examples/profile-ui/native/proofs','-p','test_review*.py'],timeout=60)
    ctx.run(name+'-native',['cargo','test','--offline','--locked','-p',NATIVE,'--features','review-preview'],timeout=1200)


def isolation(ctx):
    """Prove normal/release assets and native graphs omit the installed review surface."""
    from proofs.runtime import TARGET
    ctx.run('installed-normal-debug-clean',['cargo','clean','-p',NATIVE])
    ctx.run('installed-normal-debug',['cargo','build','--offline','--locked','-p',NATIVE],timeout=1200)
    retained=[]
    debug=TARGET/'debug/libstackpress_example_profile_ui_native.rlib'
    shutil.copy2(debug,ctx.logs/'normal-native-debug.rlib');retained.append(ctx.logs/'normal-native-debug.rlib')
    # A real release-profile attempt must fail with the deliberate guard.
    try:
        ctx.run('installed-release-refusal',['cargo','build','--offline','--locked','-p',NATIVE,
                '--release','--features','review-preview'],timeout=1200,
                extra={'CARGO_PROFILE_RELEASE_OPT_LEVEL':'0','CARGO_PROFILE_RELEASE_DEBUG':'0'})
    except SystemExit as error:
        if error.code!=101 or 'review-preview is restricted to debug builds' not in (ctx.logs/'installed-release-refusal.log').read_text():raise
    else:raise AssertionError('Release review feature unexpectedly compiled')
    ctx.run('installed-normal-release',['cargo','build','--offline','--locked','-p',NATIVE,'--release'],
            extra={'CARGO_PROFILE_RELEASE_OPT_LEVEL':'0','CARGO_PROFILE_RELEASE_DEBUG':'0'},timeout=1200)
    ctx.run('installed-normal-graph',['cargo','tree','--offline','--locked','-p',NATIVE,'-e','normal'])
    graph=(ctx.logs/'installed-normal-graph.log').read_text()
    assert review.PACKAGE not in graph and 'generated-admin' not in graph
    native=TARGET/'release/libstackpress_example_profile_ui_native.rlib'
    shutil.copy2(native,ctx.logs/'normal-native-release.rlib');retained.append(ctx.logs/'normal-native-release.rlib')
    for path in retained:
        data=path.read_bytes()
        assert all(marker not in data for marker in (review_assets.MARKER,b'review-wait',b'admin-review.html')),path
    ctx.run('installed-client-graph',['cargo','tree','--offline','--locked','-p',CLIENT,
            '--target','wasm32-unknown-unknown','--features','csr','-e','normal'])
    client_graph=(ctx.logs/'installed-client-graph.log').read_text()
    assert review.PACKAGE not in client_graph and 'generated-admin' not in client_graph
    ctx.run('installed-release-csr',['cargo','build','--offline','--locked','-p',CLIENT,'--release',
            '--target','wasm32-unknown-unknown','--features','csr'],timeout=1200,
            extra={'CARGO_PROFILE_RELEASE_OPT_LEVEL':'0','CARGO_PROFILE_RELEASE_DEBUG':'0'})
    output=ctx.logs/'normal-release-frontend'
    if output.exists():shutil.rmtree(output)
    shutil.copytree(ctx.logs/'normal-debug-frontend',output)
    ctx.run('installed-release-bindgen',['wasm-bindgen','--target','web','--no-typescript','--out-dir',str(output),
            '--out-name','profile_ui',str(TARGET/'wasm32-unknown-unknown/release/stackpress_example_profile_ui_client.wasm')])
    for folder in (review.ROOT/'examples/profile-ui/dist/csr',ctx.logs/'normal-debug-frontend',output):
        review_assets.assert_clean(folder)
    config=json.loads((review.ROOT/'examples/profile-ui/native/tauri.conf.json').read_text())
    ios=json.loads((review.ROOT/'examples/profile-ui/native/tauri.ios.conf.json').read_text())
    assert config['build']['frontendDist']=='../dist/csr'
    assert ios['build']['frontendDist']=='gen/csr-bundle'
    (ctx.logs/'installed-isolation.json').write_text(json.dumps(dict(release_review_refused=True,
        normal_native_artifacts=list(map(str,retained)),normal_graph_has_review=False,
        clean_frontends=['normal CSR source','normal debug iOS staging','normal release CSR staging'],
        normal_configs_unchanged=True,mobile_build_executed=False),indent=2)+'\n')


def coverage(ctx):
    """Measure the changed native library; app event-loop/platform branches remain unclaimed."""
    from proofs.runtime import Context
    from coverage import report
    measured=Context('installed-coverage')
    measured.run('native-tests',['cargo','llvm-cov','--offline','--locked','-p',NATIVE,
                 '--features','review-preview','--no-report'],timeout=1200)
    report(measured,['-p',NATIVE],packages=[NATIVE],prefixes=['examples/profile-ui/native/src/'])
    ctx.run('installed-python-coverage',['python3','examples/profile-ui/native/proofs/review_coverage.py',
            str(ctx.logs/'installed-python-coverage.json')],timeout=60)


def quality(ctx):
    """Keep app compilation separate from library execution and forbid warning drift."""
    ctx.run('installed-fmt-check',['cargo','fmt','-p',NATIVE,'--','--check'])
    ctx.run('installed-app-clippy',['cargo','clippy','--offline','--locked','-p',NATIVE,
            '--features','app,review-preview','--all-targets','--','-D','warnings'],timeout=1800)
    ctx.run('installed-doc',['cargo','doc','--offline','--locked','-p',NATIVE,
            '--features','review-preview','--no-deps'],extra={'RUSTDOCFLAGS':'-D warnings'},timeout=1200)
    ctx.run('installed-normal-tests',['cargo','test','--offline','--locked','-p',NATIVE],timeout=1200)


def main():
    """All child processes use the same capped target and existing Context watchdog."""
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action',choices=['assets','tests','isolation','browser','coverage','quality'])
    parser.add_argument('--directory',type=Path,required=True)
    parser.add_argument('--name',default='part-b')
    args=parser.parse_args();ctx=review.context(args.directory)
    if args.action=='assets':assets(ctx)
    elif args.action=='tests':tests(ctx,args.name)
    elif args.action=='isolation':isolation(ctx)
    elif args.action=='coverage':coverage(ctx)
    elif args.action=='quality':quality(ctx)
    else:ctx.run(args.name+'-browser',['node','examples/review-preview/installed-browser.cjs',str(ctx.logs.parent)],
                 extra={'ADMIN_PLAYWRIGHT':review.PLAYWRIGHT},timeout=180)


if __name__=='__main__':main()
