#!/usr/bin/env python3
"""Build/check content-addressed client inventories; no installation or listener."""
import hashlib
import json
from pathlib import Path
import re
import stat
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
MAX_ASSET_BYTES = 64 * 1024 * 1024
MAX_TOTAL_BYTES = 128 * 1024 * 1024
MAX_FILES = 128
MAX_MANIFEST_BYTES = 64 * 1024


def asset_folder(mode):
    """Admit only an existing regular output directory, matching native startup."""
    if mode not in ("hydrate", "csr"):
        raise ValueError("choose hydrate or csr")
    folder = PROFILE / "dist" / mode
    if folder.is_symlink():
        raise ValueError(f"asset root symlink refused: {folder}")
    if not folder.is_dir():
        raise ValueError(f"asset output missing: {folder}; build {mode} first")
    return folder


def read_bounded(file, limit, excess):
    """Reject special/oversized files before opening, and bound a growing file's read."""
    metadata = file.lstat()
    if stat.S_ISLNK(metadata.st_mode):
        raise ValueError(f"asset symlink refused: {file}")
    if not stat.S_ISREG(metadata.st_mode):
        raise ValueError(f"asset is not a regular file: {file}")
    if metadata.st_size > limit:
        raise ValueError(f"{excess}: {file}")
    with file.open("rb") as source:
        data = source.read(limit + 1)
    if len(data) > limit:
        raise ValueError(f"{excess}: {file}")
    return data


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
    folder = asset_folder(mode)
    files = {}
    total = 0
    # Admission is streaming; only the bounded accepted inventory is sorted for hashing.
    for file in folder.rglob("*"):
        if file.is_symlink():
            raise ValueError(f"asset symlink refused: {file}")
        if file.is_dir() or file == folder / "manifest.json":
            continue
        name = file.relative_to(folder).as_posix()
        if any(part in ("", ".", "..") or not re.fullmatch(r"[A-Za-z0-9_.-]+", part)
               for part in name.split("/")):
            raise ValueError(f"invalid asset name: {file}")
        mime = {".js": "text/javascript; charset=utf-8", ".wasm": "application/wasm",
                ".html": "text/html; charset=utf-8"}.get(file.suffix)
        if mime is None:
            raise ValueError(f"unsupported asset: {file}")
        # Count and remaining total are admission policy, not after-the-read checks.
        if len(files) >= MAX_FILES:
            raise ValueError("asset inventory exceeds count limit")
        limit = min(MAX_ASSET_BYTES, MAX_TOTAL_BYTES - total)
        excess = "asset exceeds 64 MiB" if limit == MAX_ASSET_BYTES else "asset inventory exceeds total byte limit"
        data = read_bounded(file, limit, excess)
        total += len(data)
        files[name] = {"sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data), "mime": mime}
    for required in ("start.js", "profile_ui.js", "profile_ui_bg.wasm"):
        if required not in files:
            raise ValueError(f"asset missing: {folder / required}; build {mode} first")
    revision, build, fingerprint = contract()
    manifest = {"format": 1, "revision": revision, "build": build, "mode": mode,
                "wasm_bindgen": "0.2.129", "contract_sha256": fingerprint, "files": files}
    manifest["content_id"] = identity(manifest)
    return manifest


def write(mode):
    """Publish one inventory only after all assets and the seed contract have been checked."""
    manifest = inventory(mode)
    path = asset_folder(mode) / "manifest.json"
    text = json.dumps(manifest, indent=2, sort_keys=True) + "\n"
    if len(text.encode("utf-8")) > MAX_MANIFEST_BYTES:
        raise ValueError(f"manifest exceeds 64 KiB: {path}")
    # Refuse an existing redirect/special file before replacing owned build metadata.
    if path.exists() or path.is_symlink():
        read_bounded(path, MAX_MANIFEST_BYTES, "manifest exceeds 64 KiB")
    path.write_text(text)
    return manifest


def check(mode):
    """Reject stale source, missing/modified assets, extra files and metadata drift."""
    path = asset_folder(mode) / "manifest.json"
    actual = json.loads(read_bounded(path, MAX_MANIFEST_BYTES, "manifest exceeds 64 KiB").decode("utf-8"))
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
