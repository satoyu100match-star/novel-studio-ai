import { useEffect, useState } from "react";
import { ContextInspector } from "@/features/ai/ContextInspector";
import { AI_ANALYSIS_TYPE_LABELS, type AiAnalysisReport, type ContextBlock } from "@/types/tauriCommands";
import * as analysisService from "@/services/analysisService";
import * as settingsService from "@/services/settingsService";
import * as characterService from "@/services/characterService";

const ANALYSIS_TYPES = ["contradiction", "character_tone", "timeline", "setting", "foreshadowing", "style"];
const REQUIRES_CHARACTER = new Set(["character_tone"]);

type Stage = "idle" | "reviewing";

/**
 * Phase 7: 高度AI分析(矛盾チェック/Character口調/Timeline矛盾/設定矛盾/
 * 未回収伏線/文体講評)。
 *
 * Phase6の執筆支援機能と同じく「内容確認 → 送信」の順を必ず踏む
 * (Context Inspectorを経由しない送信経路は作らない)。分析結果は原稿等の
 * データを一切書き換えない読み取り専用の機能なので、Phase6のような
 * 「適用」ボタンは存在しない -- 結果は保存され、いつでも見返せる。
 */
export function AiAnalysisPanel({ projectId }: { projectId: string }) {
  const [analysisType, setAnalysisType] = useState("contradiction");
  const [characters, setCharacters] = useState<{ id: string; name: string }[]>([]);
  const [characterId, setCharacterId] = useState("");
  const [stage, setStage] = useState<Stage>("idle");
  const [posting, setPosting] = useState(false);
  const [blocks, setBlocks] = useState<ContextBlock[]>([]);
  const [selectedLabels, setSelectedLabels] = useState<Set<string>>(new Set());
  const [error, setError] = useState("");
  const [reports, setReports] = useState<AiAnalysisReport[]>([]);
  const [reportsLoading, setReportsLoading] = useState(false);

  const requiresCharacter = REQUIRES_CHARACTER.has(analysisType);

  async function loadReports() {
    setReportsLoading(true);
    const list = await analysisService.listAiAnalysisReports(projectId, analysisType);
    setReports(list);
    setReportsLoading(false);
  }

  useEffect(() => {
    setStage("idle");
    setError("");
    loadReports();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [analysisType, projectId]);

  useEffect(() => {
    characterService.listCharacters(projectId).then((list) => setCharacters(list.map((c) => ({ id: c.id, name: c.name }))));
  }, [projectId]);

  async function handleReview() {
    setError("");
    if (requiresCharacter && !characterId) {
      setError("対象キャラクターを選択してください。");
      return;
    }
    const candidates = await analysisService.buildAiAnalysisContext(projectId, analysisType, characterId || null);
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
      await analysisService.runAiAnalysis(
        projectId,
        analysisType,
        requiresCharacter ? characterId : null,
        provider,
        model,
        baseUrl || null,
        temperature ? Number(temperature) : null,
        maxOutputTokens ? Number(maxOutputTokens) : null,
        selectedBlocks,
      );
      setStage("idle");
      await loadReports();
    } catch (err) {
      setError(String(err));
    } finally {
      setPosting(false);
    }
  }

  async function handleDelete(id: string) {
    await analysisService.deleteAiAnalysisReport(id);
    await loadReports();
  }

  return (
    <div className="detail-form" style={{ maxWidth: 760 }}>
      <label className="form-label">
        分析の種類
        <select value={analysisType} onChange={(e) => setAnalysisType(e.target.value)}>
          {ANALYSIS_TYPES.map((t) => (
            <option key={t} value={t}>
              {AI_ANALYSIS_TYPE_LABELS[t] ?? t}
            </option>
          ))}
        </select>
      </label>

      {requiresCharacter && (
        <label className="form-label">
          対象キャラクター
          <select value={characterId} onChange={(e) => setCharacterId(e.target.value)}>
            <option value="">選択してください</option>
            {characters.map((c) => (
              <option key={c.id} value={c.id}>
                {c.name}
              </option>
            ))}
          </select>
        </label>
      )}

      {stage === "idle" && (
        <div className="field-row">
          <button className="primary" type="button" onClick={handleReview}>
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
              {posting ? "分析中..." : "この内容で分析"}
            </button>
          </div>
        </>
      )}

      {error && <div className="status-line">{error}</div>}

      <div className="detail-form__group">
        <legend>分析結果の履歴({AI_ANALYSIS_TYPE_LABELS[analysisType] ?? analysisType})</legend>
        {reportsLoading && <p className="status-line">読み込み中...</p>}
        {!reportsLoading && reports.length === 0 && <p className="status-line">まだ分析結果はありません。</p>}
        <ul className="entity-list">
          {reports.map((r) => (
            <li key={r.id} className="comment-card">
              <div className="comment-card__meta">
                <span>{new Date(r.created_at).toLocaleString()}</span>
                <span>
                  {r.provider} / {r.model}
                </span>
                {r.target_summary && <span>対象: {r.target_summary}</span>}
              </div>
              <div className="comment-card__quote">{r.result}</div>
              <div className="field-row">
                <button className="secondary" type="button" onClick={() => handleDelete(r.id)}>
                  削除
                </button>
              </div>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
