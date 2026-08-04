#!/usr/bin/env bash
# Nudge agents toward classic git: few code PRs for related clusters.
# Never auto-commits or pushes — only returns followup_message for the agent.
set -euo pipefail

input="$(cat)"
status="$(printf '%s' "$input" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("status",""))' 2>/dev/null || true)"
loop_count="$(printf '%s' "$input" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("loop_count",0))' 2>/dev/null || echo 0)"

if [[ "$status" != "completed" ]]; then
  echo '{}'
  exit 0
fi

# Cap auto-followups from this hook (also capped by hooks.json loop_limit).
if [[ "${loop_count:-0}" -ge 1 ]]; then
  echo '{}'
  exit 0
fi

root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [[ -z "$root" ]]; then
  echo '{}'
  exit 0
fi
cd "$root"

# Clean tree → nothing to do.
if git diff --quiet && git diff --cached --quiet && [[ -z "$(git ls-files --others --exclude-standard)" ]]; then
  echo '{}'
  exit 0
fi

# Count dirty paths; also whether any non-docs/handoff code changed.
porcelain="$(git status --porcelain)"
dirty_count="$(printf '%s\n' "$porcelain" | sed '/^$/d' | wc -l | tr -d ' ')"
code_dirty=0
if printf '%s\n' "$porcelain" | grep -qE '^.. (crates/|apps/|Cargo\.(toml|lock)|\.github/)'; then
  code_dirty=1
fi

# Active gate only if marker exists AND is not already MERGED/ARCHIVED.
gate_active=0
for gate in docs/handoffs/PR-GATE.md docs/handoffs/SPRINT-GATE.md; do
  if [[ -f "$gate" ]]; then
    if grep -qiE '^\*\*PR:\*\*.*MERGED|^\*\*Status:\*\*.*(MERGED|ARCHIVED)|^# .*ARCHIVED' "$gate"; then
      continue
    fi
    gate_active=1
  fi
done

msg=""
if [[ "$gate_active" -eq 1 && "$code_dirty" -eq 1 ]]; then
  msg="Git policy (PR gate): active gate + code changes. If the code cluster is ready and no open PR exists for this branch, commit the cluster and open one PR (prefer squash). Do not open a PR for handoffs/docs only. See docs/12-development.md."
elif [[ "$gate_active" -eq 1 && "$code_dirty" -eq 0 ]]; then
  # Handoffs/docs-only under a stale or leftover gate — do not ask for a PR.
  echo '{}'
  exit 0
elif [[ "${dirty_count:-0}" -ge 12 && "$code_dirty" -eq 1 ]]; then
  msg="Git policy: ${dirty_count} dirty paths including code. If this related code cluster is ready (or user asked for PR), commit with Task/Epic IDs and open one PR — not a handoff/docs-only PR. See docs/12-development.md."
fi

if [[ -z "$msg" ]]; then
  echo '{}'
  exit 0
fi

python3 -c 'import json,sys; print(json.dumps({"followup_message": sys.argv[1]}, ensure_ascii=False))' "$msg"
exit 0
