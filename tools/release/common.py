"""Shared, dependency-free release inventory and command helpers."""
from pathlib import Path
import json
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / ".agents/resources/release-manifests-00050"
VERSION = "0.2.1"
# Explicit G59/G61 scope; private examples are never inferred as releases.
CANDIDATES = [
    "crates/leptos", "crates/framework/framework-language", "crates/sql/sql", "crates/lib",
    "crates/idea/idea-parser", "crates/sql/sql-sqlite", "crates/sql/sql-postgres",
    "crates/sql/sql-mysql", "crates/server/server", "crates/idea/idea-transformer",
    "crates/server/http", "tools/cli", "crates/framework/framework-view",
    "crates/framework/framework-csrf", "crates/framework/framework-email", "crates/idea/idea",
    "crates/framework/framework-schema", "crates/framework/framework-sql",
    "crates/framework/framework-admin", "crates/framework/framework-session",
    "crates/framework/framework-api", "crates/framework/framework", ".",
]
COMPATIBLE = (
    "framework-aggregate-full,framework-aggregate-ui-session,framework-aggregate-ui-api,"
    "framework-aggregate-ui-admin,framework-aggregate-ui-view,framework-aggregate-ssr,"
    "framework-api-codegen,framework-csrf-server,framework-email-server,"
    "stackpress-framework/codegen,stackpress-framework-schema/providers,"
    "stackpress-framework-sql/sqlite-maintenance,stackpress-framework-sql/codegen,"
    "stackpress-framework-sql/server,stackpress-framework-schema/source,stackpress-framework-sql/source,"
    "server-http-sse,server-http-host-sse,server-http-host-websocket,server-http-host-tls,"
    "leptos-sse,leptos-websocket"
)


def manifest(path):
    """Read TOML without resolving or refreshing dependencies."""
    return tomllib.loads(path.read_text())


def candidates():
    """Return approved packages in dependency-first publication order."""
    return [(ROOT / path, manifest(ROOT / path / "Cargo.toml")["package"]["name"])
            for path in CANDIDATES]


def registry(data):
    """Compare exact registry identity, source and checksum; retain duplicate versions."""
    return {(p["name"], p["version"], p["source"], p.get("checksum"))
            for p in tomllib.loads(data.decode())["package"]
            if p.get("source", "").startswith("registry+")}


def run(command, **kwargs):
    """Expose nested Cargo output to the single enclosing benchmark record."""
    print("COMMAND:", *map(str, command), flush=True)
    return subprocess.run(command, cwd=ROOT, check=True, **kwargs)


def seed_lock(directory, env):
    """Resolve a new consumer once offline, then assert a root registry subset."""
    lock = directory / "Cargo.lock"
    root_lock = (ROOT / "Cargo.lock").read_bytes()
    if not lock.exists():
        lock.write_bytes(root_lock)
        run(["cargo", "tree", "--manifest-path", str(directory / "Cargo.toml"),
             "--offline", "-e", "normal", "--depth", "1"], env=env)
    assert registry(lock.read_bytes()) <= registry(root_lock), "new consumer registry tuple"
    assert (ROOT / "Cargo.lock").read_bytes() == root_lock, "root lock changed"


def write_report(path, data):
    """Keep complete machine-readable evidence beside its command log."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2) + "\n")
