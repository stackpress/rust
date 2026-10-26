#!/usr/bin/env python3
"""Build/check content-addressed client inventories; no installation or listener."""
import hashlib
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[2]
PROFILE = ROOT / "examples/profile-ui"
CONTRACT_FILES = (
    "examples/profile-ui/contracts/src/lib.rs",
    "examples/profile-ui/contracts/src/reply.rs",
    "examples/profile-ui/route-contract.rs",
    "examples/generated-profile/app/src/generated/contracts/profile.rs",
    "crates/framework/framework-view/src/lib.rs",
    "crates/framework/framework-view/src/policy.rs",
    "crates/framework/framework-view/src/routes.rs",
)


def contract():
    """Read the actual Rust wire/build constants and fingerprint their contract sources."""
    source = (PROFILE / "contracts/src/lib.rs").read_text()
    revision = int(re.search(r'pub const REVISION: u32 = (\d+);', source)[1])
    build = re.search(r'pub const BUILD: &str = "([^"]+)";', source)[1]
    digest = hashlib.sha256()
    for name in CONTRACT_FILES:
        digest.update(name.encode() + b"\0" + (ROOT / name).read_bytes() + b"\0")
    return revision, build, digest.hexdigest()


def identity(manifest):
    """Use the same explicit NUL-delimited digest grammar as the native gateway."""
    digest = hashlib.sha256()
    fields = ["profile-ui-assets-v1", str(manifest["revision"]), manifest["build"],
              manifest["mode"], manifest["wasm_bindgen"], manifest["contract_sha256"]]
    for name, entry in sorted(manifest["files"].items()):
        fields += [name, entry["mime"], str(entry["bytes"]), entry["sha256"]]
    for field in fields:
        digest.update(field.encode() + b"\0")
    return digest.hexdigest()


def inventory(mode):
    """Hash every actual output, rejecting ambiguous names and unsupported assets."""
    if mode not in ("hydrate", "csr"):
        raise ValueError("choose hydrate or csr")
    folder = PROFILE / "dist" / mode
    files = {}
    for file in sorted(folder.rglob("*")):
        if file.is_symlink():
            raise ValueError(f"asset symlink refused: {file}")
        if not file.is_file() or file.name == "manifest.json":
            continue
        name = file.relative_to(folder).as_posix()
        if any(part in ("", ".", "..") or not re.fullmatch(r"[A-Za-z0-9_.-]+", part)
               for part in name.split("/")):
            raise ValueError(f"invalid asset name: {file}")
        mime = {".js": "text/javascript; charset=utf-8", ".wasm": "application/wasm",
                ".html": "text/html; charset=utf-8"}.get(file.suffix)
        if mime is None:
            raise ValueError(f"unsupported asset: {file}")
        data = file.read_bytes()
        if len(data) > 64 * 1024 * 1024:
            raise ValueError(f"asset exceeds 64 MiB: {file}")
        files[name] = {"sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data), "mime": mime}
    for required in ("start.js", "profile_ui.js", "profile_ui_bg.wasm"):
        if required not in files:
            raise ValueError(f"asset missing: {folder / required}; build {mode} first")
    if len(files) > 128 or sum(entry["bytes"] for entry in files.values()) > 128 * 1024 * 1024:
        raise ValueError("asset inventory exceeds count/total byte limit")
    revision, build, fingerprint = contract()
    manifest = {"format": 1, "revision": revision, "build": build, "mode": mode,
                "wasm_bindgen": "0.2.129", "contract_sha256": fingerprint, "files": files}
    manifest["content_id"] = identity(manifest)
    return manifest


def write(mode):
    """Publish one inventory only after all assets and the seed contract have been checked."""
    manifest = inventory(mode)
    (PROFILE / "dist" / mode / "manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    return manifest


def check(mode):
    """Reject stale source, missing/modified assets, extra files and metadata drift."""
    path = PROFILE / "dist" / mode / "manifest.json"
    actual = json.loads(path.read_text())
    expected = inventory(mode)
    if actual != expected:
        raise ValueError(f"asset/seed contract mismatch: {path}; rebuild {mode}")
    return actual


if __name__ == "__main__":
    try:
        if len(sys.argv) != 3 or sys.argv[1] not in ("write", "check"):
            raise ValueError("usage: asset_manifest.py <write|check> <hydrate|csr>")
        result = (write if sys.argv[1] == "write" else check)(sys.argv[2])
        print(f"{sys.argv[2]}: {result['build']} /assets/{result['content_id']}/start.js")
    except (ValueError, OSError, KeyError, TypeError) as error:
        raise SystemExit(str(error)) from error
