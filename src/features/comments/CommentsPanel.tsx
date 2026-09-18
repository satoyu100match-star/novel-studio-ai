import { useEffect, useState } from "react";
import { useManuscriptStore } from "@/features/manuscript/manuscriptStore";
import type { Comment } from "@/types/tauriCommands";
import * as commentService from "@/services/commentService";

/**
 * 本文内コメント(仕様#15周辺, 見送りだった機能をPhase4で実装)。
 *
 * 実装方針: contentEditableへの移行はIME入力を壊すリスクがあるため行わず
 * (docs/EDITOR.md最優先方針)、代わりに標準エディタのtextareaから
 * selectionStart/Endと引用テキストのスナップショットだけを読み取り、
 * このサイドパネルでコメント本文を追加する方式にする。本文が後から
 * 編集されて文字位置がズレても、引用テキストを目視確認できることで
 * 実用上困らない設計(0004マイグレーションのコメント参照)。
 */
export function CommentsPanel({ projectId }: { projectId: string }) {
  const selection = useManuscriptStore((s) => s.selection);
  const pending = useManuscriptStore((s) => s.pendingCommentSelection);
  const clearPending = useManuscriptStore((s) => s.clearPendingCommentSelection);

  const [comments, setComments] = useState<Comment[]>([]);
  const [newBody, setNewBody] = useState("");

  useEffect(() => {
    if (!selection) {
      setComments([]);
      return;
    }
    commentService.listComments(selection.ownerType, selection.ownerId).then(setComments);
  }, [selection]);

  useEffect(() => {
    // 選択範囲キャプチャが来たら新規フォームにフォーカスしやすいよう
    // 入力欄を空にしておく(引用は別枠でプレビュー表示する)。
    if (pending) setNewBody("");
  }, [pending]);

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    if (!selection) return;
    const body = newBody.trim();
    if (!body) return;
    const created = await commentService.createComment(
      projectId,
      selection.ownerType,
      selection.ownerId,
      pending?.start ?? null,
      pending?.end ?? null,
      pending?.quote || null,
      body,
    );
    setComments((cs) => [...cs, created]);
    setNewBody("");
    clearPending();
  }

  async function toggleResolved(comment: Comment) {
    const updated = await commentService.setCommentResolved(comment.id, !comment.resolved);
    setComments((cs) => cs.map((c) => (c.id === updated.id ? updated : c)));
  }

  async function handleDelete(id: string) {
    await commentService.deleteComment(id);
    setComments((cs) => cs.filter((c) => c.id !== id));
  }

  if (!selection) {
    return <p className="status-line">章またはシーンを選択するとコメントを表示できます。</p>;
  }

  return (
    <div className="comments-panel">
      <form onSubmit={handleCreate} className="detail-form">
        {pending && (
          <div className="comment-card__quote">
            {pending.quote ? `「${pending.quote}」に対して` : "本文全体に対して(範囲未選択)"}
          </div>
        )}
        <textarea
          rows={3}
          value={newBody}
          onChange={(e) => setNewBody(e.target.value)}
          placeholder="コメントを入力..."
        />
        <div className="field-row">
          {pending && (
            <button type="button" className="secondary" onClick={() => clearPending()}>
              引用を取り消す
            </button>
          )}
          <button className="primary" type="submit">
            追加
          </button>
        </div>
      </form>
      <ul className="comments-panel__list">
        {comments.map((c) => (
          <li key={c.id} className={`comment-card ${c.resolved ? "comment-card--resolved" : ""}`}>
            {c.quote && <div className="comment-card__quote">「{c.quote}」</div>}
            <div className="comment-card__body">{c.body}</div>
            <div className="comment-card__actions">
              <button className="secondary" onClick={() => handleDelete(c.id)}>
                削除
              </button>
              <button className="secondary" onClick={() => toggleResolved(c)}>
                {c.resolved ? "未解決に戻す" : "解決済みにする"}
              </button>
            </div>
          </li>
        ))}
        {comments.length === 0 && <li className="status-line">コメントはまだありません。</li>}
      </ul>
    </div>
  );
}
