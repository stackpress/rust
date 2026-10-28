#!/usr/bin/env python3
"""One-off historical proof for issue #15 M2: run M1's early-draft assertion against unchanged pre-M1 (3ff7591) sources.

Usage: CARGO_TARGET_DIR=<task target> PROFILE_UI_PLAYWRIGHT=<playwright> python3 <this file> <fresh work dir>
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tarfile
import sys

ROOT = Path(__file__).resolve().parents[4]
REVISION = '3ff7591'


def main():
    """A saved-Dark case isolates draft adoption from the newer unset-theme protocol."""
    work = Path(sys.argv[1]).resolve()
    baseline = work / 'baseline'
    work.mkdir(parents=True, exist_ok=True)
    assert not baseline.exists(), 'use a fresh task-owned destination'
    baseline.mkdir()
    archive = work / 'before.tar'
    # Extract the exact committed tree; no production file is patched for this proof.
    with archive.open('wb') as output:
        subprocess.run(['git','archive',REVISION],cwd=ROOT,stdout=output,check=True)
    with tarfile.open(archive) as source:
        source.extractall(baseline,filter='data')
    archive.unlink()
    env = dict(os.environ, PROFILE_UI_TARGET=os.environ['CARGO_TARGET_DIR'])
    subprocess.run(['cargo','build','--locked','--offline','-p','stackpress-example-profile-ui-web'],cwd=baseline,env=env,check=True)
    subprocess.run([sys.executable,'examples/profile-ui/build-client.py','hydrate'],cwd=baseline,env=env,check=True)
    original = subprocess.check_output(['git','show','0d0d868:examples/profile-ui/proofs/browser-initial-theme.cjs'],cwd=ROOT,text=True)
    script = original.replace("for(const base of ['','/demo/v1'])",'for(const base of [process.env.BASELINE_MOUNT])')
    script = script.replace("for(const display of ['fr-unset','fr-dark'])","for(const display of ['fr-dark'])")
    needle = "        assert.equal(await page.locator('input').inputValue(),'EARLY-profile-draft');"
    assert needle in script
    # Keep the original failing assertion. Instrument its immediate observable input only.
    script = script.replace(needle,"        console.log('BASELINE_DRAFT '+JSON.stringify({before:'EARLY-profile-draft',after:await page.locator('input').inputValue(),sameNode:await page.evaluate(()=>inputBefore===document.querySelector('input'))}));\n        await page.screenshot({path:path.join(output,'baseline-draft-'+(base?'mounted':'root')+'.png'),fullPage:true});\n"+needle)
    proof = baseline/'examples/profile-ui/proofs/browser-initial-theme.cjs'
    proof.write_text(script)
    evidence = work/'evidence'
    evidence.mkdir()
    cases=[]
    for base in ['', '/demo/v1']:
        result=subprocess.run(['node',str(proof)],cwd=baseline,env=dict(env,BASELINE_MOUNT=base,PROFILE_UI_IOS_LOGS=str(evidence)),text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=120)
        (evidence/('baseline-'+('mounted' if base else 'root')+'.log')).write_text(result.stdout)
        observations=[json.loads(line.removeprefix('BASELINE_DRAFT ')) for line in result.stdout.splitlines() if line.startswith('BASELINE_DRAFT ')]
        assert len(observations)==1 and result.returncode!=0, result.stdout
        observed=observations[0]
        assert observed=={'before':'EARLY-profile-draft','after':'abc','sameNode':True},observed
        assert 'EARLY-profile-draft' in result.stdout and 'AssertionError' in result.stdout,result.stdout
        cases.append({'base':base,'exit':result.returncode,**observed})
    # The sole extra/modified tracked-tree path is the copied/instrumented M1 proof.
    report={'revision':subprocess.check_output(['git','rev-parse',REVISION],cwd=ROOT,text=True).strip(),
            'm1_proof_sha256':hashlib.sha256(original.encode()).hexdigest(),
            'production_source_patches':0,'adaptation':'saved Dark only; one mount per process; immediate observation before original assertion',
            'predates_m1':True,'cases':cases,'listener':False}
    (evidence/'early-draft-before-m1.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report),flush=True)


if __name__=='__main__':
    main()
