import { useEffect, useMemo, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useThemeStore } from "@/stores/themeStore";
import { useWorkspaceUiStore, type WorkspaceSection } from "@/stores/workspaceUiStore";
import { useManuscriptStore } from "@/features/manuscript/manuscriptStore";
import { useProjectListStore } from "@/stores/projectStore";

type CommandItem = {
  id: string;
  label: string;
  hint?: string;
  run: () => void;
};

const SECTION_LABELS: { id: WorkspaceSection; label: string }[] = [
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

/**
 * Phase 11: Command Palette。Ctrl+K(macOSはCmd+K)でどこからでも開き、
 * 「今いるプロジェクトのセクションへ切り替える」「章へジャンプする」
 * 「作品一覧へ戻る」「サンプル作品を開く」等をキーボードだけで実行
 * できるようにする。仕様が明示的に対象外とするSemantic Search/Map/
 * Plugin architecture等とは異なり、既存機能への「ショートカットでの
 * 到達手段」を追加するだけで、新しいデータや機能そのものは増やさない
 * (docs/ROADMAP.md Phase11参照)。
 */
export function CommandPalette({
  open,
  onOpenChange,
  onOpenAbout,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onOpenAbout: () => void;
}) {
  const [query, setQuery] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  const navigate = useNavigate();
  const toggleTheme = useThemeStore((s) => s.toggleTheme);
  const createSample = useProjectListStore((s) => s.createSample);

  const manuscriptProjectId = useManuscriptStore((s) => s.projectId);
  const chapters = useManuscriptStore((s) => s.chapters);
  const selectNode = useManuscriptStore((s) => s.selectNode);
  const setSection = useWorkspaceUiStore((s) => s.setSection);

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      const isModK = (e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k";
      if (isModK) {
        e.preventDefault();
        onOpenChange(!open);
        return;
      }
      if (e.key === "Escape" && open) {
        onOpenChange(false);
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  useEffect(() => {
    if (open) {
      setQuery("");
      // ダイアログが描画されてからフォーカスする(同フレームだと失敗するため)。
      setTimeout(() => inputRef.current?.focus(), 0);
    }
  }, [open]);

  function close() {
    onOpenChange(false);
  }

  function goToProjectSection(section: WorkspaceSection) {
    return () => {
      setSection(section);
      if (manuscriptProjectId) navigate(`/projects/${manuscriptProjectId}`);
      close();
    };
  }

  const items: CommandItem[] = useMemo(() => {
    const list: CommandItem[] = [
      { id: "nav-projects", label: "作品一覧へ移動", run: () => (navigate("/"), close()) },
      {
        id: "action-sample",
        label: "サンプル作品を開く",
        hint: "人物・世界観・プロット等が入った作品をすぐ試せます",
        run: async () => {
          close();
          const project = await createSample();
          navigate(`/projects/${project.id}`);
        },
      },
      { id: "action-theme", label: "テーマ切替(ライト/ダーク)", run: () => (toggleTheme(), close()) },
      { id: "action-about", label: "このアプリについて を開く", run: () => (onOpenAbout(), close()) },
    ];

    if (manuscriptProjectId) {
      for (const s of SECTION_LABELS) {
        list.push({
          id: `section-${s.id}`,
          label: `${s.label}セクションへ移動`,
          run: goToProjectSection(s.id),
        });
      }
      for (const chapter of chapters) {
        list.push({
          id: `chapter-${chapter.id}`,
          label: chapter.title || "(無題の章)",
          hint: "章の原稿を開く",
          run: async () => {
            close();
            setSection("manuscript");
            await selectNode({ ownerType: "chapter", ownerId: chapter.id });
            navigate(`/projects/${manuscriptProjectId}`);
          },
        });
      }
    }

    return list;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [manuscriptProjectId, chapters, navigate, createSample, toggleTheme, onOpenAbout, selectNode, setSection]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return items;
    return items.filter((item) => item.label.toLowerCase().includes(q));
  }, [items, query]);

  if (!open) return null;

  return (
    <div className="modal-overlay" role="dialog" aria-label="コマンドパレット" onClick={close}>
      <div className="modal card" onClick={(e) => e.stopPropagation()}>
        <input
          ref={inputRef}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && filtered.length > 0) {
              filtered[0].run();
            }
          }}
          placeholder="コマンドを検索(章名・セクション名など)..."
          aria-label="コマンドを検索"
        />
        <ul className="entity-list" style={{ maxHeight: "50vh", overflowY: "auto" }}>
          {filtered.length === 0 && <li className="status-line">一致するコマンドがありません。</li>}
          {filtered.map((item) => (
            <li key={item.id}>
              <button type="button" className="entity-list__item" onClick={item.run}>
                <span>{item.label}</span>
                {item.hint && <span className="entity-list__meta">{item.hint}</span>}
              </button>
            </li>
          ))}
        </ul>
        <p className="status-line">Ctrl+K(macOSはCmd+K)でいつでも開閉できます。Escで閉じます。</p>
      </div>
    </div>
  );
}
