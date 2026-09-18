import { useEffect, useState } from "react";
import { readAppInfo } from "@/services/appInfoService";
import * as aboutService from "@/services/aboutService";
import * as updateService from "@/services/updateService";
import type { AppInfo } from "@/types/tauriCommands";

type Tab = "about" | "changelog" | "licenses" | "privacy";

const TABS: { id: Tab; label: string }[] = [
  { id: "about", label: "概要" },
  { id: "changelog", label: "更新履歴" },
  { id: "licenses", label: "ライセンス" },
  { id: "privacy", label: "プライバシー" },
];

/**
 * Phase 10: 「このアプリについて」画面。バージョン情報・更新履歴・
 * サードパーティライセンス・プライバシーの説明・診断ログの書き出しを
 * 1箇所にまとめる。いずれもリポジトリ直下のMarkdownファイル
 * (CHANGELOG.md/THIRD_PARTY_NOTICES.md/PRIVACY.md)をそのまま表示する
 * だけで、アプリ内に別コピーを持たない(`commands::about`参照)。
 */
export function AboutModal({ onClose }: { onClose: () => void }) {
  const [tab, setTab] = useState<Tab>("about");
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [changelog, setChangelog] = useState("");
  const [licenses, setLicenses] = useState("");
  const [privacy, setPrivacy] = useState("");
  const [diagBusy, setDiagBusy] = useState(false);
  const [diagMessage, setDiagMessage] = useState("");
  const [updateBusy, setUpdateBusy] = useState(false);
  const [updateMessage, setUpdateMessage] = useState("");

  useEffect(() => {
    readAppInfo().then(setAppInfo);
    aboutService.getChangelog().then(setChangelog);
    aboutService.getThirdPartyNotices().then(setLicenses);
    aboutService.getPrivacyNotice().then(setPrivacy);
  }, []);

  // Phase15: Auto Update。起動時の自動確認(`UpdateBanner`)とは別に、
  // いつでも手動で確認できるようにする(バナーを「後で」で閉じた後や、
  // 単に「今最新版か知りたい」という場合のため)。
  async function handleCheckUpdate() {
    setUpdateBusy(true);
    setUpdateMessage("");
    try {
      const update = await updateService.checkForUpdate();
      setUpdateMessage(
        update
          ? `新しいバージョン(v${update.version})があります。画面上部の案内から更新してください。`
          : "現在お使いのバージョンが最新です。",
      );
    } catch (err) {
      setUpdateMessage(`確認に失敗しました: ${String(err)}`);
    } finally {
      setUpdateBusy(false);
    }
  }

  async function handleExportDiagnostics() {
    setDiagBusy(true);
    setDiagMessage("");
    try {
      await aboutService.exportDiagnostics();
      setDiagMessage("診断ログを書き出しました。");
    } catch (err) {
      setDiagMessage(`書き出しに失敗しました: ${String(err)}`);
    } finally {
      setDiagBusy(false);
    }
  }

  return (
    <div className="modal-overlay" role="dialog" aria-label="このアプリについて">
      <div className="modal card">
        <div className="field-row" style={{ justifyContent: "space-between" }}>
          <h1 style={{ margin: 0 }}>このアプリについて</h1>
          <button className="secondary" type="button" onClick={onClose}>
            閉じる
          </button>
        </div>

        <div className="section-layout__tabs">
          {TABS.map((t) => (
            <button
              key={t.id}
              className={tab === t.id ? "tab tab--active" : "tab"}
              type="button"
              onClick={() => setTab(t.id)}
            >
              {t.label}
            </button>
          ))}
        </div>

        {tab === "about" && (
          <div className="detail-form">
            <p>
              <strong>{appInfo?.name ?? "Novel Studio AI"}</strong>
              {appInfo && <span> ・ バージョン {appInfo.version}</span>}
            </p>
            <p className="status-line">
              AIを本格的に活用した小説制作専用デスクトップIDE。作品データは原則ローカルに保存され、AI機能は
              すべて任意です。詳細は「プライバシー」タブを参照してください。
            </p>
            <div className="field-row" style={{ flexWrap: "wrap" }}>
              <button className="secondary" type="button" onClick={handleCheckUpdate} disabled={updateBusy}>
                {updateBusy ? "確認中..." : "アップデートを確認"}
              </button>
              <button className="secondary" type="button" onClick={handleExportDiagnostics} disabled={diagBusy}>
                {diagBusy ? "書き出し中..." : "診断ログを書き出す"}
              </button>
            </div>
            {updateMessage && <p className="status-line">{updateMessage}</p>}
            {diagMessage && <p className="status-line">{diagMessage}</p>}
            <p className="status-line">
              起動のたびに新しいバージョンがないか自動で確認します(見つかった場合のみ画面上部に案内を
              表示。ダウンロード・適用は必ずボタン操作が必要です)。不具合の報告時にログを共有したい場合は
              診断ログの書き出しをお使いください。自動的にどこかへ送信されることはなく、手元にZIPファイルと
              して保存されるだけです。
            </p>
          </div>
        )}

        {tab === "changelog" && (
          <pre className="about-text" aria-label="更新履歴">
            {changelog || "読み込み中..."}
          </pre>
        )}

        {tab === "licenses" && (
          <pre className="about-text" aria-label="ライセンス">
            {licenses || "読み込み中..."}
          </pre>
        )}

        {tab === "privacy" && (
          <pre className="about-text" aria-label="プライバシー">
            {privacy || "読み込み中..."}
          </pre>
        )}
      </div>
    </div>
  );
}
