import { useState } from "react";
import { useManuscriptStore } from "@/features/manuscript/manuscriptStore";
import type { Chapter } from "@/types/tauriCommands";

const CHAPTER_STATUS_LABEL: Record<string, string> = {
  not_started: "未着手",
  planning: "構想中",
  writing: "執筆中",
  first_draft: "初稿完成",
  revising: "推敲中",
  done: "完成",
};

function ChapterRow({ chapter }: { chapter: Chapter }) {
  const expanded = useManuscriptStore((s) => s.expandedChapterIds.has(chapter.id));
  const scenes = useManuscriptStore((s) => s.scenesByChapter[chapter.id] ?? []);
  const selection = useManuscriptStore((s) => s.selection);
  const toggleChapterExpanded = useManuscriptStore((s) => s.toggleChapterExpanded);
  const selectNode = useManuscriptStore((s) => s.selectNode);
  const addScene = useManuscriptStore((s) => s.addScene);

  const isSelected = selection?.ownerType === "chapter" && selection.ownerId === chapter.id;

  return (
    <li>
      <div className={`nav-row ${isSelected ? "nav-row--selected" : ""}`}>
        <button className="nav-row__disclosure" onClick={() => toggleChapterExpanded(chapter.id)} aria-label="開閉">
          {expanded ? "▾" : "▸"}
        </button>
        <button className="nav-row__label" onClick={() => selectNode({ ownerType: "chapter", ownerId: chapter.id })}>
          <span className="nav-row__title">{chapter.title}</span>
          <span className="nav-row__meta">
            {CHAPTER_STATUS_LABEL[chapter.status] ?? chapter.status} ・ {chapter.char_count.toLocaleString()}字
          </span>
        </button>
      </div>
      {expanded && (
        <ul className="nav-tree nav-tree--nested">
          {scenes.map((scene) => {
            const sceneSelected = selection?.ownerType === "scene" && selection.ownerId === scene.id;
            return (
              <li key={scene.id}>
                <div className={`nav-row ${sceneSelected ? "nav-row--selected" : ""}`}>
                  <span className="nav-row__disclosure" />
                  <button
                    className="nav-row__label"
                    onClick={() => selectNode({ ownerType: "scene", ownerId: scene.id })}
                  >
                    <span className="nav-row__title">{scene.title}</span>
                    <span className="nav-row__meta">{scene.char_count.toLocaleString()}字</span>
                  </button>
                </div>
              </li>
            );
          })}
          <li>
            <NewNodeButton label="+ シーンを追加" onSubmit={(title) => addScene(chapter.id, title)} indent />
          </li>
        </ul>
      )}
    </li>
  );
}

function NewNodeButton({
  label,
  onSubmit,
  indent,
}: {
  label: string;
  onSubmit: (title: string) => void;
  indent?: boolean;
}) {
  const [editing, setEditing] = useState(false);
  const [value, setValue] = useState("");

  if (!editing) {
    return (
      <button className={`nav-add ${indent ? "nav-add--indent" : ""}`} onClick={() => setEditing(true)}>
        {label}
      </button>
    );
  }

  return (
    <form
      className={`nav-add-form ${indent ? "nav-add--indent" : ""}`}
      onSubmit={(e) => {
        e.preventDefault();
        const title = value.trim();
        if (title) onSubmit(title);
        setValue("");
        setEditing(false);
      }}
    >
      <input
        autoFocus
        value={value}
        onChange={(e) => setValue(e.target.value)}
        onBlur={() => setEditing(false)}
        placeholder="タイトル"
      />
    </form>
  );
}

/**
 * 左ペイン「原稿」ナビゲーター(仕様#8, #12)。Part > Chapter > Scene。
 * ドラッグ&ドロップによる並べ替えはPhase1では未実装(上下ボタン等での
 * 並べ替えも含め、後続イテレーションで追加予定 -- CLAUDE.md「未完成機能」参照)。
 */
export function Navigator() {
  const chapters = useManuscriptStore((s) => s.chapters);
  const addChapter = useManuscriptStore((s) => s.addChapter);

  return (
    <nav className="navigator" aria-label="原稿ナビゲーター">
      <h3 className="navigator__heading">原稿</h3>
      <ul className="nav-tree">
        {chapters
          .slice()
          .sort((a, b) => a.order_index - b.order_index)
          .map((chapter) => (
            <ChapterRow key={chapter.id} chapter={chapter} />
          ))}
        <li>
          <NewNodeButton label="+ 章を追加" onSubmit={(title) => addChapter(title)} />
        </li>
      </ul>
    </nav>
  );
}
