import { describe, it, expect } from "vitest";
import { courseSection, finishView, gateSpot, ovalPoint, raceView, runnerPoint } from "./raceShow";

const names = ["A", "B", "C", "D"];

describe("raceView", () => {
  it("keeps the finishing order among the runners past the post", () => {
    const v = raceView([1000, 1000, 1000, 900], names, 1000, undefined, undefined, undefined, ["C", "A", "B", "D"]);
    expect(v.order).toEqual(["C", "A", "B", "D"]);
    expect(v.ranks).toEqual([2, 3, 1, 4]);
    expect(v.leader).toBe("C");
  });

  it("goes by the lanes for a tie with no finishing order", () => {
    expect(raceView([500, 500, 500, 500], names, 1000).order).toEqual(names);
  });

  it("orders the runners and ranks each lane", () => {
    const v = raceView([100, 300, 200, 50], names, 2400);
    expect(v.order).toEqual(["B", "C", "A", "D"]);
    expect(v.ranks).toEqual([3, 1, 2, 4]);
    expect(v.leader).toBe("B");
  });

  it("gives what the leader has left to run, and never less than nothing", () => {
    expect(raceView([1800, 0, 0, 0], names, 2400).remaining).toBe(600);
    expect(raceView([2500, 0, 0, 0], names, 2400).remaining).toBe(0);
  });

  it("rounds the distance left to tens of metres", () => {
    expect(raceView([1803, 0, 0, 0], names, 2400).remaining).toBe(600);
  });

  it("names the stage of the race", () => {
    const at = (pos: number) => raceView([pos, 0, 0, 0], names, 2400, 500).phase;
    expect(at(100)).toBe("スタート");
    expect(at(600)).toBe("序盤");
    expect(at(1300)).toBe("中盤");
    expect(at(1600)).toBe("最終コーナー");
    expect(at(1950)).toBe("最後の直線");
    expect(at(2400)).toBe("ゴール");
  });

  it("is in the straight only when the straight is known", () => {
    expect(raceView([2000, 0, 0, 0], names, 2400, 500).stretch).toBe(true);
    expect(raceView([1800, 0, 0, 0], names, 2400, 500).stretch).toBe(false);
    expect(raceView([2000, 0, 0, 0], names, 2400).stretch).toBe(false);
  });

  it("calls a close race only in the straight", () => {
    expect(raceView([2100, 2098, 0, 0], names, 2400, 500).close).toBe(true);
    expect(raceView([2100, 2080, 0, 0], names, 2400, 500).close).toBe(false);
    expect(raceView([1000, 999, 0, 0], names, 2400, 500).close).toBe(false);
    expect(raceView([2400, 2399, 0, 0], names, 2400, 500).close).toBe(false);
  });
});

describe("finishView", () => {
  const results = [
    { rank: 1, name: "A", finish_time: 120.1 },
    { rank: 2, name: "B", finish_time: 120.15 },
    { rank: 3, name: "C", finish_time: 121 },
  ];

  it("names the winner and calls a photo finish when the gap is tiny", () => {
    expect(finishView(results, [])).toEqual({ winner: "A", photo: true, mineWon: false, minePlaced: null });
    expect(finishView([results[0], results[2]], []).photo).toBe(false);
  });

  it("knows when one of the stable's horses won", () => {
    expect(finishView(results, ["A"]).mineWon).toBe(true);
  });

  it("reports the best of the stable's horses when they did not win", () => {
    expect(finishView(results, ["C", "B"]).minePlaced).toEqual({ name: "B", rank: 2 });
  });
});

describe("courseSection", () => {
  const lap = 2000;
  const straight = 500; // corners of 500m each
  const at = (toGo: number) => courseSection(toGo, lap, straight);

  it("names where a horse is by what it has left to run", () => {
    expect(at(400)).toBe("最後の直線");
    expect(at(600)).toBe("第4コーナー");
    expect(at(900)).toBe("第3コーナー");
    expect(at(1200)).toBe("向正面");
    expect(at(1600)).toBe("第2コーナー");
    expect(at(1900)).toBe("第1コーナー");
  });

  it("calls the home straight the stands on the first time past", () => {
    expect(at(2400)).toBe("正面スタンド前");
  });

  it("is used for the stage of the race once the lap is known", () => {
    const stage = (pos: number) => raceView([pos, 0, 0, 0], names, 2400, 500, 2000).phase;
    expect(stage(50)).toBe("スタート");
    expect(stage(1500)).toBe("第3コーナー");
    expect(stage(1950)).toBe("最後の直線");
    expect(stage(2400)).toBe("ゴール");
  });
});

describe("ovalPoint", () => {
  const opts = { lap: 2000, straight: 500, width: 300, height: 170 };
  const left = (toGo: number, outwards = 0) => ovalPoint(toGo, { ...opts, turn: "左回り" }, outwards);
  const right = (toGo: number) => ovalPoint(toGo, { ...opts, turn: "右回り" });

  it("stays inside the drawing all the way round", () => {
    for (let d = 0; d < 2000; d += 50) {
      const p = left(d);
      expect(p.x).toBeGreaterThan(0);
      expect(p.x).toBeLessThan(300);
      expect(p.y).toBeGreaterThan(0);
      expect(p.y).toBeLessThan(170);
    }
  });

  it("runs along the bottom of the drawing in the home straight, towards the finish", () => {
    expect(left(0).y).toBeCloseTo(left(400).y);
    expect(left(0).x).toBeGreaterThan(left(400).x);
  });

  it("curves round through a corner", () => {
    const [a, b, c] = [left(500), left(750), left(1000)];
    expect(b.x).toBeLessThan(a.x);
    expect(b.y).toBeLessThan(a.y);
    expect(c.y).toBeLessThan(b.y);
    expect(c.x).toBeGreaterThan(b.x);
  });

  it("goes round the other way on a right-handed course", () => {
    for (const d of [0, 300, 750, 1200, 1700]) {
      expect(right(d).x).toBeCloseTo(300 - left(d).x);
      expect(right(d).y).toBeCloseTo(left(d).y);
    }
  });

  it("comes back to the same place a lap later", () => {
    expect(left(2300).x).toBeCloseTo(left(300).x);
    expect(left(2300).y).toBeCloseTo(left(300).y);
  });

  it("puts a horse wide of the rail when asked", () => {
    expect(left(100, 6).y).toBeCloseTo(left(100).y + 6);
    expect(left(1200, 6).y).toBeCloseTo(left(1200).y - 6);
    expect(left(750, 6).x).toBeLessThan(left(750).x);
  });

  it("copes with a course whose straights would not leave room for corners", () => {
    const p = ovalPoint(100, { lap: 1000, straight: 600, width: 300, height: 170, turn: "左回り" });
    expect(Number.isFinite(p.x) && Number.isFinite(p.y)).toBe(true);
  });
});

describe("goal and gate", () => {
  const tokyo = { lap: 2083, straight: 526, afterGoal: 32, turn: "左回り", width: 300, height: 170 };

  it("puts the goal short of the end of the straight", () => {
    const endOfStraight = ovalPoint(tokyo.lap - tokyo.afterGoal + 0.01, tokyo);
    expect(ovalPoint(0, tokyo).x).toBeLessThan(endOfStraight.x);
    expect(ovalPoint(0, tokyo).y).toBeCloseTo(endOfStraight.y);
  });

  it("carries on past the goal round to the first corner", () => {
    const justPast = ovalPoint(tokyo.lap - 10, tokyo);
    expect(justPast.x).toBeGreaterThan(ovalPoint(0, tokyo).x);
    expect(justPast.y).toBeCloseTo(ovalPoint(0, tokyo).y);
  });

  it("finds where the gate is by the distance", () => {
    expect(gateSpot(2400, tokyo).label).toBe("正面スタンド前");
    expect(gateSpot(2400, tokyo).pocket).toBe(false);
    expect(gateSpot(1400, tokyo).label).toBe("向正面");
    expect(gateSpot(1600, tokyo).label).toBe("2コーナー奥ポケット");
    expect(gateSpot(1600, tokyo).pocket).toBe(true);
    expect(gateSpot(2000, tokyo, 3).label).toBe("1コーナー奥ポケット");
  });

  it("takes a pocket the table says skips a corner on to the next straight", () => {
    const kyoto = { lap: 1894, straight: 404, afterGoal: 93, turn: "右回り", width: 300, height: 170 };
    const spot = gateSpot(1600, kyoto, 2);
    expect(spot.pocket).toBe(true);
    expect(spot.label).toBe("2コーナー奥ポケット");
    expect(courseSection(spot.join, kyoto.lap, kyoto.straight, kyoto.afterGoal)).toBe("向正面");
  });

  it("runs a horse from the pocket along a line that joins the course", () => {
    const place = (toGo: number) => runnerPoint(toGo, 1600, tokyo);
    const spot = gateSpot(1600, tokyo);
    const joined = ovalPoint(spot.join, tokyo);
    expect(place(spot.join).x).toBeCloseTo(joined.x);
    expect(place(spot.join).y).toBeCloseTo(joined.y);
    // off the oval at the gate
    expect(place(1600).x).not.toBeCloseTo(ovalPoint(1600, tokyo).x);
    // and on it once joined
    expect(place(1000).x).toBeCloseTo(ovalPoint(1000, tokyo).x);
  });

  it("takes a gate past the goal with a corner too few to be in a pocket", () => {
    const real = { ...tokyo, afterGoal: 133 };
    const spot = gateSpot(2000, real, 3);
    expect(spot.pocket).toBe(true);
    expect(spot.label).toBe("1コーナー奥ポケット");
    expect(gateSpot(2400, real, 4).label).toBe("正面スタンド前");
    expect(gateSpot(1400, real, 2).label).toBe("向正面");
  });

  it("counts a gate just short of the home straight as on it", () => {
    const nakayama = { lap: 1667, straight: 310, afterGoal: 72, turn: "右回り", width: 300, height: 170 };
    expect(gateSpot(2000, nakayama, 4)).toEqual({ label: "正面スタンド前", pocket: false, join: 2000 });
  });

  it("says a gate further round is towards the fourth corner", () => {
    const hanshin = { lap: 1689, straight: 357, afterGoal: 14, turn: "右回り", width: 300, height: 170 };
    expect(gateSpot(2200, hanshin, 4).label).toBe("4コーナー寄り");
  });

  it("runs a horse round the course itself when the gate is on it", () => {
    expect(runnerPoint(2400, 2400, tokyo).x).toBeCloseTo(ovalPoint(2400, tokyo).x);
  });

  it("keeps a straight course on one straight line with no corners", () => {
    const straight = { lap: 1000, straight: 1000, afterGoal: 0, turn: "左回り", width: 300, height: 170, straightCourse: true };
    const [a, b] = [ovalPoint(1000, straight), ovalPoint(0, straight)];
    expect(a.y).toBeCloseTo(b.y);
    expect(a.x).toBeLessThan(b.x);
    expect(gateSpot(1000, straight).label).toBe("直線コース");
    expect(gateSpot(1000, straight).pocket).toBe(false);
  });

  it("calls a straight course's last stretch without any corner", () => {
    const stage = (pos: number) => raceView([pos, 0, 0, 0], names, 1000, 400, 0).phase;
    expect(stage(100)).toBe("スタート");
    expect(stage(400)).toBe("序盤");
    expect(stage(550)).toBe("中盤");
    expect(stage(700)).toBe("最後の直線");
  });

  it("counts the run past the goal as in front of the stands", () => {
    expect(courseSection(2083 + 20 + 2083, 2083, 526, 32)).toBe("正面スタンド前");
    expect(courseSection(2083 - 10, 2083, 526, 32)).toBe("正面スタンド前");
  });
});
