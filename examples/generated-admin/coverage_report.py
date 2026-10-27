#!/usr/bin/env python3
"""Include example packages that cargo-llvm-cov's default filename filter hides."""
import json
import shlex
from pathlib import Path

from verify import ROOT, LOGS, run


def main():
    """Export unchanged counters for all implementation sources, including generated contracts."""
    line = next(line for line in (LOGS / 'coverage-command.log').read_text().splitlines()
                if 'llvm-cov export ' in line)
    argv = shlex.split(line.split('`', 2)[1])
    index = argv.index('-ignore-filename-regex')
    del argv[index:index + 2]
    roots = ['crates/framework/framework-admin', 'examples/generated-admin/tool',
             'examples/generated-admin/shared', 'examples/generated-admin/web']
    sources = [str(path) for root in roots for path in (ROOT / root / 'src').rglob('*.rs')]
    run('coverage-raw', argv + ['--sources', *sources])
    raw = (LOGS / 'coverage-raw.log').read_text().split('\n', 1)[1]
    (LOGS / 'coverage-raw.json').write_text(raw)
    data = json.loads(raw)
    result = {}
    for root in roots:
        files = [file for item in data['data'] for file in item['files']
                 if file['filename'].startswith(str(ROOT / root / 'src') + '/')]
        covered = sum(file['summary']['lines']['covered'] for file in files)
        total = sum(file['summary']['lines']['count'] for file in files)
        result[root] = {'covered': covered, 'total': total, 'percent': covered * 100 / total if total else None}
    (LOGS / 'coverage-summary.json').write_text(json.dumps(result, indent=2))
    print(json.dumps(result, indent=2))
    below_floor = [root for root, counts in result.items() if counts['percent'] is None or counts['percent'] < 70]
    if below_floor:
        raise RuntimeError(f'crates below the development coverage floor: {below_floor}')


if __name__ == '__main__':
    main()
