import { useEffect, useState } from "react";
import { useManuscriptStore } from "@/features/manuscript/manuscriptStore";
import { ContextInspector } from "@/features/ai/ContextInspector";
import type { AiChatMessage, ContextBlock } from "@/types/tauriCommands";
import * as aiService from "@/services/aiService";
import * as settingsService from "@/services/settingsService";

type Stage = "idle" | "reviewing";

/**
 * 作品専用AIチャット(仕様#44)。送信は必ず
 * 「質問入力 → Context Inspectorで確認 → 送信」の順を踏む
 * (docs/AI.md 2章、バイパス経路を作らない方針)。
 */
export function AiChatPanel({ projectId }: { projectId: string }) {
  const selection = useManuscriptStore((s) => s.selection);
  const scenesByChapter = useManuscriptStore((s) => s.scenesByChapter);

  const [messages, setMessages] = useState<AiChatMessage[]>([]);
  const [draft, setDraft] = useState("");
  const [stage, setStage] = useState<Stage>("idle");
  const [posting, setPosting] = useState(false);
  const [blocks, setBlocks] = useState<ContextBlock[]>([]);
  const [selectedLabels, setSelectedLabels] = useState<Set<string>>(new Set());
  const [error, setError] = useState("");
  const [hasKey, setHasKey] = useState<boolean | null>(null);

  useEffect(() => {
    aiService.listAiChatMessages(projectId).then(setMessages);
  }, [projectId]);

  useEffect(() => {
    (async () => {
      const provider = (await settingsService.readSetting("ai.provider")) ?? "anthropic";
      setHasKey(await aiService.hasAiApiKey(provider));
    })();
  }, [stage]);

  function resolveChapterAndScene(): { chapterId: string | null; sceneId: string | null } {
    if (!selection) return { chapterId: null, sceneId: null };
    if (selection.ownerType === "chapter") return { chapterId: selection.ownerId, sceneId: null };
    const chapterId =
      Object.entries(scenesByChapter).find(([, scenes]) => scenes.some((s) => s.id === selection.ownerId))?.[0] ??
      null;
    return { chapterId, sceneId: selection.ownerId };
  }

  async function handleReview() {
    const text = draft.trim();
    if (!text) return;
    setError("");
    const { chapterId, sceneId } = resolveChapterAndScene();
    const candidates = await aiService.buildAiContext(projectId, chapterId, sceneId, text);
    setBlocks(candidates);
    setSelectedLabels(new Set(candidates.filter((b) => b.included_by_default).map((b) => b.label)));
    setStage("reviewing");
  }

  function toggleLabel(label: string) {
    setSelectedLabels((cur) => {
      const next = new Set(cur);
      if (next.has(label)) next.delete(label);
      else next.add(label);
      return next;
    });
  }

  async function handleSend() {
    const text = draft.trim();
    if (!text) return;
    setPosting(true);
    setError("");
    try {
      const [provider, model, baseUrl, temperature, maxOutputTokens] = await Promise.all([
        settingsService.readSetting("ai.provider"),
        settingsService.readSetting("ai.model"),
        settingsService.readSetting("ai.base_url"),
        settingsService.readSetting("ai.temperature"),
        settingsService.readSetting("ai.max_output_tokens"),
      ]);
      if (!provider || !model) {
        setError("先にAI設定でProviderとModelを設定してください。");
        setPosting(false);
        return;
      }
      const selectedBlocks = blocks.filter((b) => selectedLabels.has(b.label));
      const newMessages = await aiService.sendAiChatMessage(
        projectId,
        provider,
        model,
        baseUrl || null,
        temperature ? Number(temperature) : null,
        maxOutputTokens ? Number(maxOutputTokens) : null,
        selectedBlocks,
        text,
      );
      setMessages((ms) => [...ms, ...newMessages]);
      setDraft("");
      setBlocks([]);
      setSelectedLabels(new Set());
      setStage("idle");
    } catch (err) {
      setError(String(err));
    } finally {
      setPosting(false);
    }
  }

  async function handleClear() {
    await aiService.clearAiChat(projectId);
    setMessages([]);
  }

  return (
    <div className="section-layout">
      <div className="comments-panel" style={{ padding: 12 }}>
        {hasKey === false && (
          <div className="status-line">
            APIキーが未設定です。AI設定タブから登録すると送信できます(未設定でも他の全機能は利用できます)。
          </div>
        )}
        <ul className="comments-panel__list">
          {messages.map((m) => (
            <li key={m.id} className={`comment-card ${m.role === "assistant" ? "" : "comment-card--resolved"}`}>
              <div className="entity-list__meta">
                {m.role === "user" ? "あなた" : `AI (${m.provider ?? ""} ${m.model ?? ""})`}
                {m.context_summary && ` ・ 参照: ${m.context_summary}`}
              </div>
              <div className="comment-card__body">{m.content}</div>
            </li>
          ))}
          {messages.length === 0 && <li className="status-line">まだ会話はありません。</li>}
        </ul>

        {stage === "reviewing" && (
          <ContextInspector blocks={blocks} selectedLabels={selectedLabels} onToggle={toggleLabel} />
        )}
        {error && <div className="status-line">{error}</div>}

        <textarea
          rows={3}
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          placeholder="AIに質問・相談する内容を入力..."
          disabled={posting}
        />
        <div className="field-row">
          <button className="secondary" onClick={handleClear} type="button">
            会話をクリア
          </button>
          {stage !== "reviewing" ? (
            <button className="primary" onClick={handleReview} type="button" disabled={!draft.trim()}>
              送信内容を確認
            </button>
          ) : (
            <>
              <button
                className="secondary"
                type="button"
                disabled={posting}
                onClick={() => {
                  setStage("idle");
                  setBlocks([]);
                }}
              >
                キャンセル
              </button>
              <button className="primary" type="button" onClick={handleSend} disabled={posting}>
                {posting ? "送信中..." : "この内容で送信"}
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
