#!/usr/bin/env python3
"""Run the identical compatibility consumer against old or migrated production code."""
import argparse
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent


def main():
    """Pin exact stable output; treat legacy System normalization as the approved change."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('phase', choices=['before', 'after'])
    parser.add_argument('work', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--capture', action='store_true')
    args = parser.parse_args()
    args.work.mkdir(parents=True, exist_ok=True)
    args.output.mkdir(parents=True, exist_ok=True)
    source = (HERE / 'golden.rs').read_text()
    native = ROOT / 'examples/profile-ui/native/src/preferences.rs'
    source += '\n#[path = ' + json.dumps(str(native)) + ']\nmod native_preferences;\n'
    (args.work / 'main.rs').write_text(source)
    old = f'stackpress-framework-preferences={{path="{ROOT}/crates/framework/framework-preferences",features=["native-file"]}}' if args.phase == 'before' else ''
    view_features = ',features=["preferences","flash","native-file"]' if args.phase == 'after' else ''
    manifest = f'''[package]
name="issue15-golden"
version="0.0.0"
edition="2024"
[workspace]
[features]
after=[]
[[bin]]
name="issue15-golden"
path="main.rs"
[dependencies]
serde_json="1"
http="1"
tokio={{version="1",features=["macros","rt-multi-thread"]}}
stackpress-lib={{path="{ROOT}/crates/lib"}}
stackpress-server={{path="{ROOT}/crates/server/server",features=["http"]}}
stackpress-framework-language={{path="{ROOT}/crates/framework/framework-language"}}
stackpress-framework-view={{path="{ROOT}/crates/framework/framework-view"{view_features}}}
stackpress-example-ui-preferences-web={{path="{ROOT}/examples/ui-preferences/web"}}
{old}
'''
    (args.work / 'Cargo.toml').write_text(manifest)
    if not (args.work / 'Cargo.lock').exists():
        (args.work / 'Cargo.lock').write_bytes((ROOT / 'Cargo.lock').read_bytes())
    # Update only affected local-package edges, retaining every locked external version.
    subprocess.run(['cargo', 'metadata', '--offline', '--format-version', '1',
                    '--manifest-path', str(args.work / 'Cargo.toml')], stdout=subprocess.DEVNULL, check=True)
    command = ['cargo', 'run', '--locked', '--offline', '--manifest-path', str(args.work / 'Cargo.toml')]
    if args.phase == 'after':
        command += ['--features', 'after']
    # Keep Cargo's timing stream visible while capturing only the public JSON result.
    result = subprocess.run(command, env=dict(os.environ, ISSUE15_GOLDEN_DIRECTORY=str(args.output)), stdout=subprocess.PIPE, text=True, check=True)
    report = json.loads(result.stdout)
    (args.output / (args.phase + '.json')).write_text(result.stdout)
    golden = HERE / 'golden-v1.json'
    if args.capture:
        assert args.phase == 'before' and not golden.exists()
        golden.write_text(json.dumps(report['stable'], indent=2) + '\n')
    assert report['stable'] == json.loads(golden.read_text()), 'cookie/file/HTTP/Tauri golden changed'
    theme = 'system' if args.phase == 'before' else None
    assert report['legacy'] == {'cookie':{'locale':'fr','theme':theme}, 'file_reply':{'revision':1,'preferences':{'locale':'fr','theme':theme},'recovered':False},'file_unchanged':True}
    print(json.dumps({'phase':args.phase,'golden':'exact match','legacy_theme':theme,'legacy_french_retained':True,'legacy_file_bytes_unchanged':True}), flush=True)


if __name__ == '__main__':
    main()
