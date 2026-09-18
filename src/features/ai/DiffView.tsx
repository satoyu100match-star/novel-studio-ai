import { computeDiff } from "@/features/ai/diff";

export function DiffView({ before, after }: { before: string; after: string }) {
  const tokens = computeDiff(before, after);
  return (
    <div className="diff-view">
      {tokens.map((t, i) => {
        if (t.op === "equal") return <span key={i}>{t.text}</span>;
        if (t.op === "delete") {
          return (
            <span key={i} className="diff-view__delete">
              {t.text}
            </span>
          );
        }
        return (
          <span key={i} className="diff-view__insert">
            {t.text}
          </span>
        );
      })}
    </div>
  );
}
