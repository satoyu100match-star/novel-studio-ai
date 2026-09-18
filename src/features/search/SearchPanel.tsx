import { useState } from "react";
import * as manuscriptService from "@/services/manuscriptService";
import { useManuscriptStore } from "@/features/manuscript/manuscriptStore";
import type { SearchHit } from "@/types/tauriCommands";

/**
 * 作品全体検索(仕様#64)。Phase1時点では章/シーンのタイトルと本文のみを
 * 対象とするシンプルなLIKE検索(実装: src-tauri/src/repositories/search_repository.rs)。
 * Character/World/Plot/メモ等を対象に含める高度検索(仕様#65)は、
 * それらのデータモデルが揃うPhase3〜4以降に拡張する。
 */
export function SearchPanel() {
  const projectId = useManuscriptStore((s) => s.projectId);
  const selectNode = useManuscriptStore((s) => s.selectNode);
  const toggleChapterExpanded = useManuscriptStore((s) => s.toggleChapterExpanded);
  const scenesByChapter = useManuscriptStore((s) => s.scenesByChapter);
  const expandedChapterIds = useManuscriptStore((s) => s.expandedChapterIds);

  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchHit[]>([]);
  const [searching, setSearching] = useState(false);

  async function runSearch(q: string) {
    setQuery(q);
    if (!projectId || q.trim().length === 0) {
      setResults([]);
      return;
    }
    setSearching(true);
    const hits = await manuscriptService.searchProject(projectId, q);
    setResults(hits);
    setSearching(false);
  }

  async function jumpTo(hit: SearchHit) {
    if (hit.owner_type === "scene") {
      // Scenes live under a chapter in the tree; find and expand it first
      // (Phase1's simple LIKE search doesn't return the parent chapter id,
      // so we scan already-loaded scene lists -- good enough at Phase1
      // data volumes; revisit if this becomes a bottleneck).
      const chapterId = Object.entries(scenesByChapter).find(([, scenes]) =>
        scenes.some((s) => s.id === hit.owner_id),
      )?.[0];
      if (chapterId && !expandedChapterIds.has(chapterId)) {
        await toggleChapterExpanded(chapterId);
      }
    }
    await selectNode({ ownerType: hit.owner_type, ownerId: hit.owner_id });
  }

  return (
    <div className="search-panel">
      <input
        className="search-panel__input"
        value={query}
        onChange={(e) => runSearch(e.target.value)}
        placeholder="作品内を検索..."
        aria-label="作品内検索"
      />
      {searching && <div className="status-line">検索中...</div>}
      <ul className="search-panel__results">
        {results.map((hit) => (
          <li key={`${hit.owner_type}:${hit.owner_id}`}>
            <button className="search-hit" onClick={() => jumpTo(hit)}>
              <span className="search-hit__title">{hit.title}</span>
              <span className="search-hit__snippet">{hit.snippet}</span>
            </button>
          </li>
        ))}
        {!searching && query.trim() !== "" && results.length === 0 && (
          <li className="status-line">一致する結果はありません</li>
        )}
      </ul>
    </div>
  );
}
