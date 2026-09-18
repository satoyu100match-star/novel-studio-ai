import { isTauriRuntime } from "@/services/runtime";
import { logger } from "@/utils/logger";

/**
 * Phase15: Auto Update。`@tauri-apps/plugin-updater`をこのサービス
 * 1箇所に閉じ込め、featureやコンポーネントから直接importさせない
 * (CLAUDE.md「featureやコンポーネントから`@tauri-apps/api`を直接
 * importしない」方針をプラグインAPIにも適用)。
 *
 * 更新の確認自体は起動時に自動で行うが、実際のダウンロード・適用・
 * 再起動は必ずユーザーの明示的なボタン操作を経てから行う(バック
 * グラウンドで勝手に更新を当てて執筆中の内容を巻き込むことを避ける
 * ため -- CLAUDE.md優先順位#1「データ安全性」)。
 */

export interface AvailableUpdate {
  version: string;
  currentVersion: string;
  date: string | null;
  body: string | null;
}

// pluginのUpdateオブジェクトそのものをアプリ全体に持ち回すと型の依存が
// 広がるため、必要な情報だけを`AvailableUpdate`へ写し、実際の適用は
// このサービス内に閉じたモジュール変数へ保持したUpdateハンドルを使って
// 行う(呼び出し側はversion文字列だけで`applyUpdate`を呼べばよい)。
let pendingUpdate: import("@tauri-apps/plugin-updater").Update | null = null;

export async function checkForUpdate(): Promise<AvailableUpdate | null> {
  if (!isTauriRuntime()) return null;
  try {
    const { check } = await import("@tauri-apps/plugin-updater");
    const update = await check();
    if (!update) {
      pendingUpdate = null;
      return null;
    }
    pendingUpdate = update;
    return {
      version: update.version,
      currentVersion: update.currentVersion,
      date: update.date ?? null,
      body: update.body ?? null,
    };
  } catch (err) {
    logger.error("checkForUpdate failed", { err: String(err) });
    return null;
  }
}

/**
 * 直前の`checkForUpdate`で見つかった更新をダウンロードして適用し、
 * アプリを再起動する。`onProgress`はダウンロード進捗(0〜100のおおよその
 * 割合、contentLengthが取得できない場合はnull)を通知する任意コールバック。
 */
export async function downloadAndInstallUpdate(onProgress?: (percent: number | null) => void): Promise<void> {
  if (!pendingUpdate) {
    throw new Error("適用可能な更新がありません。先に「アップデートを確認」を実行してください。");
  }
  let downloaded = 0;
  let contentLength: number | undefined;
  await pendingUpdate.downloadAndInstall((event) => {
    if (event.event === "Started") {
      contentLength = event.data.contentLength;
      onProgress?.(contentLength ? 0 : null);
    } else if (event.event === "Progress") {
      downloaded += event.data.chunkLength;
      onProgress?.(contentLength ? Math.min(100, Math.round((downloaded / contentLength) * 100)) : null);
    } else if (event.event === "Finished") {
      onProgress?.(100);
    }
  });
  const { relaunch } = await import("@tauri-apps/plugin-process");
  await relaunch();
}
