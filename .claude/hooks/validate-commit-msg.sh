#!/bin/bash
# PreToolUse hook for Bash: validate that git commit messages follow
# Conventional Commits format (see CONTRIBUTING.md).
# Exits 0 to allow, 2 to block (stderr fed back to Claude Code).

set -euo pipefail

INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty')

# Target only `git commit`. Other bash commands pass through.
if ! echo "$COMMAND" | grep -qE '(^|[[:space:]]|;|&&|\|\|)git[[:space:]]+commit'; then
  exit 0
fi

# Skip amend / fixup / squash — these modify existing commits.
if echo "$COMMAND" | grep -qE -- '(--amend|--fixup|--squash)'; then
  exit 0
fi

# --- Extract subject from -m ... ---
SUBJECT=""

# Double-quoted: -m "..."
SUBJECT=$(echo "$COMMAND" | grep -oE -- '-m[[:space:]]+"[^"]+"' | head -1 | sed -E 's/^-m[[:space:]]+"(.*)"$/\1/' || true)

# Single-quoted: -m '...'
if [[ -z "$SUBJECT" ]]; then
  SUBJECT=$(echo "$COMMAND" | grep -oE -- "-m[[:space:]]+'[^']+'" | head -1 | sed -E "s/^-m[[:space:]]+'(.*)'\$/\\1/" || true)
fi

# Heredoc: $(cat <<'EOF' ... EOF) — take first line inside the heredoc.
if [[ -z "$SUBJECT" ]]; then
  SUBJECT=$(echo "$COMMAND" \
    | awk "/cat[[:space:]]*<<-?[[:space:]]*'?EOF'?/{flag=1;next} /^[[:space:]]*'?EOF'?[[:space:]]*\$/{flag=0} flag" \
    | head -1 || true)
fi

# Could not extract (using editor or --file <path>). Allow through.
if [[ -z "$SUBJECT" ]]; then
  exit 0
fi

FIRST_LINE=$(echo "$SUBJECT" | head -1)

# --- Validate format ---
TYPE_RE='^(feat|fix|chore|docs|refactor|test|build|ci|perf|style|revert)(\([a-z0-9_/-]+\))?!?: .+'

if ! echo "$FIRST_LINE" | grep -qE "$TYPE_RE"; then
  cat >&2 <<MSG
[hook/validate-commit-msg] Commit subject is not a valid Conventional Commit.

  Got:      $FIRST_LINE
  Expected: <type>(<scope>)?: <subject>
  Types:    feat | fix | chore | docs | refactor | test | build | ci | perf | style | revert

See CLAUDE.md and CONTRIBUTING.md for details.
MSG
  exit 2
fi

LEN=${#FIRST_LINE}
if (( LEN > 72 )); then
  cat >&2 <<MSG
[hook/validate-commit-msg] Commit subject exceeds 72 characters (got: $LEN).

  Subject: $FIRST_LINE

Keep the subject concise.
MSG
  exit 2
fi

if [[ "${FIRST_LINE: -1}" == "." ]]; then
  cat >&2 <<MSG
[hook/validate-commit-msg] Commit subject should not end with a period.

  Subject: $FIRST_LINE
MSG
  exit 2
fi

exit 0
