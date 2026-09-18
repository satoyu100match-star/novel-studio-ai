import { useEffect, useState } from "react";
import { EXPORT_FORMAT_LABELS } from "@/types/tauriCommands";
import * as exportService from "@/services/exportService";

/**
 * Phase 9: Export。「原稿を作品外へ持ち出したい」ニーズに応える画面
 * (仕様#39系)。ここで作るのは6形式(TXT/Markdown/HTML/DOCX/簡易PDF/EPUB)
 * で、いずれも読み込み専用のスナップショット書き出し -- 原稿データ自体
 * には一切書き込まない(AI機能等と違い、承認フローを挟む必要がない)。
 *
 * 「原稿用紙PDF」(縦書き・禁則処理・ルビ・傍点込みの本組版)は
 * `src-tauri/src/export/mod.rs`のモジュールコメント通り独立実装をせず、
 * 既存の原稿用紙ビューの印刷プレビューに委ねる方針のため、ここでは
 * その導線を案内するのみに留める。
 */
export function ExportPanel({ projectId }: { projectId: string }) {
  const [formats, setFormats] = useState<string[]>([]);
  const [busyFormat, setBusyFormat] = useState<string | null>(null);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    exportService.listExportFormats().then(setFormats);
  }, []);

  async function handleExport(format: string) {
    setBusyFormat(format);
    setMessage("");
    setError("");
    try {
      const output = await exportService.exportProject(projectId, format);
      exportService.downloadExportOutput(output);
      setMessage(`${output.filename} を書き出しました。`);
    } catch (err) {
      setError(`書き出しに失敗しました: ${String(err)}`);
    } finally {
      setBusyFormat(null);
    }
  }

  return (
    <div className="detail-form" style={{ maxWidth: 640 }}>
      <p className="status-line">
        作品全体(章・シーンの本文)を以下の形式で書き出せます。書き出しは原稿データを一切変更しません。
      </p>
      <ul className="entity-list">
        {formats.map((format) => (
          <li key={format} className="comment-card">
            <div className="comment-card__body">{EXPORT_FORMAT_LABELS[format] ?? format}</div>
            <div className="field-row">
              <button
                className="secondary"
                type="button"
                onClick={() => handleExport(format)}
                disabled={busyFormat !== null}
              >
                {busyFormat === format ? "書き出し中..." : "書き出す"}
              </button>
            </div>
          </li>
        ))}
      </ul>
      {message && <p className="status-line">{message}</p>}
      {error && <p className="status-line">{error}</p>}
      <p className="status-line">
        縦書き・禁則処理・ルビ・傍点を含む「原稿用紙PDF」が必要な場合は、原稿セクションの原稿用紙ビューを開き、
        印刷プレビューから「PDFとして保存」を選んでください(既に組版済みのビューをそのまま使うほうが確実なため、
        ここでは簡易な横書きPDFのみを扱っています)。
      </p>
    </div>
  );
}
