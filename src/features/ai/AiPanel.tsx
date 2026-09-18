import { useState } from "react";
import { AiSettingsForm } from "@/features/ai/AiSettingsForm";
import { AiChatPanel } from "@/features/ai/AiChatPanel";
import { AiUsagePanel } from "@/features/ai/AiUsagePanel";
import { AiWritingAssistPanel } from "@/features/ai/AiWritingAssistPanel";
import { ConceptChatPanel } from "@/features/ai/ConceptChatPanel";

type SubTab = "concept" | "settings" | "chat" | "assist" | "usage";

/**
 * AIセクション(Phase 5基盤 + Phase 6個別支援機能 + Phase12作品設計チャット)。
 * 作品設計チャット/執筆支援/チャット/AI設定/使用量のサブタブ。
 */
export function AiPanel({ projectId }: { projectId: string }) {
  const [tab, setTab] = useState<SubTab>("assist");

  return (
    <div className="section-layout">
      <div className="section-layout__tabs">
        <button className={tab === "assist" ? "tab tab--active" : "tab"} onClick={() => setTab("assist")}>
          執筆支援
        </button>
        <button className={tab === "concept" ? "tab tab--active" : "tab"} onClick={() => setTab("concept")}>
          作品設計チャット
        </button>
        <button className={tab === "chat" ? "tab tab--active" : "tab"} onClick={() => setTab("chat")}>
          チャット
        </button>
        <button className={tab === "settings" ? "tab tab--active" : "tab"} onClick={() => setTab("settings")}>
          AI設定
        </button>
        <button className={tab === "usage" ? "tab tab--active" : "tab"} onClick={() => setTab("usage")}>
          使用量
        </button>
      </div>
      <div style={{ padding: tab === "chat" ? 0 : "12px 16px", flex: 1, minHeight: 0, overflowY: "auto" }}>
        {tab === "assist" && <AiWritingAssistPanel projectId={projectId} />}
        {tab === "concept" && <ConceptChatPanel projectId={projectId} />}
        {tab === "chat" && <AiChatPanel projectId={projectId} />}
        {tab === "settings" && <AiSettingsForm />}
        {tab === "usage" && <AiUsagePanel projectId={projectId} />}
      </div>
    </div>
  );
}
