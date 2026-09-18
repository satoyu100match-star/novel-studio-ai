import { describe, expect, it } from "vitest";
import { layoutPageCount, tokenize, typeset } from "@/features/manuscript/typeset";

// 仕様#95 記載のテスト文字列。縦書き/横書き両方の見た目はコンポーネント側の
// 責務なので、ここではセル分解・改行・改頁が崩れないことだけを確認する。
const REQUIRED_TEST_STRINGS = [
  "「こんにちは」",
  "『こんにちは』",
  "……",
  "――",
  "！？",
  "123",
  "ABC",
  "漢字（かんじ）",
];

describe("typeset: required manuscript test strings (spec #95)", () => {
  for (const text of REQUIRED_TEST_STRINGS) {
    it(`does not crash or drop characters for: ${text}`, () => {
      const pages = typeset(text, { charsPerLine: 20, linesPerPage: 20, kinsokuLevel: "standard" });
      const rendered = pages.flatMap((p) => p.flatMap((l) => l.map((c) => c.char))).join("");
      expect(rendered).toBe(text);
    });
  }
});

describe("typeset: line wrapping", () => {
  it("wraps at charsPerLine for plain text", () => {
    const pages = typeset("あいうえおかきくけこ", { charsPerLine: 5, linesPerPage: 20, kinsokuLevel: "simple" });
    expect(pages[0].length).toBe(2);
    expect(pages[0][0].map((c) => c.char).join("")).toBe("あいうえお");
    expect(pages[0][1].map((c) => c.char).join("")).toBe("かきくけこ");
  });

  it("splits into pages once linesPerPage is exceeded", () => {
    const body = Array.from({ length: 25 }, (_, i) => `行${i}`).join("\n");
    const pages = typeset(body, { charsPerLine: 20, linesPerPage: 20, kinsokuLevel: "simple" });
    expect(pages.length).toBe(2);
    expect(pages[0].length).toBe(20);
    expect(pages[1].length).toBe(5);
  });
});

describe("typeset: kinsoku (禁則処理)", () => {
  it("pulls a forbidden line-start punctuation mark back (行頭禁則, standard)", () => {
    // charsPerLine=5: naive wrap would put "。" alone at the start of line 2.
    const pages = typeset("あいうえお。かきくけこ", {
      charsPerLine: 5,
      linesPerPage: 20,
      kinsokuLevel: "standard",
    });
    const lines = pages[0].map((l) => l.map((c) => c.char).join(""));
    expect(lines[0]).toBe("あいうえお。"); // hung past the nominal width
    expect(lines[1].startsWith("。")).toBe(false);
  });

  it("does not apply kinsoku when level is simple", () => {
    const pages = typeset("あいうえお。かきくけこ", {
      charsPerLine: 5,
      linesPerPage: 20,
      kinsokuLevel: "simple",
    });
    const lines = pages[0].map((l) => l.map((c) => c.char).join(""));
    expect(lines[0]).toBe("あいうえお");
    expect(lines[1][0]).toBe("。");
  });

  it("pushes a trailing opening bracket forward (行末禁則, strict only)", () => {
    const pages = typeset("あいうえ「かきくけこ」", {
      charsPerLine: 5,
      linesPerPage: 20,
      kinsokuLevel: "strict",
    });
    const lines = pages[0].map((l) => l.map((c) => c.char).join(""));
    expect(lines[0]).toBe("あいうえ");
    expect(lines[1].startsWith("「")).toBe(true);
  });
});

describe("typeset: ruby markup", () => {
  it("parses ｜base《reading》 into an atomic run carrying the reading", () => {
    const atoms = tokenize("｜漢字《かんじ》です");
    expect(atoms[0]).toEqual({ chars: ["漢", "字"], ruby: "かんじ", bouten: false });

    const pages = typeset("｜漢字《かんじ》です", { charsPerLine: 20, linesPerPage: 20, kinsokuLevel: "standard" });
    const cells = pages[0][0];
    expect(cells.map((c) => c.char).join("")).toBe("漢字です");
    expect(cells[0].ruby).toBe("かんじ");
    expect(cells[0].rubySpan).toBe(2);
    expect(cells[1].ruby).toBeUndefined();
  });

  it("keeps a ruby run atomic across a line break instead of splitting it", () => {
    const pages = typeset("あい｜漢字《かんじ》", { charsPerLine: 3, linesPerPage: 20, kinsokuLevel: "simple" });
    const lines = pages[0].map((l) => l.map((c) => c.char).join(""));
    // "あい" (2) + "漢字" (2) would be 4 > charsPerLine(3), so the ruby run
    // must move to its own line rather than being split mid-run.
    expect(lines[0]).toBe("あい");
    expect(lines[1]).toBe("漢字");
  });
});

describe("typeset: bouten markup", () => {
  it("flags each character in a [#傍点]...[#傍点終わり] run", () => {
    const pages = typeset("これは[#傍点]大事[#傍点終わり]です", {
      charsPerLine: 20,
      linesPerPage: 20,
      kinsokuLevel: "standard",
    });
    const cells = pages[0][0];
    const boutenChars = cells.filter((c) => c.bouten).map((c) => c.char);
    expect(boutenChars).toEqual(["大", "事"]);
  });
});

describe("layoutPageCount", () => {
  it("returns 0 for empty manuscript, distinct from a 1-page result", () => {
    expect(layoutPageCount(typeset("", { charsPerLine: 20, linesPerPage: 20, kinsokuLevel: "standard" }))).toBe(0);
    expect(layoutPageCount(typeset("あ", { charsPerLine: 20, linesPerPage: 20, kinsokuLevel: "standard" }))).toBe(1);
  });
});
