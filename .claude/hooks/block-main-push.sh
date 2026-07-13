#!/bin/bash
# PreToolUse hook for Bash: block direct pushes to main/master.
# Enforces the PR-required branch strategy (see CONTRIBUTING.md).
# Exits 0 to allow, 2 to block (stderr fed back to the coding agent).

set -euo pipefail

INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty')

# Target only `git push`.
if ! echo "$COMMAND" | grep -qE '(^|[[:space:]]|;|&&|\|\|)git[[:space:]]+push'; then
  exit 0
fi

# --- Explicit push target: main / master ---
if echo "$COMMAND" | grep -qE 'git[[:space:]]+push([[:space:]]+[^[:space:]]+)?[[:space:]]+(HEAD:)?(main|master)([[:space:]]|$)'; then
  cat >&2 <<MSG
[hook/block-main-push] Direct push to main/master is forbidden.

  Command: $COMMAND

Create a feature branch and open a PR:
  git checkout -b feature/<topic>
  git push -u origin HEAD
  gh pr create
MSG
  exit 2
fi

# --- Current branch is main/master and a bare push was requested ---
PROJECT_DIR=$(echo "$INPUT" | jq -r '.cwd // empty')
PROJECT_DIR="${PROJECT_DIR:-${CLAUDE_PROJECT_DIR:-.}}"
CURRENT_BRANCH=$(git -C "$PROJECT_DIR" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")

if [[ "$CURRENT_BRANCH" == "main" || "$CURRENT_BRANCH" == "master" ]]; then
  # Allow tag pushes — they don't move the main/master branch ref. Recognized
  # forms (covers both `git push origin <tag>` and the explicit `tag <name>` /
  # `--tags` / `refs/tags/...` variants):
  #   git push --tags [...]
  #   git push [remote] refs/tags/<name>
  #   git push [remote] tag <name>
  #   git push [remote] vX.Y.Z          (semver tag convention used by this repo)
  if echo "$COMMAND" | grep -qE '(^|[[:space:]])--tags([[:space:]]|$)|refs/tags/|[[:space:]]tag[[:space:]]+[^[:space:]]+|[[:space:]]v[0-9]+\.[0-9]+\.[0-9]+([[:space:]]|$)'; then
    exit 0
  fi
  cat >&2 <<MSG
[hook/block-main-push] Current branch is '$CURRENT_BRANCH' — push is forbidden.

  Command: $COMMAND

Switch to a feature branch first:
  git checkout -b feature/<topic>
MSG
  exit 2
fi

exit 0
