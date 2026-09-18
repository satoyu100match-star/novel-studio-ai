import { useEffect, useState } from "react";
import { useManuscriptStore } from "@/features/manuscript/manuscriptStore";
import { ContextInspector } from "@/features/ai/ContextInspector";
import { DiffView } from "@/features/ai/DiffView";
import { AiFlowSteps, type AiFlowStage } from "@/features/ai/AiFlowSteps";
import { parseCharacterDraft, parsePlotDrafts, parseWorldEntryDraft } from "@/features/ai/parseGenerated";
import { AI_WRITING_FEATURE_DESCRIPTIONS, AI_WRITING_FEATURE_LABELS, type ContextBlock } from "@/types/tauriCommands";
import * as aiService from "@/services/aiService";
import * as settingsService from "@/services/settingsService";
import * as projectService from "@/services/projectService";
import * as characterService from "@/services/characterService";
import * as worldService from "@/services/worldService";
import * as plotService from "@/services/plotService";

const TEXT_EDITING_FEATURES = new Set(["rewrite", "add_description", "improve_dialogue"]);

// Phase13: 「どの機能が何をするか分かりにくい」への対応として、本文編集
// (選択中の本文が必須)とアイデア生成(本文選択は不要)をプルダウン内で
// 見出し(optgroup)で分け、機能の性質が一目で分かるようにした。
const TEXT_EDITING_GROUP: { value: string; label: string }[] = [
  { value: "rewrite", label: AI_WRITING_FEATURE_LABELS.rewrite },
  { value: "continue", label: AI_WRITING_FEATURE_LABELS.continue },
  { value: "add_description", label: AI_WRITING_FEATURE_LABELS.add_description },
  { value: "improve_dialogue", label: AI_WRITING_FEATURE_LABELS.improve_dialogue },
];
const GENERATION_GROUP: { value: string; label: string }[] = [
  { value: "generate_character", label: AI_WRITING_FEATURE_LABELS.generate_character },
  { value: "generate_world", label: AI_WRITING_FEATURE_LABELS.generate_world },
  { value: "generate_plot", label: AI_WRITING_FEATURE_LABELS.generate_plot },
  { value: "generate_synopsis", label: AI_WRITING_FEATURE_LABELS.generate_synopsis },
  { value: "generate_titles", label: AI_WRITING_FEATURE_LABELS.generate_titles },
];

type Stage = AiFlowStage;

/**
 * Phase 6: AI小説支援機能(推敲/続き案/描写追加/会話改善/各種生成)。
 *
 * どの機能でも「入力 → Context Inspectorで確認 → 送信 → 結果を確認して
 * から適用」の順を必ず踏む(docs/AI.md 4章)。原稿やキャラクター等への
 * 書き込みは、ユーザーが結果を見て明示的に「適用」ボタンを押した時のみ
 * 行われる -- AIの応答を受け取った時点では何も確定しない。
 */
export function AiWritingAssistPanel({ projectId }: { projectId: string }) {
  const selection = useManuscriptStore((s) => s.selection);
  const scenesByChapter = useManuscriptStore((s) => s.scenesByChapter);
  const editorBody = useManuscriptStore((s) => s.editorBody);
  const setEditorBody = useManuscriptStore((s) => s.setEditorBody);

  const [feature, setFeature] = useState("rewrite");
  const [extraInstruction, setExtraInstruction] = useState("");
  const [stage, setStage] = useState<Stage>("idle");
  const [posting, setPosting] = useState(false);
  const [blocks, setBlocks] = useState<ContextBlock[]>([]);
  const [selectedLabels, setSelectedLabels] = useState<Set<string>>(new Set());
  const [result, setResult] = useState("");
  const [error, setError] = useState("");
  const [applyMessage, setApplyMessage] = useState("");

  useEffect(() => {
    setStage("idle");
    setResult("");
    setError("");
    setApplyMessage("");
  }, [feature]);

  const requiresManuscript = TEXT_EDITING_FEATURES.has(feature);
  const targetText = requiresManuscript ? editorBody : "";
  const manuscriptMissing = requiresManuscript && (!selection || targetText.trim() === "");

  function resolveChapterAndScene(): { chapterId: string | null; sceneId: string | null } {
    if (!selection) return { chapterId: null, sceneId: null };
    if (selection.ownerType === "chapter") return { chapterId: selection.ownerId, sceneId: null };
    const chapterId =
      Object.entries(scenesByChapter).find(([, scenes]) => scenes.some((s) => s.id === selection.ownerId))?.[0] ??
      null;
    return { chapterId, sceneId: selection.ownerId };
  }

  async function handleReview() {
    setError("");
    const { chapterId, sceneId } = resolveChapterAndScene();
    const candidates = await aiService.buildAiContext(projectId, chapterId, sceneId, extraInstruction.trim() || null);
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
        setError("先にAI設定タブでProviderとModelを設定してください。");
        return;
      }
      const selectedBlocks = blocks.filter((b) => selectedLabels.has(b.label));
      const inputText = requiresManuscript ? targetText : extraInstruction.trim() || null;
      const response = await aiService.runAiWritingFeature(
        projectId,
        feature,
        provider,
        model,
        baseUrl || null,
        temperature ? Number(temperature) : null,
        maxOutputTokens ? Number(maxOutputTokens) : null,
        selectedBlocks,
        inputText,
      );
      setResult(response.content);
      setStage("result");
    } catch (err) {
      setError(String(err));
    } finally {
      setPosting(false);
    }
  }

  function handleDiscard() {
    setStage("idle");
    setResult("");
    setApplyMessage("");
  }

  async function handleApply() {
    setApplyMessage("");
    if (feature === "rewrite" || feature === "add_description" || feature === "improve_dialogue") {
      setEditorBody(result);
      setApplyMessage("原稿に適用しました(自動保存されます)。「原稿」セクションで確認できます。");
    } else if (feature === "continue") {
      const needsBreak = editorBody.length > 0 && !editorBody.endsWith("\n");
      setEditorBody(editorBody + (needsBreak ? "\n" : "") + result);
      setApplyMessage("本文の末尾に追加しました(自動保存されます)。「原稿」セクションで確認できます。");
    } else if (feature === "generate_synopsis") {
      const project = await projectService.getProject(projectId);
      if (!project) return;
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const { id, createdAt, updatedAt, isSample, ...input } = project;
      await projectService.updateProject(projectId, { ...input, synopsis: result });
      setApplyMessage("作品設定のあらすじを更新しました。");
    } else if (feature === "generate_character") {
      const draft = parseCharacterDraft(result);
      if (!draft.name) {
        setError("名前を読み取れませんでした。出力内容を確認してください。");
        return;
      }
      await characterService.createCharacter(projectId, { ...characterService.emptyCharacterInput(), ...draft });
      setApplyMessage("新規キャラクターとして作成しました。「人物」セクションで確認・編集できます。");
    } else if (feature === "generate_world") {
      const draft = parseWorldEntryDraft(result);
      if (!draft.name) {
        setError("名前を読み取れませんでした。出力内容を確認してください。");
        return;
      }
      await worldService.createWorldEntry(projectId, { ...worldService.emptyWorldEntryInput(), ...draft });
      setApplyMessage("新規世界観項目として作成しました。「世界観」セクションで確認・編集できます。");
    } else if (feature === "generate_plot") {
      const drafts = parsePlotDrafts(result);
      if (drafts.length === 0) {
        setError("プロット案を読み取れませんでした。出力内容を確認してください。");
        return;
      }
      const lanes = await plotService.listPlotLanes(projectId);
      const lane = lanes[0];
      if (!lane) return;
      for (const draft of drafts) {
        const card = await plotService.createPlotCard(projectId, lane.id, draft.title);
        await plotService.updatePlotCard(card.id, { ...card, summary: draft.summary || null });
      }
      setApplyMessage(`${drafts.length}件のプロットカードを追加しました。「ストーリー」セクションで確認できます。`);
    }
  }

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(result);
      setApplyMessage("クリップボードにコピーしました。");
    } catch {
      setApplyMessage("コピーに失敗しました。");
    }
  }

  const canApply = feature !== "generate_titles";

  return (
    <div className="detail-form" style={{ maxWidth: 760 }}>
      <AiFlowSteps stage={stage} />

      <label className="form-label">
        機能
        <select value={feature} onChange={(e) => setFeature(e.target.value)}>
          <optgroup label="本文編集(原稿セクションで本文の選択が必要)">
            {TEXT_EDITING_GROUP.map((f) => (
              <option key={f.value} value={f.value}>
                {f.label}
              </option>
            ))}
          </optgroup>
          <optgroup label="アイデア生成(本文選択は不要)">
            {GENERATION_GROUP.map((f) => (
              <option key={f.value} value={f.value}>
                {f.label}
              </option>
            ))}
          </optgroup>
        </select>
      </label>
      <p className="ai-feature-description">{AI_WRITING_FEATURE_DESCRIPTIONS[feature]}</p>

      {requiresManuscript && (
        <div className="detail-form__group">
          <legend>対象の本文(現在選択中の章/シーンの本文全体)</legend>
          {manuscriptMissing ? (
            <p className="status-line">先に「原稿」セクションで章またはシーンを選択してください。</p>
          ) : (
            <div className="diff-view" style={{ maxHeight: 160 }}>
              {targetText}
            </div>
          )}
        </div>
      )}

      <label className="form-label">
        追加の指示(任意)
        <textarea
          rows={2}
          value={extraInstruction}
          onChange={(e) => setExtraInstruction(e.target.value)}
          placeholder="例: もっと緊迫感を出して / 主人公は内気な性格"
        />
      </label>

      {stage === "idle" && (
        <div className="field-row">
          <button
            className="primary"
            type="button"
            onClick={handleReview}
            disabled={requiresManuscript && manuscriptMissing}
          >
            送信内容を確認
          </button>
        </div>
      )}

      {stage === "reviewing" && (
        <>
          <ContextInspector blocks={blocks} selectedLabels={selectedLabels} onToggle={toggleLabel} />
          <div className="field-row">
            <button className="secondary" type="button" onClick={() => setStage("idle")} disabled={posting}>
              戻る
            </button>
            <button className="primary" type="button" onClick={handleSend} disabled={posting}>
              {posting ? "生成中..." : "この内容で送信"}
            </button>
          </div>
        </>
      )}

      {error && <div className="status-line">{error}</div>}

      {stage === "result" && (
        <div className="detail-form__group">
          <legend>AIの提案</legend>
          {(feature === "rewrite" || feature === "add_description" || feature === "improve_dialogue") && (
            <DiffView before={targetText} after={result} />
          )}
          <textarea rows={8} value={result} onChange={(e) => setResult(e.target.value)} />
          {applyMessage && <p className="status-line">{applyMessage}</p>}
          <div className="field-row">
            <button className="secondary" type="button" onClick={handleDiscard}>
              破棄
            </button>
            <button className="secondary" type="button" onClick={handleSend} disabled={posting}>
              再生成
            </button>
            <button className="secondary" type="button" onClick={handleCopy}>
              コピー
            </button>
            {canApply && (
              <button className="primary" type="button" onClick={handleApply}>
                適用
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
