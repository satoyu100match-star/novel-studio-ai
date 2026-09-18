import { useState } from "react";
import { ReadabilityPanel } from "@/features/analysis/ReadabilityPanel";
import { AiAnalysisPanel } from "@/features/analysis/AiAnalysisPanel";

type SubTab = "readability" | "ai";

/**
 * Phase 7: 高度分析セクション。可読性分析(AI不要、常に利用可能)と
 * AI分析(矛盾チェック等、AI設定が必要)をサブタブで切り替える。
 */
export function AnalysisPanel({ projectId }: { projectId: string }) {
  const [tab, setTab] = useState<SubTab>("readability");

  return (
    <div className="section-layout">
      <div className="section-layout__tabs">
        <button className={tab === "readability" ? "tab tab--active" : "tab"} onClick={() => setTab("readability")}>
          可読性
        </button>
        <button className={tab === "ai" ? "tab tab--active" : "tab"} onClick={() => setTab("ai")}>
          AI分析
        </button>
      </div>
      <div style={{ padding: "12px 16px", flex: 1, minHeight: 0, overflowY: "auto" }}>
        {tab === "readability" && <ReadabilityPanel projectId={projectId} />}
        {tab === "ai" && <AiAnalysisPanel projectId={projectId} />}
      </div>
    </div>
  );
}
