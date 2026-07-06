#!/bin/bash
# PreToolUse hook for Bash: validate gh pr create/edit title and body.
#   - Title: must follow Conventional Commits.
#   - Body:  must contain the core sections of .github/PULL_REQUEST_TEMPLATE.md.
# Exits 0 to allow, 2 to block (stderr fed back to Claude Code).

set -euo pipefail

INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty')

# Target only `gh pr create` / `gh pr edit`.
if ! echo "$COMMAND" | grep -qE 'gh[[:space:]]+pr[[:space:]]+(create|edit)'; then
  exit 0
fi

# --- Title extraction ---
TITLE=""

# --title "..."
TITLE=$(echo "$COMMAND" | grep -oE -- '--title[[:space:]]+"[^"]+"' | head -1 | sed -E 's/^--title[[:space:]]+"(.*)"$/\1/' || true)
# --title '...'
if [[ -z "$TITLE" ]]; then
  TITLE=$(echo "$COMMAND" | grep -oE -- "--title[[:space:]]+'[^']+'" | head -1 | sed -E "s/^--title[[:space:]]+'(.*)'\$/\\1/" || true)
fi
# -t "..."
if [[ -z "$TITLE" ]]; then
  TITLE=$(echo "$COMMAND" | grep -oE -- '-t[[:space:]]+"[^"]+"' | head -1 | sed -E 's/^-t[[:space:]]+"(.*)"$/\1/' || true)
fi
# -t '...'
if [[ -z "$TITLE" ]]; then
  TITLE=$(echo "$COMMAND" | grep -oE -- "-t[[:space:]]+'[^']+'" | head -1 | sed -E "s/^-t[[:space:]]+'(.*)'\$/\\1/" || true)
fi

TYPE_RE='^(feat|fix|chore|docs|refactor|test|build|ci|perf|style|revert)(\([a-z0-9_/-]+\))?!?: .+'

if [[ -n "$TITLE" ]]; then
  if ! echo "$TITLE" | grep -qE "$TYPE_RE"; then
    cat >&2 <<MSG
[hook/validate-pr] PR title is not a valid Conventional Commit.

  Got:      $TITLE
  Expected: <type>(<scope>)?: <subject>
  Types:    feat | fix | chore | docs | refactor | test | build | ci | perf | style | revert

See CLAUDE.md and CONTRIBUTING.md for details.
MSG
    exit 2
  fi
fi

# --- Body validation ---
# If no explicit body was passed (neither --body / -b / --body-file), gh will
# fall back to .github/PULL_REQUEST_TEMPLATE.md or an editor. Skip validation
# in that case — the template file already has the required sections.
if ! echo "$COMMAND" | grep -qE -- '(--body([[:space:]]|=)|--body-file|(^|[[:space:]])-b[[:space:]])'; then
  exit 0
fi

BODY=""
BODY_FILE=$(echo "$COMMAND" | grep -oE -- '--body-file[[:space:]]+[^[:space:]]+' | head -1 | awk '{print $2}' | tr -d '"' | tr -d "'" || true)

if [[ -n "$BODY_FILE" ]] && [[ -f "$BODY_FILE" ]]; then
  BODY=$(cat "$BODY_FILE")
else
  # --body "..." or heredoc — scan the entire command string.
  BODY="$COMMAND"
fi

MISSING=()
for SECTION in "## Summary" "## Type of Change" "## Changes Made" "## How Has This Been Tested?"; do
  if ! echo "$BODY" | grep -qF "$SECTION"; then
    MISSING+=("$SECTION")
  fi
done

if (( ${#MISSING[@]} > 0 )); then
  cat >&2 <<MSG
[hook/validate-pr] PR body is missing required section(s): ${MISSING[*]}

PR bodies must follow .github/PULL_REQUEST_TEMPLATE.md and contain at least:
  ## Summary
  ## Type of Change
  ## Changes Made
  ## How Has This Been Tested?

See CLAUDE.md "Pull Request Guidelines" for details.
MSG
  exit 2
fi

exit 0
