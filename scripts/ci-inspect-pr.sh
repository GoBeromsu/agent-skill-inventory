#!/usr/bin/env bash
set -euo pipefail

REPO="."
PR="${PR:-}"
JSON_FLAG=""
MAX_LINES="${MAX_LINES:-200}"
CONTEXT="${CONTEXT:-40}"

while [[ $# -gt 0 ]]; do
  if [[ "$1" == "--" ]]; then
    shift
    continue
  fi

  case "$1" in
    --repo)
      REPO="$2"
      shift 2
      ;;
    --pr)
      PR="$2"
      shift 2
      ;;
    --json)
      JSON_FLAG="--json"
      shift
      ;;
    --max-lines)
      MAX_LINES="$2"
      shift 2
      ;;
    --context)
      CONTEXT="$2"
      shift 2
      ;;
    --help)
      cat <<'USAGE'
Usage: pnpm run ci:inspect-pr [--repo <path>] [--pr <number|url>] [--json] [--max-lines N] [--context N]

Environment:
  PR: PR number or URL (defaults to current branch PR if available).
  MAX_LINES, CONTEXT: override default log extraction limits.
USAGE
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 2
      ;;
  esac
done

if [[ -z "$PR" ]]; then
  if [[ -d .git ]]; then
    PR="$(gh pr view --json number -q .number)"
  fi
fi

if [[ -z "$PR" ]]; then
  echo "No PR specified. Pass --pr or set PR env var." >&2
  exit 2
fi

python "$HOME/.codex/skills/gh-fix-ci/scripts/inspect_pr_checks.py" \
  --repo "$REPO" \
  --pr "$PR" \
  --max-lines "$MAX_LINES" \
  --context "$CONTEXT" \
  ${JSON_FLAG}
