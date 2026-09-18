import { useEffect, useState } from "react";
import type { WorldCategory, WorldEntry, WorldEntryInput } from "@/types/tauriCommands";
import * as worldService from "@/services/worldService";
import { TagInput } from "@/components/TagInput";

function toInput(e: WorldEntry): WorldEntryInput {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const { id, project_id, ...rest } = e;
  return rest;
}

function EntryForm({ projectId, entry, categories, onSaved, onDeleted }: {
  projectId: string;
  entry: WorldEntry;
  categories: WorldCategory[];
  onSaved: (e: WorldEntry) => void;
  onDeleted: () => void;
}) {
  const [form, setForm] = useState<WorldEntryInput>(toInput(entry));
  const [tags, setTags] = useState<string[]>([]);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    setForm(toInput(entry));
    worldService.getWorldEntryTags(entry.id).then(setTags);
  }, [entry]);

  async function handleSave() {
    setSaving(true);
    const updated = await worldService.updateWorldEntry(entry.id, form);
    await worldService.setWorldEntryTags(projectId, entry.id, tags);
    setSaving(false);
    onSaved(updated);
  }

  return (
    <div className="detail-form">
      <label className="form-label">
        名前
        <input value={form.name} onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))} />
      </label>
      <label className="form-label">
        読み
        <input value={form.reading ?? ""} onChange={(e) => setForm((f) => ({ ...f, reading: e.target.value }))} />
      </label>
      <label className="form-label">
        カテゴリ
        <select
          value={form.category_id ?? ""}
          onChange={(e) => setForm((f) => ({ ...f, category_id: e.target.value || null }))}
        >
          <option value="">(未分類)</option>
          {categories.map((c) => (
            <option key={c.id} value={c.id}>{c.name}</option>
          ))}
        </select>
      </label>
      <label className="form-label">
        概要
        <textarea rows={2} value={form.summary ?? ""} onChange={(e) => setForm((f) => ({ ...f, summary: e.target.value }))} />
      </label>
      <label className="form-label">
        詳細
        <textarea rows={5} value={form.detail ?? ""} onChange={(e) => setForm((f) => ({ ...f, detail: e.target.value }))} />
      </label>
      <label className="form-label">
        メモ
        <textarea rows={2} value={form.memo ?? ""} onChange={(e) => setForm((f) => ({ ...f, memo: e.target.value }))} />
      </label>
      <fieldset className="detail-form__group">
        <legend>タグ</legend>
        <TagInput tags={tags} onChange={setTags} />
      </fieldset>
      <div className="field-row">
        <button className="primary" onClick={handleSave} disabled={saving}>保存</button>
        <button className="secondary" onClick={async () => { await worldService.deleteWorldEntry(entry.id); onDeleted(); }}>削除</button>
      </div>
    </div>
  );
}

export function WorldEntriesTab({ projectId }: { projectId: string }) {
  const [categories, setCategories] = useState<WorldCategory[]>([]);
  const [entries, setEntries] = useState<WorldEntry[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [categoryFilter, setCategoryFilter] = useState<string>("");
  const [newName, setNewName] = useState("");
  const [newCategoryName, setNewCategoryName] = useState("");

  useEffect(() => {
    worldService.listWorldCategories(projectId).then(setCategories);
    worldService.listWorldEntries(projectId).then(setEntries);
  }, [projectId]);

  const selected = entries.find((e) => e.id === selectedId) ?? null;
  const filtered = categoryFilter ? entries.filter((e) => e.category_id === categoryFilter) : entries;

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    const name = newName.trim();
    if (!name) return;
    const created = await worldService.createWorldEntry(projectId, {
      ...worldService.emptyWorldEntryInput(),
      name,
      category_id: categoryFilter || null,
    });
    setEntries((es) => [...es, created]);
    setNewName("");
    setSelectedId(created.id);
  }

  async function handleAddCategory(e: React.FormEvent) {
    e.preventDefault();
    const name = newCategoryName.trim();
    if (!name) return;
    const created = await worldService.createWorldCategory(projectId, name);
    setCategories((cs) => [...cs, created]);
    setNewCategoryName("");
  }

  return (
    <div className="section-layout">
      <div className="section-layout__body section-layout__body--with-categories">
        <div className="section-layout__categories">
          <button className={categoryFilter === "" ? "tab tab--active" : "tab"} onClick={() => setCategoryFilter("")}>
            すべて
          </button>
          {categories.map((c) => (
            <button
              key={c.id}
              className={categoryFilter === c.id ? "tab tab--active" : "tab"}
              onClick={() => setCategoryFilter(c.id)}
            >
              {c.name}
            </button>
          ))}
          <form onSubmit={handleAddCategory} className="field-row" style={{ marginTop: 8 }}>
            <input
              value={newCategoryName}
              onChange={(e) => setNewCategoryName(e.target.value)}
              placeholder="新カテゴリ"
              style={{ fontSize: 12 }}
            />
            <button className="secondary" type="submit">追加</button>
          </form>
        </div>
        <div className="section-layout__list">
          <form onSubmit={handleCreate} className="field-row">
            <input value={newName} onChange={(e) => setNewName(e.target.value)} placeholder="新しい項目名" />
            <button className="primary" type="submit">追加</button>
          </form>
          <ul className="entity-list">
            {filtered.map((entry) => (
              <li key={entry.id}>
                <button
                  className={`entity-list__item ${selectedId === entry.id ? "entity-list__item--selected" : ""}`}
                  onClick={() => setSelectedId(entry.id)}
                >
                  <span>{entry.name}</span>
                </button>
              </li>
            ))}
          </ul>
        </div>
        <div className="section-layout__detail">
          {selected ? (
            <EntryForm
              projectId={projectId}
              entry={selected}
              categories={categories}
              onSaved={(updated) => setEntries((es) => es.map((e) => (e.id === updated.id ? updated : e)))}
              onDeleted={() => {
                setEntries((es) => es.filter((e) => e.id !== selected.id));
                setSelectedId(null);
              }}
            />
          ) : (
            <p className="status-line">左の一覧から項目を選択してください。</p>
          )}
        </div>
      </div>
    </div>
  );
}
