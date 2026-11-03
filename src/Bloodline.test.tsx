import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor, within } from "@testing-library/react";
import Bloodline from "./Bloodline";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
import { invoke } from "@tauri-apps/api/core";

const stats = { speed: 85, stamina: 70.5, temper: 40, power: 60, turf: 75, dirt: 55 };

function horse(over: Record<string, unknown>): Record<string, unknown> {
  return {
    sire_id: null,
    dam_id: null,
    gender: "male",
    lineage: "Ignis",
    family: "Sire",
    gen: 1,
    coat_color: "鹿毛",
    genes: { G: "gg", E: "EE", A: "AA" },
    growth_factor: 0.5,
    growth_type: "普通",
    peak_age: 4,
    aptitude_min: 1600,
    aptitude_max: 2400,
    running_style: "先行",
    personality: "普通",
    inbreed_vol: 0,
    stats,
    ...over,
  };
}

// 1,2 founding sires; 3,4 founding dams; 5..9 descend from them
const db = [
  horse({ id: 1, name: "イグニス", lineage: "Ignis" }),
  horse({ id: 2, name: "アクア", lineage: "Aqua", coat_color: "芦毛" }),
  horse({ id: 3, name: "ハルカ", gender: "female", lineage: "None", family: "F1" }),
  horse({ id: 4, name: "アカネ", gender: "female", lineage: "None", family: "F2" }),
  horse({ id: 5, name: "ミナモ", gender: "female", gen: 2, sire_id: 1, dam_id: 3, lineage: "Ignis", family: "F1" }),
  horse({ id: 6, name: "ソラオー", gen: 2, sire_id: 2, dam_id: 4, lineage: "Aqua", family: "F2" }),
  horse({ id: 7, name: "ホシカゲ", gen: 3, sire_id: 6, dam_id: 5, lineage: "Aqua", family: "F1" }),
  horse({ id: 8, name: "ツキヒメ", gender: "female", gen: 3, sire_id: 1, dam_id: 4, lineage: "Ignis", family: "F2" }),
  horse({
    id: 9, name: "ハヤテノ", gen: 4, sire_id: 7, dam_id: 8, lineage: "Aqua", family: "F2",
    coat_color: "栗毛", growth_type: "晩成", peak_age: 6, aptitude_min: 1600, aptitude_max: 2400,
    running_style: "差し", personality: "冷静", inbreed_vol: 0.125,
    stats: { speed: 85, stamina: 70.5, temper: 40, power: 60, turf: 75, dirt: 55 },
  }),
];

// what the backend's track-effect calculation would answer
const effectsFor = (a: { speed: number; stamina: number }) => ({
  aptitude: 27,
  conditions: [
    { condition: "good", speed: a.speed, stamina: a.stamina },
    { condition: "soft", speed: 85.0, stamina: 55.6 },
    { condition: "yielding", speed: 80.1, stamina: 51.2 },
    { condition: "heavy", speed: 70.3, stamina: 41.9 },
  ],
});

function backend(data: unknown[], effects: (a: never) => unknown = effectsFor as never) {
  vi.mocked(invoke).mockImplementation(async (cmd: string, args?: unknown) => {
    if (cmd === "get_bloodline") return data;
    if (cmd === "get_track_effects") return effects(args as never);
    throw new Error("unexpected " + cmd);
  });
}

async function open(data: unknown[] = db) {
  backend(data);
  render(<Bloodline />);
  await screen.findByText(/全\d+頭/);
}

const list = () => screen.getByRole("table", { name: "馬の一覧" });
const detail = () => screen.getByRole("region", { name: "馬の詳細" });

describe("Bloodline database", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("loads the pedigree once and shows how many horses there are", async () => {
    await open();
    expect(invoke).toHaveBeenCalledTimes(1); // the track effects are only asked for once a horse is picked
    expect(invoke).toHaveBeenCalledWith("get_bloodline");
    expect(screen.getByText("全9頭")).toBeTruthy();
    expect(within(list()).getByText("ハヤテノ")).toBeTruthy();
  });

  it("asks for a horse to be chosen before showing details", async () => {
    await open();
    expect(screen.getByText("馬を選ぶと、詳しい情報と血統表が表示されます。")).toBeTruthy();
    expect(screen.queryByRole("region", { name: "馬の詳細" })).toBeNull();
  });

  it("filters by generation, sex, lineage and name", async () => {
    await open();
    const rows = () => within(list()).getAllByRole("row").length - 1;
    expect(rows()).toBe(9);
    fireEvent.change(screen.getByLabelText("世代"), { target: { value: "4" } });
    expect(rows()).toBe(1);
    expect(screen.getByText("該当 1頭")).toBeTruthy();
    fireEvent.change(screen.getByLabelText("世代"), { target: { value: "all" } });
    fireEvent.change(screen.getByLabelText("性別"), { target: { value: "female" } });
    expect(rows()).toBe(4); // ハルカ アカネ ミナモ ツキヒメ
    fireEvent.change(screen.getByLabelText("性別"), { target: { value: "all" } });
    fireEvent.change(screen.getByLabelText("系統"), { target: { value: "Aqua" } });
    expect(rows()).toBe(4); // アクア ソラオー ホシカゲ ハヤテノ
    fireEvent.change(screen.getByLabelText("系統"), { target: { value: "all" } });
    fireEvent.change(screen.getByLabelText("名前で検索"), { target: { value: "ホシ" } });
    expect(rows()).toBe(1);
    expect(within(list()).getByText("ホシカゲ")).toBeTruthy();
  });

  it("combines filters and says so when nothing matches", async () => {
    await open();
    fireEvent.change(screen.getByLabelText("世代"), { target: { value: "4" } });
    fireEvent.change(screen.getByLabelText("性別"), { target: { value: "female" } });
    expect(screen.getByText("該当する馬がいません。")).toBeTruthy();
    expect(screen.getByText("該当 0頭")).toBeTruthy();
  });

  it("offers every generation and lineage that is in the data", async () => {
    await open();
    const values = (label: string) =>
      Array.from((screen.getByLabelText(label) as HTMLSelectElement).options).map((o) => o.value);
    expect(values("世代")).toEqual(["all", "1", "2", "3", "4"]);
    expect(values("系統")).toEqual(["all", "Aqua", "Ignis", "None"]);
  });

  it("shows a horse's profile when it is picked", async () => {
    await open();
    fireEvent.click(within(list()).getByRole("button", { name: "ハヤテノ" }));
    const d = within(detail());
    expect(d.getByRole("heading", { name: /ハヤテノ/ })).toBeTruthy();
    expect(d.getByLabelText("ハヤテノの性別: 牡")).toBeTruthy();
    expect(d.getByText("栗毛")).toBeTruthy();
    expect(d.getByText("Aqua")).toBeTruthy();
    expect(d.getByText("F2")).toBeTruthy();
    expect(d.getByText("第4世代")).toBeTruthy();
    expect(d.getByText("晩成(6歳ごろがピーク)")).toBeTruthy();
    expect(d.getByText("1600m〜2400m")).toBeTruthy();
    expect(d.getByText("差し")).toBeTruthy();
    expect(d.getByText("冷静")).toBeTruthy();
    expect(d.getByText("0.125(やや強い)")).toBeTruthy();
  });

  it("shows the six stats as meters", async () => {
    await open();
    fireEvent.click(within(list()).getByRole("button", { name: "ハヤテノ" }));
    const d = within(detail());
    expect(d.getByRole("meter", { name: "スピード: 85" }).getAttribute("aria-valuenow")).toBe("85");
    expect(d.getByRole("meter", { name: "スタミナ: 70.5" })).toBeTruthy();
    expect(d.getByRole("meter", { name: "気性: 40" })).toBeTruthy();
    expect(d.getByRole("meter", { name: "パワー: 60" })).toBeTruthy();
    expect(d.getByRole("meter", { name: "芝適性: 75" })).toBeTruthy();
    expect(d.getByRole("meter", { name: "ダート適性: 55" })).toBeTruthy();
  });

  it("describes inbreeding strength", async () => {
    const withVol = (inbreed_vol: number) => [
      horse({ id: 1, name: "テスト", inbreed_vol }),
    ];
    for (const [vol, text] of [
      [0, "なし"],
      [0.05, "0.050(弱い)"],
      [0.125, "0.125(やや強い)"],
      [0.5, "0.500(強い)"],
    ] as const) {
      backend(withVol(vol));
      const { unmount } = render(<Bloodline />);
      await screen.findByText("全1頭");
      fireEvent.click(within(list()).getByRole("button", { name: "テスト" }));
      expect(within(detail()).getByText(text)).toBeTruthy();
      unmount();
    }
  });

  it("draws the pedigree back four generations", async () => {
    await open();
    fireEvent.click(within(list()).getByRole("button", { name: "ハヤテノ" }));
    const d = within(detail());
    expect(d.getByLabelText("父: ホシカゲ")).toBeTruthy();
    expect(d.getByLabelText("母: ツキヒメ")).toBeTruthy();
    expect(d.getByLabelText("父父: ソラオー")).toBeTruthy();
    expect(d.getByLabelText("父母: ミナモ")).toBeTruthy();
    expect(d.getByLabelText("母父: イグニス")).toBeTruthy();
    expect(d.getByLabelText("母母: アカネ")).toBeTruthy();
    expect(d.getByLabelText("父父父: アクア")).toBeTruthy();
    expect(d.getByLabelText("父父母: アカネ")).toBeTruthy();
    expect(d.getByLabelText("父母父: イグニス")).toBeTruthy();
    expect(d.getByLabelText("父母母: ハルカ")).toBeTruthy();
    // founders have no parents of their own
    expect(d.getByLabelText("母父父: 不明")).toBeTruthy();
    expect(d.getByLabelText("父父父父: 不明")).toBeTruthy();
    // 2 + 4 + 8 + 16 ancestors
    expect(d.getAllByRole("listitem").length).toBe(30);
  });

  it("jumps to an ancestor when its name is clicked", async () => {
    await open();
    fireEvent.click(within(list()).getByRole("button", { name: "ハヤテノ" }));
    fireEvent.click(within(within(detail()).getByLabelText("父: ホシカゲ")).getByRole("button"));
    expect(within(detail()).getByRole("heading", { name: /ホシカゲ/ })).toBeTruthy();
    expect(within(detail()).getByText("第3世代")).toBeTruthy();
    expect(within(detail()).getByLabelText("父: ソラオー")).toBeTruthy();
  });

  it("marks the chosen horse in the list", async () => {
    await open();
    fireEvent.click(within(list()).getByRole("button", { name: "ミナモ" }));
    const row = within(list()).getByRole("button", { name: "ミナモ" }).closest("tr")!;
    expect(row.className).toContain("picked");
  });

  it("pages through a long list", async () => {
    const many = Array.from({ length: 40 }, (_, i) => horse({ id: i + 1, name: `ウマ${i + 1}` }));
    await open(many);
    expect(screen.getByText("1 / 3 ページ")).toBeTruthy();
    expect(within(list()).getAllByRole("row").length - 1).toBe(15);
    expect((screen.getByText("前のページ") as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(screen.getByText("次のページ"));
    expect(screen.getByText("2 / 3 ページ")).toBeTruthy();
    expect(within(list()).getByText("ウマ16")).toBeTruthy();
    fireEvent.click(screen.getByText("次のページ"));
    expect(within(list()).getAllByRole("row").length - 1).toBe(10);
    expect((screen.getByText("次のページ") as HTMLButtonElement).disabled).toBe(true);
    // filtering goes back to the first page
    fireEvent.change(screen.getByLabelText("名前で検索"), { target: { value: "ウマ3" } });
    expect(screen.getByText("1 / 1 ページ")).toBeTruthy();
  });

  it("shows an error when the pedigree cannot be loaded", async () => {
    vi.mocked(invoke).mockRejectedValue("読み込めませんでした");
    render(<Bloodline />);
    await waitFor(() => expect(screen.getByText("読み込めませんでした")).toBeTruthy());
  });

  // ---- bad-track aptitude (道悪適性) ----

  it("asks the backend for the track effects of the horse that is picked", async () => {
    await open();
    fireEvent.click(within(list()).getByRole("button", { name: "ハヤテノ" }));
    await within(detail()).findByRole("meter", { name: "道悪適性: 27" });
    const call = vi.mocked(invoke).mock.calls.find((c) => c[0] === "get_track_effects")!;
    expect(call[1]).toEqual({ speed: 85, stamina: 70.5, power: 60, dirt: 55 });
  });

  it("shows the bad-track aptitude as a meter", async () => {
    await open();
    fireEvent.click(within(list()).getByRole("button", { name: "ハヤテノ" }));
    const meter = await within(detail()).findByRole("meter", { name: "道悪適性: 27" });
    expect(meter.getAttribute("aria-valuenow")).toBe("27");
    expect(within(detail()).getByText(/パワー70%/)).toBeTruthy();
  });

  it("lists speed and stamina on 良, 稍重, 重 and 不良 tracks", async () => {
    await open();
    fireEvent.click(within(list()).getByRole("button", { name: "ハヤテノ" }));
    const table = await within(detail()).findByRole("table", { name: "馬場状態別の能力" });
    const rows = within(table).getAllByRole("row").slice(1);
    const text = rows.map((r) => within(r).getAllByRole("cell").map((c) => c.textContent));
    expect(text).toEqual([
      ["良", "85", "70.5"],
      ["稍重", "85", "55.6"],
      ["重", "80.1", "51.2"],
      ["不良", "70.3", "41.9"],
    ]);
  });

  it("asks again when another horse is picked", async () => {
    await open();
    fireEvent.click(within(list()).getByRole("button", { name: "ハヤテノ" }));
    await within(detail()).findByRole("meter", { name: "道悪適性: 27" });
    fireEvent.click(within(list()).getByRole("button", { name: "ミナモ" }));
    await waitFor(() =>
      expect(vi.mocked(invoke).mock.calls.filter((c) => c[0] === "get_track_effects").length).toBe(2)
    );
  });

  it("shows nothing about tracks until a horse is picked", async () => {
    await open();
    expect(screen.queryByText("道悪適性")).toBeNull();
    expect(vi.mocked(invoke).mock.calls.some((c) => c[0] === "get_track_effects")).toBe(false);
  });

  it("copes with the track effects failing to load", async () => {
    backend(db, () => {
      throw "計算できませんでした";
    });
    render(<Bloodline />);
    await screen.findByText("全9頭");
    fireEvent.click(within(list()).getByRole("button", { name: "ハヤテノ" }));
    await within(detail()).findByText("計算できませんでした");
    // the rest of the profile is still there
    expect(within(detail()).getByText("栗毛")).toBeTruthy();
  });

  // ---- mutation (突然変異) ----

  // 7 (ホシカゲ) has a mutated speed and power; its foal 9 does not
  const mutant = {
    mutations: ["speed", "power"],
    stats: { speed: 112, stamina: 70.5, temper: 40, power: 118.5, turf: 75, dirt: 55 },
  };
  const withMutation = db.map((h) => (h.id === 7 ? { ...h, ...mutant } : h));
  const openMutant = async () => {
    backend(withMutation);
    render(<Bloodline />);
    await screen.findByText("全9頭");
  };

  it("marks mutated horses in the list and says how many there are", async () => {
    await openMutant();
    const row = (name: string) => within(list()).getByRole("button", { name }).closest("tr")!;
    expect(within(row("ホシカゲ")).getByLabelText("突然変異あり").textContent).toBe("✦");
    expect(within(row("ハヤテノ")).queryByLabelText("突然変異あり")).toBeNull();
    expect(within(list()).getAllByLabelText("突然変異あり").length).toBe(1);
    expect(screen.getByText("突然変異: 1頭")).toBeTruthy();
  });

  it("does not mention mutations when there are none", async () => {
    await open();
    expect(screen.queryByText(/突然変異:/)).toBeNull();
    expect(screen.queryByLabelText("突然変異あり")).toBeNull();
  });

  it("filters to the mutated horses only", async () => {
    await openMutant();
    fireEvent.click(screen.getByLabelText("突然変異のみ"));
    expect(within(list()).getAllByRole("row").length - 1).toBe(1);
    expect(screen.getByText("該当 1頭")).toBeTruthy();
    expect(within(list()).getByText("ホシカゲ")).toBeTruthy();
    fireEvent.click(screen.getByLabelText("突然変異のみ"));
    expect(within(list()).getAllByRole("row").length - 1).toBe(9);
  });

  it("lists which stats mutated in the detail", async () => {
    await openMutant();
    fireEvent.click(within(list()).getByRole("button", { name: "ホシカゲ" }));
    const d = within(detail());
    expect(d.getByText("突然変異")).toBeTruthy();
    expect(d.getByText("スピード・パワー")).toBeTruthy();
  });

  it("shows no mutation row for an ordinary horse", async () => {
    await openMutant();
    fireEvent.click(within(list()).getByRole("button", { name: "ハヤテノ" }));
    expect(within(detail()).queryByText("突然変異")).toBeNull();
  });

  it("shows stats above 100 in full but never draws the bar past its track", async () => {
    await openMutant();
    fireEvent.click(within(list()).getByRole("button", { name: "ホシカゲ" }));
    const d = within(detail());
    const speed = d.getByRole("meter", { name: "スピード: 112" });
    expect(speed.getAttribute("aria-valuenow")).toBe("112");
    expect(Number(speed.getAttribute("aria-valuemax"))).toBeGreaterThanOrEqual(112);
    expect((speed.firstElementChild as HTMLElement).style.width).toBe("100%");
    expect(d.getByRole("meter", { name: "パワー: 118.5" })).toBeTruthy();
    // an ordinary stat keeps its own proportion
    expect((d.getByRole("meter", { name: "スタミナ: 70.5" }).firstElementChild as HTMLElement).style.width).toBe("70.5%");
  });

  it("points out exactly which stat meters are mutations", async () => {
    await openMutant();
    fireEvent.click(within(list()).getByRole("button", { name: "ホシカゲ" }));
    const d = within(detail());
    expect(d.getByLabelText("スピードは突然変異")).toBeTruthy();
    expect(d.getByLabelText("パワーは突然変異")).toBeTruthy();
    expect(d.queryByLabelText("スタミナは突然変異")).toBeNull();
    expect(d.getByRole("meter", { name: "スピード: 112" }).className).toContain("mutated");
    expect(d.getByRole("meter", { name: "スタミナ: 70.5" }).className).not.toContain("mutated");
  });

  it("makes a mutated ancestor stand out in the pedigree", async () => {
    await openMutant();
    fireEvent.click(within(list()).getByRole("button", { name: "ハヤテノ" }));
    const d = within(detail());
    const sire = d.getByLabelText("父: ホシカゲ");
    expect(sire.className).toContain("mutant");
    expect(within(sire).getByLabelText("突然変異あり")).toBeTruthy();
    expect(d.getByLabelText("母: ツキヒメ").className).not.toContain("mutant");
  });

  it("copes with data from before mutations existed", async () => {
    // the db fixture has no `mutations` field at all
    await open();
    fireEvent.click(within(list()).getByRole("button", { name: "ハヤテノ" }));
    expect(within(detail()).getByRole("meter", { name: "スピード: 85" })).toBeTruthy();
  });

  it("caps a bad-track aptitude bar above 100 too", async () => {
    backend(withMutation, () => ({
      aptitude: 112.5,
      conditions: [{ condition: "good", speed: 112, stamina: 70.5 }],
    }));
    render(<Bloodline />);
    await screen.findByText("全9頭");
    fireEvent.click(within(list()).getByRole("button", { name: "ホシカゲ" }));
    const meter = await within(detail()).findByRole("meter", { name: "道悪適性: 112.5" });
    expect(meter.getAttribute("aria-valuenow")).toBe("112.5");
    expect((meter.firstElementChild as HTMLElement).style.width).toBe("100%");
  });

  // ---- horses of the player's own stable ----

  const withOwned = db.map((h) => (h.id === 9 || h.id === 5 ? { ...h, owned: true } : h));
  const openOwned = async () => {
    backend(withOwned);
    render(<Bloodline />);
    await screen.findByText("全9頭");
  };

  it("marks the horses of my stable in the list", async () => {
    await openOwned();
    const row = (name: string) => within(list()).getByRole("button", { name }).closest("tr")!;
    expect(within(row("ハヤテノ")).getByLabelText("厩舎の馬")).toBeTruthy();
    expect(within(row("ミナモ")).getByLabelText("厩舎の馬")).toBeTruthy();
    expect(within(row("ホシカゲ")).queryByLabelText("厩舎の馬")).toBeNull();
    expect(screen.getByText("厩舎の馬: 2頭")).toBeTruthy();
  });

  it("says nothing about a stable when none of the horses is mine", async () => {
    await open();
    expect(screen.queryByLabelText("厩舎の馬")).toBeNull();
    expect(screen.queryByText(/^厩舎の馬:/)).toBeNull();
  });

  it("filters to my stable's horses", async () => {
    await openOwned();
    fireEvent.click(screen.getByLabelText("厩舎の馬のみ"));
    expect(within(list()).getAllByRole("row").length - 1).toBe(2);
    expect(screen.getByText("該当 2頭")).toBeTruthy();
    fireEvent.click(screen.getByLabelText("厩舎の馬のみ"));
    expect(within(list()).getAllByRole("row").length - 1).toBe(9);
  });

  it("tells that a chosen horse is one of mine", async () => {
    await openOwned();
    fireEvent.click(within(list()).getByRole("button", { name: "ハヤテノ" }));
    expect(within(detail()).getByText("厩舎の馬です。")).toBeTruthy();
    fireEvent.click(within(list()).getByRole("button", { name: "ホシカゲ" }));
    expect(within(detail()).queryByText("厩舎の馬です。")).toBeNull();
  });

  it("combines the stable filter with the others", async () => {
    await openOwned();
    fireEvent.click(screen.getByLabelText("厩舎の馬のみ"));
    fireEvent.change(screen.getByLabelText("性別"), { target: { value: "female" } });
    expect(within(list()).getAllByRole("row").length - 1).toBe(1); // ミナモ
    expect(within(list()).getByText("ミナモ")).toBeTruthy();
  });
});
