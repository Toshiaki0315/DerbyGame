import { describe, it, expect } from "vitest";
import { GATES, gateFor, gateForRunner } from "./gates";

describe("gates (枠番)", () => {
  it("has 8 gates in the official colour order", () => {
    expect(GATES.map((g) => g.label)).toEqual(["白", "黒", "赤", "青", "黄", "緑", "橙", "桃"]);
    expect(GATES.map((g) => g.no)).toEqual([1, 2, 3, 4, 5, 6, 7, 8]);
  });

  it("uses readable text colours (dark on white/yellow/pink, white on the rest)", () => {
    const dark = GATES.filter((g) => g.fg === "#111").map((g) => g.label);
    expect(dark).toEqual(["白", "黄", "橙", "桃"]);
  });

  it("maps a field index to its gate", () => {
    expect(gateFor(0).no).toBe(1);
    expect(gateFor(7).no).toBe(8);
  });

  it("gives each runner its own gate in small fields", () => {
    for (let n = 1; n <= 8; n++) {
      for (let i = 0; i < n; i++) expect(gateForRunner(i, n).no).toBe(i + 1);
    }
  });

  it("fills big fields from the back, two horses per gate, as in real racing", () => {
    const gates = (n: number) => Array.from({ length: n }, (_, i) => gateForRunner(i, n).no);
    expect(gates(9)).toEqual([1, 2, 3, 4, 5, 6, 7, 8, 8]);
    expect(gates(12)).toEqual([1, 2, 3, 4, 5, 5, 6, 6, 7, 7, 8, 8]);
    expect(gates(14)).toEqual([1, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8]);
    expect(gates(16)).toEqual([1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8]);
  });

  it("puts the extra horses of a 17 or 18 runner field into the last gates", () => {
    const gates = (n: number) => Array.from({ length: n }, (_, i) => gateForRunner(i, n).no);
    expect(gates(17)).toEqual([1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 8]);
    expect(gates(18)).toEqual([1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 7, 8, 8, 8]);
  });

  it("always uses every colour, front to back, with at most three horses per gate", () => {
    for (let n = 8; n <= 18; n++) {
      const gates = Array.from({ length: n }, (_, i) => gateForRunner(i, n).no);
      expect(gates).toEqual([...gates].sort((a, b) => a - b));
      expect(new Set(gates).size).toBe(8);
      for (const g of new Set(gates)) expect(gates.filter((x) => x === g).length).toBeLessThanOrEqual(3);
    }
  });
});
