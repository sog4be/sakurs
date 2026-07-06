---
description: Push the current branch and open a PR following the repository template
allowed-tools: Bash(git status), Bash(git diff:*), Bash(git log:*), Bash(git push:*), Bash(gh pr:*), Bash(gh repo view:*)
argument-hint: "[additional context (optional)]"
---

# /pr

現在のブランチを push し、**Conventional Commits タイトル**と **`.github/PULL_REQUEST_TEMPLATE.md` 構成の本文** で Pull Request を作成してください。

## 手順

1. `git status` で push 状態と未コミット変更を確認
2. `git log main..HEAD --oneline` でこの PR に含まれるコミットを把握
3. `git diff main...HEAD` で変更の実質を把握
4. 必要ならブランチを upstream に push: `git push -u origin HEAD`
5. PR タイトルを決める:
   - Conventional Commits 形式（コミットと同じ type 体系）
   - 代表的な変更を 1 文で。PR が複数 type の変更を含む場合、最も支配的なものに合わせる（またはブランチを分ける）
6. PR 本文は **`.github/PULL_REQUEST_TEMPLATE.md` の全セクションを埋める**（CLAUDE.md「PR Template Compliance Checklist」参照）:
   - 必須: `## Summary` / `## Type of Change` / `## Changes Made` / `## How Has This Been Tested?`
   - チェックボックスは該当項目を `[x]` にする。該当しない項目は `[ ]` のまま理由を書く
   - work ブランチ（`work/*`）からの PR は base を親 feature ブランチにする（CONTRIBUTING.md 参照）
7. `gh pr create --title "..." --body "$(cat <<'EOF' ... EOF)"` で作成
8. 最後に PR URL を返す

## 本文のバッククォート（重要）

PR 本文にコードスパン（`` `foo` ``）を含める場合、**heredoc デリミタは必ずシングルクォートで囲む** (`<<'EOF'`)。

- `<<EOF`（クォート無し）や `--body "..."` に直接書くと、シェルが `` ` `` をコマンド置換として解釈する
- 必ず `<<'EOF' ... EOF` でラップする。もしくは `--body-file <path>` を使う
- **PR 本文に `\`` を書かない**。書いている時点でエスケープミス

## 制約

- **Co-Authored-By / Generated with Claude Code フッターは付けない**
- 他 PR を参照する時は markdown link ではなく素の URL を貼る（GitHub が inline mention にレンダリングする）
- 追加メモ: `$ARGUMENTS`

## Hook との協調

`.claude/hooks/validate-pr.sh` がタイトル形式と本文セクションの存在をチェックします。hook が exit 2 で止まったらエラー出力を読んで修正してください。
