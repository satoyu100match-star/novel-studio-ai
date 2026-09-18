import { useState } from "react";
import type { ContextBlock } from "@/types/tauriCommands";

/**
 * Context Inspector(仕様#42)。実際にAIへ送信される情報を、送信前に
 * ユーザーが確認・チェックボックスで取捨選択できる画面。プライバシーと
 * API費用削減の両方に効くため、送信ボタンの手前に必ずこのステップを挟む
 * (バイパスするショートカットは作らない -- docs/AI.md 2章方針)。
 */
export function ContextInspector({
  blocks,
  selectedLabels,
  onToggle,
}: {
  blocks: ContextBlock[];
  selectedLabels: Set<string>;
  onToggle: (label: string) => void;
}) {
  const [expandedLabel, setExpandedLabel] = useState<string | null>(null);
  const totalChars = blocks
    .filter((b) => selectedLabels.has(b.label))
    .reduce((sum, b) => sum + b.char_count, 0);

  return (
    <div className="detail-form__group">
      <legend>送信内容の確認(Context Inspector)</legend>
      {blocks.length === 0 && <p className="status-line">送信候補の情報はありません。</p>}
      <ul className="entity-list">
        {blocks.map((block) => (
          <li key={block.label}>
            <label className="field-row" style={{ alignItems: "flex-start" }}>
              <input
                type="checkbox"
                checked={selectedLabels.has(block.label)}
                onChange={() => onToggle(block.label)}
              />
              <div style={{ flex: 1 }}>
                <button
                  type="button"
                  className="entity-list__item"
                  style={{ padding: 0 }}
                  onClick={() => setExpandedLabel((cur) => (cur === block.label ? null : block.label))}
                >
                  <span>{block.label}</span>
                  <span className="entity-list__meta">{block.char_count.toLocaleString()}字</span>
                </button>
                {expandedLabel === block.label && (
                  <div className="comment-card__quote" style={{ marginTop: 4 }}>
                    {block.content}
                  </div>
                )}
              </div>
            </label>
          </li>
        ))}
      </ul>
      <p className="status-line">送信予定: 合計 約{totalChars.toLocaleString()}字</p>
    </div>
  );
}
