#!/usr/bin/env python3
"""Finite compiler and strings checks for debug-only native proof isolation."""
import argparse
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[4]
NATIVE = ROOT / 'examples/profile-ui/native'
MARKERS = ('PROFILE_UI_PROOF', 'PROFILE_UI_PROOF_MODE', 'PROFILE_UI_MOBILE_PROOF',
           'window.profileProofWindow', 'window.profileProofMode', 'window.profileHangEntered',
           'window.mobileProofMode', 'mobile-proof-mode', 'mobile-proof-control', 'PRIVATE_PROOF_CAUSE')
DESKTOP = ('PROFILE_UI_PROOF', 'PROFILE_UI_PROOF_MODE', 'window.profileProofWindow',
           'window.profileProofMode', 'window.profileHangEntered', 'PRIVATE_PROOF_CAUSE')


def compile_gates():
    """Type-check the real proof files without dependencies: release removes both."""
    target = Path(os.environ['CARGO_TARGET_DIR']) / 'proof-isolation'
    target.mkdir(parents=True, exist_ok=True)
    fixture = target / 'release_gates.rs'
    # Each real module has an inner compile-time guard. Removing either guard
    # makes this dependency-free fixture fail on its Tauri/application imports.
    fixture.write_text('\n'.join([
        '#[path = ' + json.dumps(str(NATIVE / 'src/proof.rs'), ensure_ascii=False) + '] mod desktop;',
        '#[path = ' + json.dumps(str(NATIVE / 'src/mobile_proof.rs'), ensure_ascii=False) + '] mod mobile;',
        'pub fn release_boundary() {}',
    ]) + '\n')
    triples = ('aarch64-apple-darwin', 'aarch64-apple-ios-sim', 'aarch64-linux-android')
    for triple in triples:
        subprocess.run(['rustc', '--edition=2024', '--crate-type=lib',
                        '--emit=metadata', '--cfg', 'feature="app"', '-C', 'debug-assertions=no', '--target', triple,
                        str(fixture), '-o', str(target / (triple + '.rmeta'))],
                       check=True, timeout=30)
    return dict(release_module_exclusion=list(triples), compiler=subprocess.check_output(['rustc', '--version'], text=True).strip(), mobile_runtime_launched=False)


def artifact_strings(debug, release):
    """Inspect actual linked artifacts; a debug-positive check guards the detector."""
    reports = {}
    for kind, binary in (('debug', debug), ('release', release)):
        raw = subprocess.check_output(['strings', '-a', str(binary)], timeout=30)
        reports[kind] = {marker: marker.encode() in raw for marker in MARKERS}
    # Mobile hooks are target-gated and are not expected in the desktop debug
    # artifact. Their source exclusion is checked separately on all three targets.
    assert all(reports['debug'][marker] for marker in DESKTOP), reports['debug']
    assert not any(reports['release'].values()), reports['release']
    return dict(binary_strings=reports, debug=str(debug), release=str(release))


def main():
    """Select an explicit check; missing artifacts or compiler failures are failures."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=('compile', 'strings'))
    parser.add_argument('--debug', type=Path)
    parser.add_argument('--release', type=Path)
    args = parser.parse_args()
    if args.mode == 'compile':
        report = compile_gates()
    else:
        if args.debug is None or args.release is None:
            parser.error('strings requires both --debug and --release')
        report = artifact_strings(args.debug, args.release)
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
