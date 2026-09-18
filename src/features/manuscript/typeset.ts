/**
 * Manuscript-paper typesetting engine (Phase 2).
 *
 * Pure functions only -- no React, no DOM. Turns plain manuscript text
 * into a grid of pages/lines/cells for the genkou-youshi view. This is
 * the one place that implements Japanese typesetting rules (kinsoku,
 * ruby, bouten); `GenkouYoushiView.tsx` only renders whatever this
 * produces. See docs/MANUSCRIPT.md for why this has to stay a pure
 * function separate from the editable text (仕様#18).
 *
 * Supported inline markup (documented here since it's Novel Studio AI's
 * own convention, not a general Markdown dialect -- spec #15 explicitly
 * asks that authors not have to see raw formatting symbols in the normal
 * editor; this markup is only ever interpreted by this typesetting layer
 * for the manuscript-paper *view*, never shown to the author as "the way
 * to write"):
 *   ｜基底《よみ》   ルビ (ruby). The ｜ marks where the base text run
 *                    starts, ending at the character before 《.
 *   [#傍点]文字[#傍点終わり]   傍点 (emphasis dots), one dot per base
 *                    character in the run.
 */

export type KinsokuLevel = "strict" | "standard" | "simple";

export interface TypesetOptions {
  charsPerLine: number;
  linesPerPage: number;
  kinsokuLevel: KinsokuLevel;
}

export const DEFAULT_TYPESET_OPTIONS: TypesetOptions = {
  charsPerLine: 20,
  linesPerPage: 20,
  kinsokuLevel: "standard",
};

export interface TypesetCell {
  char: string;
  bouten: boolean;
  /** Present only on the first cell of a ruby run. */
  ruby?: string;
  /** How many cells (including this one) the ruby run spans. Only set alongside `ruby`. */
  rubySpan?: number;
  /** True for half-width alphanumeric runs that should be tate-chu-yoko combined in vertical mode. */
  tcy: boolean;
}

export type TypesetLine = TypesetCell[];
export type TypesetPage = TypesetLine[];

// 行頭禁則: must not be the first character on a line.
const FORBIDDEN_LINE_START = new Set(
  "、。，．・：；！？ヽヾゝゞ々〻ー）〕］｝〉》」』】〙〗〟’”ｧｨｩｪｫｬｭｮｯｰぁぃぅぇぉっゃゅょゎァィゥェォッャュョヮ",
);

// 行末禁則: must not be the last character on a line (opening brackets).
const FORBIDDEN_LINE_END = new Set("（〔［｛〈《「『【〘〖〝‘“");

interface Atom {
  chars: string[]; // 1 char, unless this atom is an atomic ruby run
  ruby?: string;
  bouten: boolean;
}

function isAsciiAlnum(ch: string): boolean {
  return /[0-9A-Za-z]/.test(ch);
}

/** Parses inline ruby/bouten markup into atoms. Plain text becomes 1-char atoms. */
export function tokenize(body: string): Atom[] {
  const atoms: Atom[] = [];
  let i = 0;
  while (i < body.length) {
    // Ruby: ｜base《reading》
    if (body[i] === "｜") {
      const rubyOpen = body.indexOf("《", i + 1);
      const rubyClose = rubyOpen >= 0 ? body.indexOf("》", rubyOpen + 1) : -1;
      if (rubyOpen > i && rubyClose > rubyOpen) {
        const base = body.slice(i + 1, rubyOpen);
        const reading = body.slice(rubyOpen + 1, rubyClose);
        atoms.push({ chars: [...base], ruby: reading, bouten: false });
        i = rubyClose + 1;
        continue;
      }
    }
    // Bouten: [#傍点]text[#傍点終わり]
    if (body.startsWith("[#傍点]", i)) {
      const end = body.indexOf("[#傍点終わり]", i);
      if (end >= 0) {
        const inner = body.slice(i + "[#傍点]".length, end);
        for (const ch of inner) atoms.push({ chars: [ch], bouten: true });
        i = end + "[#傍点終わり]".length;
        continue;
      }
    }
    atoms.push({ chars: [body[i]], bouten: false });
    i += 1;
  }
  return atoms;
}

/** Atom width in cells (a ruby run is atomic and must not be split across a line break). */
function atomWidth(atom: Atom): number {
  return atom.chars.length;
}

function atomToCells(atom: Atom): TypesetCell[] {
  const isTcyGroup = atom.chars.length > 1 && atom.chars.every(isAsciiAlnum) && !atom.ruby;
  return atom.chars.map((char, idx) => ({
    char,
    bouten: atom.bouten,
    tcy: isTcyGroup || (atom.chars.length === 1 && isAsciiAlnum(char)),
    ...(atom.ruby && idx === 0 ? { ruby: atom.ruby, rubySpan: atom.chars.length } : {}),
  }));
}

function fillLinesForParagraph(atoms: Atom[], charsPerLine: number): Atom[][] {
  const lines: Atom[][] = [];
  let current: Atom[] = [];
  let width = 0;

  for (const atom of atoms) {
    const w = atomWidth(atom);
    if (width > 0 && width + w > charsPerLine) {
      lines.push(current);
      current = [];
      width = 0;
    }
    current.push(atom);
    width += w;
  }
  lines.push(current);
  return lines;
}

/** 行頭禁則 / 行末禁則 adjustment, applied to already width-filled lines. */
function applyKinsoku(lines: Atom[][], level: KinsokuLevel): Atom[][] {
  if (level === "simple") return lines;

  const result = lines.map((l) => [...l]);

  for (let pass = 0; pass < 2; pass++) {
    for (let i = 0; i < result.length - 1; i++) {
      const line = result[i];
      const next = result[i + 1];
      if (next.length === 0) continue;

      // 行頭禁則: pull the offending leading atom back onto the previous
      // line (allowed to hang past the nominal width -- ぶら下げ).
      const nextFirst = next[0];
      if (
        nextFirst.chars.length === 1 &&
        FORBIDDEN_LINE_START.has(nextFirst.chars[0]) &&
        line.length > 0
      ) {
        line.push(next.shift()!);
      }

      // 行末禁則 (strict only): push a trailing opening-bracket atom
      // forward onto the next line instead of ending the line with it.
      if (level === "strict" && line.length > 0) {
        const lineLast = line[line.length - 1];
        if (lineLast.chars.length === 1 && FORBIDDEN_LINE_END.has(lineLast.chars[0])) {
          next.unshift(line.pop()!);
        }
      }
    }
  }

  return result.filter((l) => l.length > 0);
}

export function typeset(body: string, options: TypesetOptions = DEFAULT_TYPESET_OPTIONS): TypesetPage[] {
  const paragraphs = body.split("\n");
  const allAtomLines: Atom[][] = [];

  for (const paragraph of paragraphs) {
    const atoms = tokenize(paragraph);
    if (atoms.length === 0) {
      allAtomLines.push([]); // preserve blank lines
      continue;
    }
    const rawLines = fillLinesForParagraph(atoms, options.charsPerLine);
    const kinsokuLines = applyKinsoku(rawLines, options.kinsokuLevel);
    allAtomLines.push(...(kinsokuLines.length > 0 ? kinsokuLines : [[]]));
  }

  const lines: TypesetLine[] = allAtomLines.map((atomLine) => atomLine.flatMap(atomToCells));

  const pages: TypesetPage[] = [];
  for (let i = 0; i < lines.length; i += options.linesPerPage) {
    pages.push(lines.slice(i, i + options.linesPerPage));
  }
  return pages.length > 0 ? pages : [[]];
}

/** 実際の組版結果によるページ数(実レイアウト枚数)。換算枚数とは区別する -- docs/MANUSCRIPT.md 4章。 */
export function layoutPageCount(pages: TypesetPage[]): number {
  // A body that typesets to a single, entirely empty page counts as 0
  // pages laid out yet, not "1 blank page".
  if (pages.length === 1 && pages[0].every((line) => line.length === 0)) return 0;
  return pages.length;
}

export const MANUSCRIPT_SIZE_PRESETS: { label: string; charsPerLine: number; linesPerPage: number }[] = [
  { label: "20×20 (400字)", charsPerLine: 20, linesPerPage: 20 },
  { label: "20×40 (800字)", charsPerLine: 20, linesPerPage: 40 },
  { label: "30×40 (1200字)", charsPerLine: 30, linesPerPage: 40 },
];
