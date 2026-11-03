/** 枠番 colours as used in Japanese horse racing (1白 2黒 3赤 4青 5黄 6緑 7橙 8桃). */
export interface Gate {
  no: number;
  label: string;
  bg: string;
  fg: string;
}

export const GATES: Gate[] = [
  { no: 1, label: "白", bg: "#ffffff", fg: "#111" },
  { no: 2, label: "黒", bg: "#111111", fg: "#fff" },
  { no: 3, label: "赤", bg: "#e60012", fg: "#fff" },
  { no: 4, label: "青", bg: "#0067c0", fg: "#fff" },
  { no: 5, label: "黄", bg: "#ffd800", fg: "#111" },
  { no: 6, label: "緑", bg: "#00a73c", fg: "#fff" },
  { no: 7, label: "橙", bg: "#f58220", fg: "#111" },
  { no: 8, label: "桃", bg: "#f78fb3", fg: "#111" },
];

export const gateFor = (index: number): Gate => GATES[index % GATES.length];

/**
 * Gate for runner `index` (0-based, in post order) in a field of `count`.
 * Follows the real rule: up to 8 runners each get their own gate; with 9-16
 * the later gates take two horses each; with 17-18 the last gates take three.
 */
export const gateForRunner = (index: number, count: number): Gate => {
  const gates = GATES.length;
  if (count <= gates) return GATES[index];
  // how many horses each gate holds, front to back
  const sizes =
    count <= gates * 2
      ? Array.from({ length: gates }, (_, g) => (g < gates * 2 - count ? 1 : 2))
      : Array.from({ length: gates }, (_, g) => (g >= gates - (count - gates * 2) ? 3 : 2));
  let seen = 0;
  for (let g = 0; g < gates; g++) {
    seen += sizes[g];
    if (index < seen) return GATES[g];
  }
  return GATES[gates - 1];
};
