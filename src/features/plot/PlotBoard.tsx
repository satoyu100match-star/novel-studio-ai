import { useEffect, useState } from "react";
import type { PlotCard, PlotLane } from "@/types/tauriCommands";
import * as plotService from "@/services/plotService";

/**
 * プロットボード(仕様#30〜32相当)。カンバン方式: レーン=進行状況、
 * カード=プロット単位のイベント/展開。ドラッグ&ドロップでのカード移動は
 * HTML5ネイティブDnD(`draggable`属性)で実装 -- 追加ライブラリなしで
 * 動く最小実装(package.jsonに既存のDnDライブラリがないため)。
 */
export function PlotBoard({ projectId }: { projectId: string }) {
  const [lanes, setLanes] = useState<PlotLane[]>([]);
  const [cards, setCards] = useState<PlotCard[]>([]);
  const [newCardTitle, setNewCardTitle] = useState<Record<string, string>>({});
  const [newLaneName, setNewLaneName] = useState("");
  const [draggingId, setDraggingId] = useState<string | null>(null);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editSummary, setEditSummary] = useState("");

  useEffect(() => {
    plotService.listPlotLanes(projectId).then(setLanes);
    plotService.listPlotCards(projectId).then(setCards);
  }, [projectId]);

  const cardsForLane = (laneId: string) =>
    cards.filter((c) => c.lane_id === laneId).sort((a, b) => a.order_index - b.order_index);

  async function handleAddCard(laneId: string) {
    const title = (newCardTitle[laneId] ?? "").trim();
    if (!title) return;
    const created = await plotService.createPlotCard(projectId, laneId, title);
    setCards((cs) => [...cs, created]);
    setNewCardTitle((m) => ({ ...m, [laneId]: "" }));
  }

  async function handleAddLane(e: React.FormEvent) {
    e.preventDefault();
    const name = newLaneName.trim();
    if (!name) return;
    const created = await plotService.createPlotLane(projectId, name);
    setLanes((ls) => [...ls, created]);
    setNewLaneName("");
  }

  async function handleDrop(laneId: string) {
    if (!draggingId) return;
    const card = cards.find((c) => c.id === draggingId);
    setDraggingId(null);
    if (!card || card.lane_id === laneId) return;
    const nextOrder = cardsForLane(laneId).length;
    const moved = await plotService.movePlotCard(card.id, laneId, nextOrder);
    setCards((cs) => cs.map((c) => (c.id === moved.id ? moved : c)));
  }

  function startEdit(card: PlotCard) {
    setEditingId(card.id);
    setEditSummary(card.summary ?? "");
  }

  async function saveEdit(card: PlotCard) {
    const updated = await plotService.updatePlotCard(card.id, { ...card, summary: editSummary || null });
    setCards((cs) => cs.map((c) => (c.id === updated.id ? updated : c)));
    setEditingId(null);
  }

  async function handleDeleteCard(id: string) {
    await plotService.deletePlotCard(id);
    setCards((cs) => cs.filter((c) => c.id !== id));
    if (editingId === id) setEditingId(null);
  }

  return (
    <div className="plot-board">
      {lanes.map((lane) => (
        <div
          key={lane.id}
          className="plot-board__lane"
          onDragOver={(e) => e.preventDefault()}
          onDrop={() => handleDrop(lane.id)}
        >
          <div className="plot-board__lane-header">
            <span>{lane.name}</span>
            <span className="plot-board__lane-count">{cardsForLane(lane.id).length}</span>
          </div>
          <div className="plot-board__cards">
            {cardsForLane(lane.id).map((card) => (
              <div
                key={card.id}
                className={`plot-board__card ${draggingId === card.id ? "plot-board__card--dragging" : ""}`}
                draggable
                onDragStart={() => setDraggingId(card.id)}
                onDragEnd={() => setDraggingId(null)}
                onClick={() => startEdit(card)}
              >
                <span className="plot-board__card-title">{card.title}</span>
                {editingId === card.id ? (
                  <>
                    <textarea
                      rows={3}
                      value={editSummary}
                      onChange={(e) => setEditSummary(e.target.value)}
                      onClick={(e) => e.stopPropagation()}
                    />
                    <div className="plot-board__card-actions">
                      <button
                        className="secondary"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDeleteCard(card.id);
                        }}
                      >
                        削除
                      </button>
                      <button
                        className="primary"
                        onClick={(e) => {
                          e.stopPropagation();
                          saveEdit(card);
                        }}
                      >
                        保存
                      </button>
                    </div>
                  </>
                ) : (
                  card.summary && <span className="plot-board__card-summary">{card.summary}</span>
                )}
              </div>
            ))}
          </div>
          <form
            className="field-row"
            onSubmit={(e) => {
              e.preventDefault();
              handleAddCard(lane.id);
            }}
          >
            <input
              value={newCardTitle[lane.id] ?? ""}
              onChange={(e) => setNewCardTitle((m) => ({ ...m, [lane.id]: e.target.value }))}
              placeholder="カード追加"
              style={{ fontSize: 12 }}
            />
          </form>
        </div>
      ))}
      <form className="plot-board__add-lane field-row" onSubmit={handleAddLane}>
        <input value={newLaneName} onChange={(e) => setNewLaneName(e.target.value)} placeholder="新しいレーン" />
        <button className="secondary" type="submit">
          追加
        </button>
      </form>
    </div>
  );
}
