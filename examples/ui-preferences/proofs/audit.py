#!/usr/bin/env python3
"""Audit source scope and proof syntax without building or starting a listener."""
import ast
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]


def main():
    """Keep the source migration, generated metadata and retired scaffold reviewable."""
    changed = subprocess.check_output(['git','diff','--name-only'],cwd=ROOT,text=True).splitlines()
    new = subprocess.check_output(['git','ls-files','--others','--exclude-standard'],cwd=ROOT,text=True).splitlines()
    allowed = ('Cargo.toml','Cargo.lock','src/lib.rs','crates/framework/framework-language/',
               'crates/framework/framework-view/','crates/leptos/','crates/framework/framework-preferences/',
               'examples/profile-ui/','examples/ui-preferences/','examples/ui-gallery/',
               'examples/ui-stream/web/src/lib.rs','examples/review-preview/Cargo.toml',
               'examples/review-preview/src/panel.rs',
               '.agents/specs/00001-stackpress-rust-foundation/tasks/00016-',
               '.agents/specs/00001-stackpress-rust-foundation/tasks/00013-receipt-lane-b-m1.md')
    files = sorted(set(changed + new))
    assert all(file.startswith(allowed) for file in files), [file for file in files if not file.startswith(allowed)]
    for removed in ['crates/framework/framework-preferences','crates/leptos/src/components/system.css',
                    'crates/framework/framework-view/proofs/issue15-m0']:
        assert not (ROOT / removed).exists(), removed
    for name in files:
        file = ROOT / name
        if not file.is_file():
            continue
        if file.suffix == '.py':
            ast.parse(file.read_text(),filename=name)
        if file.suffix in ('.js','.cjs'):
            subprocess.run(['node','--check',str(file)],check=True)
        if file.suffix == '.rs' and file.name != 'golden.rs':
            text=file.read_text()
            assert all(token not in text for token in ['Theme::System','ThemePreference::System','SystemThemeStyles','SYSTEM_THEME_STYLES']), name
    subprocess.run(['git','diff','--check'],cwd=ROOT,check=True)
    result={'files':files,'scope':'passed','python_node_syntax':'passed','old_crate_system_css_prototype':'removed','whitespace':'passed'}
    Path(sys.argv[1]).write_text(json.dumps(result,indent=2)+'\n')
    print('PASS M1 scope, source/syntax and whitespace audit:',len(files),'paths')


if __name__ == '__main__':
    main()
