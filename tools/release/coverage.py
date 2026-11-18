#!/usr/bin/env python3
"""Instrument locked Cargo tests directly and report first-party source by release crate."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile
from common import ROOT, COMPATIBLE, candidates

def run(command, env):
    """Preserve commands under the enclosing benchmark; every child has a finite budget."""
    print('COMMAND:', *map(str,command if len(command)<40 else [*command[:3],f'<{len(command)-5} profile files>',*command[-2:]]), flush=True)
    subprocess.run(command,cwd=ROOT,env=env,check=True,timeout=1800)

def mysql_generated_ddl(scope, features, env):
    """Include the existing opt-in DDL group that ordinary nextest discovery cannot select."""
    command=['cargo','test','--locked','--offline',*scope,'--features',features,
             '--test','m2','--no-run','--message-format=json']
    print('COMMAND:',*command,flush=True)
    output=subprocess.check_output(command,cwd=ROOT,env=env,text=True,timeout=1800)
    source=ROOT/'crates/sql/sql-mysql/src/m2.rs'
    executables=[]
    for line in output.splitlines():
        artifact=json.loads(line)
        if (artifact.get('reason')=='compiler-artifact' and artifact.get('executable')
                and Path(artifact['target']['src_path']).resolve()==source):
            executables.append(artifact['executable'])
    assert len(executables)==1, 'expected exactly one MySQL M2 proof executable'
    run([executables[0],'--g37-ddl'],env)

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--selected',action='store_true',help='edit-loop measurement of the three changed crates')
    args=parser.parse_args()
    target=Path(os.environ['CARGO_TARGET_DIR']).parent/'coverage'
    target.mkdir(parents=True,exist_ok=True)
    profiles=Path(tempfile.mkdtemp(prefix='run-',dir=target))
    # LLVM locks and merges each binary's counters into a four-file pool. A PID
    # suffix would retain a separate multi-megabyte profile for every nextest case.
    env=dict(os.environ,CARGO_TARGET_DIR=str(target),RUSTFLAGS='-C instrument-coverage',LLVM_PROFILE_FILE=str(profiles/'run-%4m.profraw'))
    scope=['-p','stackpress-server','-p','stackpress-server-http','-p','stackpress-leptos'] if args.selected else ['--workspace']
    features='stackpress-server/http-sse,stackpress-server/http-upgrade,stackpress-server-http/sse,stackpress-server-http/websocket,stackpress-server-http/tls,stackpress-leptos/ssr,stackpress-leptos/client-sse,stackpress-leptos/client-websocket' if args.selected else COMPATIBLE
    run(['cargo','nextest','run','--locked','--offline',*scope,'--all-targets','--features',features,'--test-threads','2','--no-fail-fast'],env)
    # Ignored live suites are enabled by their explicit URL environment, through their watchdogs.
    # Realtime Chromium also executes instrumented native code in its finite child process.
    if os.environ.get('BLOG_PLAYWRIGHT'):
        env['REALTIME_ARTIFACTS']=str(target.parent/'browser-artifacts')
        # Keep the selected feature unification identical to the instrumented ordinary run.
        run(['cargo','test','--locked','--offline',*scope,'--features',features,'--test','realtime_browser','browser_watchdog','--','--ignored','--exact','--nocapture'],env)
    if not args.selected and os.environ.get('STACKPRESS_TEST_MYSQL_URL'):
        mysql_generated_ddl(scope,features,env)
    export_coverage(target,profiles,args.selected,env)

def export_coverage(target, profiles, selected, env):
    """Merge the completed proof profiles and retain ordinary per-candidate source summaries."""
    sysroot=Path(subprocess.check_output(['rustc','--print','sysroot'],text=True).strip())
    host=next(line.split(': ',1)[1] for line in subprocess.check_output(['rustc','-vV'],text=True).splitlines() if line.startswith('host:'))
    llvm=sysroot/'lib/rustlib'/host/'bin'
    merged=profiles/'coverage.profdata'
    run([str(llvm/'llvm-profdata'),'merge','-sparse',*map(str,profiles.glob('*.profraw')),'-o',str(merged)],env)
    # Include test, application and example executables. Their own files are filtered below,
    # but first-party library code executed through those callers must contribute as well.
    objects=[]
    for directory in [target/'debug',target/'debug/deps',target/'debug/examples']:
        if directory.exists():
            objects.extend(p for p in directory.iterdir()
                           if p.is_file() and p.suffix=='' and os.access(p,os.X_OK))
    # Per-crate accounting uses file summaries; omit unused region/function payloads.
    command=[str(llvm/'llvm-cov'),'export','-format=text','--summary-only',f'-instr-profile={merged}']
    for file in objects:command+=['-object',str(file)]
    raw=subprocess.check_output(command,text=True,timeout=120)
    (profiles/'raw.json').write_text(raw)
    source_files=json.loads(raw)['data'][0]['files']
    report={}
    for directory,name in candidates():
        if name=='stackpress' or selected and name not in ['stackpress-server','stackpress-server-http','stackpress-leptos']:continue
        files=[]
        for file in source_files:
            path=Path(file['filename']).resolve()
            if not path.is_relative_to(directory/'src'):continue
            # Match prior release receipts: ordinary LLVM src/ rows, including in-source tests.
            # Separate test/example/dependency files are outside this per-candidate source scope.
            files.append(file)
        covered=sum(f['summary']['lines']['covered'] for f in files)
        total=sum(f['summary']['lines']['count'] for f in files)
        report[name]={'covered':covered,'total':total,'percent':covered*100/total if total else 0,'files':[{ 'path':f['filename'],'lines':f['summary']['lines']} for f in files]}
        print(f'{name}: {covered}/{total} = {report[name]["percent"]:.2f}%',flush=True)
    (profiles/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    (target/'latest.txt').write_text(str(profiles)+'\n')
    print('COVERAGE REPORT:',profiles,flush=True)
    if not selected:assert all(r['percent']>=90 for r in report.values()),'release crate below 90%; inspect per-file report'
if __name__=='__main__':main()
