import { useEffect, useState } from "react";
import { HashRouter } from "react-router-dom";
import { AppRouter } from "@/app/AppRouter";
import { THEME_LABELS, THEME_OPTIONS, useThemeStore, type Theme } from "@/stores/themeStore";
import * as backupService from "@/services/backupService";
import * as settingsService from "@/services/settingsService";
import { OnboardingModal } from "@/features/onboarding/OnboardingModal";
import { AboutModal } from "@/features/about/AboutModal";
import { CommandPalette } from "@/features/commandPalette/CommandPalette";
import { UpdateBanner } from "@/features/update/UpdateBanner";

const ONBOARDING_SEEN_KEY = "app.onboarding_seen";

/**
 * `HashRouter` rather than `BrowserRouter`: the app is served from a
 * `tauri://` / `file://`-style origin in production, which has no server
 * to fall back to a real path for client-side routes. It wraps the whole
 * shell (not just `<AppRouter>`) so that things like `CommandPalette`
 * (Phase11), which live outside any `<Route>` element but still need
 * `useNavigate`, are inside the Router context too.
 */
export function App() {
  return (
    <HashRouter>
      <AppShell />
    </HashRouter>
  );
}

function AppShell() {
  const theme = useThemeStore((s) => s.theme);
  const hydrate = useThemeStore((s) => s.hydrate);
  const setTheme = useThemeStore((s) => s.setTheme);
  const [showRecoveryBanner, setShowRecoveryBanner] = useState(false);
  const [showOnboarding, setShowOnboarding] = useState(false);
  const [showAbout, setShowAbout] = useState(false);
  const [showCommandPalette, setShowCommandPalette] = useState(false);

  useEffect(() => {
    hydrate();
  }, [hydrate]);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  // Phase 8: Crash Recovery。前回アプリが正常終了しなかった形跡があれば、
  // 一度だけ案内する(原稿は保存のたびにSQLiteへ同期書き込みされている
  // ため実際のデータ喪失リスクは小さいが、不安を残さないための表示)。
  useEffect(() => {
    backupService.wasUncleanShutdown().then((was) => setShowRecoveryBanner(was));
  }, []);

  // Phase 10: Onboarding。初回起動時のみ表示する(`app_settings`にフラグを
  // 立てて次回以降は出さない。スキップ/完了のどちらでも同じフラグを立てる
  // -- 何度も同じ案内を見せて執筆の邪魔をしないことを優先)。
  useEffect(() => {
    settingsService.readSetting(ONBOARDING_SEEN_KEY).then((seen) => {
      if (!seen) setShowOnboarding(true);
    });
  }, []);

  function handleOnboardingDone() {
    setShowOnboarding(false);
    settingsService.writeSetting(ONBOARDING_SEEN_KEY, "1");
  }

  return (
    <div className="app-shell">
      <div className="app-shell__topbar">
        <select
          className="theme-select"
          aria-label="テーマ"
          value={theme}
          onChange={(e) => setTheme(e.target.value as Theme)}
        >
          {THEME_OPTIONS.map((t) => (
            <option key={t} value={t}>
              {THEME_LABELS[t]}
            </option>
          ))}
        </select>
        <button className="secondary" type="button" onClick={() => setShowCommandPalette(true)}>
          コマンド(Ctrl+K)
        </button>
        <button className="secondary" type="button" onClick={() => setShowAbout(true)}>
          このアプリについて
        </button>
      </div>
      <UpdateBanner />
      {showRecoveryBanner && (
        <div className="recovery-banner">
          <span>
            前回、アプリが正常に終了しなかった可能性があります。原稿は自動保存されているため大きなデータ
            喪失の心配はありませんが、心配な場合は各章/シーンの「履歴」タブやバックアップから内容を
            確認できます。
          </span>
          <button className="secondary" type="button" onClick={() => setShowRecoveryBanner(false)}>
            閉じる
          </button>
        </div>
      )}
      <div className="app-shell__content">
        <AppRouter />
      </div>
      {showOnboarding && <OnboardingModal onDone={handleOnboardingDone} />}
      {showAbout && <AboutModal onClose={() => setShowAbout(false)} />}
      <CommandPalette
        open={showCommandPalette}
        onOpenChange={setShowCommandPalette}
        onOpenAbout={() => setShowAbout(true)}
      />
    </div>
  );
}
