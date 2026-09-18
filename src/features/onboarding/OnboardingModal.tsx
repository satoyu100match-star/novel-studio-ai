import { useState } from "react";
import { APP_NAME_FALLBACK } from "@/app/config";

const STEPS = [
  {
    title: `${APP_NAME_FALLBACK}へようこそ`,
    body: "AIを本格的に活用した小説制作専用デスクトップIDEです。執筆・原稿用紙・キャラクター・世界観・プロット・時系列・伏線・資料・AI相談・矛盾チェック・推敲・Exportまでを1つのアプリで完結できます。",
  },
  {
    title: "まずは書いてみる、触ってみる",
    body: "「新しい小説」から作品を作ってすぐに書き始められます。何もない状態から試したい場合は、作品一覧の「サンプル作品を開く」から、登場人物・世界観・プロットなどが最初から入ったサンプルを開いてみてください。",
  },
  {
    title: "ワークスペースの構成",
    body: "作品を開くと、原稿/人物/世界観/ストーリー/AI/分析/ゴミ箱/エクスポートのタブで機能を切り替えられます。AI機能はすべて任意で、設定しなくても他の機能は通常どおり使えます。",
  },
  {
    title: "データは自動で守られます",
    body: "本文は入力のたびに自動保存され、章/シーンごとの履歴(バージョン管理)や、アプリ全体の自動バックアップも用意されています。詳しくはいつでも「このアプリについて」から確認できます。",
  },
];

/**
 * Phase 10: 初回起動時のOnboarding。`app_settings`の"app.onboarding_seen"
 * が立っていない場合にのみApp.tsxから表示される(1回だけ)。ブロッキング
 * にはせず、いつでも「スキップ」で閉じられる(何もない状態からでも
 * すぐ執筆を始められることを優先 -- CLAUDE.md「執筆体験」優先度)。
 */
export function OnboardingModal({ onDone }: { onDone: () => void }) {
  const [stepIndex, setStepIndex] = useState(0);
  const step = STEPS[stepIndex];
  const isLast = stepIndex === STEPS.length - 1;

  return (
    <div className="modal-overlay" role="dialog" aria-label="ようこそ">
      <div className="modal card">
        <h1 style={{ margin: 0 }}>{step.title}</h1>
        <p className="status-line">{step.body}</p>
        <div className="field-row" style={{ justifyContent: "space-between" }}>
          <button className="secondary" type="button" onClick={onDone}>
            スキップ
          </button>
          <div className="field-row">
            <span className="status-line" style={{ alignSelf: "center" }}>
              {stepIndex + 1} / {STEPS.length}
            </span>
            {stepIndex > 0 && (
              <button className="secondary" type="button" onClick={() => setStepIndex((i) => i - 1)}>
                戻る
              </button>
            )}
            <button
              className="primary"
              type="button"
              onClick={() => (isLast ? onDone() : setStepIndex((i) => i + 1))}
            >
              {isLast ? "始める" : "次へ"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
