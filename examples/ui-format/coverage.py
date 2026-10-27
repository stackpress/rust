#!/usr/bin/env python3
"""Export existing instrumentation including the example; count production lines only."""
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
OUT = Path('/private/tmp/s8-format')
if '--run' in sys.argv:
    # Run after all other Cargo jobs: source, test executables and profiles must
    # agree, and no normal cdylib rebuild may replace an instrumented object.
    env = dict(os.environ, CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0',
               CARGO_TARGET_DIR=str(ROOT/'target/s8-native'),
               CARGO_LLVM_COV_TARGET_DIR=str(ROOT/'target/s8-native'))
    steps = [
        ('precoverage-clean', ['cargo', 'clean']),
        ('coverage-tests', ['cargo', 'llvm-cov', '--no-rustc-wrapper', '--no-report', '--locked', '--offline',
                            '-p', 'stackpress-framework-ui', '-p', 'stackpress-example-ui-format', '--features', 'ssr,rich,format']),
        ('coverage-native', ['cargo', 'llvm-cov', 'run', '--no-rustc-wrapper', '--no-report', '--locked', '--offline',
                             '-p', 'stackpress-example-ui-format', '--features', 'ssr,rich,format', '--bin', 'ui-format']),
        ('coverage-report', ['cargo', 'llvm-cov', 'report', '--json', '--output-path', str(OUT/'coverage.json'),
                             '--locked', '--offline', '-p', 'stackpress-framework-ui', '-p', 'stackpress-example-ui-format', '-v']),
    ]
    for name, args in steps:
        with (OUT/(name+'.log')).open('w') as output:
            output.write('$ '+' '.join(args)+'\n')
            output.flush()
            subprocess.run(args, cwd=ROOT, env=env, stdout=output, stderr=subprocess.STDOUT, timeout=1200, check=True)
        print('PASS '+name, flush=True)
log = (OUT / 'coverage-report.log').read_text()
line = next(line for line in log.splitlines() if 'llvm-cov export ' in line)
command = shlex.split(line.split('`')[1])
# cargo-llvm-cov normally excludes examples, and report metadata without selected
# features can exclude optional crates. Keep all raw source data, then select exact
# first-party production paths below rather than hide exclusions in a regex.
index = command.index('-ignore-filename-regex')
del command[index:index+2]
command.append('-num-threads=1')
with (OUT / 'coverage-full.json').open('w') as stdout, (OUT / 'coverage-export.log').open('w') as stderr:
    subprocess.run(command, cwd=ROOT, stdout=stdout, stderr=stderr, timeout=120, check=True)
# LCOV provides LLVM's actual per-line DA counts instead of reconstructing them
# from region segments or counting multi-line expressions by hand.
lcov = [part if part != '-format=text' else '-format=lcov' for part in command]
with (OUT / 'coverage-full.lcov').open('w') as stdout, (OUT / 'coverage-lcov.log').open('w') as stderr:
    subprocess.run(lcov, cwd=ROOT, stdout=stdout, stderr=stderr, timeout=120, check=True)
files = {}
for record in (OUT / 'coverage-full.lcov').read_text().split('end_of_record'):
    source = re.search(r'^SF:(.+)$', record, re.M)
    if not source:
        continue
    path = Path(source[1])
    if not path.is_relative_to(ROOT):
        continue
    relative = str(path.relative_to(ROOT))
    if not (relative.startswith('crates/framework/framework-ui/src/') or relative.startswith('examples/ui-format/src/')):
        continue
    if relative.endswith('_tests.rs'):
        continue
    text = path.read_text().splitlines()
    cutoff = next((i+1 for i, line in enumerate(text) if re.match(r'#\[cfg\((?:all\()?test\b', line)), len(text)+1)
    counts = {}
    for number, count in re.findall(r'^DA:(\d+),(\d+)', record, re.M):
        number, count = int(number), int(count)
        if number < cutoff:
            counts[number] = max(counts.get(number, 0), count)
    files[relative] = dict(covered=sum(c>0 for c in counts.values()), executable=len(counts),
                           uncovered=[n for n,c in counts.items() if not c], test_cutoff=cutoff)
summary = {}
for name, prefix in [('framework-ui', 'crates/framework/framework-ui/'), ('ui-format', 'examples/ui-format/')]:
    selected = {p: v for p,v in files.items() if p.startswith(prefix)}
    covered = sum(v['covered'] for v in selected.values())
    total = sum(v['executable'] for v in selected.values())
    summary[name] = dict(covered=covered, executable=total, percent=100*covered/total, files=selected)
    if covered/total < .7:
        raise RuntimeError(f'{name} below the per-crate development floor')
(OUT / 'coverage-summary.json').write_text(json.dumps(summary, indent=2))
print(json.dumps({name:{k:v for k,v in data.items() if k!='files'} for name,data in summary.items()}, indent=2))
