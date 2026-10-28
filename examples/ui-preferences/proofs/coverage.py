#!/usr/bin/env python3
"""Measure each changed framework crate's native production executable lines."""
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]


def main():
    """Keep per-crate floors visible; WASM/installed-platform evidence stays separate."""
    report = Path(os.environ['ISSUE15_COVERAGE_REPORT'])
    scopes = ['-p', 'stackpress-framework-language', '-p', 'stackpress-framework-view', '-p', 'stackpress-framework-ui']
    features = 'stackpress-framework-view/native-file,stackpress-framework-view/web-preferences,stackpress-framework-view/web-flash,stackpress-framework-ui/stream,stackpress-framework-ui/rich'
    if '--report-only' not in sys.argv:
        subprocess.run(['cargo', 'llvm-cov', '--offline', '--locked', *scopes, '--features', features, '--all-targets', '--no-report'], check=True)
    subprocess.run(['cargo', 'llvm-cov', 'report', *scopes, '--json', '--output-path', str(report)], check=True)
    totals = {name:[0,0] for name in ['framework-language','framework-view','framework-ui']}
    for file in json.loads(report.read_text())['data'][0]['files']:
        path = Path(file['filename'])
        try:
            relative = path.relative_to(ROOT / 'crates/framework')
        except ValueError:
            continue
        if relative.parts[0] not in totals or 'src' not in relative.parts:
            continue
        if path.name == 'tests.rs' or path.name.endswith('_tests.rs'):
            continue
        lines = file['summary']['lines']
        totals[relative.parts[0]][0] += lines['covered']
        totals[relative.parts[0]][1] += lines['count']
    summary = {name:{'covered':covered,'count':count,'percent':100*covered/count if count else None} for name,(covered,count) in totals.items()}
    report.with_suffix('.summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    print(json.dumps(summary,indent=2),flush=True)
    assert all(value['percent'] is not None and value['percent']>=70 for value in summary.values())


if __name__ == '__main__':
    main()
