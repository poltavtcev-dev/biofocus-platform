#!/usr/bin/env bash
# Opt in to the small local interpreter (Ollama llama3.2:1b).
# Does not start BioFocus and does not call the model.
# The model only paraphrases an already-computed report when you press
# Generate report. It does not compute Features or Insights.
set -euo pipefail

MODEL="llama3.2:1b"
HOME_DIR="${BIOFOCUS_HOME:-${HOME}/.biofocus}"
FLAG="${HOME_DIR}/local_llm_enabled"

if [[ -n "${BIOFOCUS_LOCAL_LLM_BASE_URL:-}" && "${BIOFOCUS_LOCAL_LLM_BASE_URL}" != http://127.0.0.1:11434* && "${BIOFOCUS_LOCAL_LLM_BASE_URL}" != http://localhost:11434* ]]; then
  echo "This shell already points BioFocus at a non-local model endpoint."
  echo "Unset BIOFOCUS_LOCAL_LLM, BIOFOCUS_LOCAL_LLM_BASE_URL, BIOFOCUS_LOCAL_LLM_MODEL,"
  echo "BIOFOCUS_LOCAL_LLM_API_KEY, and BIOFOCUS_LOCAL_LLM_TIMEOUT_SECS before starting Desktop."
  echo "Otherwise Generate report still sends the prompt to that endpoint, not to Ollama."
  echo
fi

if ! command -v ollama >/dev/null 2>&1; then
  echo "Ollama is not installed."
  echo "Install it from https://ollama.com then re-run: scripts/enable-local-ai.sh"
  exit 1
fi

echo "Pulling ${MODEL} (small local model, about 1 GB)…"
ollama pull "${MODEL}"

mkdir -p "${HOME_DIR}"
printf '1\n' > "${FLAG}"

echo
echo "Local AI opt-in written to ${FLAG}"
echo "Keep Ollama running (the Ollama app, or: ollama serve)."
echo "Restart BioFocus Desktop, then press Generate report."
echo "The model only interprets the offline report. It does not diagnose."
