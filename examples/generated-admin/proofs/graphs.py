"""Compare normal-edge packages with a read-only trial/3 snapshot, without another target."""
import io
import json
import re
import subprocess
import tarfile
import tempfile
from pathlib import Path
from .runtime import ROOT


def packages(log):
    """Ignore tree indentation and local checkout paths; retain exact package versions."""
    return sorted(set(re.findall(r'([A-Za-z0-9_-]+) v([0-9][^\s()]*)', log)))


def compare(ctx):
    """Existing URL/IDNA ICU is permitted; no new default package is permitted."""
    LOGS = ctx.logs
    run = ctx.run
    LOGS.mkdir(parents=True, exist_ok=True)
    commit = subprocess.check_output(['git', 'rev-parse', 'trial/3'], cwd=ROOT,
                                     text=True, timeout=30).strip()
    archive = subprocess.check_output(['git', 'archive', '--format=tar', commit],
                                      cwd=ROOT, timeout=60)
    reports = {}
    with tempfile.TemporaryDirectory(prefix='trial3-graph-', dir=LOGS) as temp:
        with tarfile.open(fileobj=io.BytesIO(archive)) as source:
            source.extractall(temp, filter='data')
        for mode in ['hydrate', 'csr']:
            command = ['cargo', 'tree', '--offline', '--locked', '-p',
                       'generated-admin-client', '--target', 'wasm32-unknown-unknown',
                       '--features', mode, '--edges', 'normal']
            baseline = f'graph-trial3-default-{mode}'
            current = f'graph-current-default-{mode}'
            run(baseline, command + ['--manifest-path', str(Path(temp)/'Cargo.toml')])
            run(current, command)
            old = packages((LOGS/f'{baseline}.log').read_text())
            new = packages((LOGS/f'{current}.log').read_text())
            added = [package for package in new if package not in old]
            removed = [package for package in old if package not in new]
            reports[mode] = dict(baseline=old, current=new, added=added, removed=removed)
            assert not added and not removed, reports[mode]
    (LOGS/'default-graph-comparison.json').write_text(json.dumps(
        dict(baselineCommit=commit, modes=reports), indent=2))

