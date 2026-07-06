---
description: Commit staged changes in Conventional Commits format
allowed-tools: Bash(git status), Bash(git diff:*), Bash(git log:*), Bash(git add:*), Bash(git commit:*), Bash(git restore:*), Bash(cargo fmt:*), Bash(cargo clippy:*)
argument-hint: "[scope or extra note (optional)]"
---

# /commit

ステージ済み（または未ステージの関連）変更を **Conventional Commits 形式** でコミットしてください。

## 手順

1. `git status` と `git diff --staged`（未ステージがあれば `git diff` も）で変更内容を把握
2. コミット対象に含めてはいけないものが無いか確認（CLAUDE.md「Committing Changes with Git」参照: カバレッジ成果物、target/、temp/、IDE/OS ファイル等）
3. Rust コードに変更がある場合は CI 検証を先に実行:
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace -- -D warnings`
4. 論理的に独立した変更が複数あれば、**コミットを分ける**（例: 機能追加と無関係なフォーマット修正は別コミット）
5. 各コミットの subject を以下に従って作る:
   - 形式: `<type>(<scope>)?: <subject>`
   - type: `feat` | `fix` | `chore` | `docs` | `refactor` | `test` | `build` | `ci` | `perf` | `style` | `revert`
   - 1 文・72 文字以内・末尾ピリオド無し。本文が必要な変更（アルゴリズム・挙動変更）は heredoc で理由を書く
6. 最後に `git status` で結果を確認

## 制約

- **`Co-Authored-By: Claude` や `🤖 Generated with [Claude Code]` フッターは付けない**（`.claude/settings.json` で抑止済み）
- 追加の指示やメモ: `$ARGUMENTS`（指定があればそれを反映）

## 例

- `git commit -m "fix: keep final sentence boundary in multi-chunk mode"`
- scope 付き: `git commit -m "feat(cli): add --quiet flag"`
- Breaking change: `git commit -m "feat!: narrow public API surface"`

`.claude/hooks/validate-commit-msg.sh` がメッセージ形式を検証します。形式違反で hook が失敗したら、エラーメッセージに従って修正して再実行してください。
