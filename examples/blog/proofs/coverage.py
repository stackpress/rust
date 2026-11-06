#!/usr/bin/env python3
"""Measure native first-party source and executed generated families without hiding examples."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

BLOG=Path(__file__).resolve().parents[1]
ROOT=BLOG.parents[1]


def measure(run):
    """Execute every regression, export fresh counters, and report each native crate."""
    target=Path(os.environ['CARGO_TARGET_DIR']).parent/'coverage'
    target.mkdir(parents=True,exist_ok=True)
    # Use rustc's source-based instrumentation directly: Cargo calls remain explicitly
    # locked/offline, without a helper's implicit metadata or automatic examples exclusion.
    env=dict(os.environ,CARGO_TARGET_DIR=str(target))
    env['RUSTFLAGS']=(env.get('RUSTFLAGS','')+' -C instrument-coverage').strip()
    # Historical profiles remain evidence, but cannot contribute to this gate's counts.
    # Unique run paths also prevent a rerun from merging counters from older binaries.
    profiles_dir=Path(tempfile.mkdtemp(prefix='run-',dir=target))
    env['LLVM_PROFILE_FILE']=str(profiles_dir/'blog-%m-%p.profraw')
    common=['--manifest-path',str(BLOG/'Cargo.toml'),'--locked','--offline']
    features=['--features','blog-tool/native-proof,blog-shared/ssr']
    # Catalog redirects, native Date output and cookie scope remain enabled unchanged.
    run('coverage-nextest',['cargo','nextest','run',*common,'--workspace','--all-targets',*features,'--test-threads','2','--no-fail-fast'],env=env,seconds=900)
    return finish(run,target,profiles_dir)


def finish(run,target,profiles_dir):
    """Finish native browser coverage/export from this run's already completed test profiles."""
    env=dict(os.environ,CARGO_TARGET_DIR=str(target))
    env['RUSTFLAGS']=(env.get('RUSTFLAGS','')+' -C instrument-coverage').strip()
    env['LLVM_PROFILE_FILE']=str(profiles_dir/'blog-%m-%p.profraw')
    common=['--manifest-path',str(BLOG/'Cargo.toml'),'--locked','--offline']
    if run('coverage-binaries',['cargo','build',*common,'-p','blog-web','--bins'],env=env,seconds=900)==0:
        run('coverage-native-http-browser',['python3',str(BLOG/'proofs/browser.py'),'--mode','hydrate'],env=env,seconds=180)
    profiles=sorted(profiles_dir.glob('blog-*.profraw'))
    assert profiles,'native coverage produced no profiles'
    sysroot=Path(subprocess.check_output(['rustc','--print','sysroot'],text=True).strip())
    host=next(line.split(': ',1)[1] for line in subprocess.check_output(['rustc','-vV'],text=True).splitlines() if line.startswith('host:'))
    llvm=sysroot/'lib/rustlib'/host/'bin'
    merged=profiles_dir/'blog.profdata'
    subprocess.run([str(llvm/'llvm-profdata'),'merge','-sparse',*map(str,profiles),'-o',str(merged)],check=True)
    objects=[file for file in (target/'debug/deps').iterdir() if file.is_file() and file.suffix=='' and os.access(file,os.X_OK) and file.name.startswith(('main-','blog_web-','blog_tool-','blog_shared-','blog_client-','probe-','setup-','serve-'))]
    objects.extend(file for name in ('blog-web','blog-tool','setup','probe','serve') if (file:=target/'debug'/name).exists())
    command=[str(llvm/'llvm-cov'),'export','-format=text',f'-instr-profile={merged}']
    for file in objects:command+=['-object',str(file)]
    raw=subprocess.check_output(command,text=True,timeout=120)
    output=BLOG/'.build/gate'
    (output/'coverage-raw.json').write_text(raw)
    groups={name:[] for name in ['blog-tool','blog-web','blog-shared']}
    for file in json.loads(raw)['data'][0]['files']:
        location=Path(file['filename']).resolve()
        group=next((name for name,folder in [('blog-tool','tool'),('blog-web','web'),('blog-shared','shared')] if location.is_relative_to(BLOG/folder/'src')),None)
        generated=BLOG/'.build/app-generated'
        if location.is_relative_to(generated):
            group='blog-shared' if location.relative_to(generated).parts[0]=='ui' else 'blog-web'
        # Dedicated tests, disposable extension/bootstrap fixtures and dependencies are not
        # shipped app source. Actual app-generated UI/source/stores/client remain counted.
        if group:groups[group].append(file)
    report={}
    for name,files in groups.items():
        covered=sum(file['summary']['lines']['covered'] for file in files)
        total=sum(file['summary']['lines']['count'] for file in files)
        report[name]={'covered':covered,'executable':total,'percent':covered*100/total if total else None,
                      'files':[{ 'file':file['filename'],'covered':file['summary']['lines']['covered'],'executable':file['summary']['lines']['count']}for file in files]}
        print(f'{name}: {covered}/{total} native executable lines = {report[name]["percent"]}',flush=True)
    report['blog-client']={
        'status':'exempt', 'decision':'G56',
        'scope':'startup/mount/disposal and thin DOM/navigation/fetch/AbortController glue only',
        'evidence':'tests/browser.cjs public matrix exercises actual hydration/CSR entries; gate requires its passing phase',
        'decisions':'shared/src/{form,browser,root}.rs are target-independent and included in native coverage',
    }
    (output/'coverage.json').write_text(json.dumps(report,indent=2))
    assert all(value['percent'] is not None and value['percent']>=70 for name,value in report.items() if name!='blog-client'),'native crate below70%; see per-crate report'
    return report
