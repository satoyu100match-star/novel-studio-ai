import { useEffect, useRef, useState } from "react";
import type { Note } from "@/types/tauriCommands";
import * as noteService from "@/services/noteService";

const SUGGESTED_FOLDERS = ["アイデア", "没案", "調べ物", "タイトル候補", "セリフ候補", "設定メモ"];

export function NotesPanel({ projectId }: { projectId: string }) {
  const [notes, setNotes] = useState<Note[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [folder, setFolder] = useState("アイデア");
  const [newTitle, setNewTitle] = useState("");
  const [body, setBody] = useState("");
  const [title, setTitle] = useState("");
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    noteService.listNotes(projectId).then(setNotes);
  }, [projectId]);

  const selected = notes.find((n) => n.id === selectedId) ?? null;
  const folders = Array.from(new Set([...SUGGESTED_FOLDERS, ...notes.map((n) => n.folder)]));

  function selectNote(note: Note) {
    setSelectedId(note.id);
    setTitle(note.title);
    setBody(note.body);
  }

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    const t = newTitle.trim();
    if (!t) return;
    const created = await noteService.createNote(projectId, folder, t);
    setNotes((ns) => [created, ...ns]);
    setNewTitle("");
    selectNote(created);
  }

  function scheduleSave(nextTitle: string, nextBody: string) {
    if (!selected) return;
    if (saveTimer.current) clearTimeout(saveTimer.current);
    saveTimer.current = setTimeout(async () => {
      const updated = await noteService.saveNote(selected.id, selected.folder, nextTitle, nextBody);
      setNotes((ns) => ns.map((n) => (n.id === updated.id ? updated : n)));
    }, 800);
  }

  return (
    <div className="section-layout">
      <div className="section-layout__body section-layout__body--with-categories">
        <div className="section-layout__categories">
          {folders.map((f) => (
            <div key={f} className="notes-folder-label">{f}</div>
          ))}
        </div>
        <div className="section-layout__list">
          <form onSubmit={handleCreate} className="field-row" style={{ flexWrap: "wrap" }}>
            <select value={folder} onChange={(e) => setFolder(e.target.value)}>
              {folders.map((f) => (
                <option key={f} value={f}>{f}</option>
              ))}
            </select>
            <input value={newTitle} onChange={(e) => setNewTitle(e.target.value)} placeholder="新規メモのタイトル" />
            <button className="primary" type="submit">追加</button>
          </form>
          <ul className="entity-list">
            {notes.map((note) => (
              <li key={note.id}>
                <button
                  className={`entity-list__item ${selectedId === note.id ? "entity-list__item--selected" : ""}`}
                  onClick={() => selectNote(note)}
                >
                  <span>{note.title}</span>
                  <span className="entity-list__meta">{note.folder}</span>
                </button>
              </li>
            ))}
          </ul>
        </div>
        <div className="section-layout__detail">
          {selected ? (
            <div className="detail-form">
              <label className="form-label">
                タイトル
                <input
                  value={title}
                  onChange={(e) => {
                    setTitle(e.target.value);
                    scheduleSave(e.target.value, body);
                  }}
                />
              </label>
              <textarea
                className="editor__textarea"
                style={{ minHeight: 300 }}
                value={body}
                onChange={(e) => {
                  setBody(e.target.value);
                  scheduleSave(title, e.target.value);
                }}
              />
              <div className="field-row">
                <button
                  className="secondary"
                  onClick={async () => {
                    await noteService.deleteNote(selected.id);
                    setNotes((ns) => ns.filter((n) => n.id !== selected.id));
                    setSelectedId(null);
                  }}
                >
                  削除
                </button>
              </div>
            </div>
          ) : (
            <p className="status-line">左の一覧からメモを選択してください。</p>
          )}
        </div>
      </div>
    </div>
  );
}
