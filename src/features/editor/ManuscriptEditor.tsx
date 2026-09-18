import { useEffect, useRef, useState } from "react";
import { useManuscriptStore } from "@/features/manuscript/manuscriptStore";
import { GenkouYoushiView } from "@/features/manuscript/GenkouYoushiView";

const SAVE_STATUS_LABEL: Record<string, string> = {
  idle: "",
  saving: "保存中...",
  saved: "保存済み",
  error: "保存エラー",
};

/**
 * 標準エディタ (Phase 1)。
 *
 * 実装方針: プレーンな `<textarea>` を採用する。日本語IME変換中の挙動は
 * ブラウザ/OSネイティブに完全に委ね、compositionイベントに介入する処理を
 * 一切行わない -- IME入力を壊さないことを最優先する(最重要方針#12,
 * docs/EDITOR.md 1章)。Undo/Redoもブラウザネイティブの textarea undo
 * スタックをそのまま使うため、独自実装での破損リスクがない。
 *
 * リッチテキスト整形(見出し・強調・ルビ・傍点・本文内コメント)は、
 * IMEに影響を与えないリッチテキストエディタ統合が必要なため、
 * Phase 2以降に持ち越す(CLAUDE.md「未完成機能」参照。ダミーボタンを
 * 置くくらいなら未実装のままにする、という方針に基づく判断)。
 */
export function ManuscriptEditor() {
  const selection = useManuscriptStore((s) => s.selection);
  const editorBody = useManuscriptStore((s) => s.editorBody);
  const setEditorBody = useManuscriptStore((s) => s.setEditorBody);
  const flushSave = useManuscriptStore((s) => s.flushSave);
  const saveStatus = useManuscriptStore((s) => s.saveStatus);
  const captureCommentSelection = useManuscriptStore((s) => s.captureCommentSelection);

  const [focusMode, setFocusMode] = useState(false);
  const [viewMode, setViewMode] = useState<"editor" | "genkou">("editor");
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Save on unmount / selection change, and on window blur (app losing
  // focus, e.g. Alt+Tab) so nothing typed is lost longer than necessary.
  useEffect(() => {
    const handler = () => {
      flushSave();
    };
    window.addEventListener("blur", handler);
    return () => {
      window.removeEventListener("blur", handler);
      flushSave();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selection?.ownerId]);

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
        e.preventDefault();
        flushSave();
      }
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toUpperCase() === "F") {
        e.preventDefault();
        setFocusMode((v) => !v);
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [flushSave]);

  const charCount = editorBody.length;
  const pageEstimate = (charCount / 400).toFixed(1);

  if (!selection) {
    return (
      <div className="editor editor--empty">
        <p>左のナビゲーターから章またはシーンを選択してください。</p>
      </div>
    );
  }

  return (
    <div className={`editor ${focusMode ? "editor--focus" : ""}`}>
      <div className="editor__toolbar">
        <button className="secondary" onClick={() => setViewMode((v) => (v === "editor" ? "genkou" : "editor"))}>
          {viewMode === "editor" ? "原稿用紙プレビュー" : "標準エディタに戻る"}
        </button>
        <button className="secondary" onClick={() => setFocusMode((v) => !v)}>
          {focusMode ? "Focus Mode 解除" : "Focus Mode"}
        </button>
        {viewMode === "editor" && (
          <button
            className="secondary"
            onClick={() => {
              const el = textareaRef.current;
              if (!el) return;
              const start = el.selectionStart ?? 0;
              const end = el.selectionEnd ?? 0;
              const quote = editorBody.slice(start, end);
              captureCommentSelection(start, end, quote);
            }}
          >
            選択範囲にコメント追加
          </button>
        )}
      </div>

      {viewMode === "genkou" ? (
        <GenkouYoushiView body={editorBody} />
      ) : (
        <>
          <textarea
            ref={textareaRef}
            className="editor__textarea"
            value={editorBody}
            onChange={(e) => setEditorBody(e.target.value)}
            placeholder="ここに本文を入力してください..."
            aria-label="本文エディタ"
            spellCheck={false}
          />
          <div className="editor__statusline">
            <span>{charCount.toLocaleString()}字</span>
            <span>換算 {pageEstimate}枚 (400字詰め)</span>
            <span className="editor__save-status">{SAVE_STATUS_LABEL[saveStatus]}</span>
          </div>
        </>
      )}
    </div>
  );
}
