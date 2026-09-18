import { useEffect, useState } from "react";
import type { TimelineEvent } from "@/types/tauriCommands";
import * as timelineService from "@/services/timelineService";

/**
 * 時系列(仕様#33)。実世界の日付ではなく作中の自由記述日時
 * (架空暦を許容)を並べる年表。order_indexで著者が任意順に並べ替え可能
 * にする土台のみ持たせ(ドラッグ&ドロップでの並べ替えUIはPhase11の
 * Navigator D&D実装とあわせて検討)、現状は追加順で表示する。
 */
export function TimelinePanel({ projectId }: { projectId: string }) {
  const [events, setEvents] = useState<TimelineEvent[]>([]);
  const [newTitle, setNewTitle] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editDate, setEditDate] = useState("");
  const [editDescription, setEditDescription] = useState("");

  useEffect(() => {
    timelineService.listTimelineEvents(projectId).then(setEvents);
  }, [projectId]);

  async function handleAdd(e: React.FormEvent) {
    e.preventDefault();
    const title = newTitle.trim();
    if (!title) return;
    const created = await timelineService.createTimelineEvent(projectId, title);
    setEvents((es) => [...es, created]);
    setNewTitle("");
  }

  function startEdit(ev: TimelineEvent) {
    setEditingId(ev.id);
    setEditDate(ev.event_date ?? "");
    setEditDescription(ev.description ?? "");
  }

  async function saveEdit(ev: TimelineEvent) {
    const updated = await timelineService.updateTimelineEvent(ev.id, {
      ...ev,
      event_date: editDate || null,
      description: editDescription || null,
    });
    setEvents((es) => es.map((e) => (e.id === updated.id ? updated : e)));
    setEditingId(null);
  }

  async function handleDelete(id: string) {
    await timelineService.deleteTimelineEvent(id);
    setEvents((es) => es.filter((e) => e.id !== id));
    if (editingId === id) setEditingId(null);
  }

  return (
    <div className="section-layout">
      <form className="field-row" style={{ padding: "10px 12px" }} onSubmit={handleAdd}>
        <input value={newTitle} onChange={(e) => setNewTitle(e.target.value)} placeholder="新しい出来事" />
        <button className="primary" type="submit">
          追加
        </button>
      </form>
      <ul className="timeline-list">
        {events.map((ev) => (
          <li key={ev.id} className="timeline-list__item">
            {editingId === ev.id ? (
              <div className="detail-form">
                <label className="form-label">
                  日付(自由記述)
                  <input value={editDate} onChange={(e) => setEditDate(e.target.value)} placeholder="例: 建国暦512年 春" />
                </label>
                <label className="form-label">
                  詳細
                  <textarea rows={3} value={editDescription} onChange={(e) => setEditDescription(e.target.value)} />
                </label>
                <div className="field-row">
                  <button className="secondary" onClick={() => handleDelete(ev.id)}>
                    削除
                  </button>
                  <button className="primary" onClick={() => saveEdit(ev)}>
                    保存
                  </button>
                </div>
              </div>
            ) : (
              <button className="entity-list__item" style={{ padding: 0 }} onClick={() => startEdit(ev)}>
                <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-start", gap: 4 }}>
                  <span className="timeline-list__date">{ev.event_date || "(日付未設定)"}</span>
                  <span>{ev.title}</span>
                  {ev.description && <span className="entity-list__meta">{ev.description}</span>}
                </div>
              </button>
            )}
          </li>
        ))}
        {events.length === 0 && <li className="status-line">まだ出来事がありません。</li>}
      </ul>
    </div>
  );
}
