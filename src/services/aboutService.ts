import * as api from "@/types/tauriCommands";
import { downloadExportOutput } from "@/services/exportService";
import { logger } from "@/utils/logger";

export async function getChangelog(): Promise<string> {
  try {
    return await api.getChangelog();
  } catch (err) {
    logger.error("getChangelog failed", { err: String(err) });
    return "";
  }
}

export async function getThirdPartyNotices(): Promise<string> {
  try {
    return await api.getThirdPartyNotices();
  } catch (err) {
    logger.error("getThirdPartyNotices failed", { err: String(err) });
    return "";
  }
}

export async function getPrivacyNotice(): Promise<string> {
  try {
    return await api.getPrivacyNotice();
  } catch (err) {
    logger.error("getPrivacyNotice failed", { err: String(err) });
    return "";
  }
}

/**
 * 診断ログをZIPとして書き出し、そのままダウンロードさせる。自動送信は
 * 一切行わない(PRIVACY.md参照) -- 書き出したファイルをどう使うかは
 * 常にユーザー自身の判断。
 */
export async function exportDiagnostics(): Promise<void> {
  const output = await api.exportDiagnostics();
  downloadExportOutput(output);
}
