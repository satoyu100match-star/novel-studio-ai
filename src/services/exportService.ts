import * as api from "@/types/tauriCommands";
import type { ExportOutput } from "@/types/tauriCommands";
import { logger } from "@/utils/logger";

export async function listExportFormats(): Promise<string[]> {
  try {
    return await api.listExportFormats();
  } catch (err) {
    logger.error("listExportFormats failed", { err: String(err) });
    return [];
  }
}

export const exportProject = api.exportProject;

/**
 * `ExportOutput`のバイト列からBlobを作り、ブラウザのダウンロード動線
 * (`<a download>`のクリック)でファイルとして保存する。フォーマットを
 * 問わずこの1経路だけで扱えるのが`ExportOutput`を統一形にした狙い
 * (`src-tauri/src/export/mod.rs`のモジュールコメント参照)。
 */
export function downloadExportOutput(output: ExportOutput): void {
  const blob = new Blob([new Uint8Array(output.bytes)], { type: output.mime_type });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = output.filename;
  document.body.appendChild(anchor);
  anchor.click();
  document.body.removeChild(anchor);
  URL.revokeObjectURL(url);
}
