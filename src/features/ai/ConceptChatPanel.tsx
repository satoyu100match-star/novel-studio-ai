import { useState } from "react";
import { ContextInspector } from "@/features/ai/ContextInspector";
import { AiFlowSteps, type AiFlowStage } from "@/features/ai/AiFlowSteps";
import { parseConceptDraft, type ConceptDraft } from "@/features/ai/parseGenerated";
import type { ContextBlock } from "@/types/tauriCommands";
import * as aiService from "@/services/aiService";
import * as settingsService from "@/services/settingsService";
import * as projectService from "@/services/projectService";
import * as characterService from "@/services/characterService";
import * as worldService from "@/services/worldService";
import * as plotService from "@/services/plotService";

type Stage = AiFlowStage;

const EMPTY_DRAFT: ConceptDraft = { titles: [], synopsis: "", characters: [], worldEntries: [], plotCards: [] };

/**
 * Phase12: 作品設計チャット。「こういう作品にしたい」という自由記述の
 * アイデアから、タイトル案・あらすじ・キャラクター・世界観項目・
 * プロットカードをAIに一括で提案させる専用画面。
 *
 * 既存のAI小説支援機能(Phase6)と同じく「入力 → Context Inspectorで
 * 送信内容を確認 → 送信 → 結果を確認してから適用」の順を必ず踏む
 * (docs/AI.md 4章)。提案を受け取った時点では何も確定させず、各項目に
 * チェックボックスを設けて、ユーザーが選んだ項目だけを実際の
 * Character/World項目/プロットカードとして作成する(1人物・1世界観
 * 項目でも他機能と同じ`create*`コマンドを流用するだけなので、DBの
 * 扱いとしては手動で1件ずつ追加するのと変わらない)。
 */
export function ConceptChatPanel({ projectId }: { projectId: string }) {
  const [idea, setIdea] = useState("");
  const [stage, setStage] = useState<Stage>("idle");
  const [posting, setPosting] = useState(false);
  const [applying, setApplying] = useState(false);
  const [blocks, setBlocks] = useState<ContextBlock[]>([]);
  const [selectedLabels, setSelectedLabels] = useState<Set<string>>(new Set());
  const [rawResult, setRawResult] = useState("");
  const [draft, setDraft] = useState<ConceptDraft>(EMPTY_DRAFT);
  const [error, setError] = useState("");
  const [applyMessage, setApplyMessage] = useState("");

  const [applySynopsis, setApplySynopsis] = useState(false);
  const [selectedCharacters, setSelectedCharacters] = useState<Set<number>>(new Set());
  const [selectedWorldEntries, setSelectedWorldEntries] = useState<Set<number>>(new Set());
  const [selectedPlotCards, setSelectedPlotCards] = useState<Set<number>>(new Set());

  function toggleIndex(setter: (fn: (cur: Set<number>) => Set<number>) => void, index: number) {
    setter((cur) => {
      const next = new Set(cur);
      if (next.has(index)) next.delete(index);
      else next.add(index);
      return next;
    });
  }

  function toggleContextLabel(label: string) {
    setSelectedLabels((cur) => {
      const next = new Set(cur);
      if (next.has(label)) next.delete(label);
      else next.add(label);
      return next;
    });
  }

  async function handleReview() {
    setError("");
    if (idea.trim() === "") {
      setError("作品のアイデアや要望を入力してください。");
      return;
    }
    const candidates = await aiService.buildAiContext(projectId, null, null, idea.trim());
    setBlocks(candidates);
    setSelectedLabels(new Set(candidates.filter((b) => b.included_by_default).map((b) => b.label)));
    setStage("reviewing");
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
        setError("先に「AI設定」タブでProviderとModelを設定してください。");
        return;
      }
      const selectedBlocks = blocks.filter((b) => selectedLabels.has(b.label));
      const response = await aiService.runAiWritingFeature(
        projectId,
        "generate_concept",
        provider,
        model,
        baseUrl || null,
        temperature ? Number(temperature) : null,
        maxOutputTokens ? Number(maxOutputTokens) : null,
        selectedBlocks,
        idea.trim(),
      );
      const parsed = parseConceptDraft(response.content);
      setRawResult(response.content);
      setDraft(parsed);
      setSelectedCharacters(new Set(parsed.characters.map((_, i) => i)));
      setSelectedWorldEntries(new Set(parsed.worldEntries.map((_, i) => i)));
      setSelectedPlotCards(new Set(parsed.plotCards.map((_, i) => i)));
      setApplySynopsis(false);
      setApplyMessage("");
      if (
        parsed.titles.length === 0 &&
        !parsed.synopsis &&
        parsed.characters.length === 0 &&
        parsed.worldEntries.length === 0 &&
        parsed.plotCards.length === 0
      ) {
        setError(
          "AIの応答から項目を読み取れませんでした。下の生成結果(原文)を確認するか、再生成してみてください。",
        );
      }
      setStage("result");
    } catch (err) {
      setError(String(err));
    } finally {
      setPosting(false);
    }
  }

  function handleDiscard() {
    setStage("idle");
    setDraft(EMPTY_DRAFT);
    setRawResult("");
    setApplyMessage("");
    setError("");
  }

  async function handleCopyTitles() {
    try {
      await navigator.clipboard.writeText(draft.titles.join("\n"));
      setApplyMessage("タイトル候補をクリップボードにコピーしました。");
    } catch {
      setApplyMessage("コピーに失敗しました。");
    }
  }

  async function handleApply() {
    setApplying(true);
    setApplyMessage("");
    setError("");
    try {
      let createdCharacters = 0;
      let createdWorldEntries = 0;
      let createdPlotCards = 0;

      if (applySynopsis && draft.synopsis) {
        const project = await projectService.getProject(projectId);
        if (project) {
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          const { id, createdAt, updatedAt, isSample, ...input } = project;
          await projectService.updateProject(projectId, { ...input, synopsis: draft.synopsis });
        }
      }

      for (const index of selectedCharacters) {
        const characterDraft = draft.characters[index];
        if (!characterDraft?.name) continue;
        await characterService.createCharacter(projectId, {
          ...characterService.emptyCharacterInput(),
          ...characterDraft,
        });
        createdCharacters += 1;
      }

      for (const index of selectedWorldEntries) {
        const worldDraft = draft.worldEntries[index];
        if (!worldDraft?.name) continue;
        await worldService.createWorldEntry(projectId, { ...worldService.emptyWorldEntryInput(), ...worldDraft });
        createdWorldEntries += 1;
      }

      if (selectedPlotCards.size > 0) {
        const lanes = await plotService.listPlotLanes(projectId);
        const lane = lanes[0];
        if (lane) {
          for (const index of selectedPlotCards) {
            const plotDraft = draft.plotCards[index];
            if (!plotDraft?.title) continue;
            const card = await plotService.createPlotCard(projectId, lane.id, plotDraft.title);
            await plotService.updatePlotCard(card.id, { ...card, summary: plotDraft.summary || null });
            createdPlotCards += 1;
          }
        }
      }

      const parts: string[] = [];
      if (applySynopsis && draft.synopsis) parts.push("あらすじ");
      if (createdCharacters > 0) parts.push(`キャラクター${createdCharacters}件`);
      if (createdWorldEntries > 0) parts.push(`世界観項目${createdWorldEntries}件`);
      if (createdPlotCards > 0) parts.push(`プロットカード${createdPlotCards}件`);
      setApplyMessage(
        parts.length > 0
          ? `${parts.join("・")}を作成しました。各セクションで確認・編集できます。`
          : "作成対象が選択されていません。チェックボックスで項目を選んでから実行してください。",
      );
    } catch (err) {
      setError(String(err));
    } finally {
      setApplying(false);
    }
  }

  return (
    <div className="detail-form" style={{ maxWidth: 820 }}>
      <AiFlowSteps stage={stage} />
      <p className="status-line">
        作りたい作品のイメージを自由に書いてください。既存の作品情報(あれば)も踏まえて、AIが
        タイトル案・あらすじ・キャラクター・世界観項目・プロットのアイデアを一括で提案します。
        提案はあくまで下書きです。気に入ったものだけを選んで作成でき、何も選ばなければ何も
        作成されません。
      </p>

      <label className="form-label">
        作品のアイデア・要望
        <textarea
          rows={4}
          value={idea}
          onChange={(e) => setIdea(e.target.value)}
          placeholder="例: 異世界の刑務所を舞台にしたダークファンタジー。主人公は元冒険者の新人看守で、\n収監されている元凶悪犯たちとの奇妙な共同生活を通して成長していく物語にしたい。"
        />
      </label>

      {stage === "idle" && (
        <div className="field-row">
          <button className="primary" type="button" onClick={handleReview}>
            送信内容を確認
          </button>
        </div>
      )}

      {stage === "reviewing" && (
        <>
          <ContextInspector blocks={blocks} selectedLabels={selectedLabels} onToggle={toggleContextLabel} />
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
        <>
          {draft.titles.length > 0 && (
            <fieldset className="detail-form__group">
              <legend>タイトル案</legend>
              <ul className="entity-list">
                {draft.titles.map((title, i) => (
                  <li key={i}>
                    <span>{title}</span>
                  </li>
                ))}
              </ul>
              <div className="field-row">
                <button className="secondary" type="button" onClick={handleCopyTitles}>
                  コピー
                </button>
              </div>
            </fieldset>
          )}

          {draft.synopsis && (
            <fieldset className="detail-form__group">
              <legend>あらすじ案</legend>
              <div className="diff-view" style={{ maxHeight: 160 }}>
                {draft.synopsis}
              </div>
              <label className="form-label" style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
                <input
                  type="checkbox"
                  checked={applySynopsis}
                  onChange={(e) => setApplySynopsis(e.target.checked)}
                />
                作品設定のあらすじとして採用する(既存のあらすじは上書きされます)
              </label>
            </fieldset>
          )}

          {draft.characters.length > 0 && (
            <fieldset className="detail-form__group">
              <legend>キャラクター案({draft.characters.length}件)</legend>
              <ul className="entity-list">
                {draft.characters.map((c, i) => (
                  <li key={i}>
                    <label className="entity-list__item" style={{ alignItems: "flex-start", cursor: "pointer" }}>
                      <input
                        type="checkbox"
                        checked={selectedCharacters.has(i)}
                        onChange={() => toggleIndex(setSelectedCharacters, i)}
                      />
                      <span>
                        <strong>{c.name || "(名前未取得)"}</strong>
                        {c.role ? ` — ${c.role}` : ""}
                        {c.personality ? `／${c.personality}` : ""}
                      </span>
                    </label>
                  </li>
                ))}
              </ul>
            </fieldset>
          )}

          {draft.worldEntries.length > 0 && (
            <fieldset className="detail-form__group">
              <legend>世界観項目案({draft.worldEntries.length}件)</legend>
              <ul className="entity-list">
                {draft.worldEntries.map((w, i) => (
                  <li key={i}>
                    <label className="entity-list__item" style={{ alignItems: "flex-start", cursor: "pointer" }}>
                      <input
                        type="checkbox"
                        checked={selectedWorldEntries.has(i)}
                        onChange={() => toggleIndex(setSelectedWorldEntries, i)}
                      />
                      <span>
                        <strong>{w.name || "(名前未取得)"}</strong>
                        {w.summary ? ` — ${w.summary}` : ""}
                      </span>
                    </label>
                  </li>
                ))}
              </ul>
            </fieldset>
          )}

          {draft.plotCards.length > 0 && (
            <fieldset className="detail-form__group">
              <legend>プロット案({draft.plotCards.length}件)</legend>
              <ul className="entity-list">
                {draft.plotCards.map((p, i) => (
                  <li key={i}>
                    <label className="entity-list__item" style={{ alignItems: "flex-start", cursor: "pointer" }}>
                      <input
                        type="checkbox"
                        checked={selectedPlotCards.has(i)}
                        onChange={() => toggleIndex(setSelectedPlotCards, i)}
                      />
                      <span>
                        <strong>{p.title}</strong>
                        {p.summary ? ` — ${p.summary}` : ""}
                      </span>
                    </label>
                  </li>
                ))}
              </ul>
            </fieldset>
          )}

          <details>
            <summary className="status-line" style={{ cursor: "pointer" }}>
              生成結果(原文)を表示
            </summary>
            <div className="diff-view" style={{ maxHeight: 200 }}>
              {rawResult}
            </div>
          </details>

          {applyMessage && <p className="status-line">{applyMessage}</p>}

          <div className="field-row">
            <button className="secondary" type="button" onClick={handleDiscard} disabled={applying}>
              破棄
            </button>
            <button className="secondary" type="button" onClick={handleSend} disabled={posting || applying}>
              {posting ? "生成中..." : "再生成"}
            </button>
            <button className="primary" type="button" onClick={handleApply} disabled={applying}>
              {applying ? "作成中..." : "選択した項目を作成"}
            </button>
          </div>
        </>
      )}
    </div>
  );
}
