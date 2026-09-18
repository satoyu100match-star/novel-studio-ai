import { beforeEach, describe, expect, it, vi } from "vitest";
import { useManuscriptStore } from "@/features/manuscript/manuscriptStore";
import * as manuscriptService from "@/services/manuscriptService";

// This exercises the store's autosave/flush contract against a mocked
// service layer (no real Tauri backend) -- the same "must not throw and
// must behave predictably outside Tauri" guarantee as
// settingsService.test.ts, but for the higher-stakes manuscript body.
vi.mock("@/services/manuscriptService");

const mockedService = vi.mocked(manuscriptService);

describe("useManuscriptStore", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.clearAllMocks();
    useManuscriptStore.setState({
      projectId: "proj-1",
      parts: [],
      chapters: [],
      scenesByChapter: {},
      expandedChapterIds: new Set(),
      selection: { ownerType: "chapter", ownerId: "ch-1" },
      totalCharCount: 0,
      editorBody: "",
      editorDirty: false,
      saveStatus: "idle",
      autosaveTimer: null,
    });
    mockedService.saveDocumentBody.mockResolvedValue({
      id: "doc-1",
      owner_type: "chapter",
      owner_id: "ch-1",
      body: "更新後の本文",
      char_count: 6,
      updated_at: "2026-01-01T00:00:00Z",
    });
    mockedService.projectCharCount.mockResolvedValue(6);
  });

  it("debounces autosave and flushes after the delay", async () => {
    useManuscriptStore.getState().setEditorBody("更新後の本文");
    expect(useManuscriptStore.getState().editorDirty).toBe(true);
    expect(mockedService.saveDocumentBody).not.toHaveBeenCalled();

    await vi.advanceTimersByTimeAsync(800);

    expect(mockedService.saveDocumentBody).toHaveBeenCalledWith("chapter", "ch-1", "更新後の本文");
    expect(useManuscriptStore.getState().editorDirty).toBe(false);
    expect(useManuscriptStore.getState().saveStatus).toBe("saved");
  });

  it("flushSave is a no-op when nothing is dirty", async () => {
    await useManuscriptStore.getState().flushSave();
    expect(mockedService.saveDocumentBody).not.toHaveBeenCalled();
  });

  it("selectNode flushes pending edits for the previous node before switching", async () => {
    useManuscriptStore.getState().setEditorBody("switching away with unsaved text");
    mockedService.getDocument.mockResolvedValue({
      id: "doc-2",
      owner_type: "scene",
      owner_id: "sc-1",
      body: "シーン本文",
      char_count: 5,
      updated_at: "2026-01-01T00:00:00Z",
    });

    await useManuscriptStore.getState().selectNode({ ownerType: "scene", ownerId: "sc-1" });

    expect(mockedService.saveDocumentBody).toHaveBeenCalledWith(
      "chapter",
      "ch-1",
      "switching away with unsaved text",
    );
    expect(useManuscriptStore.getState().selection).toEqual({ ownerType: "scene", ownerId: "sc-1" });
    expect(useManuscriptStore.getState().editorBody).toBe("シーン本文");
  });
});
