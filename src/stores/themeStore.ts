import { create } from "zustand";
import { readSetting, writeSetting } from "@/services/settingsService";
import { logger } from "@/utils/logger";

// "sepia" is designed in (spec #86) but not implemented until a later
// phase -- keep the union ready rather than reshaping it twice.
//
// Phase13: Light/Darkの2択だったテーマを、色使い・フォントの異なる
// 「見た目のテーマ」として拡張(かわいい/かっこいい/パンク)。既存の
// CSS変数ベースの設計(theme.css)をそのまま踏襲し、`data-theme`属性の
// 値を増やすだけで済むようにした(コンポーネント側に変更は不要)。
export type Theme = "light" | "dark" | "cute" | "cool" | "punk";

export const THEME_OPTIONS: Theme[] = ["light", "dark", "cute", "cool", "punk"];

export const THEME_LABELS: Record<Theme, string> = {
  light: "ライト",
  dark: "ダーク",
  cute: "かわいい系",
  cool: "かっこいい系",
  punk: "パンク系",
};

const SETTING_KEY = "ui.theme";
const DEFAULT_THEME: Theme = "light";

/** Light/Darkの単純な切り替え用。かわいい/かっこいい/パンクなど他の
 *  テーマから呼んだ場合はlightへ戻す(未定義の遷移を作らないための
 *  安全策)。 */
export function nextTheme(current: Theme): Theme {
  return current === "light" ? "dark" : "light";
}

function isTheme(value: string | null): value is Theme {
  return (THEME_OPTIONS as string[]).includes(value ?? "");
}

interface ThemeState {
  theme: Theme;
  hydrated: boolean;
  hydrate: () => Promise<void>;
  setTheme: (theme: Theme) => void;
  toggleTheme: () => void;
}

export const useThemeStore = create<ThemeState>((set, get) => ({
  theme: DEFAULT_THEME,
  hydrated: false,

  hydrate: async () => {
    const stored = await readSetting(SETTING_KEY);
    const theme = isTheme(stored) ? stored : DEFAULT_THEME;
    set({ theme, hydrated: true });
  },

  setTheme: (theme: Theme) => {
    set({ theme });
    writeSetting(SETTING_KEY, theme).catch((err) =>
      logger.error("theme persist failed", { err: String(err) }),
    );
  },

  toggleTheme: () => {
    get().setTheme(nextTheme(get().theme));
  },
}));
