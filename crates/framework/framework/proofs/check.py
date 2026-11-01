#!/usr/bin/env python3
"""Check complete inventory/family/risk coverage and current source anchors without builds."""
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[4]
SPEC = ROOT/'.agents/specs/00001-stackpress-rust-foundation'
BASE = Path(__file__).resolve().parent


def rows(text, skip):
    """Keep source row identities exact, including aggregate family spellings."""
    return [line.strip('|').split('|')[0].strip() for line in text.splitlines()
            if line.startswith('| ') and not line.startswith(tuple('| '+item for item in skip))]


def main():
    """Every declared row has an existing source/test/proof and an explicit limitation list."""
    data = json.loads((BASE/'coverage-map.json').read_text())
    inventory = (SPEC/'slice-11-design-inventory.md').read_text().split('## Command inventory')
    assert [row['source'] for row in data['inventory']] == rows(inventory[0], ['Source file', '---'])
    assert [row['source'] for row in data['commands']] == rows(inventory[1], ['Command family', '---'])
    exports = (SPEC/'slice-11-design-exports.md').read_text().split('## Declarations')
    assert [row['family'] for row in data['export_families']] == rows(exports[0], ['Family', '---'])
    declared = {line.strip('|').split('|')[-1].strip() for line in exports[1].splitlines()
                if line.startswith('| src/')}
    mapped = {family.strip() for row in data['export_families'] for family in row['family'].split('/')}
    assert declared == mapped
    risks = rows((SPEC/'slice-11-design-verification.md').read_text().split('## Consumer contracts')[0], ['ID', '---'])
    assert [row['id'] for row in data['risks']] == risks == [f'R{i:02}' for i in range(1, 21)]
    for category in ['inventory', 'commands', 'export_families', 'risks']:
        for row in data[category]:
            assert row['evidence']
            for ref in row['evidence']:
                name, _, anchor = ref.partition('#')
                path = ROOT/name
                assert path.is_file(), ref
                assert not anchor or anchor in path.read_text(), ref
            assert all(name in data['limitations'] for name in row['limitations'])
    # Keep all prior session assertions; only the cache precondition and artifact name change.
    original = (ROOT/'crates/framework/framework-session/proofs/browser.cjs').read_text()
    expected = original.replace('const {chromium}=require(process.env.SESSION_PLAYWRIGHT);',
                                "assert.equal(process.env.AGGREGATE_PLAYWRIGHT,'/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright','use the approved cached Playwright path');\nconst {chromium}=require(process.env.AGGREGATE_PLAYWRIGHT);")
    assert (BASE/'session-browser.cjs').read_text() == expected.replace('slice9_account_browser', 'slice11_website')
    # Source attribution is checked against the whole read-only upstream package, not old target output.
    upstream = Path('/Users/cblanquera/server/projects/stackpress/stackpress/packages')
    hashes = re.findall(r'^\| ([^|]+?) \| ([0-9a-f]{64}) \|$', (SPEC/'slice-11-design-provenance.md').read_text(), re.M)
    assert len(hashes) == 43
    for name, digest in hashes:
        assert hashlib.sha256((upstream/'stackpress'/name).read_bytes()).hexdigest() == digest, name
    for name, key in [('source-order.json', 'source_sha256'), ('source-commands.json', 'files'), ('source-scripts.json', 'files')]:
        oracle = json.loads((BASE.parent/'tests/fixtures'/name).read_text())
        for path, digest in oracle[key].items():
            assert hashlib.sha256((upstream/path).read_bytes()).hexdigest() == digest, path
    print(json.dumps(dict(inventory=len(data['inventory']), commands=len(data['commands']),
                          export_families=len(mapped), declaration_rows=len(rows(exports[1], ['Source:line', '---'])),
                          risks=len(data['risks']), source_files=len(hashes), limitations=sorted(data['limitations'])), indent=2))


if __name__ == '__main__':
    main()
