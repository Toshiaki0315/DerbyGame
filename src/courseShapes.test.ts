import { describe, it, expect } from "vitest";
import { COURSE_SHAPES, courseShape, gateOf, shapeKey, traceOf } from "./courseShapes";
import { gatePoint, gateSpot, ovalPoint, runnerPoint } from "./raceShow";

const VENUES = ["札幌", "函館", "福島", "新潟", "東京", "中山", "中京", "京都", "阪神", "小倉"];
const shape = (name: string, surface: "turf" | "dirt", layout: string | null, lap: number, distance: number, turn: string) => ({
  lap,
  straight: 400,
  turn,
  width: 300,
  height: 170,
  trace: traceOf(name, surface, layout, 300, 170)!,
  gate: gateOf(name, surface, distance, 300, 170, layout),
});

describe("the course shapes", () => {
  it("has a turf and a dirt course for every racecourse", () => {
    for (const v of VENUES) {
      expect(COURSE_SHAPES[shapeKey(v, "dirt", null)], v).toBeDefined();
      const hasTwo = ["新潟", "中山", "京都", "阪神"].includes(v);
      for (const l of hasTwo ? ["内回り", "外回り"] : [null]) {
        expect(COURSE_SHAPES[shapeKey(v, "turf", l)], `${v} ${l}`).toBeDefined();
      }
    }
  });

  it("has no shape for a straight course or a place it does not know", () => {
    expect(traceOf("どこか", "turf", null, 300, 170)).toBeUndefined();
    expect(traceOf("新潟", "turf", "直線", 300, 170)).toBeUndefined();
  });

  it("fits every shape inside the drawing", () => {
    for (const key of Object.keys(COURSE_SHAPES)) {
      const [name, surface, layout] = key.split("/");
      const trace = traceOf(name, surface as "turf" | "dirt", layout || null, 300, 170)!;
      for (const [x, y] of trace.points) {
        expect(x, key).toBeGreaterThanOrEqual(0);
        expect(x, key).toBeLessThanOrEqual(300);
        expect(y, key).toBeGreaterThanOrEqual(0);
        expect(y, key).toBeLessThanOrEqual(170);
      }
    }
  });

  it("measures the race along the shape, from the goal round to the goal", () => {
    const s = shape("東京", "turf", null, 2083, 2400, "左回り");
    const goal = ovalPoint(0, s);
    expect(goal.x).toBeCloseTo(s.trace.points[0][0]);
    expect(goal.y).toBeCloseTo(s.trace.points[0][1]);
    const again = ovalPoint(2083, s);
    expect(again.x).toBeCloseTo(goal.x);
    expect(again.y).toBeCloseTo(goal.y);
    // spaced by the distance run: half a lap is about half way round
    const half = ovalPoint(2083 / 2, s);
    expect(half.y).toBeLessThan(goal.y - 50);
  });

  it("puts Tokyo's goal where the official plan has it", () => {
    const s = shape("東京", "turf", null, 2083, 2400, "左回り");
    const xs = s.trace.points.map(([x]) => x);
    const share = (ovalPoint(0, s).x - Math.min(...xs)) / (Math.max(...xs) - Math.min(...xs));
    expect(share).toBeGreaterThan(0.7);
    expect(share).toBeLessThan(0.85);
  });

  it("puts a right-handed course's goal on the left of its stands straight", () => {
    const s = shape("中山", "turf", "内回り", 1667, 2000, "右回り");
    const xs = s.trace.points.map(([x]) => x);
    expect(ovalPoint(0, s).x).toBeLessThan((Math.min(...xs) + Math.max(...xs)) / 2);
    // running up to the goal from the right
    expect(ovalPoint(200, s).x).toBeGreaterThan(ovalPoint(0, s).x);
  });

  it("moves a horse wide of its line, away from the middle", () => {
    const s = shape("東京", "turf", null, 2083, 2400, "左回り");
    expect(ovalPoint(100, s, 5).y).toBeGreaterThan(ovalPoint(100, s).y);
    expect(ovalPoint(1200, s, 5).y).toBeLessThan(ovalPoint(1200, s).y);
  });

  it("knows the gates off the course from the plan", () => {
    const s = shape("札幌", "turf", null, 1641, 1500, "右回り");
    const spot = gateSpot(1500, s);
    expect(spot).toMatchObject({ label: "2コーナー奥ポケット", pocket: true });
    expect(spot.join).toBeLessThan(1500);
    const gate = gatePoint(1500, s);
    expect(gate.x).toBeCloseTo(s.gate!.x);
    expect(gate.y).toBeCloseTo(s.gate!.y);
  });

  it("runs a horse from a gate on a chute to the course and on round it", () => {
    const s = shape("札幌", "turf", null, 1641, 1500, "右回り");
    const spot = gateSpot(1500, s);
    expect(runnerPoint(1500, 1500, s).x).toBeCloseTo(s.gate!.x);
    const joined = ovalPoint(spot.join, s);
    expect(runnerPoint(spot.join, 1500, s).x).toBeCloseTo(joined.x);
    expect(runnerPoint(600, 1500, s).x).toBeCloseTo(ovalPoint(600, s).x);
  });

  describe("Niigata's straight course", () => {
    const course = { name: "新潟", turn: "左回り", straight: 1000, lap: 1000, layout: "直線", after_goal: 0, straight_course: true };
    const s = courseShape(course, "turf", 1000, 300, 170);

    it("is one straight line with the start to the left of the goal", () => {
      const [start, goal] = [ovalPoint(1000, s), ovalPoint(0, s)];
      expect(start.y).toBeCloseTo(goal.y);
      expect(start.x).toBeLessThan(goal.x);
    });

    it("runs in line with the bottom of the outer course, which it is drawn against", () => {
      expect(s.backdrop!.length).toBeGreaterThan(20);
      const lowest = Math.max(...s.backdrop!.map(([, y]) => y));
      expect(ovalPoint(0, s).y).toBeCloseTo(lowest, -1);
    });

    it("reaches further left than the outer course", () => {
      const leftmost = Math.min(...s.backdrop!.map(([x]) => x));
      expect(ovalPoint(1000, s).x).toBeLessThan(leftmost);
    });

    it("has the goal where the other Niigata courses have it", () => {
      const outer = courseShape({ ...course, layout: "外回り", lap: 2223, straight: 659, straight_course: false }, "turf", 1600, 300, 170);
      const others = outer.trace!.points.map(([x]) => x);
      expect(others.length).toBeGreaterThan(0);
      const share = (ovalPoint(0, s).x - Math.min(...s.backdrop!.map(([x]) => x))) / (Math.max(...s.backdrop!.map(([x]) => x)) - Math.min(...s.backdrop!.map(([x]) => x)));
      expect(share).toBeGreaterThan(0.7);
      expect(share).toBeLessThan(0.85);
    });

    it("stays inside the drawing", () => {
      for (const toGo of [0, 500, 1000]) {
        const p = ovalPoint(toGo, s);
        expect(p.x).toBeGreaterThan(0);
        expect(p.x).toBeLessThan(300);
      }
    });
  });

  it("has no made-up pocket on a course it has the line of", () => {
    // two corners short of the goal by distance, with no gate read off the plan
    const s = shape("札幌", "dirt", null, 1487, 1050, "右回り");
    expect(s.gate).toBeUndefined();
    const spot = gateSpot(1050, s);
    expect(spot.pocket).toBe(false);
    expect(spot.label).not.toContain("ポケット");
    expect(spot.join).toBe(1050);
  });

  it("starts Hanshin's outer mile in the back straight, as the plan has it", () => {
    const s = shape("阪神", "turf", "外回り", 2089, 1600, "右回り");
    expect(gateSpot(1600, s).label).toBe("向正面");
  });

  it("keeps a gate on the course measured back from the goal", () => {
    const s = shape("東京", "turf", null, 2083, 2400, "左回り");
    expect(s.gate).toBeUndefined();
    expect(gateSpot(2400, s).pocket).toBe(false);
    expect(runnerPoint(2400, 2400, s).x).toBeCloseTo(ovalPoint(2400, s).x);
  });
});
