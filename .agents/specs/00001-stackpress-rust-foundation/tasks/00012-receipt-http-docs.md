# Task 00012 HTTP documentation receipt

Closeout docs follow-up completed on 2026-10-27 in `trial/3-s7-http-m0`.
Current `trial/3` was fast-forward merged first: `a2c463a` → `a268b96`.
The coordinator authorized only server-http docs/README/example, server/docs,
and this receipt. All changes remain uncommitted for coordinator review.

## Delivered

- [HTTP host entry](../../../../crates/server-http/docs/README.md), getting-started production main, concepts, six task guides, and seven API/index pages. The crate README now routes readers into this set.
- Covers NetworkOptions/from_options/bind/serve_until, TLS files, explicit proxy trust, startup-verified immutable assets, bounded redacted observers, finite limits/deadlines/statuses, G31 authoritative-Hyper framing, sequential reuse, cancellation, and drain/service cleanup.
- [Server streaming guide](../../../../crates/server/docs/guides/streaming-http.md) and [reference](../../../../crates/server/docs/api/streaming-http.md), with entry/API/buffered-guide links. Removed stale M0 text claiming stream time controls awaited M1.
- Documents owned Send/static producers, cloned-view identity, replacement/error isolation, preflight/publication/failure, HEAD/bodyless behavior, immutable bytes, and local completion versus remote acknowledgement.
- Example `/health` now uses Response::set with `text/plain; charset=utf-8` and the unchanged `healthy\n` text. This is the only Rust source change.
- Managed development recipe follows devmetrics; no listener was started for this follow-up. Coordinator's earlier M2 managed smoke remains separate evidence; the MIME correction was compile-checked here.

## Verification results

Rust 1.96.0 (`ac68faa20`, aarch64-apple-darwin); CARGO_INCREMENTAL=0,
CARGO_PROFILE_DEV_DEBUG=0; only configured target was this worktree's `target`.

| Check | Result |
| --- | --- |
| Exact fenced Rust examples, disposable consumer, default | 10/10 compile; TLS-only fence correctly feature-gated. |
| Same consumer with tls | 11/11 compile, including full main and TLS guide. |
| Host deployment example, default | cargo check passed; not executed. |
| Host all-targets Clippy with tls | Passed with -D warnings; includes deployment example and tests. |
| cargo fmt --all -- --check | Passed. |
| Server http-stream doctests | 71 runnable tests plus 5 compile-fail tests passed. |
| Host tls doctests | 5 passed, including compile-only listener/file examples. |
| Host default doctests | Passed. |
| Server http-stream + host tls Rustdoc | Passed with RUSTDOCFLAGS=-D warnings. |
| Documentation structure/links | 23 new/affected pages: one H1, tagged fences, heading orientation, spacing, relative paths/anchors all clean. |
| JSON manifest example | Equal to checked fixture; exact 27-byte file size and SHA-256 verified. |
| git diff --check | Passed. |

Plain Rust fences were extracted verbatim. The older buffered API page's three
snippets used its explicitly documented shared setup/async return context;
all other snippets are independent declarations/programs. Default compilation
was repeated after the final main error-wrapper adjustment. No bind/main was
executed. Cargo children completed normally; no watchdog timeout occurred.

Review passes checked reader progression/task-versus-reference separation,
source-backed API/default/status/trust claims, then formatting/link/example
consistency. No new public API or source refactor was necessary. No manifest,
lock, production library, or test file changed. Full workspace/regression and
coverage reruns were not repeated for documentation plus a MIME-only example
fix; M2's accepted evidence remains in its own receipt, not a new measurement.

## Durable reproduction

Temporary logs and extracted consumers were under `/private/tmp/s7-http-docs/`
and may disappear on restart. The results above and this complete reproducer
are durable. Run from the worktree root; the Python creates and removes an
external consumer, and only compile-checks listener code:

```sh
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
export CARGO_TARGET_DIR="$PWD/target"
python3 - <<'PY'
import os, re, subprocess, tempfile
from pathlib import Path
root = Path.cwd()
pages = sorted((root/'crates/server-http/docs').rglob('*.md'))
pages += [root/'crates/server/docs'/p for p in [
    'guides/streaming-http.md', 'api/streaming-http.md',
    'guides/using-http.md', 'api/http.md']]
with tempfile.TemporaryDirectory(prefix='s7-http-docs-') as directory:
    consumer = Path(directory)
    bins = consumer/'src/bin'
    bins.mkdir(parents=True)
    manifest = '''[package]
name = "http-docs-consumer"
version = "0.0.0"
edition = "2024"
[workspace]
[features]
tls = ["stackpress-server-http/tls"]
[dependencies]
'''
    for name, folder, features in [
        ('stackpress-lib', 'crates/lib', ''),
        ('stackpress-server', 'crates/server', ', features = ["http-stream"]'),
        ('stackpress-server-http', 'crates/server-http', ''),
    ]:
        manifest += f'{name} = {{ path = "{root/folder}"{features} }}\n'
    manifest += '''tokio = { version = "1", features = ["macros", "rt-multi-thread", "signal"] }
futures-core = "=0.3.34"
serde_json = "1"
'''
    count = 0
    for page in pages:
        blocks = re.findall(r'```rust\n(.*?)```', page.read_text(), re.S)
        for index, code in enumerate(blocks):
            name = f'example_{count:02d}'
            if page.name == 'http.md':
                setup = blocks[0] if index else ''
                code = ('async fn example() -> Result<(), Box<dyn std::error::Error>> {\n'
                        + setup + code + '\nOk(())\n}\n')
            if not re.search(r'\bfn main\s*\(', code):
                code += '\nfn main() {}\n'
            (bins/f'{name}.rs').write_text('#![allow(dead_code)]\n'+code)
            manifest += f'\n[[bin]]\nname = "{name}"\npath = "src/bin/{name}.rs"\n'
            if '/guides/tls.md' in str(page):
                manifest += 'required-features = ["tls"]\n'
            count += 1
    assert count == 11, count
    (consumer/'Cargo.toml').write_text(manifest)
    cargo = ['cargo', '--manifest-path', str(consumer/'Cargo.toml')]
    subprocess.run(['cargo', 'generate-lockfile', '--offline', *cargo[1:]], check=True)
    for features in [[], ['--features', 'tls']]:
        subprocess.run(['cargo', 'check', '--locked', '--offline', '--bins',
                        *cargo[1:], *features], check=True)
PY
cargo fmt --all -- --check
cargo check --locked --offline -p stackpress-server-http --example deployment
cargo clippy --locked --offline -p stackpress-server-http --all-targets --features tls -- -D warnings
cargo test --locked --offline --doc -p stackpress-server -p stackpress-server-http --features stackpress-server/http-stream,stackpress-server-http/tls
cargo test --locked --offline --doc -p stackpress-server-http
RUSTDOCFLAGS='-D warnings' cargo doc --locked --offline --no-deps -p stackpress-server -p stackpress-server-http --features stackpress-server/http-stream,stackpress-server-http/tls
```

The manifest fixture and relative links can be independently checked against
tracked files. No devmetrics start is needed to review or compile these docs.
The development guide contains the separately managed listener command.

## Restart-safe closeout

No implementation work remains in this follow-up. Next step is coordinator
review of the docs and MIME correction, followed by coordinator-owned commit
or further scoped feedback. No new shared request or unresolved product choice.

The KB validator before/after has the same 146 existing missing upstream-source
link errors and 22 warnings; no new findings. These are the established
worktree-relative source-link baseline and were not repaired out of scope.
The receipt is under 200 lines. Target cleanup and stopped-build status are
verified at final closeout; source remains uncommitted and ready for restart.
