import { useEffect, useState } from "react";
import { useManuscriptStore } from "@/features/manuscript/manuscriptStore";
import { DiffView } from "@/features/ai/DiffView";
import type { Revision } from "@/types/tauriCommands";
import * as revisionService from "@/services/revisionService";

const TRIGGER_LABELS: Record<string, string> = { auto: "自動", manual: "手動" };

/**
 * Phase 8: バージョン履歴。「上書き保存で前の内容が消えてしまう不安」に
 * 対応する(仕様の主要不安要素、CLAUDE.md優先順位#1「データ安全性」)。
 * 一定間隔での自動スナップショットに加え、いつでも手動でスナップショット
 * を残せる。復元は必ず「現在の内容との差分を確認してから」の1テンポを
 * 置く(Phase6のAI提案と同様、いきなり上書きしない設計)。
 */
export function RevisionsPanel() {
  const selection = useManuscriptStore((s) => s.selection);
  const editorBody = useManuscriptStore((s) => s.editorBody);
  const setEditorBody = useManuscriptStore((s) => s.setEditorBody);

  const [revisions, setRevisions] = useState<Revision[]>([]);
  const [loading, setLoading] = useState(false);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [label, setLabel] = useState("");
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState("");

  async function load() {
    if (!selection) {
      setRevisions([]);
      return;
    }
    setLoading(true);
    const list = await revisionService.listRevisions(selection.ownerType, selection.ownerId);
    setRevisions(list);
    setLoading(false);
  }

  useEffect(() => {
    load();
    setExpandedId(null);
    setMessage("");
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selection?.ownerType, selection?.ownerId]);

  async function handleSnapshot() {
    if (!selection) return;
    setSaving(true);
    await revisionService.createManualRevision(selection.ownerType, selection.ownerId, label.trim() || null);
    setLabel("");
    setSaving(false);
    setMessage("スナップショットを保存しました。");
    await load();
  }

  async function handleRestore(revision: Revision) {
    if (!window.confirm("このバージョンの内容で現在の本文を上書きします。復元前の内容は自動でバックアップされますが、よろしいですか?")) {
      return;
    }
    const doc = await revisionService.restoreRevision(revision.id);
    setEditorBody(doc.body);
    setMessage("復元しました。復元前の内容は履歴に残っています。");
    await load();
  }

  if (!selection) {
    return <p className="status-line">章またはシーンを選択すると履歴が表示されます。</p>;
  }

  return (
    <div className="revisions-panel">
      <div className="field-row" style={{ flexWrap: "wrap" }}>
        <input
          value={label}
          onChange={(e) => setLabel(e.target.value)}
          placeholder="スナップショットのラベル(任意)"
          style={{ flex: 1, minWidth: 120 }}
        />
        <button className="secondary" type="button" onClick={handleSnapshot} disabled={saving}>
          {saving ? "保存中..." : "現在の内容を保存"}
        </button>
      </div>
      {message && <p className="status-line">{message}</p>}
      {loading && <p className="status-line">読み込み中...</p>}
      {!loading && revisions.length === 0 && <p className="status-line">まだ履歴はありません。</p>}
      <ul className="entity-list">
        {revisions.map((r) => (
          <li key={r.id} className="comment-card">
            <div className="comment-card__meta">
              <span>{new Date(r.created_at).toLocaleString()}</span>
              <span>{TRIGGER_LABELS[r.trigger] ?? r.trigger}</span>
              <span>{r.char_count.toLocaleString()}字</span>
              {r.label && <span>{r.label}</span>}
            </div>
            <div className="field-row">
              <button
                className="secondary"
                type="button"
                onClick={() => setExpandedId((cur) => (cur === r.id ? null : r.id))}
              >
                {expandedId === r.id ? "差分を隠す" : "現在との差分を見る"}
              </button>
              <button className="secondary" type="button" onClick={() => handleRestore(r)}>
                このバージョンを復元
              </button>
            </div>
            {expandedId === r.id && <DiffView before={editorBody} after={r.body} />}
          </li>
        ))}
      </ul>
    </div>
  );
}
