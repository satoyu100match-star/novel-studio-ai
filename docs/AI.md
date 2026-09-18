# AI.md — AI機能設計（Phase 5〜7）

## 0. 基本思想

> 「AIに全部書かせるアプリ」ではなく「作者を支援するアプリ」（最重要方針#20）。

- AI機能が利用不能でも、通常の小説執筆機能（Phase 0〜4の全機能）は完全に動作する
  こと（最重要方針#8）。AI関連コードは独立レイヤーとし、他レイヤーから
  AI Gatewayを直接参照させない（依存が逆転しないようにする）。
- AIが本文を書き換える場合、必ずユーザーの承認を挟む（最重要方針#19、後述4章）。
- 作品データは原則ローカル保存。AI機能利用時のみ、選択された文章・関連設定が
  選択中のAIプロバイダーへ送信される（仕様#81）。ユーザー作品を無断でクラウドへ
  アップロードしない（最重要方針#9）。

## 1. Provider Abstraction

```
UI (features/ai/*)
  → AI Feature (ユースケース: 推敲/続き/矛盾チェック等)
  → Context Builder
  → AI Gateway (Rust側、AiProviderトレイト)
  → Provider実装 (Anthropic / OpenAI / Gemini / OpenAI-Compatible / 将来Local AI)
```

Provider差し替えでアプリ本体（UI/Application Services/Domain）を書き直さずに
済むよう、Gateway境界を1本のトレイト（Rust）または1本のインターフェース
（フロントで完結させる場合はTS側）に固定する。設定画面の項目: AI Provider /
Model / API Key / Base URL / Temperature / Max Output（仕様#45）。

## 2. Context Builder / Context Inspector

AIに毎回作品全文を丸ごと渡さない。Context Builderが必要な情報だけを選択して
組み立てる。候補: System instructions / 作品概要 / 現在の章 / 現在のシーン /
関係キャラクター / 関係場所 / 関連世界設定 / 直前本文 / 関連伏線 / ユーザー質問
（仕様#41）。

**Context Inspector**（仕様#42）は、実際に送信される情報をユーザーが送信前に
確認・チェックボックスで取捨選択できる画面。プライバシーとAPI費用削減の両方に
効くため、AI機能のUIとほぼ必ずセットで提供する（「送信する」ボタンの手前に
必ずこの確認ステップを挟む設計とし、バイパスするショートカットを作らない）。

## 3. APIキーの扱い

- ソースコードに埋め込まない（最重要方針#11）。
- SQLiteへ平文保存しない。OS Credential Store（Windows: Credential
  Manager / macOS: Keychain / Linux: Secret Service）を`keyring` crate
  経由で利用する（Phase17）。Tauri側で管理しフロントには渡さない。
- Gitへcommitしない、ログへ表示しない、エラー画面へ全文表示しない（仕様#82）。

## 4. AIによる本文変更フロー（必ずこの順序）

```
変更前 → AI提案 → 差分表示 → [採用 / 部分採用 / 再生成 / コピー / 破棄]
```

AIが原稿を直接上書きすることは禁止（禁止事項リスト）。承認後の反映は
Undo/Redoスタックに乗る通常の編集操作として扱う（`docs/EDITOR.md` 4章）。

## 5. AIチャット・矛盾チェックの参照ソース

作品専用チャットの回答には、可能な範囲で根拠リンク（例: 「第3章 Scene 2」
「Character: Alice」「Timeline: 4/12」）を付け、クリックで該当項目へ遷移できる
ようにする（仕様#44, #106）。矛盾チェック（Phase 7）も同じContext Builder /
参照解決の仕組みを再利用する。

## 6. AI使用量表示

本日・今月のリクエスト数・推定トークンを可能な範囲で表示する。コスト不明な
モデルについては金額を無理に推測しない（仕様#46）——不正確な数値を断定的に
出さない。

## 7. テスト方針

Mock Providerを用意し（`src-tauri` 側でトレイトのテスト用実装、または
フロント側でGatewayをモック）、CI・通常のテスト実行では実APIキーを消費しない
（仕様#96）。実プロバイダーとの疎通確認は手動または専用の統合テストに限定する。

## 8. 現状 (Phase 5 完了)

Provider Abstraction（`src-tauri/src/ai/provider.rs`、Anthropic/OpenAI/
Gemini/OpenAI互換の4種）・Context Builder（`ai/context_builder.rs`）・
Context Inspector（`src/features/ai/ContextInspector.tsx`）・作品専用
チャット・AI使用量表示まで実装済み。Phase 6以降は個別のAI支援機能
（推敲・続き案・矛盾チェック等）をこの基盤の上に積み上げる。

### 8.1 既知の制約(重要)

この開発環境（クラウドサンドボックス）からcrates.ioへの新規クレート
取得がネットワーク障害（index.crates.ioへの新規パッケージ問い合わせが
継続的にタイムアウト）で行えなかったため、当初の設計から2点フォール
バックしている。ネットワーク制約が解消され次第、優先的に本来の設計へ
置き換えるべき「技術的負債」として明記する:

1. **HTTP通信**: `ureq`/`reqwest`等の専用クレートではなく、OS標準搭載の
   `curl`をサブプロセス起動して通信している
   （`src-tauri/src/ai/http_client.rs`）。APIキー等の機微ヘッダーは
   `curl -K -`（stdin経由の設定ファイル）でのみ渡し、コマンドライン
   引数には一切載せない設計にしているため、`ps`等でのAPIキー漏洩は
   防いでいる。Windows 10 (1803以降)/macOS/Linuxいずれも`curl`を標準
   搭載しているため追加インストールは不要という前提。
2. **APIキーの保存**: Phase5時点では`keyring`クレートが未導入のため、
   ディスクへの永続化を一切行わずアプリ実行中のみRust側メモリ上に
   保持する方式にしていた。**Phase17でネットワーク制約が解消され、
   `keyring`クレート経由でOS資格情報ストア（Windows Credential
   Manager / macOS Keychain / Linux Secret Service）に保存する本来の
   設計へ置き換え済み**（`src-tauri/src/ai/keystore.rs`）。自前の暗号化
   コードは一切書かず、各OSが提供する既存の安全な仕組みにそのまま
   委ねている。これにより、一度設定したAPIキーはアプリを再起動しても
   再入力不要になった。OS資格情報ストア自体が利用できない環境（D-Bus
   Secret Serviceが起動していないLinux環境等）では、保存・読み込みの
   失敗をログに警告として記録した上で、その場のセッション中のみメモリ
   上で動作を継続する（AI機能自体やアプリの起動を止めない）。

いずれもCI/この環境でのテストでは実APIキーを消費しない
（`ai::keystore`はユニットテストで直接検証済み。実際のOS資格情報ストア
との往復動作はCI/クラウドサンドボックスからは検証できないため、
Windows実機での確認が必要。実プロバイダーとの疎通は本サンドボックスの
ネットワーク許可リスト外のホストが多く未検証 — Windows実機検証と同様の
既知の制約）。
