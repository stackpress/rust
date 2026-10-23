#!/usr/bin/env python3
"""Install or refresh managed .agents workspace rule files."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path


MARKER_START = "<!-- agent-workspace-rules:start -->"
MARKER_END = "<!-- agent-workspace-rules:end -->"

SKILL_DIR = Path(__file__).resolve().parents[1]
ASSET_ROOT = SKILL_DIR / "assets" / "dot-agents"

TEMPLATE_FILES = (
    Path("AGENTS.md"),
    Path("TERMS.md"),
    Path("references/00001-agent-workspace-rules.md"),
    Path("references/00002-intersection-points.md"),
    Path("references/00003-reference-recovery-points.md"),
    Path("workflows/agent-file-creation.md"),
    Path("workflows/agent-file-update.md"),
    Path("workflows/agent-file-ingestion.md"),
    Path("workflows/context-initialization.md"),
    Path("workflows/spec-driven-development.md"),
    Path("workflows/spec-task-implementation.md"),
    Path("workflows/spec-task-acceptance.md"),
    Path("workflows/spec-grill-session.md"),
    Path("workflows/spec-user-journeys.md"),
    Path("workflows/repair-zombie-reference-files.md"),
)

SCRIPT_FILES = (
    (
        Path("scripts/validate-agent-workspace.py"),
        SKILL_DIR / "scripts" / "validate_agent_workspace.py",
    ),
)

REQUIRED_DIRS = (
    Path("references"),
    Path("scripts"),
    Path("workflows"),
)

MCP_STATE = Path(".knowledge-mcp.json")
MCP_FILES = (
    ".gitignore", "package.json", "package-lock.json", "config.example.json",
    "config.mjs", "corpus.mjs", "embeddings.mjs", "snapshots.mjs",
    "retrieval.mjs", "references.mjs", "responses.mjs", "evaluation.mjs", "tokens.mjs", "server.mjs", "cli.mjs",
)
MCP_ROUTE = (
    "\n## Serve Knowledge To Other Projects\n\n"
    "The optional MCP capability is enabled. Use the "
    "[Serve KB Workflow](workflows/serve-kb.md) to configure, index, serve, "
    "or repair access from other projects. Runtime assets live in "
    "`.agents/scripts/mcp/`; their dependencies, models, and generated data "
    "are tooling rather than Agent Files.\n"
)


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def marked_block(text: str) -> str | None:
    start = text.find(MARKER_START)
    end = text.find(MARKER_END)
    if start == -1 or end == -1 or end < start:
        return None
    end += len(MARKER_END)
    return text[start:end]


def replace_marked(existing: str, template: str) -> tuple[str | None, str]:
    template_block = marked_block(template)
    existing_block = marked_block(existing)
    has_start = MARKER_START in existing
    has_end = MARKER_END in existing

    if template_block is None:
        raise ValueError("template is missing managed markers")

    if existing_block is not None:
        updated = existing.replace(existing_block, template_block)
        return updated, "refresh managed section"

    if has_start != has_end:
        return None, "conflict: incomplete managed markers"

    return None, "conflict: target exists without managed markers"


def plan_file(
    target: Path, template: Path, exact_managed: bool = False,
    template_text_override: str | None = None,
) -> tuple[str, str | None]:
    template_text = template_text_override if template_text_override is not None else read_text(template)
    if not target.exists():
        return "create", template_text

    existing = read_text(target)
    if existing == template_text:
        return "ok", None

    if exact_managed:
        return "refresh managed file", template_text

    updated, reason = replace_marked(existing, template_text)
    if updated is None:
        if target.name in {"AGENTS.md", "TERMS.md"}:
            block = marked_block(template_text)
            assert block is not None
            suffix = "\n\n" if not existing.endswith("\n") else "\n"
            return "append managed section", existing + suffix + block + "\n"
        return reason, None

    if updated == existing:
        return "ok", None
    return reason, updated


def install(target_root: Path, apply: bool, with_mcp: bool = False) -> int:
    agents_dir = target_root / ".agents"
    errors: list[str] = []
    actions: list[str] = []
    planned_dirs: list[Path] = []
    planned_files: list[tuple[Path, str, bool]] = []
    changes_needed = False
    state_file = agents_dir / MCP_STATE
    state = {}
    if state_file.exists():
        try:
            state = json.loads(read_text(state_file))
            if (state.get("schema_version") != 1 or state.get("enabled") is not True
                    or not isinstance(state.get("files"), dict)):
                raise ValueError("unsupported MCP installation state")
        except (ValueError, AttributeError) as error:
            print(f"Invalid {state_file}: {error}; inspect before repair", file=sys.stderr)
            return 1
    mcp_enabled = with_mcp or bool(state)

    for rel_dir in REQUIRED_DIRS:
        target_dir = agents_dir / rel_dir
        if target_dir.exists():
            actions.append(f"ok dir {target_dir}")
        else:
            actions.append(f"create dir {target_dir}")
            changes_needed = True
            planned_dirs.append(target_dir)

    templates = TEMPLATE_FILES + ((Path("workflows/serve-kb.md"),) if mcp_enabled else ())
    for rel_file in templates:
        target_file = agents_dir / rel_file
        template_file = ASSET_ROOT / rel_file
        if not template_file.exists():
            errors.append(f"missing template {template_file}")
            continue

        override = None
        if rel_file == Path("AGENTS.md") and mcp_enabled:
            override = read_text(template_file).replace(MARKER_END, MCP_ROUTE + MARKER_END)
        action, content = plan_file(target_file, template_file, template_text_override=override)
        actions.append(f"{action} {target_file}")
        if content is not None or action.startswith("conflict"):
            changes_needed = True

        if action.startswith("conflict"):
            errors.append(f"{target_file}: {action}; merge manually")
            continue

        if content is not None:
            planned_files.append((target_file, content, False))

    for rel_file, source_file in SCRIPT_FILES:
        target_file = agents_dir / rel_file
        if not source_file.exists():
            errors.append(f"missing script source {source_file}")
            continue

        action, content = plan_file(target_file, source_file, exact_managed=True)
        actions.append(f"{action} {target_file}")
        if content is not None:
            changes_needed = True

        if content is not None:
            planned_files.append((target_file, content, True))

    if mcp_enabled:
        hashes = {}
        for name in MCP_FILES:
            rel_file = Path("scripts/mcp") / name
            target_file = agents_dir / rel_file
            template_file = ASSET_ROOT / "scripts/mcp" / ("gitignore.template" if name == ".gitignore" else name)
            if not template_file.is_file():
                errors.append(f"missing MCP template {template_file}")
                continue
            content = read_text(template_file)
            digest = hashlib.sha256(content.encode()).hexdigest()
            hashes[name] = digest
            action = "create"
            if target_file.exists():
                existing = read_text(target_file)
                existing_hash = hashlib.sha256(existing.encode()).hexdigest()
                if existing == content:
                    actions.append(f"ok {target_file}")
                    continue
                if state.get("files", {}).get(name) != existing_hash:
                    errors.append(f"{target_file}: modified or unmanaged MCP asset; merge manually")
                    continue
                action = "refresh managed file"
            actions.append(f"{action} {target_file}")
            planned_files.append((target_file, content, False))
            changes_needed = True

        config_file = agents_dir / "scripts/mcp/config.json"
        if not config_file.exists():
            config = json.loads(read_text(ASSET_ROOT / "scripts/mcp/config.example.json"))
            slug = re.sub(r"[^a-z0-9-]+", "-", target_root.name.lower()).strip("-")[:48] or "project"
            config["name"] = f"{slug}-knowledge"
            config["project_id"] = f"{slug}-{hashlib.sha256(str(target_root).encode()).hexdigest()[:8]}"
            planned_files.append((config_file, json.dumps(config, indent=2) + "\n", False))
            actions.append(f"create project configuration {config_file}")
            changes_needed = True
        else:
            actions.append(f"preserve project configuration {config_file}")
        state_text = json.dumps({"schema_version": 1, "enabled": True, "files": hashes}, indent=2) + "\n"
        if not state_file.exists() or read_text(state_file) != state_text:
            planned_files.append((state_file, state_text, False))
            actions.append(f"record MCP enablement {state_file}")
            changes_needed = True

    # Preflight every destination before any write, including ancestor symlinks.
    for target in [agents_dir, *planned_dirs, *(item[0] for item in planned_files)]:
        for candidate in [target, *target.parents]:
            if candidate == target_root:
                break
            if candidate.is_symlink():
                errors.append(f"refusing symlink destination {candidate}")
        if target.exists() and target in [item[0] for item in planned_files] and not target.is_file():
            errors.append(f"file destination is not a regular file: {target}")

    mode = "APPLY" if apply else "DRY RUN"
    print(f"{mode}: agent workspace rules install")
    for action in actions:
        print(action)

    if errors:
        print("\nErrors:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        if apply:
            print("\nNo changes were written.", file=sys.stderr)
        return 1

    if apply:
        for target_dir in planned_dirs:
            target_dir.mkdir(parents=True, exist_ok=True)

        for target_file, content, executable in planned_files:
            target_file.parent.mkdir(parents=True, exist_ok=True)
            target_file.write_text(content, encoding="utf-8")
            if executable:
                target_file.chmod(target_file.stat().st_mode | 0o755)

    if not apply and changes_needed:
        print("\nRe-run with --apply to write these changes.")
    elif not apply:
        print("\nNo changes needed.")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Install or refresh managed .agents workspace rule files."
    )
    parser.add_argument(
        "--target",
        default=".",
        help="Project root containing or receiving the .agents workspace.",
    )
    parser.add_argument(
        "--with-mcp",
        action="store_true",
        help="Enable the optional workflow and runtime for serving this project's knowledge to other projects.",
    )
    parser.add_argument(
        "--apply",
        action="store_true",
        help="Write changes. Without this flag the script only prints a dry run.",
    )
    args = parser.parse_args()

    target_root = Path(args.target).expanduser().resolve()
    return install(target_root, args.apply, args.with_mcp)


if __name__ == "__main__":
    raise SystemExit(main())
