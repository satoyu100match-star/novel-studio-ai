import { useEffect, useState } from "react";
import type { GlossaryEntry } from "@/types/tauriCommands";
import * as glossaryService from "@/services/glossaryService";

/** 用語辞典(仕様#27)。本文から用語をクリックして参照する機能はPhase5以降
 * (AI Context Builderと合わせて実装する方が自然なため)に見送る。 */
export function GlossaryTab({ projectId }: { projectId: string }) {
  const [entries, setEntries] = useState<GlossaryEntry[]>([]);
  const [term, setTerm] = useState("");
  const [reading, setReading] = useState("");
  const [definition, setDefinition] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);

  useEffect(() => {
    glossaryService.listGlossaryEntries(projectId).then(setEntries);
  }, [projectId]);

  function resetForm() {
    setTerm("");
    setReading("");
    setDefinition("");
    setEditingId(null);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!term.trim()) return;
    if (editingId) {
      const updated = await glossaryService.updateGlossaryEntry(editingId, { term, reading, definition });
      setEntries((es) => es.map((x) => (x.id === editingId ? updated : x)));
    } else {
      const created = await glossaryService.createGlossaryEntry(projectId, { term, reading, definition });
      setEntries((es) => [...es, created].sort((a, b) => a.term.localeCompare(b.term, "ja")));
    }
    resetForm();
  }

  return (
    <div className="card" style={{ maxWidth: "none" }}>
      <h1>用語辞典</h1>
      <form onSubmit={handleSubmit} className="form-grid" style={{ marginBottom: 12 }}>
        <label className="form-label">
          用語
          <input value={term} onChange={(e) => setTerm(e.target.value)} required />
        </label>
        <label className="form-label">
          読み
          <input value={reading} onChange={(e) => setReading(e.target.value)} />
        </label>
        <label className="form-label" style={{ gridColumn: "span 2" }}>
          説明
          <textarea rows={2} value={definition} onChange={(e) => setDefinition(e.target.value)} />
        </label>
        <div className="field-row">
          <button className="primary" type="submit">{editingId ? "更新" : "追加"}</button>
          {editingId && <button className="secondary" type="button" onClick={resetForm}>キャンセル</button>}
        </div>
      </form>
      <ul className="glossary-list">
        {entries.map((entry) => (
          <li key={entry.id}>
            <div>
              <strong>{entry.term}</strong>
              {entry.reading && <span className="entity-list__meta"> （{entry.reading}）</span>}
              <p style={{ margin: "2px 0", fontSize: 13 }}>{entry.definition}</p>
            </div>
            <div className="field-row">
              <button
                className="secondary"
                onClick={() => {
                  setEditingId(entry.id);
                  setTerm(entry.term);
                  setReading(entry.reading ?? "");
                  setDefinition(entry.definition ?? "");
                }}
              >
                編集
              </button>
              <button
                className="secondary"
                onClick={async () => {
                  await glossaryService.deleteGlossaryEntry(entry.id);
                  setEntries((es) => es.filter((x) => x.id !== entry.id));
                }}
              >
                削除
              </button>
            </div>
          </li>
        ))}
      </ul>
    </div>
  );
}
