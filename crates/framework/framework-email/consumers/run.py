#!/usr/bin/env python3
"""Prove standalone graphs and executable imports, using a distinct target for each consumer.

Run this whole script once through the repository build recorder. Cargo runs offline;
consumer lock snapshots are checked with --locked except during initial lock reconciliation.
"""
import argparse
import json
import shutil
import os
from pathlib import Path
import subprocess
import tomllib


def run(command, env, expected=None):
    """Keep exact commands in the outer recorder and propagate failures immediately."""
    print('COMMAND:', *map(str, command), flush=True)
    result=subprocess.run(command, env=env, check=True, text=True, stdout=subprocess.PIPE if expected else None)
    if expected:
        print(result.stdout,end='',flush=True)
        assert result.stdout.strip()==expected,result.stdout
    return result


def main():
    """Assert feature boundaries, then format/lint/run each independent application."""
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reconcile-locks', action='store_true')
    args=parser.parse_args()
    root=Path(__file__).resolve().parents[4]
    base=Path(__file__).resolve().parent
    target=Path(os.environ['CARGO_TARGET_DIR']).resolve()
    baseline=tomllib.loads((root/'Cargo.lock').read_text())['package']
    external={(p['name'],p['version'],p.get('source'),p.get('checksum')) for p in baseline if 'source' in p}
    for kind in ['direct','event','disabled']:
        env={**os.environ,'CARGO_TARGET_DIR':str(target/f'consumer-{kind}')}
        manifest=base/kind/'Cargo.toml'
        flags=['--manifest-path',str(manifest),'--offline']
        if not args.reconcile_locks: flags+=['--locked']
        graph=subprocess.check_output(['cargo','tree',*flags,'-e','normal','--prefix','none'],env=env,text=True)
        print(f'GRAPH {kind}:\n{graph}',flush=True)
        if kind=='direct':
            assert 'stackpress-server ' not in graph
            assert 'stackpress-framework-session ' not in graph
        elif kind=='event':
            assert 'stackpress-server ' in graph
            assert 'stackpress-framework-session ' not in graph
        else:
            assert 'stackpress-framework-csrf ' not in graph
            assert 'stackpress-framework-email ' not in graph
            assert 'stackpress-server ' not in graph
        if '--locked' not in flags: flags+=['--locked']
        packages=tomllib.loads((manifest.parent/'Cargo.lock').read_text())['package']
        assert {(p['name'],p['version'],p.get('source'),p.get('checksum')) for p in packages if 'source' in p} <= external
        run(['cargo','fmt','--manifest-path',str(manifest),'--check'],env)
        run(['cargo','clippy',*flags,'--','-D','warnings'],env)
        run(['cargo','run',*flags],env,f'slice9b {kind} consumer passed')
        # Consumers have identical package/binary names deliberately: directories isolate outputs.
        size=int(subprocess.check_output(['du','-sk',str(target)],text=True).split()[0])
        print(json.dumps({'consumer':kind,'combined_target_KiB':size}),flush=True)
        assert size < 20*1024*1024
        # Trim only idle incremental output in this disposable workspace when the combined effort nears its cap.
        # The dependency artifacts, main warm target and other consumers' binaries are retained.
        if size > 19.4*1024*1024:
            incremental=Path(env['CARGO_TARGET_DIR'])/'debug/incremental'
            if incremental.exists() and not incremental.is_symlink(): shutil.rmtree(incremental)
            print(f'Trimmed only idle incremental output: {incremental}',flush=True)
    return 0


if __name__=='__main__':
    raise SystemExit(main())
