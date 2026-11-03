import { describe, it, expect } from "vitest";
import {
  badTrackMark,
  classOf,
  fitReasons,
  gainText,
  gearWords,
  isYoung,
  mutationText,
  raceKey,
  raceValue,
  recordText,
  selectionLabel,
  signed,
  starRecord,
  statLabel,
  temperMark,
} from "./labels";

describe("the words the screens share", () => {
  it("names a horse's class from its wins", () => {
    expect(classOf({ starts: 0, wins: 0 })).toBe("新馬");
    expect(classOf({ starts: 3, wins: 0 })).toBe("未勝利");
    expect(classOf({ starts: 3, wins: 1 })).toBe("1勝クラス");
    expect(recordText({ starts: 3, wins: 1 })).toBe("3戦1勝・1勝クラス");
  });

  it("says what gear a horse wears, or nothing", () => {
    expect(gearWords(null)).toBeFalsy();
    expect(gearWords({ menko: false, shadow_roll: false })).toBeFalsy();
    expect(gearWords({ menko: true, shadow_roll: false })).toContain("メンコ");
    const both = gearWords({ menko: true, shadow_roll: true });
    expect(both).toContain("メンコ");
    expect(both).toContain("シャドーロール");
  });

  it("falls back to the key for a stat it does not know", () => {
    expect(statLabel("speed")).toBe("スピード");
    expect(statLabel("something")).toBe("something");
  });

  it("counts only a horse known to be under two as young", () => {
    expect(isYoung({ age: 0 })).toBe(true);
    expect(isYoung({ age: 2 })).toBe(false);
    expect(isYoung({})).toBe(false);
    expect(isYoung(undefined)).toBe(false);
  });

  it("names what has mutated", () => {
    expect(mutationText(["speed"])).toContain("スピード");
  });

  it("tells a rival's record", () => {
    expect(starRecord({ starts: 5, wins: 3, g1_wins: 1 } as never)).toBe("5戦3勝(G1 1勝)");
  });

  it("says why a course suits a horse", () => {
    expect(fitReasons({ percent: 0, rating: "fair", reasons: [] } as never)).toBe("");
  });

  it("keys a race by its place on the card", () => {
    const race = { day: "sat", venue: "kyoto", race_no: 11, week: 1, name: "x" } as never;
    expect(raceValue(race)).toBe("sat:kyoto:11");
    expect(raceKey(race)).toEqual({ day: "sat", venue: "kyoto", race_no: 11 });
  });

  it("writes a selection with arrows only when the order matters", () => {
    expect(selectionLabel("exacta", ["A", "B"])).toBe("A→B");
    expect(selectionLabel("trifecta", ["A", "B", "C"])).toBe("A→B→C");
    expect(selectionLabel("quinella", ["A", "B"])).toBe("A-B");
    expect(selectionLabel("trio", ["A", "B", "C"])).toBe("A-B-C");
  });

  it("writes what a workout gives", () => {
    const text = gainText(
      {
        workout: "dirt",
        name: "ダートコース",
        description: "",
        gains: [{ ability: "dirt", amount: 1 }, { ability: "power", amount: 1 }],
      } as never,
      3
    );
    expect(text).toContain("ダート");
    expect(text).toContain("パワー");
  });

  it("grades temperament and bad-track ability at the same steps each time", () => {
    expect([65, 64, 50, 49, 30, 29].map(temperMark)).toEqual(["◎", "○", "○", "△", "△", "×"]);
    expect([75, 74, 60, 59, 45, 44].map(badTrackMark)).toEqual(["◎", "○", "○", "△", "△", "×"]);
  });

  it("puts a plus on a gain and leaves nothing on nothing", () => {
    expect(signed(5)).toBe("+5");
    expect(signed(0)).toBe("0");
    expect(signed(-5)).toBe("-5");
  });
});
