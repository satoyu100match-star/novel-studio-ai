# Novel Studio AI

AIを活用した小説制作専用デスクトップIDE（開発中・Phase 0）。執筆・原稿用紙・
キャラクター・世界観・プロット・時系列・伏線・資料・AI相談・矛盾チェック・
推敲・Exportを1つに統合することを目標にしています。

詳しい設計方針は `CLAUDE.md` と `docs/` を参照してください。

## 現在の状態

Phase 0（プロジェクト基盤）が完了した段階です。アプリの起動、SQLiteの
読み書き、テーマ切替の永続化までが動作します。小説を実際に書く機能は
Phase 1以降で追加されます。

## 必要環境

- Node.js 20+ / pnpm 9+
- Rust (stable) + Cargo
- Windows: Tauri公式ドキュメントの「Prerequisites」に従いWebView2等をセットアップ
- Linux (開発確認用): `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `librsvg2-dev`,
  `libssl-dev`, `libayatana-appindicator3-dev`, `libxdo-dev` 等

## セットアップ

```bash
pnpm install
```

## 開発サーバー起動

```bash
pnpm tauri dev
```

## テスト

```bash
pnpm typecheck
pnpm lint
pnpm test
cd src-tauri && cargo test
```

## ビルド

```bash
pnpm build              # フロントエンドのみ
cd src-tauri && cargo build   # Rustバックエンド込みの開発ビルド
pnpm tauri build         # 配布用バンドル（Phase 10でWindowsインストーラ含め本格対応）
```

## ディレクトリ構成

`docs/ARCHITECTURE.md` を参照してください。
