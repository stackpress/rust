#!/usr/bin/env python3
"""Finite unsigned simulator build. G32 permits only its temporary options service."""
import argparse
import json
import os
from pathlib import Path
import signal
import shutil
import socket
import subprocess
import time

ROOT=Path(__file__).resolve().parents[4]
NATIVE=ROOT/'examples/profile-ui/native'
LOGS=Path(os.environ.get('PROFILE_UI_IOS_LOGS','/private/tmp/s8-lane-b-ios'))
TARGET=ROOT/'target/s8-ios-native'
WASM=ROOT/'target/s8-ios-wasm'


def budget(deadline=None):
    """A manual-review build measures its single native/WASM/Xcode target every 15s."""
    if not TARGET.exists():return
    deadline=deadline or time.monotonic()+30
    # rustc can remove a path while du traverses it. Retry a failed measurement,
    # but never turn it into a zero-sized target or extend the child's deadline.
    for _ in range(3):
        remaining=deadline-time.monotonic()
        if remaining<=0:raise TimeoutError('iOS target measurement timed out')
        measured=subprocess.run(['du','-sk',str(TARGET)],text=True,capture_output=True,timeout=remaining)
        if measured.returncode==0:
            size=int(measured.stdout.split()[0])
            if size>8*1024*1024:raise RuntimeError('iOS review target exceeded 8 GiB')
            return
    raise RuntimeError('Cannot measure iOS target: '+measured.stderr)


def build_command(variant=None, bundle=None):
    """Make review feature/assets one selection; ordinary builds keep the existing config."""
    command=['cargo','tauri','ios','build','--ci','--debug','--target','aarch64-sim',
             '--features','app'+(',review-preview' if variant else ''),'--no-sign','--archive-only']
    if variant:
        if variant not in ('disabled','plugin') or bundle is None:
            raise ValueError('A review build requires a compiled variant and checked assets')
        override=LOGS/'review-config.json'
        override.write_text(json.dumps({'build':{'frontendDist':str(bundle),'devUrl':None}}))
        command+=['--config',str(override)]
    return command+['--','--locked']


def run(name,command,env,cwd=ROOT,timeout=900):
    """Own the whole child group, kill/reap on timeout, and retain a finite log."""
    with (LOGS/(name+'.log')).open('w') as log:
        log.write('$ '+' '.join(map(str,command))+'\n');log.flush()
        with subprocess.Popen(list(map(str,command)),cwd=cwd,env=env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True) as child:
            try:
                deadline=time.monotonic()+timeout
                while child.poll() is None:
                    if time.monotonic()>=deadline:raise TimeoutError(name+' timed out')
                    budget(min(deadline,time.monotonic()+30))
                    try:child.wait(timeout=min(15,max(.001,deadline-time.monotonic())))
                    except subprocess.TimeoutExpired:pass
                code=child.returncode
            finally:
                if child.poll() is None:
                    try:os.killpg(child.pid,signal.SIGKILL)
                    except ProcessLookupError:pass
                child.wait()
        if code:raise RuntimeError(name+' failed; see '+str(LOGS/(name+'.log')))
    print('PASS '+name,flush=True)


def main():
    """Use installed tooling only; never start a dev command or packaged devUrl."""
    global TARGET,WASM
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--review-variant',choices=['disabled','plugin'])
    parser.add_argument('--directory',type=Path)
    args=parser.parse_args()
    if args.review_variant:
        if args.directory is None:parser.error('--review-variant requires --directory')
        directory=args.directory.resolve()
        TARGET=WASM=directory/'target'
        from review_assets import verify
        review_bundle=directory/'ios-assets'/args.review_variant
        verify(review_bundle,args.review_variant)
    elif args.directory is not None:parser.error('--directory requires --review-variant')
    LOGS.mkdir(parents=True,exist_ok=True)
    guard=LOGS/'no-install-bin';guard.mkdir(exist_ok=True)
    for name in ['brew','gem','rustup','pod']:
        real=shutil.which(name)
        if not real:raise RuntimeError('required installed command missing: '+name)
        script='''#!/usr/bin/env python3
import os,sys
from pathlib import Path
if any(arg in {'install','reinstall','upgrade','update','uninstall','add','remove','default','override','self'} for arg in sys.argv[1:]):
 Path(%r).write_text(%r+' '+repr(sys.argv[1:]))
 sys.exit(97)
os.execv(%r,[%r]+sys.argv[1:])
'''%(str(LOGS/'INSTALL-ATTEMPT.txt'),name,real,real)
        (guard/name).write_text(script);(guard/name).chmod(0o755)
    # A simulator build must not inspect/select signing credentials or provisioning.
    (guard/'security').write_text('#!/bin/sh\nexit 1\n');(guard/'security').chmod(0o755)
    project=NATIVE/'gen/apple'
    wrapper='''#!/usr/bin/env python3
import os,sys
args=[arg for arg in sys.argv[1:] if arg!='-allowProvisioningUpdates']
if '-scheme' in args:
 if '-derivedDataPath' not in args:args+=['-derivedDataPath',%r]
 args+=['CODE_SIGNING_ALLOWED=NO','CODE_SIGNING_REQUIRED=NO','CODE_SIGN_IDENTITY=','DEVELOPMENT_TEAM=',%r]
os.execv('/usr/bin/xcodebuild',['/usr/bin/xcodebuild']+args)
'''%(str(TARGET/'ios-derived-data' if args.review_variant else project/'build/DerivedData'),'CLANG_MODULE_CACHE_PATH='+str(TARGET/'clang-module-cache'))
    (guard/'xcodebuild').write_text(wrapper);(guard/'xcodebuild').chmod(0o755)
    env=dict(os.environ,PATH=str(guard)+os.pathsep+os.environ['PATH'],CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0',CARGO_TARGET_DIR=str(TARGET),PROFILE_UI_TARGET=str(WASM),CLANG_MODULE_CACHE_PATH=str(TARGET/'clang-module-cache'),SWIFT_MODULE_CACHE_PATH=str(TARGET/'swift-module-cache'))
    for key in ['GEM_HOME','GEM_PATH','APPLE_DEVELOPMENT_TEAM']:env.pop(key,None)
    # A fresh checkout can regenerate its ignored platform project, but must never
    # install a Rust target or host tool. All dependency commands remain guarded.
    if not (project/'project.yml').is_file():
        run('ios-init-help',['cargo','tauri','ios','init','--help'],env,cwd=NATIVE)
        if '--skip-targets-install' not in (LOGS/'ios-init-help.log').read_text():
            raise RuntimeError('installed CLI lacks the required no-target-installs flag')
        run('ios-init',['cargo','tauri','ios','init','--ci','--skip-targets-install'],env,cwd=NATIVE)
    if not args.review_variant:
        run('ios-csr-build',['python3','examples/profile-ui/build-client.py','csr'],dict(env,CARGO_TARGET_DIR=str(WASM)))
        from review_assets import ordinary
        ordinary(ROOT/'examples/profile-ui/dist/csr',NATIVE/'gen/csr-bundle')
    # The mobile configuration contains bundled paths only, and no dev URL.
    config=json.loads((NATIVE/'tauri.ios.conf.json').read_text())
    assert config['build'].get('devUrl') is None
    options=project/'.tauri/cli-options-server.json'
    if options.exists():raise RuntimeError('stale options-service descriptor exists; inspect before proceeding')
    command=build_command(args.review_variant,review_bundle if args.review_variant else None)
    observed=set();start=time.monotonic();next_measure=start
    with (LOGS/'ios-build.log').open('w') as log:
        log.write('$ '+' '.join(command)+'\n');log.flush()
        with subprocess.Popen(command,cwd=NATIVE,env=env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True) as child:
            try:
                while child.poll() is None:
                    if (LOGS/'INSTALL-ATTEMPT.txt').exists():raise RuntimeError('tooling attempted an install; stopped')
                    if time.monotonic()-start>1200:raise TimeoutError('iOS build timed out')
                    if time.monotonic()>=next_measure:
                        budget(min(start+1200,time.monotonic()+30));next_measure=time.monotonic()+15
                    if options.exists():
                        try:observed.add(json.loads(options.read_text())['addr'])
                        except (OSError,ValueError,KeyError):pass
                    time.sleep(.1)
                code=child.returncode
            finally:
                if child.poll() is None:
                    try:os.killpg(child.pid,signal.SIGKILL)
                    except ProcessLookupError:pass
                child.wait()
    # The command and its options service must be gone before any proof can launch.
    for address in observed:
        host,port=address.rsplit(':',1)
        try:
            with socket.create_connection((host,int(port)),timeout=1):raise RuntimeError('build options service remains active')
        except ConnectionRefusedError:pass
    report={'exit_code':code,'build_options_service_ran':bool(observed),'observed_addresses':sorted(observed),'closed_before_proof':not options.exists(),'seconds':round(time.monotonic()-start,2),'unsigned':True,'installer_attempted':(LOGS/'INSTALL-ATTEMPT.txt').exists()}
    (LOGS/'ios-build.json').write_text(json.dumps(report,indent=2)+'\n')
    if code:raise RuntimeError('iOS build failed; see ios-build.log')
    assert observed and not options.exists()
    if args.review_variant:
        # Retain the actual built app outside the sole disposable target. The next
        # variant's build must not silently replace the app selected by a launcher.
        archive=project/'build/stackpress-example-profile-ui-native_iOS.xcarchive/Products/Applications'
        apps=list(archive.glob('*.app'))
        if len(apps)!=1:raise RuntimeError('Expected exactly one built simulator app')
        destination=directory/'ios-apps'/args.review_variant/apps[0].name
        if destination.is_symlink():raise RuntimeError('Refusing symlink app destination')
        if destination.exists():shutil.rmtree(destination)
        destination.parent.mkdir(parents=True,exist_ok=True)
        shutil.copytree(apps[0],destination,symlinks=True)
        report.update(review_variant=args.review_variant,bundle=str(destination),frontend=str(review_bundle))
    (LOGS/'ios-build.json').write_text(json.dumps(report,indent=2)+'\n')
    print('PASS ios-build; options service exited',flush=True)


if __name__=='__main__':main()
