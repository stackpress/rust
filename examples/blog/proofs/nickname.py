#!/usr/bin/env python3
"""Prepare/check the approved copied-out app's real SQLite upgrade and generated browser UI."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess

BLOG = Path(__file__).resolve().parents[1]
ROOT = BLOG.parents[1]
STATE = BLOG / '.build/nickname.json'
PROJECT = BLOG / '.build/readme/project'


def digest(path):
    """Hash actual runtime inputs/artifacts so the gate never uses an older copied binary."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def signature():
    """Include every authored runtime file, selected manifests/lock and the framework commit."""
    files = [file for directory in ['tool/src', 'web/src', 'shared/src', 'client/src', 'assets', 'config']
             for file in (BLOG / directory).rglob('*') if file.is_file()]
    files += [BLOG / name for name in ['schema.idea', 'bootstrap.js', 'Cargo.lock', 'Cargo.toml',
              'tool/Cargo.toml', 'web/Cargo.toml', 'shared/Cargo.toml', 'client/Cargo.toml']]
    return {'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
            'files': {str(file.relative_to(BLOG)): digest(file) for file in sorted(files)}}


def check():
    """Require the same framework/runtime source and exact generated/native/browser products."""
    state = json.loads(STATE.read_text())
    assert state['source'] == signature(), 'nickname fixture source is stale; prepare it again'
    for name, expected in state['products'].items():
        assert digest(Path(name)) == expected, name
    assert (PROJECT / 'Cargo.lock').read_bytes() == (BLOG / 'Cargo.lock').read_bytes()
    print('Nickname fixture: current source/HEAD, upgraded SQLite, exact emitted UI/native/browser products and unchanged lock.')
    return state


def prepare():
    """Follow the README's checkout-path step, then revise/generate/push the copied app."""
    source = signature()
    root_lock = (ROOT / 'Cargo.lock').read_bytes()
    source_lock = (BLOG / 'Cargo.lock').read_bytes()
    source_schema = (BLOG / 'schema.idea').read_bytes()
    native = Path(os.environ['CARGO_TARGET_DIR'])
    target = native.parent
    try:
        # The copy proof derives only existing repo-local edges from this checkout.
        subprocess.run(['python3', str(BLOG / 'proofs/readme.py')], check=True, timeout=420)
        env = dict(os.environ, CARGO_TARGET_DIR=str(target / 'readme'))

        def cargo(*args):
            """All consumer commands use its byte-identical lock, offline, in its own target."""
            subprocess.run(['cargo', *args, '--locked', '--offline'], cwd=PROJECT, env=env,
                           check=True, timeout=300)

        database = PROJECT / '.build/blog.sqlite'
        with sqlite3.connect(database) as connection:
            before = connection.execute('SELECT id,title FROM article ORDER BY id').fetchall()
            assert len(before) == 2
        header = 'model Profile {\n'
        schema = source_schema.decode()
        assert schema.count(header) == 1
        schema = schema.replace(header, header + '  nickname String? @label("Nickname") @searchable @field.string @list.string @view.string\n', 1)
        (PROJECT / 'schema.idea').write_text(schema)
        # Generator options follow --; locked/offline must remain Cargo's arguments.
        def run(package, *args):
            subprocess.run(['cargo', 'run', '-p', package, '--locked', '--offline', *args],
                           cwd=PROJECT, env=env, check=True, timeout=300)
        run('blog-tool', '--', 'generate-app')
        run('blog-tool', '--', 'generate-app', '--check')
        run('blog-web', '--bin', 'blog-web', '--', '-b', 'config/develop.json', 'emit', 'revision')
        run('blog-web', '--bin', 'blog-web', '--', '-b', 'config/develop.json', 'emit', 'push')
        with sqlite3.connect(database) as connection:
            assert connection.execute('SELECT id,title FROM article ORDER BY id').fetchall() == before
            assert 'nickname' in {row[1] for row in connection.execute('PRAGMA table_info(profile)')}
            seed = PROJECT / '.build/nickname-seed.sqlite'
            with sqlite3.connect(seed) as snapshot:
                connection.backup(snapshot)
        cargo('build', '-p', 'blog-web', '--bin', 'probe')
        frozen = BLOG / '.build/nickname/probe'
        frozen.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(target / 'readme/debug/probe', frozen)
        # This native consumer has finished: retain its frozen binary and upgraded database,
        # and free only its compiled target before the two browser graphs need that budget.
        consumer = target / 'readme'
        assert consumer.resolve().parent == target.resolve()
        shutil.rmtree(consumer)
        # The copied app's browser modes get their own consumer targets; main products stay warm.
        browser_env = dict(os.environ, CARGO_TARGET_DIR=str(target / 'nickname/native'))
        subprocess.run(['python3', str(PROJECT / 'proofs/build.py')], env=browser_env,
                       cwd=PROJECT, check=True, timeout=600)
        products = [frozen, seed, PROJECT / 'schema.idea', PROJECT / 'Cargo.lock']
        products += [file for directory in ['.build/app-generated', '.build/browser']
                     for file in (PROJECT / directory).rglob('*') if file.is_file()]
        # A concurrent authored/runtime change invalidates the copy rather than blessing an
        # older consumer with a newer source signature. Proof/docs edits are not runtime inputs.
        assert signature() == source, 'runtime source changed during copied-out preparation'
        state = {'source': source, 'project': str(PROJECT), 'probe': str(frozen),
                 'seed': str(seed), 'preserved_articles': before,
                 'products': {str(file): digest(file) for file in sorted(products)}}
        STATE.write_text(json.dumps(state, indent=2))
        check()
        # Completed consumer compilation outputs are disposable; exact products remain above.
        for directory in [target / 'readme', target / 'nickname']:
            assert directory.resolve().parent == target.resolve()
            if directory.exists():
                shutil.rmtree(directory)
        print('Copied-out first success and real nullable-column push preserved two articles; both regenerated browser modes prepared.')
    finally:
        assert (ROOT / 'Cargo.lock').read_bytes() == root_lock
        assert (BLOG / 'Cargo.lock').read_bytes() == source_lock
        assert (BLOG / 'schema.idea').read_bytes() == source_schema


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=['prepare', 'check', 'browser'])
    args = parser.parse_args()
    if args.mode == 'prepare':
        prepare()
        return 0
    state = check()
    if args.mode == 'browser':
        env = dict(os.environ, BLOG_BROWSER_PROJECT=state['project'], BLOG_BROWSER_PROBE=state['probe'],
                   BLOG_BROWSER_SEED=state['seed'], BLOG_NICKNAME='1')
        return subprocess.run(['python3', str(BLOG / 'proofs/browser.py')], env=env,
                              check=False, timeout=300).returncode
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
