import { create } from "zustand";

/**
 * 作品ワークスペース(`ProjectWorkspacePage`)のセクション切り替え状態を、
 * コンポーネントローカルのuseStateからグローバルストアへ引き上げたもの
 * (Phase11、Command Palette)。Command Paletteはアプリのどこからでも
 * 開けるため、「原稿セクションへジャンプする」ような操作を実現するには
 * セクション状態がコンポーネント外から参照・変更できる必要がある。
 */
export type WorkspaceSection =
  | "manuscript"
  | "characters"
  | "world"
  | "story"
  | "ai"
  | "analysis"
  | "notes"
  | "trash"
  | "export";

interface WorkspaceUiState {
  section: WorkspaceSection;
  setSection: (section: WorkspaceSection) => void;
}

export const useWorkspaceUiStore = create<WorkspaceUiState>((set) => ({
  section: "manuscript",
  setSection: (section) => set({ section }),
}));
