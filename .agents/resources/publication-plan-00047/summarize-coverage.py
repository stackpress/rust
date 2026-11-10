"""Attribute the one LLVM export to package source roots without hiding generated code."""
import argparse
import json
import math
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('report', type=Path)
parser.add_argument('--inventory', type=Path, default=Path(__file__).with_name('inventory.json'))
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
root = Path.cwd().resolve()
inventory = json.loads(args.inventory.read_text())
raw = json.loads(args.report.read_text())
assert len(raw['data']) == 1, 'Do not silently double-count multiple LLVM data units'
files = raw['data'][0]['files']
assert len({row['filename'] for row in files}) == len(files)
result = {'root': str(root), 'source': str(args.report), 'tool': 'cargo-llvm-cov 0.8.6',
          'scope': 'Every LLVM-mapped file under each package src directory, including in-source tests and shipped generated code; separate tests/examples/consumers and dependencies are not attributed as implementation.',
          'packages': []}
for package in inventory:
    if package['path'].startswith('examples/'):
        continue
    src = (root / package['path'] / 'src').resolve()
    selected = [row for row in files if Path(row['filename']).resolve().is_relative_to(src)]
    covered = sum(row['summary']['lines']['covered'] for row in selected)
    total = sum(row['summary']['lines']['count'] for row in selected)
    assert 0 <= covered <= total
    percent = covered * 100 / total if total else None
    record = {'name': package['name'], 'path': package['path'], 'covered': covered, 'total': total,
              'percent': percent, 'gap_percentage_points': max(0, 90 - percent) if total else None,
              'additional_covered_lines_for_90': max(0, math.ceil(9 * total / 10) - covered) if total else None,
              'files': [{'file': str(Path(row['filename']).resolve().relative_to(root)),
                         'covered': row['summary']['lines']['covered'], 'total': row['summary']['lines']['count']}
                        for row in selected]}
    result['packages'].append(record)
    print(package['name'], f'{covered}/{total}', f'{percent:.2f}%' if total else 'N/A',
          'gap', record['additional_covered_lines_for_90'])
result['combined'] = {'covered': sum(row['covered'] for row in result['packages']),
                      'total': sum(row['total'] for row in result['packages'])}
combined = result['combined']
combined['percent'] = combined['covered'] * 100 / combined['total'] if combined['total'] else None
result['raw_tool_totals'] = raw['data'][0]['totals']
args.output.write_text(json.dumps(result, indent=2) + '\n')
