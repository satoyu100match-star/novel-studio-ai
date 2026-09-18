import { useEffect, useState } from "react";
import type { Foreshadowing } from "@/types/tauriCommands";
import { FORESHADOWING_STATUSES, FORESHADOWING_STATUS_LABELS } from "@/types/tauriCommands";
import * as foreshadowingService from "@/services/foreshadowingService";

/**
 * 伏線トラッカー(仕様#35)。ステータスは
 * 構想→設置予定→設置済→ヒント提示済→回収予定→回収済 の一直線とは限らず
 * (破棄もある)、ユーザーが自由に選び直せるプルダウンとして実装する。
 */
export function ForeshadowingPanel({ projectId }: { projectId: string }) {
  const [items, setItems] = useState<Foreshadowing[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [newTitle, setNewTitle] = useState("");
  const [form, setForm] = useState<Foreshadowing | null>(null);

  useEffect(() => {
    foreshadowingService.listForeshadowings(projectId).then(setItems);
  }, [projectId]);

  const selected = items.find((f) => f.id === selectedId) ?? null;

  function select(f: Foreshadowing) {
    setSelectedId(f.id);
    setForm(f);
  }

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    const title = newTitle.trim();
    if (!title) return;
    const created = await foreshadowingService.createForeshadowing(projectId, title);
    setItems((fs) => [...fs, created]);
    setNewTitle("");
    select(created);
  }

  async function handleSave() {
    if (!form) return;
    const updated = await foreshadowingService.updateForeshadowing(form.id, form);
    setItems((fs) => fs.map((f) => (f.id === updated.id ? updated : f)));
    setForm(updated);
  }

  async function handleDelete() {
    if (!selected) return;
    await foreshadowingService.deleteForeshadowing(selected.id);
    setItems((fs) => fs.filter((f) => f.id !== selected.id));
    setSelectedId(null);
    setForm(null);
  }

  return (
    <div className="section-layout">
      <div className="section-layout__body">
        <div className="section-layout__list">
          <form onSubmit={handleCreate} className="field-row">
            <input value={newTitle} onChange={(e) => setNewTitle(e.target.value)} placeholder="新しい伏線" />
            <button className="primary" type="submit">
              追加
            </button>
          </form>
          <ul className="entity-list">
            {items.map((f) => (
              <li key={f.id}>
                <button
                  className={`entity-list__item ${selectedId === f.id ? "entity-list__item--selected" : ""}`}
                  onClick={() => select(f)}
                >
                  <span>{f.title}</span>
                  <span className="entity-list__meta">
                    {FORESHADOWING_STATUS_LABELS[f.status as keyof typeof FORESHADOWING_STATUS_LABELS] ?? f.status}
                  </span>
                </button>
              </li>
            ))}
            {items.length === 0 && <li className="status-line">まだ伏線がありません。</li>}
          </ul>
        </div>
        <div className="section-layout__detail">
          {form ? (
            <div className="detail-form">
              <label className="form-label">
                タイトル
                <input value={form.title} onChange={(e) => setForm({ ...form, title: e.target.value })} />
              </label>
              <label className="form-label">
                ステータス
                <select value={form.status} onChange={(e) => setForm({ ...form, status: e.target.value })}>
                  {FORESHADOWING_STATUSES.map((s) => (
                    <option key={s} value={s}>
                      {FORESHADOWING_STATUS_LABELS[s]}
                    </option>
                  ))}
                </select>
              </label>
              <label className="form-label">
                詳細(何を、どう仕込むか)
                <textarea rows={4} value={form.detail ?? ""} onChange={(e) => setForm({ ...form, detail: e.target.value })} />
              </label>
              <label className="form-label">
                メモ
                <textarea rows={2} value={form.memo ?? ""} onChange={(e) => setForm({ ...form, memo: e.target.value })} />
              </label>
              <div className="field-row">
                <button className="secondary" onClick={handleDelete}>
                  削除
                </button>
                <button className="primary" onClick={handleSave}>
                  保存
                </button>
              </div>
            </div>
          ) : (
            <p className="status-line">左の一覧から伏線を選択してください。</p>
          )}
        </div>
      </div>
    </div>
  );
}
