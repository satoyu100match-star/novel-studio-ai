export type AiFlowStage = "idle" | "reviewing" | "result";

const STEPS: { key: AiFlowStage; label: string }[] = [
  { key: "idle", label: "① 入力" },
  { key: "reviewing", label: "② 送信内容の確認" },
  { key: "result", label: "③ 結果の確認・適用" },
];

/**
 * Phase13: 「操作の流れが分かりにくい」というフィードバックを受けて追加。
 * AI機能はどれも「入力 → 送信内容を確認 → 結果を確認して適用」という
 * 3段階を必ず踏む設計(docs/AI.md 4章)だが、その段階が画面上で見える形に
 * なっていなかった。現在どの段階にいるかを常に表示することで、
 * 「今何をしていて、次に何をすればいいか」を分かりやすくする。
 */
export function AiFlowSteps({ stage }: { stage: AiFlowStage }) {
  const currentIndex = STEPS.findIndex((s) => s.key === stage);
  return (
    <div className="ai-flow-steps">
      {STEPS.map((s, i) => (
        <span
          key={s.key}
          className={i <= currentIndex ? "ai-flow-steps__step ai-flow-steps__step--active" : "ai-flow-steps__step"}
        >
          {s.label}
        </span>
      ))}
    </div>
  );
}
