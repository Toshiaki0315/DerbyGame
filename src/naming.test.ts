import { describe, it, expect } from "vitest";
import { isValidHorseName, NAME_RULE } from "./naming";

describe("horse naming rule", () => {
  it("accepts 2-9 full-width katakana", () => {
    for (const n of ["サクラフブキ", "オウゴンオーシャン", "アサ", "ニューホープ"]) {
      expect(isValidHorseName(n), n).toBe(true);
    }
  });

  it("rejects everything else", () => {
    for (const n of ["", "ア", "オウゴンオーシャンズ", "桜吹雪", "さくらふぶき", "Sakura", "ｻｸﾗﾌﾞｷ", "ーサクラ", "サクラ ブ"]) {
      expect(isValidHorseName(n), n).toBe(false);
    }
  });

  it("describes the rule for display", () => {
    expect(NAME_RULE).toBe("全角カタカナ2〜9文字");
  });
});
