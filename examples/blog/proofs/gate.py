#!/usr/bin/env python3
"""Bootstrap and verify a fresh checkout, retaining every phase and enabled regression."""
import json
import os
from pathlib import Path
import subprocess
import sys
import time

BLOG=Path(__file__).resolve().parents[1]
ROOT=BLOG.parents[1]
OUTPUT=BLOG/'.build/gate'
OUTPUT.mkdir(parents=True,exist_ok=True)
RESULTS=[]


def reserve_coverage(target):
    """Reserve the measured native instrumented graph before adding it to warm targets."""
    import shutil
    def size(directory):
        return int(subprocess.check_output(['du','-sk',str(directory)],text=True).split()[0])*1024 if directory.exists() else 0
    # The measured full test+binary graph occupies about5.55GiB. Count existing warm
    # coverage bytes once and reserve5.75GiB, rather than treating it as an edit-loop build.
    projected=lambda:size(target)-size(target/'coverage')+max(size(target/'coverage'),int(5.75*1024**3))
    for name in ['browser-entry','readme','csr','hydrate']:
        directory=target/name
        if projected()>12*1024**3 and directory.exists():
            # Every browser/consumer check has completed; emitted/frozen products live in
            # .build and remain available to the native HTTP browser coverage phase.
            assert directory.resolve().parent==target.resolve()
            shutil.rmtree(directory)
            print('Retired completed own compilation target:',directory,flush=True)
    assert projected()<=12*1024**3, f'coverage reserve requires more space: {projected()}'
    print('Coverage reserve projected bytes:',projected(),flush=True)


def run(name,command,env=None,seconds=600):
    """Record a finite phase, continuing independent checks after an expected/unexpected failure."""
    print(f'GATE {name}: {command}',flush=True)
    start=time.monotonic()
    log=OUTPUT/(name+'.log')
    with log.open('w')as handle:
        process=subprocess.Popen(command,cwd=ROOT,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,start_new_session=True)
        # The outer recorder bounds this gate; each child is reaped by its explicit deadline.
        import threading,signal
        timed_out=[]
        def expire():
            timed_out.append(True)
            try:os.killpg(process.pid,signal.SIGKILL)
            except ProcessLookupError:pass
        timer=threading.Timer(seconds,expire)
        timer.start()
        try:
            for line in process.stdout:print(line,end='',flush=True);handle.write(line)
            code=process.wait()
        finally:timer.cancel()
    # Sample only this effort's targets after each phase; never inspect another lane's cache.
    target=Path(os.environ['CARGO_TARGET_DIR']).parent
    footprint=int(subprocess.check_output(['du','-sk',str(target)],text=True).split()[0])*1024
    RESULTS.append({'name':name,'command':command,'code':code,'seconds':round(time.monotonic()-start,6),
                    'timeout':bool(timed_out),'target_bytes':footprint})
    (OUTPUT/'results.json').write_text(json.dumps(RESULTS,indent=2))
    assert footprint<12*1024**3, f'own target budget exceeded: {footprint}'
    return code


def main():
    """Verify authored/generated code, all native targets, separate renderers, browser and coverage."""
    # The documented target is dedicated to this gate; first-run diagnostics also need its
    # directory to exist before the initial size sample, without requiring a prior Cargo run.
    target=Path(os.environ['CARGO_TARGET_DIR']).parent
    target.mkdir(parents=True,exist_ok=True)
    manifest=['--manifest-path',str(BLOG/'Cargo.toml')]
    common=manifest+['--locked','--offline']
    features=['--features','blog-tool/native-proof,blog-shared/ssr']
    # Generation is an explicit prerequisite, not ignored state inherited from an edit loop.
    # The default tool is independent of its output; full and native partitions stay separate.
    generator=['cargo','run',*common,'-p','blog-tool','--']
    for action in ['generate-app','generate']:
        if run('bootstrap-'+action,[*generator,action]):return 1
    if run('bootstrap-extension',['python3',str(BLOG/'proofs/extension.py')]):return 1
    # Native HTTP tests read the actual debug WASM, so build it before compiling/running tests.
    if run('browser-build',['python3',str(BLOG/'proofs/build.py')],seconds=1200):return 1
    run('format',['python3',str(BLOG/'proofs/format.py')])
    run('javascript',['node','--check',str(BLOG/'tests/browser.cjs')])
    run('javascript-flows',['node','--check',str(BLOG/'tests/flows.cjs')])
    run('generated-freshness',['python3',str(BLOG/'proofs/freshness.py')])
    run('clippy-native',['cargo','clippy',*common,'--workspace','--all-targets',*features,'--','-D','warnings'])
    run('nextest-native',['cargo','nextest','run',*common,'--workspace','--all-targets',*features,'--test-threads','2','--no-fail-fast'])
    run('doctests',['cargo','test',*common,'--workspace','--doc',*features])
    run('rustdoc',['cargo','doc',*common,'--workspace','--no-deps',*features],env=dict(os.environ,RUSTDOCFLAGS='-D warnings'))
    run('graphs',['python3',str(BLOG/'proofs/graphs.py')])
    for mode in ['hydrate','csr']:
        target=str(Path(os.environ['CARGO_TARGET_DIR']).parent/mode)
        run('clippy-'+mode,['cargo','clippy',*common,'-p','blog-client','--target','wasm32-unknown-unknown','--no-default-features','--features',mode,'--','-D','warnings'],env=dict(os.environ,CARGO_TARGET_DIR=target))
    # Nextest's test executables are not the finite stdin/stdout adapter browser.py spawns.
    if run('browser-probe',['cargo','build',*common,'-p','blog-web','--bin','probe']):return 1
    run('browser-public-matrix',['python3',str(BLOG/'proofs/browser.py'),'--public-only','--matrix'])
    run('browser-full',['python3',str(BLOG/'proofs/browser.py')])
    # Prepare the copied-out schema/database/native/browser products in this same invocation.
    if run('nickname-prepare',['python3',str(BLOG/'proofs/nickname.py'),'prepare'],seconds=1800):return 1
    if run('nickname-freshness',['python3',str(BLOG/'proofs/nickname.py'),'check'])==0:
        run('nickname-browser',['python3',str(BLOG/'proofs/nickname.py'),'browser'])
    target=Path(os.environ['CARGO_TARGET_DIR']).parent
    reserve_coverage(target)
    try:
        from coverage import measure
        measure(run)
        browser_passed=any(item['name']=='browser-public-matrix' and item['code']==0
                           for item in RESULTS)
        RESULTS.append({'name':'coverage-blog-client-g56','code':0 if browser_passed else 1,
                        'status':'exempt','decision':'G56',
                        'scope':'startup/mount/disposal and thin DOM/navigation/fetch/AbortController glue',
                        'browser_evidence':'browser-public-matrix exercises both real entries; browser-full is a separate gate phase'})
    except Exception as error:
        print('COVERAGE GAP:',repr(error),flush=True)
        RESULTS.append({'name':'coverage-report','code':1,'error':repr(error)})
    run('lock-invariants',['python3',str(BLOG/'proofs/locks.py'),'check'])
    (OUTPUT/'results.json').write_text(json.dumps(RESULTS,indent=2))
    print('GATE RESULTS',json.dumps(RESULTS),flush=True)
    return int(any(item['code']for item in RESULTS))


if __name__=='__main__':raise SystemExit(main())
