---
description: Tidy up after a task wraps — sync main, prune merged local branches, update task tracker
allowed-tools: Bash(git status), Bash(git status:*), Bash(git fetch:*), Bash(git checkout:*), Bash(git pull:*), Bash(git branch), Bash(git branch:*), Bash(git log:*), Bash(git worktree:*), Bash(git rev-parse:*), Bash(gh pr:*), Bash(gh api:*)
argument-hint: "[--dry-run | --no-status-update | --keep-branches | <task-file-path-or-notion-url>]"
---

# /wrap-up

タスク（多くの場合 1 PR）を終えたあとの片付けを 1 コマンドで完了させる。

スコープ:
1. **main をローカル反映**
2. **マージ済みローカルブランチ・関連 worktree の削除**
3. **タスクトラッカー（Notion ページ or Markdown ファイル、あれば）のステータス更新**
4. **残 follow-up のサマリ**

`git push` などリモートに影響する操作は **行わない**。ローカル整理専用。

## 手順

### 1. 状況確認

1. `git status` で未コミット変更を確認。dirty なら停止して報告のみ（取りこぼしの可能性）
2. `git rev-parse --abbrev-ref HEAD` で現在ブランチを把握
3. `git worktree list` で worktree 構成を把握（複数 worktree なら main がどこに checkout されているか）
4. `git fetch origin --prune` でリモート参照を最新化（リモートで自動削除されたブランチ参照もここで消える）

### 2. main をローカル反映

現在地に応じて分岐:

- **main worktree にいる場合**: `git pull --ff-only origin main`
- **main 以外（feature 等）の worktree にいる場合**: `git fetch origin main:main`（local の main 参照を早送り。non-fast-forward なら失敗 → 報告）
- どちらも失敗したら停止し、選択肢（手動 rebase / 別 worktree からやり直す等）を提示して user に判断を委ねる

### 3. マージ済みローカルブランチを掃除

1. `git branch --merged main` でマージ済み一覧を取得し、`main` / 現在ブランチ / 行頭 `*` を除外
2. 各ブランチを `git branch -d <name>` で削除
   - **`-D`（強制削除）は使わない**（settings.json で deny 済み）
   - 削除拒否されたら理由（unmerged commits 等）を控えて skip
3. `git worktree list` で削除済みブランチに紐づく worktree があれば `git worktree remove <path>` で除去
   - **現在いる worktree は除外**
   - dirty な worktree は force せず警告のみ

### 4. タスクトラッカー（Notion ページ or Markdown ファイル）があれば状態を更新

プロジェクトの進捗管理は **Notion ページ** または **Markdown ファイル** のどちらかで行われている。**形式・ファイル名は固定しない**。

1. **対象トラッカーの特定**
   - **このセッション内でタスク作業をしていた場合**（最も多いケース）: 直前までの会話文脈から、形式（Notion / Markdown）と対象リソース（Notion ページ URL / Markdown ファイルパス）を読み取る。セッション中に参照・編集していたものをそのまま使うのが最優先
   - `$ARGUMENTS` にパス or Notion URL が指定されていればそれを優先
   - 文脈にもセッションにも手がかりが無いとき（フォールバック）:
     - **Markdown 候補**: `temp/` / `docs/` / ルート直下の `*.md` を見て、**PR / Issue / Task の進捗表 or ステータスマーカーを含むファイル**を探す（ファイル名はリポによって `prs.md` / `TODO.md` / `ROADMAP.md` / `tasks.md` 等まちまち）
     - **Notion 候補**: `notion-search` 等で関連ページを探す
   - 候補が複数 or 不明確なら user に 1 度確認する
   - **見当たらなければ skip**（このコマンドで新規作成はしない）
2. **そのトラッカーの流儀を読む**（ここが重要）
   - **Markdown の場合**: ファイル冒頭に **凡例 / ステータス定義** が書かれていることが多い（例: 「ステータス凡例: ✅ merged / 🚧 open / ⏸ draft / ⬜ pending」）— **必ず確認し、その記号体系に従う**。行のカラム構造（PR 番号 / リンク / ブランチ名 / 備考）も既存行を真似る
   - **Notion の場合**: データベースなら **Status プロパティの選択肢**、ページなら冒頭の凡例 / 既存行の表現を `notion-fetch` で取得して、その体系に従う
   - **このコマンドが固定の絵文字やステータス記号を持ち込まない**。トラッカーの流儀が最優先
3. **直近マージ済み PR を取得**
   - `gh pr list --state merged --base main --limit 20 --json number,title,mergedAt,headRefName,url`
4. **更新を起こす**
   - マージ済みになった行 / レコードのステータスを、そのトラッカーの記法で「マージ済み」に更新
   - PR 番号 / URL / ブランチ名のプレースホルダが残っていれば埋める
   - "follow-up" / "宿題" / "TODO" 系セクションは状態の整理だけに留め、内容の判断（消化扱いにするか等）が必要なら user に確認
5. 書き込む前に変更内容を提示して確認
   - **Markdown**: `Edit` で書き込む前に diff を提示
   - **Notion**: 更新するプロパティ / ブロックの内容を提示してから `notion-update-page` 等の MCP ツールで反映（権限プロンプトはそこで承認）

### 5. サマリ報告

簡潔に以下を出す:

- main に取り込んだ commit 件数（`git log <prev>..main --oneline | wc -l`）
- 削除したブランチ / 削除できなかったブランチ（理由付き）
- 削除した worktree
- 更新したトラッカー（Notion ページ / Markdown ファイル）の該当箇所（skip 時はその旨）
- **残っている follow-up / 宿題**（トラッカーから抽出してリスト化）— 次セッションへの引き継ぎ材料

## 引数（`$ARGUMENTS`）

- `--dry-run`: 何をやるかだけ列挙し、実際の変更（branch 削除 / ファイル書き込み / Notion 更新）はしない
- `--no-status-update`: タスクトラッカーの更新を skip
- `--keep-branches`: ローカルブランチ削除を skip
- 任意のパス or Notion URL（例: `temp/prs.md` / `https://www.notion.so/...`）: トラッカーの自動検出・セッション文脈推定を skip し、明示指定したものを使う

未指定なら全ステップ実行。

## 制約

- **destructive 操作は最小限**
  - `git branch -d` のみ使用（`-D` 禁止）
  - `git worktree remove` は force 無し
  - **`git push` / `git reset --hard` / `git rebase` は本コマンドでは実行しない**
- タスクトラッカー（Notion / Markdown）が見当たらないときは **新規作成しない**
- 絵文字 / ステータス記号 / セクション構造は **そのトラッカーの既存規約に従う**（コマンド側で標準を強制しない）
- 不確実な場面（候補が複数 / main pull 失敗 / ブランチ削除拒否 / セッション文脈で対象が読み取れない）は **停止して user に判断を委ねる**
