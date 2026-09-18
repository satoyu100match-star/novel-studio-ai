import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { useManuscriptStore } from "@/features/manuscript/manuscriptStore";
import { useWorkspaceUiStore, type WorkspaceSection } from "@/stores/workspaceUiStore";
import { Navigator } from "@/features/manuscript/Navigator";
import { ManuscriptEditor } from "@/features/editor/ManuscriptEditor";
import { SearchPanel } from "@/features/search/SearchPanel";
import { CommentsPanel } from "@/features/comments/CommentsPanel";
import { RevisionsPanel } from "@/features/revisions/RevisionsPanel";
import { ProjectSettingsForm } from "@/features/projects/ProjectSettingsForm";
import { CharactersPanel } from "@/features/characters/CharactersPanel";
import { WorldPanel } from "@/features/world/WorldPanel";
import { NotesPanel } from "@/features/notes/NotesPanel";
import { StoryPanel } from "@/features/plot/StoryPanel";
import { AiPanel } from "@/features/ai/AiPanel";
import { AnalysisPanel } from "@/features/analysis/AnalysisPanel";
import { TrashPanel } from "@/features/trash/TrashPanel";
import { ExportPanel } from "@/features/export/ExportPanel";
import * as projectService from "@/services/projectService";
import type { Project } from "@/types/tauriCommands";

// 実装済みのセクションのみ表示する(ダミーUI禁止)。
const SECTIONS: { id: WorkspaceSection; label: string }[] = [
  { id: "manuscript", label: "原稿" },
  { id: "characters", label: "人物" },
  { id: "world", label: "世界観" },
  { id: "story", label: "ストーリー" },
  { id: "ai", label: "AI" },
  { id: "analysis", label: "分析" },
  { id: "notes", label: "資料・メモ" },
  { id: "trash", label: "ゴミ箱" },
  { id: "export", label: "エクスポート" },
];

type SidePaneTab = "search" | "comments" | "revisions";

/**
 * メイン画面(仕様#8)。左にセクション切り替え、その中身は原稿セクションのみ
 * 3ペイン(Navigator | Editor | 検索)、他セクションはリスト+詳細の2ペイン。
 */
export function ProjectWorkspacePage() {
  const { projectId } = useParams<{ projectId: string }>();
  const navigate = useNavigate();
  const loadProject = useManuscriptStore((s) => s.loadProject);
  const totalCharCount = useManuscriptStore((s) => s.totalCharCount);
  const storeProjectId = useManuscriptStore((s) => s.projectId);

  const section = useWorkspaceUiStore((s) => s.section);
  const setSection = useWorkspaceUiStore((s) => s.setSection);

  const [project, setProject] = useState<Project | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [sidePaneTab, setSidePaneTab] = useState<SidePaneTab>("search");

  useEffect(() => {
    if (!projectId) return;
    loadProject(projectId);
    projectService.getProject(projectId).then((p) => {
      if (!p) {
        navigate("/");
        return;
      }
      setProject(p);
    });
  }, [projectId, loadProject, navigate]);

  if (!project || storeProjectId !== projectId || !projectId) {
    return <div className="app-shell__body">読み込み中...</div>;
  }

  const pageEstimate = (totalCharCount / 400).toFixed(1);

  return (
    <div className="workspace">
      <header className="workspace__header">
        <button className="secondary" onClick={() => navigate("/")}>
          ← 作品一覧
        </button>
        <h2 className="workspace__title">{project.title}</h2>
        <button className="secondary" onClick={() => setShowSettings(true)}>
          作品設定
        </button>
      </header>

      <div className="workspace__section-tabs">
        {SECTIONS.map((s) => (
          <button
            key={s.id}
            className={section === s.id ? "section-tab section-tab--active" : "section-tab"}
            onClick={() => setSection(s.id)}
          >
            {s.label}
          </button>
        ))}
      </div>

      {section === "manuscript" && (
        <div className="workspace__body">
          <div className="workspace__pane workspace__pane--nav">
            <Navigator />
          </div>
          <div className="workspace__pane workspace__pane--editor">
            <ManuscriptEditor />
          </div>
          <div className="workspace__pane workspace__pane--side">
            <div className="section-layout__tabs" style={{ padding: "0 0 8px" }}>
              <button
                className={sidePaneTab === "search" ? "tab tab--active" : "tab"}
                onClick={() => setSidePaneTab("search")}
              >
                検索
              </button>
              <button
                className={sidePaneTab === "comments" ? "tab tab--active" : "tab"}
                onClick={() => setSidePaneTab("comments")}
              >
                コメント
              </button>
              <button
                className={sidePaneTab === "revisions" ? "tab tab--active" : "tab"}
                onClick={() => setSidePaneTab("revisions")}
              >
                履歴
              </button>
            </div>
            {sidePaneTab === "search" && <SearchPanel />}
            {sidePaneTab === "comments" && <CommentsPanel projectId={projectId} />}
            {sidePaneTab === "revisions" && <RevisionsPanel />}
          </div>
        </div>
      )}
      {section === "characters" && (
        <div className="workspace__body workspace__body--single">
          <CharactersPanel projectId={projectId} />
        </div>
      )}
      {section === "world" && (
        <div className="workspace__body workspace__body--single">
          <WorldPanel projectId={projectId} />
        </div>
      )}
      {section === "story" && (
        <div className="workspace__body workspace__body--single">
          <StoryPanel projectId={projectId} />
        </div>
      )}
      {section === "ai" && (
        <div className="workspace__body workspace__body--single">
          <AiPanel projectId={projectId} />
        </div>
      )}
      {section === "analysis" && (
        <div className="workspace__body workspace__body--single">
          <AnalysisPanel projectId={projectId} />
        </div>
      )}
      {section === "notes" && (
        <div className="workspace__body workspace__body--single">
          <NotesPanel projectId={projectId} />
        </div>
      )}
      {section === "trash" && (
        <div className="workspace__body workspace__body--single">
          <TrashPanel projectId={projectId} />
        </div>
      )}
      {section === "export" && (
        <div className="workspace__body workspace__body--single">
          <ExportPanel projectId={projectId} />
        </div>
      )}

      <footer className="app-shell__statusbar">
        {totalCharCount.toLocaleString()}文字 ・ 原稿用紙換算{pageEstimate}枚
      </footer>

      {showSettings && (
        <ProjectSettingsForm
          project={project}
          onClose={() => setShowSettings(false)}
          onSaved={(p) => {
            setProject(p);
            setShowSettings(false);
          }}
        />
      )}
    </div>
  );
}
