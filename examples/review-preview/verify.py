#!/usr/bin/env python3
"""Re-run existing port-free proofs with one owned target and installed browser tooling."""
import argparse
import os
import shutil
import time
from pathlib import Path
import review


def main():
    """Existing proof drivers retain their own kill/reap and per-action watchdogs."""
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action',choices=['regressions','smoke'])
    parser.add_argument('--directory',type=Path,required=True)
    parser.add_argument('--resume',choices=['transfer','browsers','checks'])
    args=parser.parse_args();ctx=review.context(args.directory)
    root=ctx.logs.parent
    from proofs.runtime import TARGET
    common=dict(CARGO_TARGET_DIR=str(TARGET),PROFILE_UI_NATIVE_TARGET=str(TARGET),PROFILE_UI_TARGET=str(TARGET),
                GALLERY_OUTPUT_DIR=str(root/'gallery'),PREFERENCES_LOGS=str(root/'preferences'),UI_STREAM_OUTPUT_DIR=str(root/'stream'),
                UI_FORMAT_OUTPUT_DIR=str(root/'format'),ADMIN_PROOF_ROOT=str(root),
                GALLERY_PLAYWRIGHT=review.PLAYWRIGHT,PREFERENCES_PLAYWRIGHT=review.PLAYWRIGHT,UI_STREAM_PLAYWRIGHT=review.PLAYWRIGHT,
                UI_FORMAT_PLAYWRIGHT=review.PLAYWRIGHT,ADMIN_PLAYWRIGHT=review.PLAYWRIGHT,REVIEW_PLAYWRIGHT=review.PLAYWRIGHT,
                REVIEW_DIRECTORY=str(root))
    if args.action=='smoke':
        for name in ['review-browser.log','review-browser-result.json']:
            previous=ctx.logs/name
            if previous.exists():shutil.copy2(previous,ctx.logs/(name+'.attempt-'+str(time.time_ns())))
        ctx.run('review-browser',['node','examples/review-preview/browser.cjs'],extra=common,timeout=120)
        return
    if not args.resume:
        for name,script in [('gallery','examples/ui-gallery/browser.py'),('gallery-breadth','examples/ui-gallery/breadth-browser.py'),
                            ('preferences','examples/ui-preferences/browser.py'),('stream','examples/ui-stream/browser.py'),
                            ('format','examples/ui-format/browser.py'),('format-edit','examples/ui-format/edit-browser.py')]:
            ctx.run('existing-'+name,['python3',script],extra=common,timeout=180)
    for suite in (['transfer'] if args.resume=='transfer' else [] if args.resume else ['roles','crud','plugin','transfer']):
        ctx.run('existing-admin-build-'+suite,['python3','examples/generated-admin/proof.py','build','--suite',suite],extra=common,timeout=1800)
    for suite in ([] if args.resume=='checks' else ['roles','crud','format','transfer','races']):
        ctx.run('existing-admin-browser-'+suite,['python3','examples/generated-admin/proof.py','browser','--suite',suite],extra=common,timeout=1800)
    # Every browser/native artifact was copied outside target; free compilation caches
    # before broad workspace checks instead of approaching the eight-GiB ceiling.
    if args.resume!='checks':ctx.run('release-browser-build-cache',['cargo','clean'],extra=common)
    for action in (['checks','graphs'] if args.resume=='checks' else ['compile','checks','graphs']):
        ctx.run('existing-admin-'+action,['python3','examples/generated-admin/proof.py',action],extra=common,timeout=2400)


if __name__=='__main__':main()
