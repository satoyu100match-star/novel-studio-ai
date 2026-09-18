import { useMemo, useState } from "react";
import {
  DEFAULT_TYPESET_OPTIONS,
  layoutPageCount,
  MANUSCRIPT_SIZE_PRESETS,
  typeset,
  type KinsokuLevel,
  type TypesetCell,
} from "@/features/manuscript/typeset";

/**
 * 原稿用紙ビュー(Phase 2)。
 *
 * 重要: これは本文データの「表示ビュー」であり、編集はしない
 * (docs/MANUSCRIPT.md 1章: 本文とビューの分離)。実際の入力・編集は
 * Phase 1の標準エディタ(textarea, IME安全)で行い、このビューはその
 * 結果を組版して確認する画面という位置づけにする。
 *
 * セルごとの入力欄(400個のinput等)には絶対にしない -- 仕様#18。
 */

const KINSOKU_LABEL: Record<KinsokuLevel, string> = {
  strict: "厳密",
  standard: "標準",
  simple: "簡易",
};

function Cell({ cell }: { cell: TypesetCell }) {
  return (
    <span className={`ms-cell ${cell.tcy ? "ms-cell--tcy" : ""} ${cell.bouten ? "ms-cell--bouten" : ""}`}>
      {cell.ruby && (
        <span className="ms-cell__ruby" style={{ ["--ruby-span" as string]: cell.rubySpan }}>
          {cell.ruby}
        </span>
      )}
      <span className="ms-cell__char">{cell.char === "" ? " " : cell.char}</span>
    </span>
  );
}

export function GenkouYoushiView({ body }: { body: string }) {
  const [orientation, setOrientation] = useState<"vertical" | "horizontal">("vertical");
  const [presetIndex, setPresetIndex] = useState(0);
  const [kinsokuLevel, setKinsokuLevel] = useState<KinsokuLevel>(DEFAULT_TYPESET_OPTIONS.kinsokuLevel);
  const [zoom, setZoom] = useState(1);
  const [pageIndex, setPageIndex] = useState(0);

  const preset = MANUSCRIPT_SIZE_PRESETS[presetIndex];
  const options = useMemo(
    () => ({ charsPerLine: preset.charsPerLine, linesPerPage: preset.linesPerPage, kinsokuLevel }),
    [preset, kinsokuLevel],
  );

  const pages = useMemo(() => typeset(body, options), [body, options]);
  const pageCount = layoutPageCount(pages);
  const conversionPages = (body.length / 400).toFixed(1);
  const clampedPageIndex = Math.min(pageIndex, Math.max(pages.length - 1, 0));

  function handlePrint() {
    window.print();
  }

  return (
    <div className="genkou">
      <div className="genkou__toolbar">
        <label>
          サイズ
          <select value={presetIndex} onChange={(e) => setPresetIndex(Number(e.target.value))}>
            {MANUSCRIPT_SIZE_PRESETS.map((p, i) => (
              <option key={p.label} value={i}>
                {p.label}
              </option>
            ))}
          </select>
        </label>
        <label>
          禁則処理
          <select value={kinsokuLevel} onChange={(e) => setKinsokuLevel(e.target.value as KinsokuLevel)}>
            {(Object.keys(KINSOKU_LABEL) as KinsokuLevel[]).map((level) => (
              <option key={level} value={level}>
                {KINSOKU_LABEL[level]}
              </option>
            ))}
          </select>
        </label>
        <button className="secondary" onClick={() => setOrientation((o) => (o === "vertical" ? "horizontal" : "vertical"))}>
          {orientation === "vertical" ? "縦書き" : "横書き"}
        </button>
        <div className="genkou__zoom">
          <button className="secondary" onClick={() => setZoom((z) => Math.max(0.5, z - 0.1))}>
            −
          </button>
          <span>{Math.round(zoom * 100)}%</span>
          <button className="secondary" onClick={() => setZoom((z) => Math.min(2, z + 0.1))}>
            +
          </button>
        </div>
        <button className="secondary" onClick={handlePrint}>
          印刷プレビュー
        </button>
      </div>

      <div className="genkou__pager">
        <button
          className="secondary"
          disabled={clampedPageIndex <= 0}
          onClick={() => setPageIndex((p) => Math.max(0, p - 1))}
        >
          ← 前ページ
        </button>
        <span>
          {pages.length === 0 ? 0 : clampedPageIndex + 1} / {pages.length}ページ
          （原稿枚数 {pageCount}枚・換算 {conversionPages}枚）
        </span>
        <button
          className="secondary"
          disabled={clampedPageIndex >= pages.length - 1}
          onClick={() => setPageIndex((p) => Math.min(pages.length - 1, p + 1))}
        >
          次ページ →
        </button>
      </div>

      <div className="genkou__viewport" style={{ ["--ms-zoom" as string]: zoom }}>
        {pages.map((page, i) => (
          <div
            key={i}
            className={`ms-page ms-page--${orientation} ${i === clampedPageIndex ? "" : "ms-page--offscreen"}`}
            style={{ ["--ms-cols" as string]: options.charsPerLine, ["--ms-rows" as string]: options.linesPerPage }}
          >
            {page.map((line, li) => (
              <div className="ms-line" key={li}>
                {line.map((cell, ci) => (
                  <Cell cell={cell} key={ci} />
                ))}
              </div>
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}
