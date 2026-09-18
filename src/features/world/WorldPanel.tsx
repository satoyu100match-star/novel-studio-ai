import { useState } from "react";
import { WorldEntriesTab } from "@/features/world/WorldEntriesTab";
import { GlossaryTab } from "@/features/world/GlossaryTab";
import { LocationsTab } from "@/features/world/LocationsTab";

type SubTab = "entries" | "glossary" | "locations";

export function WorldPanel({ projectId }: { projectId: string }) {
  const [tab, setTab] = useState<SubTab>("entries");

  return (
    <div className="section-layout">
      <div className="section-layout__tabs">
        <button className={tab === "entries" ? "tab tab--active" : "tab"} onClick={() => setTab("entries")}>
          世界観項目
        </button>
        <button className={tab === "glossary" ? "tab tab--active" : "tab"} onClick={() => setTab("glossary")}>
          用語辞典
        </button>
        <button className={tab === "locations" ? "tab tab--active" : "tab"} onClick={() => setTab("locations")}>
          場所
        </button>
      </div>
      {tab === "entries" && <WorldEntriesTab projectId={projectId} />}
      {tab === "glossary" && <GlossaryTab projectId={projectId} />}
      {tab === "locations" && <LocationsTab projectId={projectId} />}
    </div>
  );
}
