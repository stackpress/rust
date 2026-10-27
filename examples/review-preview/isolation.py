#!/usr/bin/env python3
"""Prove explicit release refusal and absence from ordinary example dependency/artifact graphs."""
import argparse
import json
import hashlib
import shutil
from pathlib import Path
import review


def main():
    """Use the same capped target; no release command can publish or start the host."""
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--directory',type=Path,required=True)
    args=parser.parse_args();ctx=review.context(args.directory)
    from proofs.runtime import TARGET
    package=review.PACKAGE
    release=dict(CARGO_PROFILE_RELEASE_OPT_LEVEL='0',CARGO_PROFILE_RELEASE_DEBUG='0')
    try:
        ctx.run('release-review-refusal',['cargo','build','--offline','--locked','--release','-p',package,'--features','review-preview'],extra=release)
    except SystemExit as error:
        log=(ctx.logs/'release-review-refusal.log').read_text()
        assert error.code and 'STACKPRESS_REVIEW_PREVIEW requires a debug build; release review hosts are forbidden' in log,log[-3000:]
    else:raise AssertionError('Release review host unexpectedly compiled')
    graphs=[]
    for selected in ['stackpress','stackpress-example-ui-gallery-web','stackpress-example-ui-preferences-web',
                     'stackpress-example-ui-stream-web','stackpress-example-ui-format','generated-admin-web',
                     'stackpress-example-profile-ui-web','stackpress-example-profile-ui-client','stackpress-example-profile-ui-native']:
        name='normal-graph-'+selected
        ctx.run(name,['cargo','tree','--offline','--locked','-p',selected,'--edges','normal,no-proc-macro'])
        graph=(ctx.logs/(name+'.log')).read_text();assert package not in graph
        graphs.append(selected)
    ctx.run('empty-normal-review-graph',['cargo','tree','--offline','--locked','-p',package,'--no-default-features','--edges','normal'])
    graph=(ctx.logs/'empty-normal-review-graph.log').read_text().splitlines()[1:]
    assert len(graph)==1 and graph[0].startswith(package),graph
    normal=['stackpress-example-ui-gallery-web','stackpress-example-ui-preferences-web','stackpress-example-ui-stream-web','generated-admin-web']
    ctx.run('normal-examples',['cargo','build','--offline','--locked',*[value for selected in normal for value in ['-p',selected]]])
    ctx.run('normal-format',['cargo','build','--offline','--locked','-p','stackpress-example-ui-format','--features','ssr,rich,format'])
    ctx.run('normal-release-gallery',['cargo','build','--offline','--locked','--release','-p','stackpress-example-ui-gallery-web'],extra=release)
    artifacts=[TARGET/'debug'/selected for selected in normal]+[TARGET/'debug/ui-format',TARGET/'release/stackpress-example-ui-gallery-web']
    root=ctx.logs.parent
    for folder in [root/'gallery/gallery-assets',root/'preferences/assets',root/'stream/assets',root/'roles/assets',root/'format/assets']:
        artifacts.extend(path for path in folder.rglob('*') if path.suffix in ['.js','.wasm'])
    markers=[b'STACKPRESS_REVIEW_PREVIEW',b'/review/action/',b'stackpress_example_review_preview',
             b'stackpress-example-review-preview',b'slice8-review-role',b'Review fixtures',b'review-pause']
    retained=ctx.logs/'isolation-artifacts';retained.mkdir(exist_ok=True)
    records=[]
    for artifact in artifacts:
        data=artifact.read_bytes()
        assert not any(marker in data for marker in markers),str(artifact)
        if artifact.is_relative_to(TARGET):
            destination=retained/(artifact.parent.name+'-'+artifact.name)
            shutil.copy2(artifact,destination);artifact=destination
        records.append(dict(path=str(artifact),bytes=len(data),sha256=hashlib.sha256(data).hexdigest()))
    report=dict(expected_release_refusal=True,normal_graphs=graphs,normal_default_review_graph_empty=True,
                marker_free_artifacts=records,release_profile='release, opt-level=0, debug assertions disabled by default')
    (ctx.logs/'isolation.json').write_text(json.dumps(report,indent=2)+'\n')
    print('PASS release isolation; '+str(len(artifacts))+' marker-free artifacts',flush=True)


if __name__=='__main__':main()
