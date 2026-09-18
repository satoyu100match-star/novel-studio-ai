import { describe, expect, it } from "vitest";
import { parseCharacterDrafts, parseConceptDraft, parseWorldEntryDrafts } from "@/features/ai/parseGenerated";

// Phase12「作品設計チャット」(generate_concept)のパース処理。プロンプト
// (src-tauri/src/ai/features.rs)が指定した見出し形式に沿った応答を、
// 各セクションごとに正しく切り出せることを確認する。

const SAMPLE_RESPONSE = `==TITLE==
異世界刑務所
鉄格子の向こうの約束

==SYNOPSIS==
元冒険者の新人看守が、異世界の刑務所で個性豊かな元凶悪犯たちと関わりながら
成長していく物語。

==CHARACTERS==
名前: アレン
読み: あれん
年齢: 22
性別: 男性
役割: 主人公
性格: 生真面目
目標: 看守として一人前になる
弱み: 情に流されやすい
一人称: 俺
口癖: まあいいか
背景: 元冒険者

名前: ミラ
読み: みら
年齢: 30
性別: 女性
役割: 囚人
性格: 皮肉屋
目標: 出所
弱み: 人を信じない
一人称: あたし
口癖: はいはい
背景: 元盗賊

==WORLD==
名前: 中央刑務所
概要: 物語の主な舞台
詳細: 魔法犯罪者を収容する巨大な施設

名前: 看守団
概要: 刑務所を管理する組織
詳細: 元冒険者が多く所属する

==PLOT==
タイトル: 出会い
概要: アレンがミラと初めて対峙する

タイトル: 脱獄未遂
概要: 囚人の一部が脱獄を試みる
`;

describe("parseConceptDraft", () => {
  it("splits every section independently", () => {
    const draft = parseConceptDraft(SAMPLE_RESPONSE);
    expect(draft.titles).toEqual(["異世界刑務所", "鉄格子の向こうの約束"]);
    expect(draft.synopsis).toContain("元冒険者の新人看守");
    expect(draft.characters).toHaveLength(2);
    expect(draft.characters[0].name).toBe("アレン");
    expect(draft.characters[1].name).toBe("ミラ");
    expect(draft.worldEntries).toHaveLength(2);
    expect(draft.worldEntries[0].name).toBe("中央刑務所");
    expect(draft.plotCards).toHaveLength(2);
    expect(draft.plotCards[0]).toEqual({ title: "出会い", summary: "アレンがミラと初めて対峙する" });
  });

  it("tolerates a missing section without breaking the others", () => {
    const withoutWorld = SAMPLE_RESPONSE.replace(/==WORLD==[\s\S]*?(?=\n==PLOT==)/, "");
    const draft = parseConceptDraft(withoutWorld);
    expect(draft.worldEntries).toEqual([]);
    expect(draft.characters).toHaveLength(2);
    expect(draft.plotCards).toHaveLength(2);
  });

  it("returns an all-empty draft for unrelated text", () => {
    const draft = parseConceptDraft("すみません、うまく生成できませんでした。");
    expect(draft).toEqual({ titles: [], synopsis: "", characters: [], worldEntries: [], plotCards: [] });
  });
});

describe("parseCharacterDrafts / parseWorldEntryDrafts", () => {
  it("skips blocks without a name", () => {
    const text = "名前: 太郎\n役割: 主人公\n\n役割: (名前なし)\n\n名前: 花子\n役割: 相棒";
    expect(parseCharacterDrafts(text).map((c) => c.name)).toEqual(["太郎", "花子"]);
  });

  it("parses multiple world entries separated by blank lines", () => {
    const text = "名前: 王国\n概要: 舞台\n\n名前: 魔法\n概要: 力の源";
    const entries = parseWorldEntryDrafts(text);
    expect(entries).toEqual([
      { name: "王国", summary: "舞台", detail: "" },
      { name: "魔法", summary: "力の源", detail: "" },
    ]);
  });
});
