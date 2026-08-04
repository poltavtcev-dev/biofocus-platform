#!/usr/bin/env bash
# Nudge agents toward classic git: commit/PR for related work clusters.
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

dirty_count="$(git status --porcelain | wc -l | tr -d ' ')"

# Explicit PR / gate request marker (optional).
pr_gate=0
if [[ -f docs/handoffs/PR-GATE.md ]] || [[ -f docs/handoffs/SPRINT-GATE.md ]] \
  || git status --porcelain | grep -qE 'PR-GATE|SPRINT-GATE'; then
  pr_gate=1
fi

msg=""
if [[ "$pr_gate" -eq 1 ]]; then
  msg="Git policy (PR gate): gate marker present and tree is dirty. Commit the related cluster, then git push -u origin HEAD and gh pr create --base main (prefer squash merge). See docs/12-development.md."
elif [[ "${dirty_count:-0}" -ge 12 ]]; then
  msg="Git policy: ${dirty_count} dirty paths. If this related work cluster is ready (or user asked for PR), commit with Task/Epic IDs and open a PR — do not wait for a full sprint. See docs/12-development.md."
fi

if [[ -z "$msg" ]]; then
  echo '{}'
  exit 0
fi

python3 -c 'import json,sys; print(json.dumps({"followup_message": sys.argv[1]}, ensure_ascii=False))' "$msg"
exit 0
