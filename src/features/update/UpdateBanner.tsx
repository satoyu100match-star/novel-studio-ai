import { useEffect, useState } from "react";
import * as updateService from "@/services/updateService";
import type { AvailableUpdate } from "@/services/updateService";

/**
 * Phase15: Auto Update。起動のたびに一度だけ静かにバックグラウンドで
 * 更新の有無を確認し(失敗しても何もしない -- 通常の執筆機能には一切
 * 影響させない)、見つかった場合だけこのバナーで知らせる。ダウンロード・
 * 適用・再起動は必ずここでのユーザーの明示的なクリックを経てから行う
 * (Crash Recoveryバナー(Phase8)と同じ、控えめな告知+明示操作のパターン)。
 */
export function UpdateBanner() {
  const [update, setUpdate] = useState<AvailableUpdate | null>(null);
  const [dismissed, setDismissed] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [progress, setProgress] = useState<number | null>(null);
  const [error, setError] = useState("");

  useEffect(() => {
    updateService.checkForUpdate().then(setUpdate);
  }, []);

  if (!update || dismissed) return null;

  async function handleInstall() {
    setInstalling(true);
    setError("");
    try {
      await updateService.downloadAndInstallUpdate(setProgress);
      // 成功時はrelaunch()がアプリを再起動するため、通常ここには
      // 到達しない。
    } catch (err) {
      setError(String(err));
      setInstalling(false);
    }
  }

  return (
    <div className="recovery-banner">
      <span>
        新しいバージョン(v{update.version})が利用できます(現在: v{update.currentVersion})。
        {installing
          ? ` ダウンロード中...${progress !== null ? `${progress}%` : ""}`
          : " 更新すると、ダウンロード後にアプリが自動で再起動します。"}
        {error && <span> 更新に失敗しました: {error}</span>}
      </span>
      <div className="field-row" style={{ marginBottom: 0 }}>
        <button className="secondary" type="button" onClick={() => setDismissed(true)} disabled={installing}>
          後で
        </button>
        <button className="primary" type="button" onClick={handleInstall} disabled={installing}>
          {installing ? "更新中..." : "今すぐ更新"}
        </button>
      </div>
    </div>
  );
}
