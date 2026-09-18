import { useState } from "react";
import { PlotBoard } from "@/features/plot/PlotBoard";
import { TimelinePanel } from "@/features/timeline/TimelinePanel";
import { ForeshadowingPanel } from "@/features/foreshadowing/ForeshadowingPanel";
import { TodoPanel } from "@/features/todos/TodoPanel";

type SubTab = "plot" | "timeline" | "foreshadowing" | "todo";

/**
 * ストーリー管理セクション(Phase 4)。プロットボード/時系列/伏線/TODOを
 * サブタブでまとめる(WorldPanelの構成パターンを踏襲)。本文内コメントは
 * 原稿セクションの検索パネルと切り替えるサイドパネルとして実装しており
 * (章・シーンに紐付くため)、ここには含めない -- ManuscriptEditor/
 * SidePanelSwitcher参照。
 */
export function StoryPanel({ projectId }: { projectId: string }) {
  const [tab, setTab] = useState<SubTab>("plot");

  return (
    <div className="section-layout">
      <div className="section-layout__tabs">
        <button className={tab === "plot" ? "tab tab--active" : "tab"} onClick={() => setTab("plot")}>
          プロットボード
        </button>
        <button className={tab === "timeline" ? "tab tab--active" : "tab"} onClick={() => setTab("timeline")}>
          時系列
        </button>
        <button className={tab === "foreshadowing" ? "tab tab--active" : "tab"} onClick={() => setTab("foreshadowing")}>
          伏線
        </button>
        <button className={tab === "todo" ? "tab tab--active" : "tab"} onClick={() => setTab("todo")}>
          TODO
        </button>
      </div>
      {tab === "plot" && <PlotBoard projectId={projectId} />}
      {tab === "timeline" && <TimelinePanel projectId={projectId} />}
      {tab === "foreshadowing" && <ForeshadowingPanel projectId={projectId} />}
      {tab === "todo" && <TodoPanel projectId={projectId} />}
    </div>
  );
}
