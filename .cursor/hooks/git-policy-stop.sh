#!/usr/bin/env bash
# Nudge agents to commit after build / push at sprint gate when the tree is dirty.
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

# Explicit sprint-gate marker (PM sets this when closing the sprint).
sprint_gate=0
if [[ -f docs/handoffs/SPRINT-GATE.md ]] || git status --porcelain | grep -q 'SPRINT-GATE'; then
  sprint_gate=1
fi

# Fresh handoffs (build or QA just finished) — mtime within 30 minutes.
handoff_fresh=0
if find docs/handoffs -maxdepth 1 \( -name '*-dev-to-qa.md' -o -name '*-qa-to-pm.md' \) -mmin -30 2>/dev/null | grep -q .; then
  handoff_fresh=1
fi

msg=""
if [[ "$sprint_gate" -eq 1 ]]; then
  msg="Git policy (sprint gate): docs/handoffs/SPRINT-GATE.md present and tree is dirty. Commit leftovers with Task/Sprint IDs, then git push -u origin HEAD and gh pr create --base main. See docs/12-development.md."
elif [[ "$handoff_fresh" -eq 1 ]]; then
  msg="Git policy (post-build): working tree is dirty after a recent handoff under docs/handoffs/. Create the task commit now (git add + commit with Task ID). Do not push unless sprint gate. See docs/12-development.md."
else
  dirty_count="$(git status --porcelain | wc -l | tr -d ' ')"
  if [[ "${dirty_count:-0}" -ge 5 ]]; then
    msg="Git policy: ${dirty_count} dirty paths after agent turn. If you finished a Dev/UX build or PM Done, commit now with the Task ID. Push only at sprint end (SPRINT-GATE.md)."
  fi
fi

if [[ -z "$msg" ]]; then
  echo '{}'
  exit 0
fi

python3 -c 'import json,sys; print(json.dumps({"followup_message": sys.argv[1]}, ensure_ascii=False))' "$msg"
exit 0
