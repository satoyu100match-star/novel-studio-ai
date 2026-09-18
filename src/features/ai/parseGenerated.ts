/**
 * AI生成機能(キャラクター/世界観項目/プロット)の応答を、既存のフォームへ
 * 反映できる形にパースするユーティリティ。プロンプト側(Rust `ai::features`)
 * で「ラベル: 値」形式の行だけを出力するよう指示しているため、それに
 * 対応する単純な行パーサで足りる。パース結果はあくまで下書きであり、
 * 作成後は既存のCharacter/World/Plotパネルで通常どおり編集・削除できる
 * (AIの出力をそのまま確定させず、必ず人間が後で見返せる状態にする)。
 */

function parseLabeledLines(text: string): Record<string, string> {
  const result: Record<string, string> = {};
  for (const rawLine of text.split("\n")) {
    const line = rawLine.trim();
    const idx = line.indexOf(":");
    const idxFullWidth = line.indexOf("：");
    const sep = idx >= 0 && (idxFullWidth < 0 || idx < idxFullWidth) ? idx : idxFullWidth;
    if (sep < 0) continue;
    const label = line.slice(0, sep).trim();
    const value = line.slice(sep + 1).trim();
    if (label && value) result[label] = value;
  }
  return result;
}

export function parseCharacterDraft(text: string): { name: string } & Record<string, string> {
  const labels = parseLabeledLines(text);
  return {
    name: labels["名前"] ?? "",
    reading: labels["読み"] ?? "",
    age: labels["年齢"] ?? "",
    gender: labels["性別"] ?? "",
    role: labels["役割"] ?? "",
    personality: labels["性格"] ?? "",
    goal: labels["目標"] ?? "",
    weaknesses: labels["弱み"] ?? "",
    first_person: labels["一人称"] ?? "",
    catchphrase: labels["口癖"] ?? "",
    past: labels["背景"] ?? "",
  };
}

export function parseWorldEntryDraft(text: string): { name: string; summary: string; detail: string } {
  const labels = parseLabeledLines(text);
  return {
    name: labels["名前"] ?? "",
    summary: labels["概要"] ?? "",
    detail: labels["詳細"] ?? "",
  };
}

export function parsePlotDrafts(text: string): { title: string; summary: string }[] {
  const blocks = text.split(/\n\s*\n/);
  const drafts: { title: string; summary: string }[] = [];
  for (const block of blocks) {
    const labels = parseLabeledLines(block);
    const title = labels["タイトル"];
    if (title) {
      drafts.push({ title, summary: labels["概要"] ?? "" });
    }
  }
  return drafts;
}

/**
 * 空行区切りの複数キャラクター案(Phase12「作品設計チャット」の
 * `generate_concept`が返す`==CHARACTERS==`セクション内で使う)。
 * 単一案用の`parseCharacterDraft`を再利用し、名前が読み取れたブロック
 * のみ採用する。
 */
export function parseCharacterDrafts(text: string): ReturnType<typeof parseCharacterDraft>[] {
  const blocks = text.split(/\n\s*\n/);
  const drafts: ReturnType<typeof parseCharacterDraft>[] = [];
  for (const block of blocks) {
    const draft = parseCharacterDraft(block);
    if (draft.name) drafts.push(draft);
  }
  return drafts;
}

/** 空行区切りの複数世界観項目案(Phase12「作品設計チャット」用)。 */
export function parseWorldEntryDrafts(text: string): ReturnType<typeof parseWorldEntryDraft>[] {
  const blocks = text.split(/\n\s*\n/);
  const drafts: ReturnType<typeof parseWorldEntryDraft>[] = [];
  for (const block of blocks) {
    const draft = parseWorldEntryDraft(block);
    if (draft.name) drafts.push(draft);
  }
  return drafts;
}

export interface ConceptDraft {
  titles: string[];
  synopsis: string;
  characters: ReturnType<typeof parseCharacterDraft>[];
  worldEntries: ReturnType<typeof parseWorldEntryDraft>[];
  plotCards: { title: string; summary: string }[];
}

function extractSection(text: string, name: string): string {
  const pattern = new RegExp(`==${name}==\\s*\\n([\\s\\S]*?)(?=\\n==[A-Z]+==|$)`);
  const match = text.match(pattern);
  return match ? match[1].trim() : "";
}

/**
 * `generate_concept`(Phase12「作品設計チャット」)の応答をパースする。
 * `==TITLE==`等の見出し(プロンプト側で指定)ごとに本文を切り出し、
 * 各セクションは既存の単一/複数案パーサへそのまま委譲する。見出しが
 * 一部欠けていたり順序が崩れていても、他のセクションの読み取りには
 * 影響しない(セクションごとに独立して正規表現で抽出するため)。
 */
export function parseConceptDraft(text: string): ConceptDraft {
  const titleSection = extractSection(text, "TITLE");
  const synopsisSection = extractSection(text, "SYNOPSIS");
  const charactersSection = extractSection(text, "CHARACTERS");
  const worldSection = extractSection(text, "WORLD");
  const plotSection = extractSection(text, "PLOT");
  return {
    titles: titleSection
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean),
    synopsis: synopsisSection,
    characters: parseCharacterDrafts(charactersSection),
    worldEntries: parseWorldEntryDrafts(worldSection),
    plotCards: parsePlotDrafts(plotSection),
  };
}
