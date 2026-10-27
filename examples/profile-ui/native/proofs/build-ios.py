#!/usr/bin/env python3
"""Finite unsigned simulator build. G32 permits only its temporary options service."""
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


def run(name,command,env,cwd=ROOT,timeout=900):
    """Own the whole child group, kill/reap on timeout, and retain a finite log."""
    with (LOGS/(name+'.log')).open('w') as log:
        log.write('$ '+' '.join(map(str,command))+'\n');log.flush()
        with subprocess.Popen(list(map(str,command)),cwd=cwd,env=env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True) as child:
            try:
                code=child.wait(timeout=timeout)
            finally:
                if child.poll() is None:os.killpg(child.pid,signal.SIGKILL)
                child.wait()
        if code:raise RuntimeError(name+' failed; see '+str(LOGS/(name+'.log')))
    print('PASS '+name,flush=True)


def main():
    """Use installed tooling only; never start a dev command or packaged devUrl."""
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
'''%(str(project/'build/DerivedData'),'CLANG_MODULE_CACHE_PATH='+str(TARGET/'clang-module-cache'))
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
    run('ios-csr-build',['python3','examples/profile-ui/build-client.py','csr'],dict(env,CARGO_TARGET_DIR=str(WASM)))
    bundle=NATIVE/'gen/csr-bundle'
    if bundle.exists():shutil.rmtree(bundle)
    shutil.copytree(ROOT/'examples/profile-ui/dist/csr',bundle)
    shutil.copyfile(NATIVE/'native-bootstrap.js',bundle/'start.js')
    # The mobile configuration contains bundled paths only, and no dev URL.
    config=json.loads((NATIVE/'tauri.ios.conf.json').read_text())
    assert config['build'].get('devUrl') is None
    options=project/'.tauri/cli-options-server.json'
    if options.exists():raise RuntimeError('stale options-service descriptor exists; inspect before proceeding')
    command=['cargo','tauri','ios','build','--ci','--debug','--target','aarch64-sim','--features','app','--no-sign','--archive-only','--','--locked']
    observed=set();start=time.monotonic()
    with (LOGS/'ios-build.log').open('w') as log:
        log.write('$ '+' '.join(command)+'\n');log.flush()
        with subprocess.Popen(command,cwd=NATIVE,env=env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True) as child:
            try:
                while child.poll() is None:
                    if (LOGS/'INSTALL-ATTEMPT.txt').exists():raise RuntimeError('tooling attempted an install; stopped')
                    if time.monotonic()-start>1200:raise TimeoutError('iOS build timed out')
                    if options.exists():
                        try:observed.add(json.loads(options.read_text())['addr'])
                        except (OSError,ValueError,KeyError):pass
                    time.sleep(.1)
                code=child.returncode
            finally:
                if child.poll() is None:os.killpg(child.pid,signal.SIGKILL)
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
    print('PASS ios-build; options service exited',flush=True)


if __name__=='__main__':main()
