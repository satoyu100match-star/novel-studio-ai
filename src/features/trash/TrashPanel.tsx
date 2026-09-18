import { useEffect, useState } from "react";
import { TRASH_ENTITY_TYPE_LABELS, type TrashItem } from "@/types/tauriCommands";
import * as trashService from "@/services/trashService";

/**
 * Phase 8: ゴミ箱。既存のSoft Delete方針(CLAUDE.md、各エンティティの
 * `deleted_at`列)を横断的に一覧・復元・完全削除する画面。ここに来るまで
 * 「削除」操作自体は各パネルの既存ボタンのまま変えていない(すべて
 * Soft Deleteのため、確認ダイアログなしでも即座に取り返しがつかなくは
 * ならない設計 -- 完全削除はこのゴミ箱を経由した時だけ)。
 */
export function TrashPanel({ projectId }: { projectId: string }) {
  const [items, setItems] = useState<TrashItem[]>([]);
  const [loading, setLoading] = useState(false);

  async function load() {
    setLoading(true);
    setItems(await trashService.listTrash(projectId));
    setLoading(false);
  }

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [projectId]);

  async function handleRestore(item: TrashItem) {
    await trashService.restoreTrashItem(item.entity_type, item.id);
    await load();
  }

  async function handlePurge(item: TrashItem) {
    if (!window.confirm(`「${item.title}」を完全に削除します。この操作は取り消せません。よろしいですか?`)) return;
    await trashService.purgeTrashItem(item.entity_type, item.id);
    await load();
  }

  async function handlePurgeAll() {
    if (items.length === 0) return;
    if (!window.confirm(`ゴミ箱内の${items.length}件をすべて完全に削除します。この操作は取り消せません。よろしいですか?`)) return;
    await trashService.purgeAllTrash(projectId);
    await load();
  }

  return (
    <div className="detail-form" style={{ maxWidth: 760 }}>
      <div className="field-row">
        <button className="secondary" type="button" onClick={load} disabled={loading}>
          {loading ? "読み込み中..." : "再読み込み"}
        </button>
        <button className="secondary" type="button" onClick={handlePurgeAll} disabled={items.length === 0}>
          ゴミ箱を空にする
        </button>
      </div>
      {!loading && items.length === 0 && <p className="status-line">ゴミ箱は空です。</p>}
      <ul className="entity-list">
        {items.map((item) => (
          <li key={`${item.entity_type}-${item.id}`} className="comment-card">
            <div className="comment-card__meta">
              <span>{TRASH_ENTITY_TYPE_LABELS[item.entity_type] ?? item.entity_type}</span>
              <span>削除日時: {new Date(item.deleted_at).toLocaleString()}</span>
            </div>
            <div className="comment-card__body">{item.title || "(無題)"}</div>
            <div className="field-row">
              <button className="secondary" type="button" onClick={() => handleRestore(item)}>
                復元
              </button>
              <button className="secondary" type="button" onClick={() => handlePurge(item)}>
                完全に削除
              </button>
            </div>
          </li>
        ))}
      </ul>
    </div>
  );
}
