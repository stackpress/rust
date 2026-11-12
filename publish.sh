#!/usr/bin/env bash
# Publish the 23 Stackpress release crates to crates.io in dependency order.
#
# Usage (from the repository root, after `cargo login`):
#   ./publish.sh            publish everything not yet on crates.io
#   ./publish.sh --plan     only print what would be published and the pacing
#
# The version comes from `[workspace.package] version` in Cargo.toml, so bump
# that (and the internal `version = "..."` requirements) before a new release.
#
# crates.io rate limits (defaults in rust-lang/crates.io src/rate_limiter.rs):
#   - brand-new crate names: a burst of 5, then 1 more every 10 minutes;
#   - new versions of existing crates: a burst of 30, then 1 per minute.
# The script paces each kind separately. A new version of all 23 existing
# crates fits inside the burst of 30, so it publishes back to back. Brand-new
# crates beyond the first 5 wait a little over 10 minutes each. If crates.io
# still answers "too many requests" (for example because of other recent
# publishes), the script waits out that kind's interval and retries the crate.
#
# The script is safe to re-run: crates already on crates.io at this version are
# skipped, so after a failure you fix the cause and run it again. It never logs
# in and never uses --allow-dirty.
set -euo pipefail

# Release version, read from [workspace.package] in the root Cargo.toml.
VERSION=$(python3 -c "import tomllib;print(tomllib.load(open('Cargo.toml','rb'))['workspace']['package']['version'])")
# crates.io buckets: burst size and refill interval (plus a small margin).
NEW_BURST=5
NEW_PACE_SECONDS=610
UPDATE_BURST=30
UPDATE_PACE_SECONDS=65
# How many times to retry one crate after a rate-limit (429) answer.
MAX_RATE_LIMIT_RETRIES=6
# crates.io asks API clients to identify themselves; no personal details.
USER_AGENT="stackpress-release-publish"

# Dependency order: every crate comes after everything it depends on.
CRATES=(
  stackpress-leptos
  stackpress-framework-language
  stackpress-sql
  stackpress-lib
  stackpress-idea-parser
  stackpress-sql-sqlite
  stackpress-sql-postgres
  stackpress-sql-mysql
  stackpress-server
  stackpress-idea-transformer
  stackpress-server-http
  stackpress-cli
  stackpress-framework-view
  stackpress-framework-csrf
  stackpress-framework-email
  stackpress-idea
  stackpress-framework-schema
  stackpress-framework-sql
  stackpress-framework-admin
  stackpress-framework-session
  stackpress-framework-api
  stackpress-framework
  stackpress
)

plan_only=false
if [[ "${1:-}" == "--plan" ]]; then
  plan_only=true
fi

log() { printf '[%s] %s\n' "$(date '+%H:%M:%S')" "$*"; }

# HTTP status crates.io gives for a crate (and optionally one version of it).
status() {
  curl -s -o /dev/null -w '%{http_code}' -A "$USER_AGENT" "https://crates.io/api/v1/crates/$1"
}

# True when crates.io already has this crate at the release version.
is_published() {
  [[ "$(status "$1/$VERSION")" == "200" ]]
}

# True when the crate name already exists on crates.io (any version).
exists() {
  [[ "$(status "$1")" == "200" ]]
}

# Refuse to publish anything that isn't exactly what's committed and pushed.
preflight() {
  if [[ ! -f Cargo.toml || ! -d tools/release ]]; then
    echo "Run this from the Stackpress repository root." >&2
    exit 1
  fi
  if [[ -n "$(git status --porcelain)" ]]; then
    echo "The working tree has uncommitted changes; commit or stash them first." >&2
    exit 1
  fi
  git fetch -q origin
  if [[ "$(git rev-parse HEAD)" != "$(git rev-parse origin/main)" ]]; then
    echo "HEAD is not the same commit as origin/main; push or pull first." >&2
    exit 1
  fi
}

# Publish one crate, waiting and retrying while crates.io rate-limits us.
publish_one() {
  local crate="$1" pace="$2" attempt=0 output
  while true; do
    log "Publishing $crate $VERSION"
    if output=$(cargo publish -p "$crate" --locked 2>&1); then
      printf '%s\n' "$output" | tail -n 3
      return 0
    fi
    printf '%s\n' "$output" | tail -n 15
    # A 429 means "too many new crates"; anything else is a real failure.
    if printf '%s' "$output" | grep -qiE '429|too many'; then
      attempt=$((attempt + 1))
      if (( attempt > MAX_RATE_LIMIT_RETRIES )); then
        echo "Still rate-limited after $MAX_RATE_LIMIT_RETRIES retries; re-run later." >&2
        exit 1
      fi
      log "Rate-limited by crates.io; waiting ${pace}s before retrying $crate"
      sleep "$pace"
      continue
    fi
    echo "Publishing $crate failed. Fix the cause, then re-run ./publish.sh." >&2
    exit 1
  done
}

main() {
  local pending=() kinds=() crate new_count=0 update_count=0
  log "Release version $VERSION"
  for crate in "${CRATES[@]}"; do
    if is_published "$crate"; then
      log "Skipping $crate (already on crates.io at $VERSION)"
    elif exists "$crate"; then
      pending+=("$crate"); kinds+=("update"); update_count=$((update_count + 1))
    else
      pending+=("$crate"); kinds+=("new"); new_count=$((new_count + 1))
    fi
  done

  if (( ${#pending[@]} == 0 )); then
    log "All ${#CRATES[@]} crates are already published at $VERSION."
    return 0
  fi

  local paced_new=$(( new_count > NEW_BURST ? new_count - NEW_BURST : 0 ))
  local paced_update=$(( update_count > UPDATE_BURST ? update_count - UPDATE_BURST : 0 ))
  local wait_minutes=$(( (paced_new * NEW_PACE_SECONDS + paced_update * UPDATE_PACE_SECONDS) / 60 ))
  log "${#pending[@]} to publish: $update_count new version(s) of existing crates, $new_count brand-new crate(s); about $wait_minutes minute(s) of rate-limit waiting."

  if $plan_only; then
    local i
    for i in "${!pending[@]}"; do
      printf '  %-34s %s\n' "${pending[$i]}" "${kinds[$i]}"
    done
    return 0
  fi

  preflight

  local i new_done=0 update_done=0
  for i in "${!pending[@]}"; do
    crate="${pending[$i]}"
    if [[ "${kinds[$i]}" == "new" ]]; then
      if (( new_done >= NEW_BURST )); then
        log "Waiting ${NEW_PACE_SECONDS}s for crates.io's new-crate limit (next: $crate)"
        sleep "$NEW_PACE_SECONDS"
      fi
      publish_one "$crate" "$NEW_PACE_SECONDS"
      new_done=$((new_done + 1))
    else
      if (( update_done >= UPDATE_BURST )); then
        log "Waiting ${UPDATE_PACE_SECONDS}s for crates.io's update limit (next: $crate)"
        sleep "$UPDATE_PACE_SECONDS"
      fi
      publish_one "$crate" "$UPDATE_PACE_SECONDS"
      update_done=$((update_done + 1))
    fi
  done

  log "Done: published ${#pending[@]} crate(s) at $VERSION."
}

main
