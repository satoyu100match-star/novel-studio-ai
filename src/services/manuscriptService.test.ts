import { describe, expect, it } from "vitest";
import { toManuscriptPages } from "@/services/manuscriptService";

describe("toManuscriptPages", () => {
  it("converts character count to 400-character manuscript pages", () => {
    expect(toManuscriptPages(400)).toBe(1);
    expect(toManuscriptPages(800)).toBe(2);
    expect(toManuscriptPages(200)).toBe(0.5);
    expect(toManuscriptPages(0)).toBe(0);
  });
});
