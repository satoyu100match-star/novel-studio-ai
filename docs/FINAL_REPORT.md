# 最終レポート — Novel Studio AI (Phase 0〜11)

作成日: 2026-09-18

このドキュメントは、Phase 0から計画していた全Phase(0〜11)完了時点での
状態をまとめた最終レポート。各Phaseごとの詳細な完了報告は
`docs/ROADMAP.md`と`CHANGELOG.md`に記録済みで、このレポートはその
横断的な要約と、実運用前に必要な残作業の一覧を目的とする。

## 1. 完了したこと

Tauri 2 + React + TypeScript + Rust(SQLite)構成のAI小説制作IDEを、
以下の12Phase(0〜11)すべて完了した。

| Phase | 内容 | 状態 |
|---|---|---|
| 0 | プロジェクト基盤(Tauri起動・SQLite接続・マイグレーション基盤) | 完了 |
| 1 | 小説を書くための最小完成版(Project/Chapter/Scene/標準エディタ/自動保存/検索) | 完了 |
| 2 | 原稿用紙モード(縦書き/横書き/禁則処理/ルビ/傍点/印刷プレビュー) | 完了 |
| 3 | 小説設計(Characters/World Bible/Glossary/Locations/Notes/Tags) | 完了 |
| 4 | ストーリー管理(Plot Board/Timeline/Foreshadowing/TODO/本文内コメント) | 完了 |
| 5 | AI基盤(Provider Abstraction/Context Builder・Inspector/作品専用AIチャット/使用量表示) | 完了 |
| 6 | AI小説支援機能(推敲/続き候補/描写追加/会話改善/各種生成、全9機能) | 完了 |
| 7 | 高度AI分析(矛盾チェック等6種)・可読性分析(AI不要のローカル集計) | 完了 |
| 8 | Revision履歴/Crash Recovery/Auto Backup/ゴミ箱 | 完了 |
| 9 | Export(TXT/Markdown/HTML/DOCX/PDF/EPUB) | 完了 |
| 10 | 製品化(Onboarding/Sample Project/About/License/Privacy/Release build) | 完了(Auto Updateを除く) |
| 11 | Command Palette | 完了(他の高度機能は意図的にスコープ外) |

各Phaseは「実装 → `cargo build`/`cargo test` → `pnpm typecheck`/
`pnpm lint`/`pnpm test`/`pnpm build` → Xvfb実機起動での動作確認 → docs
更新(CLAUDE.md/ROADMAP.md/CHANGELOG.md) → git commit」という一貫した
手順で進め、途中でテストやビルドが失敗した状態のまま次のPhaseに進んだ
ことは一度もない。

## 2. 最終的な規模

- Rustバックエンド: 約7,500行(`src-tauri/src/`)
- フロントエンド: 約7,200行(`src/`、TypeScript/TSX)
- DBマイグレーション: 7ファイル(`0001_init` 〜 `0007_phase8_safety`、
  連番・追記専用)
- gitコミット: Phase単位で12コミット(`ba32199`〜`23ba7da`)、すべて
  Phase完了時点のみ(作業途中のコミットなし)

## 3. テスト結果(最終状態)

- `cargo test`: **52件、すべて成功**(リポジトリ層・AI関連ロジック・
  DB移行/バックアップ復元・Export各形式・サンプル作品生成など)
- `pnpm test`(Vitest): **31件、すべて成功**(原稿用紙組版ロジック・
  ストア・設定サービス・ErrorBoundary等)
- `pnpm typecheck` / `pnpm lint` / `pnpm build`: いずれもエラー・
  警告なし
- `cargo tauri build --bundles deb`: Linux `.deb`バンドルの生成に成功
  (`tauri.conf.json`のパッケージング設定が機能することを確認)

## 4. 現在動作する機能(要約)

「作者が1つのアプリから離れずに長編小説を完成させられる」という目標
に沿って、以下がすべて実データで動作する状態にある(ダミーUIなし):

- 執筆: Project/Part/Chapter/Scene階層、標準エディタ(自動保存・IME
  安全性・Undo/Redo)、原稿用紙ビュー(縦書き/横書き/禁則処理/ルビ/
  傍点/印刷プレビュー)、作品全体検索、本文内コメント
- 設計: Characters(仕様のほぼ全項目)、World Bible(ビルトイン20
  カテゴリ)、Glossary、Locations、Notes、Tags
- ストーリー管理: Plot Board(カンバン)、Timeline、Foreshadowing
  トラッカー(7ステータス)、TODO
- AI機能(すべて任意、未設定でも他機能は通常通り動作): 4プロバイダー
  対応のProvider Abstraction、Context Builder/Inspector(送信前確認を
  必須化)、作品専用AIチャット、9種の執筆支援機能(推敲/続き候補等)、
  6種の高度分析(矛盾チェック等)、AI不要のローカル可読性分析、AI使用量
  表示
- データ安全性: 章/シーン単位のバージョン履歴(自動+手動スナップ
  ショット、差分表示、復元)、Crash Recovery案内、DB全体のAuto Backup
  (`VACUUM INTO`、最新10件保持)、横断的なゴミ箱(Soft Delete済み
  レコードの一覧・復元・完全削除)
- Export: TXT/Markdown/HTML/DOCX/PDF(簡易・横書き、IPAゴシック埋め込み)
  /EPUBの6形式。原稿用紙PDF(縦書き・禁則処理・ルビ・傍点込みの本組版)
  は独立実装せず、既存の原稿用紙ビューの印刷プレビューに委ねる方針
- 製品化: 初回起動時のOnboarding、実データ入りのSample Project、
  「このアプリについて」画面(概要/更新履歴/ライセンス/プライバシー)、
  診断ログの手動書き出し(自動送信なし)
- Command Palette: Ctrl+K(Cmd+K)で作品一覧・各ワークスペースセクション
  ・章へキーボードだけでジャンプ

## 5. 意図的にスコープ外とした項目

仕様書自身が「初期販売版には必須ではない」と明示する以下の高度機能は、
現在のローカルSQLite単体構成から大きく外れるアーキテクチャ変更を要する
ため、今回は実装しなかった(詳細と理由: `docs/ROADMAP.md` Phase11):

- Semantic Search(埋め込みベクトル検索基盤が必要)
- Map(地図機能)
- Custom templates / Plugin architecture(外部コード読み込みの安全性
  検討が別途必要)
- Local AI(オンデバイスLLM)
- Cloud sync / Multi-device / Collaboration(サーバー基盤・同期
  プロトコル・権限モデルが必要で、「作品データは原則ローカル」という
  中核方針からの転換を伴う)

これらに着手する場合は、専用の設計ドキュメントを新設したうえで独立した
Phaseとして計画すべき。

## 6. 実運用前に対応が必要な既知の制約(重要度順)

本開発はLinuxクラウドサンドボックス内で行っており、以下はこの環境の
制約により未検証・未実装のまま残っている。実際のリリース作業に入る前に
対応すること(詳細はいずれも`CLAUDE.md`「既知の問題」に記録済み)。

### 最優先

1. **Windows実機検証が全体的に未実施**: `cargo build`によるLinux向け
   デバッグビルドとXvfb上での起動確認は行ったが、Windows実機(または
   CI)でのビルド・起動・Microsoft IME動作(仕様#94)は未検証。特に
   IMEは日本語入力アプリとして最優先で確認が必要。
2. **APIキーが非永続化**: `keyring`クレート(OS資格情報ストア連携)が
   Phase5当時のネットワーク障害で導入できず、APIキーはアプリ実行中の
   みメモリ保持(再起動のたびに再入力が必要)。**Phase9で確認した通り
   crates.ioへのネットワークは既に復旧しているため、次の作業として
   最優先で`keyring`クレートへの置き換えを検討すること。**
3. **HTTP通信がcurlサブプロセット経由**: 同じ理由で`ureq`/`reqwest`等の
   専用クレートではなく、OS標準の`curl`をサブプロセス起動する暫定実装
   (`src-tauri/src/ai/http_client.rs`)。これも同様にネットワーク復旧を
   受けて置き換えを検討すべき技術的負債。
4. **実AIプロバイダーとの疎通が未検証**: Anthropic/OpenAI/Gemini等との
   実際のAPI疎通は本サンドボックスのネットワーク許可リストの都合で
   未検証。特に高度AI分析(矛盾チェック等)は作品全文を送信するため、
   長編作品でのトークン上限抵触が実運用前に要確認。
5. **Auto Update未実装**: 署名鍵・更新サーバー・実配布環境が必要で、
   このサンドボックスでは検証も実装もできなかった。リリース運用開始前
   に必ず設計・実装が必要。

### 優先度中

6. Windows向けインストーラ(`.msi`等)の生成・署名は未検証(Linux
   `.deb`バンドルの生成には成功し、パッケージング設定自体が機能する
   ことは確認済み)。
7. PDF/DOCX/EPUB出力は`cargo test`でのバイト列非空確認・API整合性
   確認までで、実際にPDFビューア/Word/EPUBリーダーで開いて正しく表示
   されるかの目視確認は未実施。
8. E2Eテスト(Playwright等)は未導入。
9. 作品全体検索は現状LIKE検索のみ(データ量が増えた場合の性能は
   未検証、必要になった時点でFTS5等を検討)。
10. Command Palette・Onboarding等のGUI操作自体の目視確認は未実施
    (型検査・ビルド・Xvfb起動確認のみ)。

### 優先度低(意図的な設計判断、バグではない)

- 原稿用紙ビューの禁則処理は簡易実装(基本ルールのみ)
- バックアップは固定10世代、Revision同士の相互比較は無し
- 診断ログの書き出しは絞り込みなしでログフォルダ全体が対象

## 7. 次にやるべきこと(推奨順序)

1. Windows実機(または CI)でのビルド・起動・IME動作確認
2. `keyring`クレートへのAPIキー永続化の置き換え(データ安全性最優先の
   方針を保ちつつ利便性を上げる)
3. `reqwest`等へのHTTPクライアント置き換え
4. 実際のAPIキーでAnthropic/OpenAI/Gemini各プロバイダーとの疎通確認
5. Auto Updateの設計・実装(署名鍵・配布サーバーの用意を含む)
6. Windows向けインストーラの生成・署名、実機でのインストール確認
7. Export各形式(PDF/DOCX/EPUB)の実アプリでの目視確認
8. Command Palette・Onboarding等、Phase10〜11で追加したUIの実機での
   目視確認

## 8. 結論

「データ安全性 > 執筆体験 > 安定性 > 操作性 > パフォーマンス > AI機能
> 見た目」という優先順位を一貫して守りながら、計画していた全12Phase
(0〜11)を実装・検証・ドキュメント化・コミットまで完了した。ダミーの
UIやテストなしの大規模変更は一度も行っていない。実運用に進む前には、
本レポート6章に挙げたWindows実機検証とネットワーク関連の技術的負債
(APIキー永続化・HTTP通信・Auto Update)への対応が必須である。
