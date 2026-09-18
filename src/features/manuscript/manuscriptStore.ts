import { create } from "zustand";
import type { Chapter, Part, Scene } from "@/types/tauriCommands";
import * as manuscriptService from "@/services/manuscriptService";

export type NodeRef = { ownerType: "chapter" | "scene"; ownerId: string };

const AUTOSAVE_DELAY_MS = 800;

interface ManuscriptState {
  projectId: string | null;
  parts: Part[];
  chapters: Chapter[];
  scenesByChapter: Record<string, Scene[]>;
  expandedChapterIds: Set<string>;
  selection: NodeRef | null;
  totalCharCount: number;

  editorBody: string;
  editorDirty: boolean;
  saveStatus: "idle" | "saving" | "saved" | "error";
  autosaveTimer: ReturnType<typeof setTimeout> | null;

  // Phase 4: 本文内コメント用に、textareaで選択された範囲を一時的に
  // 保持する(IME安全性のためcontentEditable化はせず、selectionStart/End
  // を読み取るだけ -- docs/EDITOR.md方針)。CommentsPanel側がこれを見て
  // 新規コメントフォームを開く。
  pendingCommentSelection: { start: number; end: number; quote: string } | null;

  loadProject: (projectId: string) => Promise<void>;
  toggleChapterExpanded: (chapterId: string) => Promise<void>;
  selectNode: (ref: NodeRef) => Promise<void>;
  addPart: (title: string) => Promise<void>;
  addChapter: (title: string, partId?: string | null) => Promise<void>;
  addScene: (chapterId: string, title: string) => Promise<void>;
  setEditorBody: (body: string) => void;
  flushSave: () => Promise<void>;
  refreshChapters: () => Promise<void>;
  captureCommentSelection: (start: number, end: number, quote: string) => void;
  clearPendingCommentSelection: () => void;
}

export const useManuscriptStore = create<ManuscriptState>((set, get) => ({
  projectId: null,
  parts: [],
  chapters: [],
  scenesByChapter: {},
  expandedChapterIds: new Set(),
  selection: null,
  totalCharCount: 0,

  editorBody: "",
  editorDirty: false,
  saveStatus: "idle",
  autosaveTimer: null,
  pendingCommentSelection: null,

  captureCommentSelection: (start: number, end: number, quote: string) => {
    set({ pendingCommentSelection: { start, end, quote } });
  },
  clearPendingCommentSelection: () => set({ pendingCommentSelection: null }),

  loadProject: async (projectId: string) => {
    const [parts, chapters, totalCharCount] = await Promise.all([
      manuscriptService.listParts(projectId),
      manuscriptService.listChapters(projectId),
      manuscriptService.projectCharCount(projectId),
    ]);
    set({
      projectId,
      parts,
      chapters,
      scenesByChapter: {},
      expandedChapterIds: new Set(),
      selection: null,
      totalCharCount,
      editorBody: "",
      editorDirty: false,
      saveStatus: "idle",
    });
  },

  refreshChapters: async () => {
    const { projectId } = get();
    if (!projectId) return;
    const chapters = await manuscriptService.listChapters(projectId);
    set({ chapters });
  },

  toggleChapterExpanded: async (chapterId: string) => {
    const expanded = new Set(get().expandedChapterIds);
    if (expanded.has(chapterId)) {
      expanded.delete(chapterId);
      set({ expandedChapterIds: expanded });
      return;
    }
    expanded.add(chapterId);
    set({ expandedChapterIds: expanded });
    if (!get().scenesByChapter[chapterId]) {
      const scenes = await manuscriptService.listScenes(chapterId);
      set({ scenesByChapter: { ...get().scenesByChapter, [chapterId]: scenes } });
    }
  },

  selectNode: async (ref: NodeRef) => {
    // Flush any pending autosave for the previously selected node before
    // switching away, so an in-flight edit never gets silently dropped.
    await get().flushSave();
    const doc = await manuscriptService.getDocument(ref.ownerType, ref.ownerId);
    set({
      selection: ref,
      editorBody: doc?.body ?? "",
      editorDirty: false,
      saveStatus: "idle",
      pendingCommentSelection: null,
    });
  },

  addPart: async (title: string) => {
    const { projectId, parts } = get();
    if (!projectId) return;
    const part = await manuscriptService.createPart(projectId, title);
    set({ parts: [...parts, part] });
  },

  addChapter: async (title: string, partId: string | null = null) => {
    const { projectId, chapters } = get();
    if (!projectId) return;
    const chapter = await manuscriptService.createChapter(projectId, partId, title);
    set({ chapters: [...chapters, chapter] });
  },

  addScene: async (chapterId: string, title: string) => {
    const scene = await manuscriptService.createScene(chapterId, title);
    const existing = get().scenesByChapter[chapterId] ?? [];
    set({
      scenesByChapter: { ...get().scenesByChapter, [chapterId]: [...existing, scene] },
      expandedChapterIds: new Set(get().expandedChapterIds).add(chapterId),
    });
  },

  setEditorBody: (body: string) => {
    const timer = get().autosaveTimer;
    if (timer) clearTimeout(timer);
    set({ editorBody: body, editorDirty: true, saveStatus: "idle" });
    const newTimer = setTimeout(() => {
      get().flushSave();
    }, AUTOSAVE_DELAY_MS);
    set({ autosaveTimer: newTimer });
  },

  flushSave: async () => {
    const { selection, editorBody, editorDirty, autosaveTimer } = get();
    if (autosaveTimer) {
      clearTimeout(autosaveTimer);
      set({ autosaveTimer: null });
    }
    if (!selection || !editorDirty) return;

    set({ saveStatus: "saving" });
    const doc = await manuscriptService.saveDocumentBody(selection.ownerType, selection.ownerId, editorBody);
    if (!doc) {
      set({ saveStatus: "error" });
      return;
    }
    set({ saveStatus: "saved", editorDirty: false });

    const { projectId } = get();
    if (projectId) {
      manuscriptService.projectCharCount(projectId).then((totalCharCount) => set({ totalCharCount }));
    }

    // Keep the Navigator's char counts (shown next to each chapter/scene)
    // in sync without a full reload.
    if (selection.ownerType === "chapter") {
      set({
        chapters: get().chapters.map((c) => (c.id === selection.ownerId ? { ...c, char_count: doc.char_count } : c)),
      });
    } else {
      const chapterId = Object.keys(get().scenesByChapter).find((cid) =>
        get().scenesByChapter[cid]?.some((s) => s.id === selection.ownerId),
      );
      if (chapterId) {
        set({
          scenesByChapter: {
            ...get().scenesByChapter,
            [chapterId]: get().scenesByChapter[chapterId].map((s) =>
              s.id === selection.ownerId ? { ...s, char_count: doc.char_count } : s,
            ),
          },
        });
      }
    }
  },
}));
