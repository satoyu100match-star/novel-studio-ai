import { useEffect, useState } from "react";
import type { ReadabilityStats } from "@/types/tauriCommands";
import * as analysisService from "@/services/analysisService";

function pct(v: number): string {
  return `${(v * 100).toFixed(1)}%`;
}

/**
 * 可読性分析(AI不要)。作品全文を対象にした純粋な文字集計のみで、
 * AI設定が無くても常に表示できる(CLAUDE.md「AI機能が利用不能でも通常の
 * 執筆機能は動作すること」)。AIによる定性的な講評は「AI分析」タブの
 * 「文体講評」から行える。
 */
export function ReadabilityPanel({ projectId }: { projectId: string }) {
  const [stats, setStats] = useState<ReadabilityStats | null>(null);
  const [loading, setLoading] = useState(false);

  async function load() {
    setLoading(true);
    const result = await analysisService.computeReadabilityStats(projectId);
    setStats(result);
    setLoading(false);
  }

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [projectId]);

  if (loading && !stats) {
    return <p className="status-line">集計中...</p>;
  }
  if (!stats || stats.total_chars === 0) {
    return (
      <div className="detail-form">
        <p className="status-line">本文がまだありません。原稿を書き進めると集計されます。</p>
        <div className="field-row">
          <button className="secondary" type="button" onClick={load}>
            再計算
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="detail-form" style={{ maxWidth: 640 }}>
      <div className="field-row">
        <button className="secondary" type="button" onClick={load} disabled={loading}>
          {loading ? "集計中..." : "再計算"}
        </button>
      </div>
      <dl className="readability-stats">
        <div className="readability-stats__row">
          <dt>総文字数</dt>
          <dd>{stats.total_chars.toLocaleString()}字</dd>
        </div>
        <div className="readability-stats__row">
          <dt>文数</dt>
          <dd>{stats.sentence_count.toLocaleString()}文</dd>
        </div>
        <div className="readability-stats__row">
          <dt>平均文長</dt>
          <dd>{stats.avg_sentence_length.toFixed(1)}字</dd>
        </div>
        <div className="readability-stats__row">
          <dt>最長文</dt>
          <dd>{stats.max_sentence_length.toLocaleString()}字</dd>
        </div>
        <div className="readability-stats__row">
          <dt>100字超の長文</dt>
          <dd>{stats.long_sentence_count.toLocaleString()}文</dd>
        </div>
        <div className="readability-stats__row">
          <dt>会話文比率</dt>
          <dd>{pct(stats.dialogue_ratio)}</dd>
        </div>
        <div className="readability-stats__row">
          <dt>漢字比率</dt>
          <dd>{pct(stats.kanji_ratio)}</dd>
        </div>
        <div className="readability-stats__row">
          <dt>ひらがな比率</dt>
          <dd>{pct(stats.hiragana_ratio)}</dd>
        </div>
        <div className="readability-stats__row">
          <dt>カタカナ比率</dt>
          <dd>{pct(stats.katakana_ratio)}</dd>
        </div>
        <div className="readability-stats__row">
          <dt>一文あたりの読点数</dt>
          <dd>{stats.avg_touten_per_sentence.toFixed(1)}個</dd>
        </div>
      </dl>
      <p className="status-line">
        あくまで機械的な集計の目安です。長文・読点数の多さが必ずしも読みにくさを意味するとは
        限りません。定性的な講評は「AI分析」タブの「文体講評」で確認できます。
      </p>
    </div>
  );
}
