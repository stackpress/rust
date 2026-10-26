#!/usr/bin/env python3
"""Export actual first-party profiles without cargo-llvm-cov's blanket examples exclusion."""
from collections import defaultdict
import json
import os
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parents[3]
base = Path(os.environ.get("PROFILE_UI_COVERAGE_TARGET", str(root / "target" / "profile-ui-native")))
if len(sys.argv) > 1 and sys.argv[1] == "build-native":
    import shlex
    env = dict(os.environ, CARGO_TARGET_DIR=str(base))
    settings = subprocess.check_output(["cargo", "llvm-cov", "show-env"], cwd=root, env=env, text=True)
    for line in settings.splitlines():
        key, value = line.split("=", 1)
        env[key] = shlex.split(value)[0] if value else ""
    subprocess.run(["cargo", "build", "--locked", "-p", "stackpress-example-profile-ui-native", "--features", "app"], cwd=root, env=env, check=True)
    raise SystemExit(0)
sysroot = Path(subprocess.check_output(["rustc", "--print", "sysroot"], text=True).strip())
llvm = sysroot / "lib/rustlib/aarch64-apple-darwin/bin"
profiles = list(base.glob("*.profraw"))
if not profiles:
    raise SystemExit("no profiles; run the scoped cargo llvm-cov command after cleaning first")
merged = base / "profile-ui-complete.profdata"
subprocess.run([str(llvm / "llvm-profdata"), "merge", "-sparse", *map(str, profiles), "-o", str(merged)], check=True)
prefixes = ("stackpress_example_profile_ui_", "stackpress_framework_", "pages-", "contracts-", "executor-", "cleanup-", "wire-", "lifecycle-", "policies-", "routing-", "codec-")
objects = [p for p in (base / "debug/deps").iterdir() if p.is_file() and p.suffix == "" and p.name.startswith(prefixes) and os.access(p, os.X_OK)]
for binary in ("profile-ui", "stackpress-example-profile-ui-web"):
    p = base / "debug" / binary
    if p.exists(): objects.append(p)
command = [str(llvm / "llvm-cov"), "export", "-format=text", f"-instr-profile={merged}"]
for obj in objects: command.extend(["-object", str(obj)])
raw = subprocess.check_output(command, text=True)
output = Path(os.environ.get("PROFILE_UI_COVERAGE_REPORT", "/private/tmp/profile-ui-complete-coverage.json"))
output.write_text(raw)
totals = defaultdict(lambda: [0, 0])
for file in json.loads(raw)["data"][0]["files"]:
    name = file["filename"]
    if not name.startswith(str(root) + "/"): continue
    relative = Path(name).relative_to(root)
    # Only production src files count; dedicated test source is evidence, not production.
    if str(relative) == "examples/profile-ui/route-contract.rs": group = "web"
    elif "src" not in relative.parts or relative.name == "tests.rs": continue
    elif str(relative).startswith("examples/profile-ui/"): group = relative.parts[2]
    elif str(relative).startswith("crates/framework/"): group = relative.parts[2]
    else: continue
    lines = file["summary"]["lines"]
    totals[group][0] += lines["covered"]
    totals[group][1] += lines["count"]
for name, (covered, count) in sorted(totals.items()):
    print(f"{name}: {covered}/{count} = {100 * covered / count:.2f}%")
print(f"raw report: {output}")
