import { useEffect, useState } from "react";
import type { AiUsageSummary } from "@/types/tauriCommands";
import * as aiService from "@/services/aiService";

/**
 * AI使用量表示(仕様#46)。本日・今月のリクエスト数・推定トークンのみを
 * 表示し、金額換算は行わない(プロバイダーごとに単価体系が異なり、
 * 不正確な数値を断定的に出さない方針 -- docs/AI.md 6章)。
 */
export function AiUsagePanel({ projectId }: { projectId: string }) {
  const [projectSummary, setProjectSummary] = useState<AiUsageSummary | null>(null);
  const [appSummary, setAppSummary] = useState<AiUsageSummary | null>(null);

  useEffect(() => {
    aiService.getAiUsageSummary(projectId).then(setProjectSummary);
    aiService.getAiUsageSummary(null).then(setAppSummary);
  }, [projectId]);

  function renderTable(title: string, summary: AiUsageSummary | null) {
    return (
      <div className="detail-form__group">
        <legend>{title}</legend>
        {summary ? (
          <table style={{ fontSize: 13, borderCollapse: "collapse" }}>
            <thead>
              <tr>
                <th></th>
                <th style={{ textAlign: "right", padding: "2px 12px" }}>リクエスト数</th>
                <th style={{ textAlign: "right", padding: "2px 12px" }}>入力トークン(推定)</th>
                <th style={{ textAlign: "right", padding: "2px 12px" }}>出力トークン(推定)</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>本日</td>
                <td style={{ textAlign: "right", padding: "2px 12px" }}>{summary.today_requests}</td>
                <td style={{ textAlign: "right", padding: "2px 12px" }}>{summary.today_input_tokens.toLocaleString()}</td>
                <td style={{ textAlign: "right", padding: "2px 12px" }}>{summary.today_output_tokens.toLocaleString()}</td>
              </tr>
              <tr>
                <td>今月</td>
                <td style={{ textAlign: "right", padding: "2px 12px" }}>{summary.month_requests}</td>
                <td style={{ textAlign: "right", padding: "2px 12px" }}>{summary.month_input_tokens.toLocaleString()}</td>
                <td style={{ textAlign: "right", padding: "2px 12px" }}>{summary.month_output_tokens.toLocaleString()}</td>
              </tr>
            </tbody>
          </table>
        ) : (
          <p className="status-line">読み込み中...</p>
        )}
      </div>
    );
  }

  return (
    <div className="detail-form">
      {renderTable("この作品の使用量", projectSummary)}
      {renderTable("アプリ全体の使用量", appSummary)}
      <p className="status-line">
        トークン数はプロバイダーAPIが返す実測値です。金額換算はプロバイダーごとに単価体系が異なるため表示していません。
      </p>
    </div>
  );
}
