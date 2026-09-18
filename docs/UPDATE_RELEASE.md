# UPDATE_RELEASE.md — 自動アップデートの仕組みとリリース手順

Phase15で追加した「Auto Update」機能の設計と、新しいバージョンを
リリースする手順をまとめる。実装の背景・動作確認状況は
`CHANGELOG.md`[Phase 15]と`docs/ROADMAP.md`も参照。

## 1. アプリ側の仕組み(実装済み)

- `tauri-plugin-updater` + `tauri-plugin-process`(再起動用)を導入
  (`src-tauri/src/lib.rs`、デスクトップのみ・`cfg(desktop)`)。
- 起動のたびに一度だけ、バックグラウンドで新バージョンの有無を確認する
  (`src/features/update/UpdateBanner.tsx`)。見つかった場合だけ、画面上部に
  Crash Recoveryバナー(Phase8)と同じ見た目の案内を表示する。確認自体が
  失敗しても(オフライン等)、通常の執筆機能には一切影響しない。
- ダウンロード・適用・再起動は、必ずバナーの「今すぐ更新」ボタンを押した
  時だけ実行される(バックグラウンドで勝手に更新が当たることはない --
  CLAUDE.md優先順位#1「データ安全性」、執筆中に勝手に再起動されて内容を
  見失う不安を避けるため)。
- 「このアプリについて」→「概要」タブに、いつでも手動で確認できる
  「アップデートを確認」ボタンも用意した(`src/features/about/
  AboutModal.tsx`)。
- 呼び出しは`src/services/updateService.ts`の1箇所に閉じ込め、featureや
  コンポーネントから`@tauri-apps/plugin-updater`を直接importしない
  (CLAUDE.md「featureやコンポーネントから`@tauri-apps/api`を直接
  importしない」方針を踏襲)。

## 2. 署名鍵(重要 -- 絶対に無くさない・公開しない)

アップデート用の鍵ペアはこのセッションで生成済み:

- **秘密鍵**: `novel-studio-ai-update.key` というファイル名でお渡しした
  もの。これを使って新しいリリースに署名する。**紛失すると今後
  アップデートを配信できなくなる。他人に見せたり、Gitへコミットしたり
  しないこと**(`.gitignore`に`*.key`を追加済みで、誤ってコミットする
  経路は塞いである)。パスワードマネージャー等、安全な場所に保管する。
- **公開鍵**: `src-tauri/tauri.conf.json`の`plugins.updater.pubkey`に
  埋め込み済み。アプリ本体に同梱され、ダウンロードした更新ファイルが
  この秘密鍵で正しく署名されたものかどうかの検証に使われる(公開鍵は
  秘密ではないので、リポジトリに含めて問題ない)。

## 3. 配信先(エンドポイント) -- 設定済み

`tauri.conf.json`の`plugins.updater.endpoints`は以下の通り確定済み:

```json
"endpoints": ["https://github.com/satoyu100match-star/novel-studio-ai/releases/latest/download/latest.json"]
```

つまりGitHubの`satoyu100match-star/novel-studio-ai`というリポジトリの
Releasesページに`latest.json`が置かれることが前提になる。このリポジトリが
まだ存在しない場合は先に作成し(公開/非公開どちらでも動作する)、下記4章の
手順で最初のリリースを1つ公開するまでは、アプリのアップデート確認は
「404」相当の応答になり「更新なし」と同じ挙動になる(通常の起動・執筆
機能には影響しない)。

## 4. 新しいバージョンをリリースする手順

### 4-A. GitHub Actionsを使う方法(推奨)

`.github/workflows/release.yml`を用意済み。`v*`形式のタグ(例:
`v0.1.1`)をpushすると、GitHub純正のWindowsランナー上で自動的に
`pnpm tauri build`相当の処理が走り、署名付きの成果物一式と
`latest.json`を生成して、GitHub Releaseの下書き(Draft)として公開する。
**あなたのパソコンでビルドし直す必要が無くなる**のが最大の利点。

事前準備(初回のみ、GitHubのリポジトリ設定で):

1. リポジトリの Settings → Secrets and variables → Actions を開く
2. `TAURI_SIGNING_PRIVATE_KEY`という名前で、`novel-studio-ai-update.key`
   ファイルの中身(全文)をそのまま貼り付けて保存
3. `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`という名前で、鍵のパスワード
   (パスワード無しで生成したので空文字列)を保存

リリース手順(バージョンを上げるたび):

1. `package.json`と`src-tauri/tauri.conf.json`の`version`を上げる
   (例: `0.1.0` → `0.1.1`)
2. `CHANGELOG.md`に変更内容を追記
3. コミットしてpush
4. `git tag v0.1.1 && git push origin v0.1.1`
5. GitHubのActionsタブでビルドが完了するのを待つ(数分)
6. GitHubのReleasesページに下書き(Draft)ができているので、内容を確認して
   「Publish release」を押す(**あえて自動公開にせず下書き止まりにして
   ある**。誤って壊れたビルドを全ユーザーに配ってしまう事故を防ぐため)
7. 公開すると、既存ユーザーのアプリが次回起動時(または「アップデートを
   確認」ボタン)で気づけるようになる

### 4-B. 手元のパソコンで手動ビルドする方法

CIを使わず、今まで通り自分のパソコンでビルドしてGitHubへアップロード
することもできる。

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content -Raw "秘密鍵ファイルのパス"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
pnpm tauri build
```

`src-tauri/target/release/bundle/`に、通常のインストーラー(`.exe`/
`.msi`)に加えて、アップデート用の成果物(`.sig`署名ファイル等、
`createUpdaterArtifacts: true`設定により生成される)ができる。これらと
`latest.json`(バージョン・日付・各プラットフォームのダウンロードURLと
署名を記述したJSON)を手動でGitHub Releaseへアップロードする。
`latest.json`を手で組み立てる必要があるため、4-Aの方法より手間が多い
(`tauri-action`はこのファイルの生成も自動化してくれる)。

## 5. 動作確認状況(既知の制約)

- このサンドボックス環境では、実際にGitHub Releaseを介した更新の
  ダウンロード・適用・再起動までは検証できていない(署名鍵の生成と、
  プラグインの組み込み・`cargo build`・Xvfb起動確認までは実施済み)。
  実際にリリースを1つ公開し、旧バージョンのアプリから更新できることを
  確認するまでは、既知の制約として扱う。
- GitHub Actionsのワークフロー(`.github/workflows/release.yml`)自体も
  実際に一度動かして確認するまでは未検証。Windowsランナー上での
  `cargo`/`pnpm`のセットアップは`dtolnay/rust-toolchain`と
  `pnpm/action-setup`という広く使われている公式アクションに任せている
  ため大きな問題は起きにくいはずだが、初回実行時にエラーが出たら
  ログを確認すること。
