import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor, act, within } from "@testing-library/react";
import App from "./App";
import { gradeLabel, mayRunOver } from "./Calendar";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
import { invoke } from "@tauri-apps/api/core";

const studMare = {
  name: "ロウバイ", sex: "female", speed: 70, stamina: 72, power: 60, dirt: 0,
  retired_age: 6, retired_year: 2, bought: false, starter: false,
};

const studSire = { ...studMare, name: "タイセイオー", sex: "male" };

const state = {
  week: 1,
  year: 1,
  money: 1000,
  round: 0,
  game_over: false,
  history: [],
  breed_cost: 500,
  studs: [studSire, { ...studMare, conception: 80 }],
  dirt_sessions: 3,
  race_fatigue_limit: 50,
  train_fatigue_limit: 90,
  workouts: [
    { workout: "wood", name: "ウッドコース追い", description: "速さを磨きます。", gains: [{ ability: "speed", amount: 3 }, { ability: "stamina", amount: 1 }], fatigue: 8 },
    { workout: "hill", name: "坂路", description: "力をつけます。", gains: [{ ability: "power", amount: 3 }, { ability: "stamina", amount: 1 }] },
    { workout: "pool", name: "プール", description: "スタミナをつけます。", gains: [{ ability: "stamina", amount: 3 }, { ability: "temper", amount: 1 }] },
    { workout: "pair", name: "併せ馬", description: "気が立ちます。", gains: [{ ability: "speed", amount: 2 }, { ability: "power", amount: 2 }, { ability: "temper", amount: -2 }], fatigue: 12 },
    { workout: "walk", name: "曳き運動", description: "落ち着かせます。", gains: [{ ability: "temper", amount: 3 }] },
    { workout: "dirt", name: "ダートコース", description: "砂に慣れさせます。", gains: [{ ability: "dirt", amount: 1 }, { ability: "power", amount: 1 }] },
  ],
  field: [
    { name: "Fast", speed: 90, stamina: 80, power: 75, bad_track: 70, odds: 3.2, place_odds: 1.5, train_cost: 100, condition: "great", sex: "male", pedigree: { sire: "ミツルギオー", dam: "ハルカゼノマイ", sire_sire: "ホウオウライデン", sire_dam: "シラユキヒメ", dam_sire: "アカツキノカゼ", dam_dam: "ハナノミヤコ" } },
    { name: "Slow", speed: 40, stamina: 50, power: 30, bad_track: 47, odds: 8.5, place_odds: 2.8, train_cost: 150, condition: "poor", sex: "female", pedigree: null },
  ],
};

// the free race as the backend now builds it: a few of the stable's horses among rivals
const freeCard = {
  race: null,
  distance: 2400,
  surface: "turf",
  track: "good",
  max_entries: 3,
  first_prize: 0,
  my_horses: state.field.map((h) => ({ name: h.name, eligible: true, entered: true })),
  entrants: state.field.map((h) => ({
    name: h.name, speed: h.speed, stamina: h.stamina, power: h.power, bad_track: h.bad_track,
    condition: h.condition, sex: h.sex, dirt: 0, mine: true, odds: h.odds, place_odds: h.place_odds,
  })),
};

// a card for a calendar race when asked for it, the free race card otherwise
const cardFor = (calendarCard: unknown) => (a: { race?: unknown }) => (a?.race ? calendarCard : freeCard);

function mockBackend(
  bet?: unknown,
  train?: unknown,
  breed?: unknown,
  extra: Record<string, unknown> = {}
) {
  vi.mocked(invoke).mockImplementation(async (cmd: string, args?: unknown) => {
    if (cmd in extra) {
      const v = extra[cmd];
      if (v instanceof Error) throw v.message;
      return typeof v === "function" ? (v as (a: unknown) => unknown)(args) : v;
    }
    if (cmd === "get_game_state") return state;
    if (cmd === "place_bets" || cmd === "watch_race") {
      if (bet instanceof Error) throw bet.message;
      return bet;
    }
    if (cmd === "reset_game") return state;
    if (cmd === "get_calendar") return { meetings: [], races: [] };
    if (cmd === "get_bloodline") return [];
    if (cmd === "quote_bet") return 10;
    if (cmd === "breeding_outlook") return { inbreed_vol: 0, penalty: 0, nick: null };
    if (cmd === "get_stud_market") return [];
    if (cmd === "quote_tickets") return ((args as { tickets: unknown[] }).tickets ?? []).map(() => 10);
    if (cmd === "get_race_card") return freeCard;
    if (cmd === "set_entries") return undefined;
    if (cmd === "breed_horses") {
      if (breed instanceof Error) throw breed.message;
      return breed;
    }
    if (cmd === "train_horse") {
      if (train instanceof Error) throw train.message;
      return train;
    }
    throw new Error("unexpected " + cmd);
  });
}

/** Waits for the game to load and for the race card to be asked for and delivered. */
async function ready() {
  await waitFor(() => screen.getByText("所持金: 1000"));
  await waitFor(() => expect(vi.mocked(invoke).mock.calls.some((c) => c[0] === "get_race_card")).toBe(true));
  // let the answer arrive and be drawn (a whole timer turn, so every pending promise settles)
  await act(async () => {
    await new Promise((r) => setTimeout(r, 0));
  });
}

/** Chooses a training menu for a horse on the training tab and runs it. */
function workout(horse: string, menu: string) {
  fireEvent.change(screen.getByLabelText(`${horse}の調教メニュー`), { target: { value: menu } });
  fireEvent.click(screen.getByLabelText(`${horse}を調教`));
}

describe("App", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("shows balance and the field with odds", async () => {
    mockBackend();
    render(<App />);
    await ready();
    expect(screen.getAllByText("Fast").length).toBeGreaterThan(0);
    expect(screen.getByText("x3.2")).toBeTruthy();
  });

  it("places a bet and shows ranked results and new balance", async () => {
    mockBackend({
      results: [
        { rank: 1, name: "Fast", finish_time: 60.5 },
        { rank: 2, name: "Slow", finish_time: 70.25 },
      ],
      frames: [[600, 300], [1800, 1200], [2400, 2000]],
      distance: 2400,
      commentary: [],
      won: true,
      payout: 320,
      money: 1220,
    });
    render(<App />);
    await ready();
    fireEvent.change(screen.getByLabelText("馬"), { target: { value: "Fast" } });
    fireEvent.change(screen.getByLabelText("賭け金"), { target: { value: "100" } });
    fireEvent.click(screen.getByText("買い目に追加"));
    fireEvent.click(screen.getByText("レース開始"));
    fireEvent.click(await screen.findByText("スキップ"));
    await waitFor(() => expect(screen.getByText("1着 Fast (60.50秒)")).toBeTruthy());
    expect(screen.getByText("的中! 配当 320")).toBeTruthy();
    expect(screen.getByText("所持金: 1220")).toBeTruthy();
    const call = vi.mocked(invoke).mock.calls.find((c) => c[0] === "place_bets")!;
    expect(call[1]).toMatchObject({ tickets: [{ horses: ["Fast"], amount: 100, kind: "win" }] });
  });

  it("shows a loss message", async () => {
    mockBackend({
      results: [{ rank: 1, name: "Slow", finish_time: 60 }],
      frames: [[2400, 2400]],
      distance: 2400,
      commentary: [],
      won: false,
      payout: 0,
      money: 900,
    });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByText("買い目に追加"));
    fireEvent.click(screen.getByText("レース開始"));
    fireEvent.click(await screen.findByText("スキップ"));
    await waitFor(() => expect(screen.getByText("はずれ。-100")).toBeTruthy());
    expect(screen.getByText("所持金: 900")).toBeTruthy();
  });

  it("shows backend errors", async () => {
    mockBackend(new Error("所持金が足りません"));
    render(<App />);
    await ready();
    fireEvent.click(screen.getByText("レース開始"));
    await waitFor(() => expect(screen.getByText("所持金が足りません")).toBeTruthy());
  });

  it("offers a restart when the game is over", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) =>
      cmd === "reset_game" ? state : { ...state, money: 0, game_over: true }
    );
    render(<App />);
    await waitFor(() => screen.getByText("ゲームオーバー"));
    fireEvent.click(screen.getByText("最初から"));
    await waitFor(() => expect(screen.getByText("所持金: 1000")).toBeTruthy());
  });

  it("replays the race frame by frame before revealing results", async () => {
    mockBackend({
      results: [
        { rank: 1, name: "Fast", finish_time: 60.5 },
        { rank: 2, name: "Slow", finish_time: 70.25 },
      ],
      frames: [[600, 300], [1800, 1200], [2400, 2000]],
      distance: 2400,
      commentary: [],
      won: true,
      payout: 320,
      money: 1220,
    });
    render(<App />);
    await ready();
    vi.useFakeTimers();
    try {
      fireEvent.click(screen.getByText("買い目に追加"));
      fireEvent.click(screen.getByText("レース開始"));
      await act(async () => {});
      const bars = () => screen.getAllByRole("progressbar").map((b) => Number(b.getAttribute("aria-valuenow")));
      expect(bars()).toEqual([0, 0]);
      expect(screen.queryByText("的中! 配当 320")).toBeNull();
      expect(screen.getByText("所持金: 1000")).toBeTruthy();
      await act(async () => {
        vi.advanceTimersByTime(100);
      });
      expect(bars()).toEqual([600, 300]);
      await act(async () => {
        vi.advanceTimersByTime(300);
      });
      expect(bars()).toEqual([2400, 2000]);
      expect(screen.getByText("的中! 配当 320")).toBeTruthy();
      expect(screen.getByText("所持金: 1220")).toBeTruthy();
    } finally {
      vi.useRealTimers();
    }
  });

  it("trains a horse and refreshes stats and money", async () => {
    const trained = {
      ...state,
      money: 900,
      field: [{ ...state.field[0], speed: 93, train_cost: 150 }, state.field[1]],
    };
    mockBackend(undefined, trained);
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "育成" }));
    workout("Fast", "wood");
    await waitFor(() => expect(screen.getByText("所持金: 900")).toBeTruthy());
    expect(screen.getByText("93")).toBeTruthy();
    const call = vi.mocked(invoke).mock.calls.find((c) => c[0] === "train_horse")!;
    expect(call[1]).toEqual({ horse: "Fast", workout: "wood" });
  });

  it("shows training errors", async () => {
    mockBackend(undefined, new Error("所持金が足りません"));
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "育成" }));
    workout("Slow", "pool");
    await waitFor(() => expect(screen.getByText("所持金が足りません")).toBeTruthy());
  });

  it("disables training while a race is being replayed", async () => {
    mockBackend({
      results: [{ rank: 1, name: "Fast", finish_time: 60 }],
      frames: [[600, 300], [2400, 2000]],
      distance: 2400,
      commentary: [],
      won: true,
      payout: 320,
      money: 1220,
    });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByText("レース開始"));
    await screen.findByText("スキップ");
    fireEvent.click(screen.getByRole("tab", { name: "育成" }));
    expect((screen.getByLabelText("Fastを調教") as HTMLButtonElement).disabled).toBe(true);
  });

  it("saves the game and confirms", async () => {
    mockBackend();
    render(<App />);
    await ready();
    vi.mocked(invoke).mockResolvedValueOnce(undefined);
    fireEvent.click(screen.getByText("セーブ"));
    await waitFor(() => expect(screen.getByText("セーブしました。")).toBeTruthy());
    expect(vi.mocked(invoke).mock.calls.some((c) => c[0] === "save_game")).toBe(true);
  });

  it("loads a saved game and replaces the displayed state", async () => {
    mockBackend();
    render(<App />);
    await ready();
    vi.mocked(invoke).mockResolvedValueOnce({ ...state, money: 420, round: 5 });
    fireEvent.click(screen.getByText("ロード"));
    await waitFor(() => expect(screen.getByText("所持金: 420")).toBeTruthy());
    expect(screen.getByText("ラウンド: 5")).toBeTruthy();
    expect(screen.getByText("ロードしました。")).toBeTruthy();
  });

  it("shows an error when there is no save to load", async () => {
    mockBackend();
    render(<App />);
    await ready();
    vi.mocked(invoke).mockRejectedValueOnce("セーブデータがありません");
    fireEvent.click(screen.getByText("ロード"));
    await waitFor(() => expect(screen.getByText("セーブデータがありません")).toBeTruthy());
    expect(screen.getByText("所持金: 1000")).toBeTruthy();
  });

  it("shows past rounds from a loaded game with net result", async () => {
    mockBackend();
    render(<App />);
    await ready();
    expect(screen.getByText("まだレースがありません。")).toBeTruthy();
    vi.mocked(invoke).mockResolvedValueOnce({
      ...state,
      round: 2,
      history: [
        { kind: "win", round: 1, horse: "Fast", stake: 100, won: true, payout: 320, winner: "Fast" },
        { kind: "place", round: 2, horse: "Slow", stake: 50, won: false, payout: 0, winner: "Fast" },
      ],
    });
    fireEvent.click(screen.getByText("ロード"));
    await waitFor(() => expect(screen.getByText("収支: +170")).toBeTruthy());
    expect(screen.getByText("第2R 複勝 Slow 50 → はずれ -50 (優勝: Fast)")).toBeTruthy();
    expect(screen.getByText("第1R 単勝 Fast 100 → 的中 +220 (優勝: Fast)")).toBeTruthy();
  });

  it("adds the finished race to the history", async () => {
    mockBackend({
      results: [
        { rank: 1, name: "Fast", finish_time: 60.5 },
        { rank: 2, name: "Slow", finish_time: 70.25 },
      ],
      frames: [[2400, 2000]],
      distance: 2400,
      commentary: [],
      won: true,
      payout: 320,
      money: 1220,
    });
    render(<App />);
    await ready();
    fireEvent.change(screen.getByLabelText("馬"), { target: { value: "Fast" } });
    fireEvent.click(screen.getByText("買い目に追加"));
    fireEvent.click(screen.getByText("レース開始"));
    fireEvent.click(await screen.findByText("スキップ"));
    await waitFor(() => expect(screen.getByText("第1R 単勝 Fast 100 → 的中 +220 (優勝: Fast)")).toBeTruthy());
    expect(screen.getByText("収支: +220")).toBeTruthy();
  });

  it("shows each horse's condition", async () => {
    mockBackend();
    render(<App />);
    await ready();
    expect(screen.getByLabelText("Fastの調子: 絶好調").textContent).toBe("絶好調");
    expect(screen.getByLabelText("Slowの調子: 不調").textContent).toBe("不調");
  });

  it("shows each horse's gate number and colour", async () => {
    mockBackend();
    render(<App />);
    await ready();
    const g1 = screen.getByLabelText("Fastの枠番: 1(白)");
    const g2 = screen.getByLabelText("Slowの枠番: 2(黒)");
    expect(g1.textContent).toBe("1");
    expect(g2.textContent).toBe("2");
    expect(g2.style.background).toBe("rgb(17, 17, 17)");
  });

  it("shows win and place odds", async () => {
    mockBackend();
    render(<App />);
    await ready();
    expect(screen.getByText("x3.2")).toBeTruthy();
    expect(screen.getByText("x1.5")).toBeTruthy();
    expect(screen.getByText("単勝")).toBeTruthy();
    expect(screen.getByText("複勝")).toBeTruthy();
  });

  it("places a place (複勝) bet when selected and records it in history", async () => {
    mockBackend({
      results: [
        { rank: 1, name: "Slow", finish_time: 60 },
        { rank: 2, name: "Fast", finish_time: 61 },
      ],
      frames: [[2300, 2400]],
      distance: 2400,
      commentary: [],
      won: true,
      payout: 150,
      money: 1050,
    });
    render(<App />);
    await ready();
    fireEvent.change(screen.getByLabelText("賭け方"), { target: { value: "place" } });
    fireEvent.click(screen.getByText("買い目に追加"));
    fireEvent.click(screen.getByText("レース開始"));
    fireEvent.click(await screen.findByText("スキップ"));
    await waitFor(() => expect(screen.getByText("第1R 複勝 Fast 100 → 的中 +50 (優勝: Slow)")).toBeTruthy());
    const call = vi.mocked(invoke).mock.calls.find((c) => c[0] === "place_bets")!;
    expect(call[1]).toMatchObject({ tickets: [{ kind: "place" }] });
  });

  it("has race, training and pedigree tabs with the race tab first", async () => {
    mockBackend();
    render(<App />);
    await ready();
    const tabs = screen.getAllByRole("tab").map((t) => t.textContent);
    expect(tabs).toEqual(["レース", "育成", "厩舎", "血統", "繁殖", "カレンダー", "情報", "実績"]);
    expect(screen.getByRole("tab", { name: "レース" }).getAttribute("aria-selected")).toBe("true");
    expect(screen.getByLabelText("賭け金")).toBeTruthy();
    expect(screen.queryByLabelText("Fastを調教")).toBeNull();
  });

  it("keeps training on its own tab, away from the race screen", async () => {
    mockBackend();
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "育成" }));
    expect(screen.getByRole("tab", { name: "育成" }).getAttribute("aria-selected")).toBe("true");
    expect(screen.getByLabelText("Fastを調教")).toBeTruthy();
    expect(screen.queryByLabelText("賭け金")).toBeNull();
    expect(screen.getByText("所持金: 1000")).toBeTruthy(); // balance stays visible
    fireEvent.click(screen.getByRole("tab", { name: "レース" }));
    expect(screen.getByLabelText("賭け金")).toBeTruthy();
  });

  it("shows the pedigree of each horse on the pedigree tab", async () => {
    mockBackend();
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "血統" }));
    expect(screen.getByText("父: ミツルギオー")).toBeTruthy();
    expect(screen.getByText("母: ハルカゼノマイ")).toBeTruthy();
    expect(screen.getByText("父父: ホウオウライデン")).toBeTruthy();
    expect(screen.getByText("母母: ハナノミヤコ")).toBeTruthy();
    expect(screen.getByText("血統不明")).toBeTruthy();
    expect(screen.queryByLabelText("賭け金")).toBeNull();
  });

  it("keeps the race replay running while another tab is open", async () => {
    mockBackend({
      results: [{ rank: 1, name: "Fast", finish_time: 60 }],
      frames: [[600, 300], [2400, 2000]],
      distance: 2400,
      commentary: [],
      won: true,
      payout: 320,
      money: 1220,
    });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByText("買い目に追加"));
    fireEvent.click(screen.getByText("レース開始"));
    await screen.findByText("スキップ");
    fireEvent.click(screen.getByRole("tab", { name: "血統" }));
    fireEvent.click(screen.getByRole("tab", { name: "レース" }));
    expect(screen.getByText("スキップ")).toBeTruthy();
    fireEvent.click(screen.getByText("スキップ"));
    await waitFor(() => expect(screen.getByText("的中! 配当 320")).toBeTruthy());
  });

  it("shows each horse's sex on the pedigree tab", async () => {
    mockBackend();
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "血統" }));
    expect(screen.getByLabelText("Fastの性別: 牡").textContent).toBe("牡");
    expect(screen.getByLabelText("Slowの性別: 牝").textContent).toBe("牝");
  });

  it("offers stallions as sire and mares as dam in the breeding form", async () => {
    mockBackend();
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "繁殖" }));
    const options = (label: string) =>
      Array.from((screen.getByLabelText(label) as HTMLSelectElement).options).map((o) => o.value);
    expect(options("父馬(牡)")).toEqual(["タイセイオー"]);
    expect(options("母馬(牝)")).toEqual(["ロウバイ"]);
    expect(options("入れ替える馬")).toEqual(["Fast", "Slow"]);
    expect(screen.getByText("全角カタカナ2〜9文字")).toBeTruthy();
  });

  it("mates a pair and says the foal is on the way", async () => {
    const mated = {
      ...state,
      money: 500,
      pregnancies: [{ foal_name: "ニューホープ", sire: "Fast", dam: "ロウバイ", replace: "Slow", weeks_left: 48 }],
    };
    mockBackend(undefined, undefined, mated);
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "繁殖" }));
    fireEvent.change(screen.getByLabelText("入れ替える馬"), { target: { value: "Slow" } });
    fireEvent.change(screen.getByLabelText("仔馬の名前"), { target: { value: "ニューホープ" } });
    fireEvent.click(screen.getByText("配合 (500)"));
    await waitFor(() => expect(screen.getByText("所持金: 500")).toBeTruthy());
    expect(screen.getByText("種付けしました。「ニューホープ」は48週後に生まれます。")).toBeTruthy();
    const call = vi.mocked(invoke).mock.calls.find((c) => c[0] === "breed_horses")!;
    expect(call[1]).toEqual({ sire: "タイセイオー", dam: "ロウバイ", replace: "Slow", name: "ニューホープ" });
  });

  describe("the breeding season", () => {
    const season = { breeding_start_week: 10, breeding_end_week: 22 };
    const openTab = async (over: object) => {
      mockBackend(undefined, undefined, undefined, { get_game_state: { ...state, ...season, ...over } });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "繁殖" }));
    };

    it("says when the season is and that it is open", async () => {
      await openTab({ breeding_open: true });
      expect(screen.getByText("種付けできるのは第10週〜第22週(3月〜5月)です。今は繁殖シーズンです。")).toBeTruthy();
    });

    it("refuses a mating out of season and says why", async () => {
      await openTab({ breeding_open: false, week: 30 });
      expect(screen.getByText("種付けできるのは第10週〜第22週(3月〜5月)です。今は第30週でシーズン外です。")).toBeTruthy();
      fireEvent.change(screen.getByLabelText("仔馬の名前"), { target: { value: "ニューホープ" } });
      expect((screen.getByText("配合 (500)") as HTMLButtonElement).disabled).toBe(true);
    });

    it("lists the foals on the way", async () => {
      await openTab({
        breeding_open: true,
        pregnancies: [{ foal_name: "ハナミズキ", sire: "Fast", dam: "Slow", replace: "", weeks_left: 12 }],
      });
      const list = screen.getByRole("region", { name: "出産待ち" });
      expect(within(list).getByText("ハナミズキ")).toBeTruthy();
      expect(within(list).getByText("父 Fast・母 Slow")).toBeTruthy();
      expect(within(list).getByText("あと12週")).toBeTruthy();
    });

    it("says which horse makes way when the foal comes", async () => {
      await openTab({
        breeding_open: true,
        pregnancies: [{ foal_name: "ハナミズキ", sire: "Fast", dam: "Slow", replace: "Fast", weeks_left: 12 }],
      });
      expect(within(screen.getByRole("region", { name: "出産待ち" })).getByText("Fastと入れ替わり")).toBeTruthy();
    });

    it("offers only retired mares as dams and says so when there are none", async () => {
      await openTab({ breeding_open: true, studs: [] });
      expect(within(screen.getByLabelText("母馬(牝)")).queryByRole("option", { name: "Slow" })).toBeNull();
      expect(screen.getByText("母馬にできる引退牝馬がいません。牝馬を引退させるか、繁殖馬を購入してください。")).toBeTruthy();
      fireEvent.change(screen.getByLabelText("仔馬の名前"), { target: { value: "ニューホープ" } });
      expect((screen.getByText("配合 (500)") as HTMLButtonElement).disabled).toBe(true);
    });

    const young = (age: number, away: object | null) => ({
      ...state,
      field: [state.field[0], { ...state.field[1], name: "ハナミズキ", sex: "male", age, away }],
    });
    const farm = { kind: "growing", label: "牧場", weeks: 30 };

    it("shows a foal growing at the farm in the training table", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: young(0, farm) });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "育成" }));
      expect(screen.getByText("牧場で育成中(残り30週)")).toBeTruthy();
    });

    it("says why a young horse cannot be entered in a race", async () => {
      const card = {
        ...freeCard,
        my_horses: [
          { name: "Fast", eligible: true, entered: false },
          { name: "ハナミズキ", eligible: false, entered: false },
        ],
      };
      mockBackend(undefined, undefined, undefined, { get_game_state: young(1, null), get_race_card: card });
      render(<App />);
      await ready();
      expect(screen.getByText("(2歳から)")).toBeTruthy();
    });

    it("says a foal at the farm cannot be entered in a race", async () => {
      const card = {
        ...freeCard,
        my_horses: [
          { name: "Fast", eligible: true, entered: false },
          { name: "ハナミズキ", eligible: false, entered: false, away: farm },
        ],
      };
      mockBackend(undefined, undefined, undefined, { get_game_state: young(0, farm), get_race_card: card });
      render(<App />);
      await ready();
      expect(screen.getByText("(牧場で育成中)")).toBeTruthy();
    });

    it("does not let a horse under two retire", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: young(1, null) });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "血統" }));
      expect((screen.getByLabelText("ハナミズキを引退させる") as HTMLButtonElement).disabled).toBe(true);
    });

    it("offers only retired stallions and those on the market as sires", async () => {
      await openTab({ breeding_open: true });
      const sires = within(screen.getByLabelText("父馬(牡)"));
      expect(sires.queryByRole("option", { name: "Fast" })).toBeNull();
      expect(sires.getByRole("option", { name: "タイセイオー(繁殖)" })).toBeTruthy();
    });

    it("shows how likely the mare chosen is to take", async () => {
      await openTab({ breeding_open: true });
      expect(screen.getByText("受胎率: 80%")).toBeTruthy();
    });

    it("says when a mating has not taken", async () => {
      mockBackend(undefined, undefined, { ...state, money: 500, pregnancies: [] });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "繁殖" }));
      fireEvent.change(screen.getByLabelText("仔馬の名前"), { target: { value: "ニューホープ" } });
      fireEvent.click(screen.getByText("配合 (500)"));
      await screen.findByText("不受胎でした。費用は戻りません。シーズン中ならもう一度種付けできます。");
    });

    it("shows no list when no foal is on the way", async () => {
      await openTab({ breeding_open: true, pregnancies: [] });
      expect(screen.queryByRole("region", { name: "出産待ち" })).toBeNull();
    });

    it("marks a mare in foal and does not offer her", async () => {
      await openTab({
        breeding_open: true,
        pregnancies: [{ foal_name: "ハナミズキ", sire: "Fast", dam: "ロウバイ", replace: "", weeks_left: 12 }],
      });
      const mare = screen.getByRole("option", { name: "ロウバイ(繁殖)(受胎中)" }) as HTMLOptionElement;
      expect(mare.disabled).toBe(true);
      fireEvent.change(screen.getByLabelText("仔馬の名前"), { target: { value: "ニューホープ" } });
      expect((screen.getByText("配合 (500)") as HTMLButtonElement).disabled).toBe(true);
    });
  });

  describe("racing abroad and sire lines", () => {
    const arc = { id: 2, name: "凱旋門賞", country: "フランス", week: 40, surface: "turf", distance: 2400, first_prize: 2500, open: true, candidates: ["Fast"] };
    const abroad = { ...state, expedition_cost: 500, overseas: [arc, { ...arc, id: 3, name: "香港カップ", country: "香港", week: 50, open: false, candidates: [] }], expeditions: [] };
    const open = async (over: object = {}) => {
      mockBackend(undefined, undefined, undefined, { get_game_state: abroad, ...over });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "厩舎" }));
      return screen.getByRole("region", { name: "海外遠征" });
    };

    it("lists the races abroad and sends a horse when asked", async () => {
      const go = vi.fn(() => abroad);
      const region = await open({ go_abroad: go });
      expect(within(region).getByText("凱旋門賞")).toBeTruthy();
      expect(within(region).getByText("フランス・第40週・芝2400m・1着 2500")).toBeTruthy();
      fireEvent.click(within(region).getByRole("button", { name: "凱旋門賞に遠征 (500)" }));
      await waitFor(() => expect(go).toHaveBeenCalledWith({ horse: "Fast", race: 2 }));
    });

    it("has no button for a race that is not open", async () => {
      const region = await open();
      expect(within(region).queryByRole("button", { name: /香港カップに遠征/ })).toBeNull();
    });

    it("shows who has gone abroad", async () => {
      const region = await open({
        get_game_state: { ...abroad, expeditions: [{ horse: "Fast", race: "凱旋門賞", year: 1 }] },
      });
      expect(within(region).getByText("Fastが凱旋門賞に遠征中です。")).toBeTruthy();
    });

    it("shows a horse away on a trip in the race entries", async () => {
      const card = { ...freeCard, my_horses: [{ name: "Fast", eligible: false, entered: false, away: { kind: "expedition", label: "遠征", weeks: 3 } }] };
      mockBackend(undefined, undefined, undefined, { get_race_card: card });
      render(<App />);
      await ready();
      expect(screen.getByText("(遠征中)")).toBeTruthy();
    });

    it("shows the sire lines and which are founded", async () => {
      mockBackend(undefined, undefined, undefined, {
        get_game_state: {
          ...state,
          sire_lines: [
            { sire: "タイセイオー", foals: 6, g1_wins: 3, established: true },
            { sire: "コガネナイト", foals: 1, g1_wins: 0, established: false },
          ],
          line_foals: 5,
          line_g1_wins: 3,
        },
      });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "実績" }));
      const region = screen.getByRole("region", { name: "系統" });
      expect(within(region).getByText("タイセイオー系(確立)")).toBeTruthy();
      expect(within(region).getByText("産駒6頭・G1 3勝")).toBeTruthy();
      expect(within(region).getByText("コガネナイト")).toBeTruthy();
    });
  });

  describe("the farm and the theories", () => {
    const farm = { pasture: 1, nursery: 0, max_level: 3, pasture_cost: 1600, nursery_cost: 800, upkeep_per_level: 5 };
    const open = async (over: object) => {
      mockBackend(undefined, undefined, undefined, { get_game_state: { ...state, money: 1000, farm }, ...over });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "繁殖" }));
    };

    it("shows the facilities and improves one when asked", async () => {
      const up = vi.fn(() => ({ ...state, farm: { ...farm, nursery: 1 } }));
      await open({ upgrade_farm: up });
      const region = screen.getByRole("region", { name: "牧場" });
      expect(within(region).getByText("放牧地 Lv1/3")).toBeTruthy();
      expect(within(region).getByText("育成施設 Lv0/3")).toBeTruthy();
      expect((within(region).getByRole("button", { name: "放牧地を拡張 (1600)" }) as HTMLButtonElement).disabled).toBe(true);
      fireEvent.click(within(region).getByRole("button", { name: "育成施設を拡張 (800)" }));
      await waitFor(() => expect(up).toHaveBeenCalledWith({ facility: "nursery" }));
    });

    it("has no button on a facility at its top level", async () => {
      await open({ get_game_state: { ...state, farm: { ...farm, pasture: 3, pasture_cost: null } } });
      const region = screen.getByRole("region", { name: "牧場" });
      expect(within(region).getByText("放牧地 Lv3/3")).toBeTruthy();
      expect(within(region).queryByRole("button", { name: /放牧地を拡張/ })).toBeNull();
    });

    it("shows the rating of the mating and the theories behind it", async () => {
      await open({
        breeding_outlook: {
          inbreed_vol: 0, penalty: 0, nick: null, rating: "A",
          theories: [{ name: "血の活性化", points: 1 }, { name: "能力の補完", points: 2 }],
        },
      });
      await screen.findByText("配合評価: A");
      expect(screen.getByText("血の活性化 +1")).toBeTruthy();
      expect(screen.getByText("能力の補完 +2")).toBeTruthy();
    });
  });

  describe("course maps on the information tab", () => {
    const track = { lap: "1m", straight: "1m", elevation: "1m" };
    const nakayama = {
      venue: "nakayama", name: "中山", place: "千葉県", turn: "right", grass: "野芝", turf: track, dirt: track, features: "", suits: "",
      maps: [
        { label: "芝 内回り", surface: "turf", layout: "内回り", lap: 1667, straight: 310, after_goal: 72 },
        { label: "芝 外回り", surface: "turf", layout: "外回り", lap: 1840, straight: 310, after_goal: 72 },
        { label: "ダート", surface: "dirt", layout: null, lap: 1493, straight: 308, after_goal: 68 },
      ],
    };

    it("draws a map for each way round and for the dirt", async () => {
      mockBackend(undefined, undefined, undefined, { get_courses: [nakayama], get_course_fits: [] });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "情報" }));
      const course = await screen.findByRole("region", { name: "中山競馬場" });
      const maps = within(course).getAllByRole("img");
      expect(maps.map((m) => m.getAttribute("aria-label"))).toEqual([
        "中山 芝 内回りのコース図",
        "中山 芝 外回りのコース図",
        "中山 ダートのコース図",
      ]);
      expect(maps[1].getAttribute("data-shape")).toBe("中山/turf/外回り");
      expect(maps[2].getAttribute("data-shape")).toBe("中山/dirt/");
      expect(within(course).getByText("芝 外回り")).toBeTruthy();
    });

    it("draws Niigata's straight course against its outer course", async () => {
      const niigata = {
        ...nakayama,
        venue: "niigata",
        name: "新潟",
        turn: "left",
        maps: [{ label: "芝 直線", surface: "turf", layout: "直線", lap: 1000, straight: 1000, after_goal: 0, straight_course: true }],
      };
      mockBackend(undefined, undefined, undefined, { get_courses: [niigata], get_course_fits: [] });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "情報" }));
      const map = await screen.findByRole("img", { name: "新潟 芝 直線のコース図" });
      expect(map.querySelectorAll(".map-corner").length).toBe(0);
      expect(map.querySelector(".map-backdrop")).not.toBeNull();
      expect(map.textContent).toContain("ゴール");
      expect(map.textContent).toContain("直線");
    });

    it("shows the goal and corners but no gate or horses, since no race is being run", async () => {
      mockBackend(undefined, undefined, undefined, { get_courses: [nakayama], get_course_fits: [] });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "情報" }));
      const map = await screen.findByRole("img", { name: "中山 ダートのコース図" });
      expect(map.textContent).toContain("ゴール");
      expect(map.textContent).toContain("右回り");
      expect(map.querySelectorAll(".map-corner").length).toBe(4);
      expect(map.textContent).not.toContain("ゲート");
      expect(map.querySelectorAll("circle").length).toBe(0);
    });
  });

  describe("the track panel", () => {
    const course = { name: "東京", turn: "左回り", straight: 500 };
    const lined = { ...freeCard, course, entrants: freeCard.entrants.map((e) => ({ ...e, mine: e.name === "Slow" })) };
    const race = {
      results: [
        { rank: 1, name: "Fast", finish_time: 130.1 },
        { rank: 2, name: "Slow", finish_time: 130.15 },
      ],
      frames: [[100, 90], [1950, 1949], [2400, 2399]],
      runners: ["Fast", "Slow"],
      distance: 2400,
      commentary: [],
      won: false,
      payout: 0,
      money: 900,
    };
    const start = async (outcome: object = race) => {
      mockBackend(outcome, undefined, undefined, { get_race_card: lined });
      render(<App />);
      await ready();
      vi.useFakeTimers();
      fireEvent.click(screen.getByText("レース開始"));
      await act(async () => {});
    };
    const tick = (ms: number) =>
      act(async () => {
        vi.advanceTimersByTime(ms);
      });

    it("says where the race is before it starts", async () => {
      mockBackend(undefined, undefined, undefined, { get_race_card: { ...lined, track: "soft" } });
      render(<App />);
      await ready();
      expect(screen.getByLabelText("コース情報").textContent).toBe("東京・芝2400m・左回り・稍重");
    });

    it("shows the stage, the distance left and who is in front as the race goes on", async () => {
      await start();
      try {
        await tick(100);
        const board = screen.getByRole("region", { name: "レース状況" });
        expect(within(board).getByText("スタート")).toBeTruthy();
        expect(within(board).getByText("残り 2300m")).toBeTruthy();
        expect(within(board).getByText("1番手 Fast")).toBeTruthy();
        expect(within(board).getByText("自分の馬 Slow 2番手")).toBeTruthy();
      } finally {
        vi.useRealTimers();
      }
    });

    it("marks the leader's lane and the stable's own", async () => {
      await start();
      try {
        await tick(100);
        const lane = (name: string) => screen.getByRole("progressbar", { name }).closest(".lane")!;
        expect(lane("Fast").classList.contains("leader")).toBe(true);
        expect(lane("Slow").classList.contains("my-lane")).toBe(true);
        expect(lane("Slow").querySelector(".lane-rank")!.textContent).toBe("2");
      } finally {
        vi.useRealTimers();
      }
    });

    it("slows down and calls it in the final straight", async () => {
      await start();
      try {
        await tick(200);
        expect(document.querySelector(".track")!.classList.contains("stretch")).toBe(true);
        const board = screen.getByRole("region", { name: "レース状況" });
        expect(within(board).getByText("最後の直線")).toBeTruthy();
        expect(within(board).getByText("大接戦!")).toBeTruthy();
        await tick(100);
        expect(screen.getByRole("progressbar", { name: "Fast" }).getAttribute("aria-valuenow")).toBe("1950");
        await tick(100);
        expect(screen.getByRole("progressbar", { name: "Fast" }).getAttribute("aria-valuenow")).toBe("2400");
      } finally {
        vi.useRealTimers();
      }
    });

    it("announces the winner and a photo finish at the line", async () => {
      await start();
      try {
        await tick(1000);
        const board = screen.getByRole("region", { name: "レース状況" });
        expect(within(board).getByText("1着 Fast!")).toBeTruthy();
        expect(within(board).getByText("写真判定の大接戦!")).toBeTruthy();
        expect(within(board).getByText("自分の馬 Slow 2着")).toBeTruthy();
      } finally {
        vi.useRealTimers();
      }
    });

    it("celebrates when the stable's horse wins", async () => {
      await start({
        ...race,
        results: [
          { rank: 1, name: "Slow", finish_time: 130.1 },
          { rank: 2, name: "Fast", finish_time: 131 },
        ],
      });
      try {
        await tick(1000);
        const board = screen.getByRole("region", { name: "レース状況" });
        expect(within(board).getByText("自分の馬が勝ちました!")).toBeTruthy();
        expect(board.classList.contains("victory")).toBe(true);
      } finally {
        vi.useRealTimers();
      }
    });

    describe("the course map", () => {
      const withLap = { ...lined, course: { ...course, lap: 2083 } };
      const dots = () => Array.from(screen.getByRole("img", { name: "コース図" }).querySelectorAll("circle"));

      it("draws the course and a dot for every runner before the race", async () => {
        mockBackend(undefined, undefined, undefined, { get_race_card: withLap });
        render(<App />);
        await ready();
        expect(dots().map((c) => c.getAttribute("data-horse")).sort()).toEqual(["Fast", "Slow"]);
        expect(screen.getByRole("img", { name: "コース図" }).textContent).toContain("ゴール");
      });

      it("says where the gate is and how many corners there are", async () => {
        mockBackend(undefined, undefined, undefined, {
          get_race_card: { ...freeCard, distance: 1600, surface: "dirt", course: { name: "東京", turn: "左回り", straight: 501, lap: 1899, after_goal: 39, corners: 2, turf_start: true } },
        });
        render(<App />);
        await ready();
        expect(screen.getByLabelText("ゲート位置").textContent).toBe("ゲート: 2コーナー奥ポケット・芝スタート・コーナー2回");
        expect(screen.getByRole("img", { name: "コース図" }).textContent).toContain("ゲート");
      });

      it("names the inner or outer course", async () => {
        mockBackend(undefined, undefined, undefined, {
          get_race_card: { ...freeCard, distance: 2000, course: { name: "京都", layout: "内回り", turn: "右回り", straight: 328, lap: 1783, after_goal: 93, corners: 4 } },
        });
        render(<App />);
        await ready();
        expect(screen.getByLabelText("コース情報").textContent).toContain("京都(内回り)・");
      });

      it("draws a straight course without corners", async () => {
        mockBackend(undefined, undefined, undefined, {
          get_race_card: { ...freeCard, distance: 1000, course: { name: "新潟", layout: "直線", turn: "左回り", straight: 1000, lap: 1000, after_goal: 0, corners: 0, straight_course: true } },
        });
        render(<App />);
        await ready();
        const map = screen.getByRole("img", { name: "コース図" });
        expect(map.querySelectorAll(".map-corner").length).toBe(0);
        expect(screen.getByLabelText("ゲート位置").textContent).toBe("ゲート: 直線コース・コーナーなし");
        expect(screen.getByLabelText("コース情報").textContent).toBe("新潟(直線)・芝1000m・良");
      });

      it("draws the shape of the course the race is on", async () => {
        mockBackend(undefined, undefined, undefined, {
          get_race_card: { ...freeCard, distance: 2000, course: { name: "中山", layout: "内回り", turn: "右回り", straight: 310, lap: 1667, after_goal: 72, corners: 4 } },
        });
        render(<App />);
        await ready();
        expect(screen.getByRole("img", { name: "コース図" }).getAttribute("data-shape")).toBe("中山/turf/内回り");
      });

      it("has no map when the lap is not known", async () => {
        mockBackend(undefined, undefined, undefined, { get_race_card: lined });
        render(<App />);
        await ready();
        expect(screen.queryByRole("img", { name: "コース図" })).toBeNull();
      });

      it("moves the dots as the race goes on and names the corner", async () => {
        const going = { ...race, frames: [[100, 90], [1000, 900], [1500, 1400], [2400, 2399]] };
        mockBackend(going, undefined, undefined, { get_race_card: withLap });
        render(<App />);
        await ready();
        vi.useFakeTimers();
        try {
          fireEvent.click(screen.getByText("レース開始"));
          await act(async () => {});
          const fast = () => dots().find((c) => c.getAttribute("data-horse") === "Fast")!;
          await tick(100);
          const early = fast().getAttribute("cx");
          await tick(200);
          expect(fast().getAttribute("cx")).not.toBe(early);
          expect(within(screen.getByRole("region", { name: "レース状況" })).getByText("第3コーナー")).toBeTruthy();
        } finally {
          vi.useRealTimers();
        }
      });
    });

    it("keeps showing where the race was after it has been run", async () => {
      await start();
      try {
        await tick(1000);
        expect(screen.getByLabelText("コース情報").textContent).toBe("東京・芝2400m・左回り・良");
      } finally {
        vi.useRealTimers();
      }
    });
  });

  describe("talking with the trainer", () => {
    const trainers = [{ id: 0, name: "鬼塚 剛志", training: "スパルタ", distance: "短距離", rotation: "連闘も辞さない", aim: "堅実に賞金", description: "叩きます。" }];
    const stable = { ...state, trainers, stable_capacity: 10, field: [{ ...state.field[0], trainer: 0 }, { ...state.field[1], trainer: 0 }] };
    const advice = [
      { horse: "Fast", comment: "疲れが見えます。", proposal: "放牧に出す" },
      { horse: "Slow", comment: "順調です。", proposal: null },
    ];

    it("shows what the trainer says and acts on a proposal when asked", async () => {
      const accept = vi.fn(() => stable);
      mockBackend(undefined, undefined, undefined, { get_game_state: stable, get_advice: advice, accept_advice: accept });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "厩舎" }));
      await screen.findByText("疲れが見えます。");
      expect(screen.getByText("順調です。")).toBeTruthy();
      expect(screen.queryByRole("button", { name: "Slowへの提案を受ける" })).toBeNull();
      fireEvent.click(screen.getByRole("button", { name: "Fastへの提案を受ける" }));
      await waitFor(() => expect(accept).toHaveBeenCalledWith({ horse: "Fast" }));
      expect(screen.getByRole("button", { name: "Fastへの提案を受ける" }).textContent).toBe("放牧に出す");
      await screen.findByText("Fast: 放牧に出す");
    });
  });

  describe("the entry rules", () => {
    it("says why a horse cannot run", async () => {
      const card = {
        ...freeCard,
        my_horses: [
          { name: "Fast", eligible: false, entered: false, barred: "賞金不足" },
          { name: "Slow", eligible: false, entered: false, barred: "抽選で除外" },
        ],
      };
      mockBackend(undefined, undefined, undefined, { get_race_card: card });
      render(<App />);
      await ready();
      expect(screen.getByText("(賞金不足)")).toBeTruthy();
      expect(screen.getByText("(抽選で除外)")).toBeTruthy();
    });

    it("says what a trial leads to", async () => {
      mockBackend(undefined, undefined, undefined, { get_race_card: { ...freeCard, trial_for: ["疾風賞"] } });
      render(<App />);
      await ready();
      expect(screen.getByText("トライアル: 3着までに「疾風賞」の優先出走権")).toBeTruthy();
    });
  });

  describe("orders and goals", () => {
    const trainers = [{ id: 0, name: "鬼塚 剛志", training: "スパルタ", distance: "短距離", rotation: "連闘も辞さない", aim: "堅実に賞金", description: "叩きます。" }];
    const stable = { ...state, trainers, stable_capacity: 10, field: [{ ...state.field[0], trainer: 0, order: "sashi" }, { ...state.field[1], trainer: 0 }] };

    it("lets an order be given to the jockey and taken back", async () => {
      const set = vi.fn(() => stable);
      mockBackend(undefined, undefined, undefined, { get_game_state: stable, set_order: set });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "厩舎" }));
      const fast = screen.getByLabelText("Fastの作戦") as HTMLSelectElement;
      expect(fast.value).toBe("sashi");
      expect(Array.from(fast.options).map((o) => o.text)).toEqual(["騎手におまかせ", "逃げ", "先行", "差し", "追込"]);
      fireEvent.change(fast, { target: { value: "nige" } });
      await waitFor(() => expect(set).toHaveBeenCalledWith({ horse: "Fast", order: "nige" }));
      fireEvent.change(fast, { target: { value: "" } });
      await waitFor(() => expect(set).toHaveBeenCalledWith({ horse: "Fast", order: null }));
    });

    it("shows the owner's standing and the goals", async () => {
      mockBackend(undefined, undefined, undefined, {
        get_game_state: {
          ...state,
          owner_rank: "駆け出し馬主",
          goals: [
            { name: "初勝利", description: "1勝する", done: true },
            { name: "G1初制覇", description: "G1を勝つ", done: false },
          ],
        },
      });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "実績" }));
      const region = screen.getByRole("region", { name: "目標" });
      expect(within(region).getByText("馬主の称号: 駆け出し馬主")).toBeTruthy();
      expect(within(region).getByText("目標 1/2 達成")).toBeTruthy();
      expect(within(region).getByText("✓ 初勝利")).toBeTruthy();
      expect(within(region).getByText("G1初制覇")).toBeTruthy();
    });
  });

  describe("the yearling sale", () => {
    const lot = { lot: 1, sire: "タイセイオー", dam: "ロウバイ", sex: "female", estimate: "A", start_price: 400, closed: false };
    const sale = { auction_open: true, auction_start_week: 27, auction_end_week: 30, week: 28 };
    const openSale = async (over: object = {}) => {
      mockBackend(undefined, undefined, undefined, {
        get_game_state: { ...state, ...sale },
        get_auction: [lot],
        bid_at_auction: { won: true, price: 620, state: { ...state, ...sale, money: 2380 } },
        ...over,
      });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "繁殖" }));
      return screen.findByRole("region", { name: "セリ" });
    };
    const fill = (max: string, name: string) => {
      fireEvent.change(screen.getByLabelText("上場1の上限額"), { target: { value: max } });
      fireEvent.change(screen.getByLabelText("上場1の馬名"), { target: { value: name } });
    };

    it("says when the sale is held and shows no lots out of season", async () => {
      mockBackend(undefined, undefined, undefined, {
        get_game_state: { ...state, auction_open: false, auction_start_week: 27, auction_end_week: 30, week: 5 },
      });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "繁殖" }));
      expect(screen.getByText("セリは7月(第27週〜第30週)に開かれます。今は第5週です。")).toBeTruthy();
      expect(vi.mocked(invoke).mock.calls.some((c) => c[0] === "get_auction")).toBe(false);
    });

    it("lists the lots with their pedigree and estimate", async () => {
      const region = await openSale();
      await within(region).findByText("父 タイセイオー・母 ロウバイ");
      expect(within(region).getByText("牝1歳")).toBeTruthy();
      expect(within(region).getByText("見込み A")).toBeTruthy();
      expect(within(region).getByText("開始 400")).toBeTruthy();
    });

    it("keeps the bid button off until the bid and name will do", async () => {
      const region = await openSale();
      const button = (await within(region).findByRole("button", { name: "上場1に入札" })) as HTMLButtonElement;
      expect(button.disabled).toBe(true);
      fill("300", "ハナミズキ");
      expect(button.disabled).toBe(true); // under the opening price
      fill("1000", "ハナミズキ");
      expect(button.disabled).toBe(true); // all the money there is
      fill("700", "bad");
      expect(button.disabled).toBe(true);
      fill("700", "ハナミズキ");
      expect(button.disabled).toBe(false);
    });

    it("bids and says what the horse went for", async () => {
      const region = await openSale();
      await within(region).findByRole("button", { name: "上場1に入札" });
      fill("700", "ハナミズキ");
      fireEvent.click(screen.getByRole("button", { name: "上場1に入札" }));
      await within(region).findByText("「ハナミズキ」を620で落札しました。");
      const call = vi.mocked(invoke).mock.calls.find((c) => c[0] === "bid_at_auction")!;
      expect(call[1]).toEqual({ lot: 1, maxBid: 700, name: "ハナミズキ" });
    });

    it("says when the bid was beaten", async () => {
      const region = await openSale({ bid_at_auction: { won: false, price: 900, state: { ...state, ...sale } } });
      await within(region).findByRole("button", { name: "上場1に入札" });
      fill("700", "ハナミズキ");
      fireEvent.click(screen.getByRole("button", { name: "上場1に入札" }));
      await within(region).findByText("競り負けました(落札額 900)。");
    });

    it("shows a closed lot without a bid button", async () => {
      const region = await openSale({ get_auction: [{ ...lot, closed: true }] });
      await within(region).findByText("終了");
      expect(within(region).queryByRole("button", { name: "上場1に入札" })).toBeNull();
    });
  });

  it("shows breeding errors from the backend", async () => {
    mockBackend(undefined, undefined, new Error("その馬名はすでに使われています"));
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "繁殖" }));
    fireEvent.change(screen.getByLabelText("仔馬の名前"), { target: { value: "ニューホープ" } });
    fireEvent.click(screen.getByText("配合 (500)"));
    await waitFor(() => expect(screen.getByText("その馬名はすでに使われています")).toBeTruthy());
  });

  it("disables breeding until the name follows the rule", async () => {
    mockBackend();
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "繁殖" }));
    const button = () => screen.getByText("配合 (500)") as HTMLButtonElement;
    expect(button().disabled).toBe(true);
    fireEvent.change(screen.getByLabelText("仔馬の名前"), { target: { value: "桜" } });
    expect(button().disabled).toBe(true);
    fireEvent.change(screen.getByLabelText("仔馬の名前"), { target: { value: "ニューホープ" } });
    expect(button().disabled).toBe(false);
  });

  it("disables breeding when the player cannot afford it", async () => {
    vi.mocked(invoke).mockImplementation(async () => ({ ...state, money: 100 }));
    render(<App />);
    await waitFor(() => screen.getByText("所持金: 100"));
    fireEvent.click(screen.getByRole("tab", { name: "繁殖" }));
    fireEvent.change(screen.getByLabelText("仔馬の名前"), { target: { value: "ニューホープ" } });
    expect((screen.getByText("配合 (500)") as HTMLButtonElement).disabled).toBe(true);
  });

  it("reveals commentary lines as the replay reaches them", async () => {
    mockBackend({
      results: [
        { rank: 1, name: "Fast", finish_time: 60.5 },
        { rank: 2, name: "Slow", finish_time: 70.25 },
      ],
      frames: [[600, 300], [1800, 1200], [2400, 2000]],
      distance: 2400,
      commentary: [
        { frame: 0, text: "各馬一斉にスタート!" },
        { frame: 1, text: "Slowが先頭に立った!" },
        { frame: 2, text: "Fast、1着でゴールイン!" },
      ],
      won: true,
      payout: 320,
      money: 1220,
    });
    render(<App />);
    await ready();
    expect(screen.queryByText("実況")).toBeNull();
    vi.useFakeTimers();
    try {
      fireEvent.click(screen.getByText("買い目に追加"));
      fireEvent.click(screen.getByText("レース開始"));
      await act(async () => {});
      expect(screen.getByText("実況")).toBeTruthy();
      expect(screen.getByText("各馬一斉にスタート!")).toBeTruthy(); // visible right at the start
      expect(screen.queryByText("Slowが先頭に立った!")).toBeNull();
      await act(async () => {
        vi.advanceTimersByTime(200);
      });
      expect(screen.getByText("Slowが先頭に立った!")).toBeTruthy();
      expect(screen.queryByText("Fast、1着でゴールイン!")).toBeNull();
      await act(async () => {
        vi.advanceTimersByTime(100);
      });
      expect(screen.getByText("Fast、1着でゴールイン!")).toBeTruthy();
      // commentary stays after the race ends
      expect(screen.getByText("的中! 配当 320")).toBeTruthy();
      expect(screen.getByText("各馬一斉にスタート!")).toBeTruthy();
    } finally {
      vi.useRealTimers();
    }
  });

  it("shows the whole commentary when the replay is skipped", async () => {
    mockBackend({
      results: [{ rank: 1, name: "Fast", finish_time: 60 }],
      frames: [[600, 300], [2400, 2000]],
      distance: 2400,
      commentary: [
        { frame: 0, text: "各馬一斉にスタート!" },
        { frame: 1, text: "Fast、1着でゴールイン!" },
      ],
      won: true,
      payout: 320,
      money: 1220,
    });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByText("レース開始"));
    fireEvent.click(await screen.findByText("スキップ"));
    await waitFor(() => expect(screen.getByText("Fast、1着でゴールイン!")).toBeTruthy());
  });

  it("lists the newest commentary first", async () => {
    mockBackend({
      results: [{ rank: 1, name: "Fast", finish_time: 60 }],
      frames: [[600, 300], [2400, 2000]],
      distance: 2400,
      commentary: [
        { frame: 0, text: "最初の実況" },
        { frame: 1, text: "最後の実況" },
      ],
      won: true,
      payout: 320,
      money: 1220,
    });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByText("レース開始"));
    fireEvent.click(await screen.findByText("スキップ"));
    await waitFor(() => screen.getByText("最後の実況"));
    const items = screen.getAllByRole("listitem").map((li) => li.textContent);
    expect(items.indexOf("最後の実況")).toBeLessThan(items.indexOf("最初の実況"));
  });

  it("loads the calendar once and reuses it for the calendar tab", async () => {
    mockBackend();
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "カレンダー" }));
    await waitFor(() => expect(screen.getByLabelText("週")).toBeTruthy());
    expect(vi.mocked(invoke).mock.calls.filter((c) => c[0] === "get_calendar").length).toBe(1);
    expect(screen.queryByLabelText("賭け金")).toBeNull();
  });

  // ---- calendar races: weeks, race choice, fields with rivals ----

  const g3 = {
    week: 1, day: "sat", race_no: 11, venue: "kyoto", name: "初春マイル賞", grade: "g3",
    surface: "turf", distance: 1600,
    eligibility: { age: "three_up", fillies_only: false, weight: "handicap" }, prize: 4300,
  };
  const week2Race = { ...g3, week: 2, race_no: 10, name: "冬芽賞", venue: "nakayama" };
  const calendarData = { meetings: [], races: [g3, week2Race] };
  const rival = (name: string, odds: number, power = 60, bad_track = 60) => ({
    name, speed: 80, stamina: 75, power, bad_track, condition: "normal", sex: "female", age: 4, dirt: 0, growth: "peak", age_form: 1, apt_min: 1600, apt_max: 2400, distance_fit: "fit", style: "senko", temper: 55,
    mine: false, odds, place_odds: 1.8,
  });
  const card = {
    race: g3,
    distance: 1600,
    surface: "turf",
    track: "yielding",
    max_entries: 3,
    first_prize: 86,
    my_horses: [
      { name: "Fast", eligible: true, entered: true },
      { name: "Slow", eligible: false, entered: false },
    ],
    entrants: [
      { ...rival("Fast", 3.2, 85, 82), mine: true, sex: "male", age: 5, condition: "great" },
      rival("ライバルA", 2.4, 70, 62.5),
      rival("ライバルB", 9.9, 40, 20),
    ],
  };
  const raceOutcome = {
    results: [
      { rank: 1, name: "ライバルA", finish_time: 90.1 },
      { rank: 2, name: "Fast", finish_time: 90.5 },
      { rank: 3, name: "ライバルB", finish_time: 91 },
    ],
    frames: [[2300, 2400, 2000]],
    commentary: [],
    runners: ["Fast", "ライバルA", "ライバルB"],
    race_name: "初春マイル賞",
    distance: 1600,
    won: true,
    payout: 240,
    money: 1140,
  };

  const calendarBackend = (bet?: unknown, extra: Record<string, unknown> = {}) =>
    mockBackend(bet, undefined, undefined, { get_calendar: calendarData, get_race_card: cardFor(card), ...extra });

  describe("how many horses a race takes", () => {
    const chooseG3 = async () => {
      render(<App />);
      await ready();
      await waitFor(() => expect((screen.getByLabelText("レース") as HTMLSelectElement).options.length).toBe(2));
      fireEvent.change(screen.getByLabelText("レース"), { target: { value: "sat:kyoto:11" } });
      await screen.findByText("ライバルA", { selector: "td.name" });
    };

    it("says how many are put forward and how many can run", async () => {
      calendarBackend(undefined, { get_race_card: cardFor({ ...card, nominated: 12, gate: 18 }) });
      await chooseG3();
      expect(screen.getByText("登録12頭 / フルゲート18頭")).toBeTruthy();
      expect(screen.queryByText(/抽選で除外されます/)).toBeNull();
    });

    it("warns of the draw when more are put forward than can run", async () => {
      calendarBackend(undefined, { get_race_card: cardFor({ ...card, nominated: 21, gate: 16 }) });
      await chooseG3();
      expect(screen.getByText("登録21頭 / フルゲート16頭")).toBeTruthy();
      expect(screen.getByText(/5頭多いので、優先出走権のない馬は抽選で除外されることがあります/)).toBeTruthy();
    });

    it("says nothing of the numbers for the free race", async () => {
      mockBackend(undefined, undefined, undefined, { get_race_card: { ...freeCard, nominated: 18, gate: 18 } });
      render(<App />);
      await ready();
      expect(screen.queryByText(/フルゲート/)).toBeNull();
    });

    it("shows a horse that lost the draw as still registered but not running", async () => {
      const lost = {
        ...card,
        nominated: 21,
        gate: 16,
        my_horses: [
          { name: "Fast", eligible: true, entered: false, registered: true, drawn_out: true },
          { name: "Slow", eligible: true, entered: false, registered: false, drawn_out: false },
        ],
      };
      calendarBackend(undefined, { get_race_card: cardFor(lost) });
      await chooseG3();
      expect((screen.getByLabelText("出走登録: Fast") as HTMLInputElement).checked).toBe(true);
      expect((screen.getByLabelText("出走登録: Slow") as HTMLInputElement).checked).toBe(false);
      expect(screen.getByText("(抽選で除外)")).toBeTruthy();
    });

    it("keeps a horse that lost the draw on the list when another is added", async () => {
      const lost = {
        ...card,
        my_horses: [
          { name: "Fast", eligible: true, entered: false, registered: true, drawn_out: true },
          { name: "Slow", eligible: true, entered: false, registered: false, drawn_out: false },
        ],
      };
      calendarBackend(undefined, { get_race_card: cardFor(lost) });
      await chooseG3();
      fireEvent.click(screen.getByLabelText("出走登録: Slow"));
      await waitFor(() =>
        expect(vi.mocked(invoke).mock.calls.find((c) => c[0] === "set_entries")?.[1]).toEqual({
          race: { day: "sat", venue: "kyoto", race_no: 11 },
          horses: ["Fast", "Slow"],
        })
      );
    });

    it("counts the horses that lost the draw towards the limit", async () => {
      const lost = {
        ...card,
        max_entries: 1,
        my_horses: [
          { name: "Fast", eligible: true, entered: false, registered: true, drawn_out: true },
          { name: "Slow", eligible: true, entered: false, registered: false, drawn_out: false },
        ],
      };
      calendarBackend(undefined, { get_race_card: cardFor(lost) });
      await chooseG3();
      expect((screen.getByLabelText("出走登録: Slow") as HTMLInputElement).disabled).toBe(true);
    });

    it("says a small field pays a place ticket on the first two only", async () => {
      calendarBackend(undefined, { get_race_card: cardFor({ ...card, place_slots: 2 }) });
      await chooseG3();
      expect(screen.getByRole("option", { name: "複勝(2着以内)" })).toBeTruthy();
      expect(screen.queryByRole("option", { name: "複勝(3着以内)" })).toBeNull();
    });
  });

  describe("east and west", () => {
    const eastAndWest = [
      { id: 0, name: "鬼塚 剛志", region: "美浦", training: "スパルタ", distance: "短距離", rotation: "連闘も辞さない", aim: "堅実に賞金", description: "叩きます。" },
      { id: 1, name: "大河内 鉄平", region: "栗東", training: "スパルタ", distance: "長距離", rotation: "標準", aim: "G1を目指す", description: "鍛えます。" },
    ];
    const riders = [
      { id: 0, name: "神崎 颯", region: "美浦", skill: 92, style: "sashi", turn: "left", tact: 8, fee: 20, description: "名手です。" },
      { id: 1, name: "獅童 蓮", region: "栗東", skill: 88, style: "nige", turn: "right", tact: 4, fee: 18, description: "勝負師です。" },
    ];
    const staffedGame = {
      ...state,
      trainers: eastAndWest,
      jockeys: riders,
      field: [{ ...state.field[0], trainer: 0 }, { ...state.field[1], trainer: 1 }],
    };
    const open = async (tab: string) => {
      mockBackend(undefined, undefined, undefined, { get_game_state: staffedGame, get_advice: [], get_courses: [], get_course_fits: [] });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: tab }));
    };

    it("groups the stables by where they are", async () => {
      await open("厩舎");
      const east = within(await screen.findByRole("region", { name: "美浦の厩舎" }));
      const west = within(screen.getByRole("region", { name: "栗東の厩舎" }));
      expect(east.getByRole("region", { name: "鬼塚 剛志厩舎" })).toBeTruthy();
      expect(east.queryByRole("region", { name: "大河内 鉄平厩舎" })).toBeNull();
      expect(west.getByRole("region", { name: "大河内 鉄平厩舎" })).toBeTruthy();
    });

    it("offers jockeys and stables by side when choosing", async () => {
      await open("厩舎");
      const groupsOf = (label: string) =>
        Array.from(screen.getByLabelText(label).querySelectorAll("optgroup")).map((g) => [
          g.label,
          Array.from(g.querySelectorAll("option")).map((o) => o.textContent),
        ]);
      await screen.findByLabelText("Fastの騎手");
      expect(groupsOf("Fastの騎手")).toEqual([
        ["美浦", ["神崎 颯(騎乗料 20)"]],
        ["栗東", ["獅童 蓮(騎乗料 18)"]],
      ]);
      expect(groupsOf("Fastの厩舎")).toEqual([
        ["美浦", ["鬼塚 剛志"]],
        ["栗東", ["大河内 鉄平"]],
      ]);
    });

    it("groups the jockeys by side on the information tab", async () => {
      await open("情報");
      const east = within(await screen.findByRole("region", { name: "美浦の騎手" }));
      const west = within(screen.getByRole("region", { name: "栗東の騎手" }));
      expect(east.getByText("神崎 颯")).toBeTruthy();
      expect(west.getByText("獅童 蓮")).toBeTruthy();
      expect(east.queryByText("獅童 蓮")).toBeNull();
    });

    it("says on the honours tab which side each trainer is on", async () => {
      await open("実績");
      const row = (await screen.findByRole("cell", { name: "大河内 鉄平" })).closest("tr")!;
      expect(within(row).getByText("栗東")).toBeTruthy();
    });
  });

  it("shows the current week and year", async () => {
    calendarBackend();
    render(<App />);
    await ready();
    expect(screen.getByText("第1週(1月)・1年目")).toBeTruthy();
    expect(screen.getByText("次の週へ")).toBeTruthy();
  });

  it("offers the free race and only this week's calendar races", async () => {
    calendarBackend();
    render(<App />);
    await ready();
    const options = await waitFor(() => {
      const o = Array.from((screen.getByLabelText("レース") as HTMLSelectElement).options).map((x) => x.textContent);
      expect(o.length).toBe(2);
      return o;
    });
    expect(options[0]).toBe("フリーレース(練習・芝2400m)");
    expect(options[1]).toBe("11R G3 初春マイル賞(芝1600m)");
    const group = (screen.getByLabelText("レース") as HTMLSelectElement).querySelector("optgroup")!;
    expect(group.label).toBe("土 京都");
    expect(options.join()).not.toContain("冬芽賞");
  });

  it("loads the field with rivals when a calendar race is chosen", async () => {
    calendarBackend();
    render(<App />);
    await ready();
    await waitFor(() => expect((screen.getByLabelText("レース") as HTMLSelectElement).options.length).toBe(2));
    fireEvent.change(screen.getByLabelText("レース"), { target: { value: "sat:kyoto:11" } });
    await screen.findByText("ライバルA", { selector: "td.name" });
    const call = vi
      .mocked(invoke)
      .mock.calls.find((c) => c[0] === "get_race_card" && (c[1] as { race?: unknown }).race)!;
    expect(call[1]).toEqual({ race: { day: "sat", venue: "kyoto", race_no: 11 } });
    expect(screen.getByText("京都 芝1600m")).toBeTruthy();
    expect(screen.getByText("3歳以上・ハンデ")).toBeTruthy();
    expect(screen.getByText("4,300万円")).toBeTruthy();
    // only the stable's horse is marked as mine
    expect(screen.getAllByLabelText("自馬").length).toBe(1);
    expect(within(screen.getByText("Fast", { selector: "td.name" }).closest("tr")!).getByLabelText("自馬")).toBeTruthy();
    expect(screen.getByText("x9.9")).toBeTruthy();
    // rivals can be bet on
    const horses = Array.from((screen.getByLabelText("馬") as HTMLSelectElement).options).map((o) => o.value);
    expect(horses).toEqual(["Fast", "ライバルA", "ライバルB"]);
  });

  it("bets on a rival in a calendar race, replays it, then moves to the next week", async () => {
    let stateCalls = 0;
    calendarBackend(raceOutcome, {
      get_game_state: () => (++stateCalls === 1 ? state : { ...state, week: 2, money: 1140, round: 1 }),
    });
    render(<App />);
    await ready();
    await waitFor(() => expect((screen.getByLabelText("レース") as HTMLSelectElement).options.length).toBe(2));
    fireEvent.change(screen.getByLabelText("レース"), { target: { value: "sat:kyoto:11" } });
    await screen.findByText("ライバルA", { selector: "td.name" });
    fireEvent.change(screen.getByLabelText("馬"), { target: { value: "ライバルA" } });
    fireEvent.click(screen.getByText("買い目に追加"));
    fireEvent.click(screen.getByText("レース開始"));
    fireEvent.click(await screen.findByText("スキップ"));
    const bet = vi.mocked(invoke).mock.calls.find((c) => c[0] === "place_bets")!;
    expect(bet[1]).toMatchObject({
      tickets: [{ horses: ["ライバルA"], kind: "win" }],
      race: { day: "sat", venue: "kyoto", race_no: 11 },
    });
    await waitFor(() => expect(screen.getByText("第2週(1月)・1年目")).toBeTruthy());
    expect(screen.getByText("所持金: 1140")).toBeTruthy();
    expect(screen.getByText("的中! 配当 240")).toBeTruthy();
    // the lanes show the race's runners, not the stable
    expect(screen.getByLabelText("ライバルB")).toBeTruthy();
    // and the race selection goes back to the free race for the new week
    expect((screen.getByLabelText("レース") as HTMLSelectElement).value).toBe("free");
  });

  it("sends no race for the free race", async () => {
    calendarBackend({ ...raceOutcome, race_name: null, runners: undefined });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByText("買い目に追加"));
    fireEvent.click(screen.getByText("レース開始"));
    await screen.findByText("スキップ");
    const bet = vi.mocked(invoke).mock.calls.find((c) => c[0] === "place_bets")!;
    expect((bet[1] as { race?: unknown }).race ?? null).toBeNull();
  });

  it("skips to the next week without racing", async () => {
    calendarBackend(undefined, { next_week: { ...state, week: 2 } });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByText("次の週へ"));
    await waitFor(() => expect(screen.getByText("第2週(1月)・1年目")).toBeTruthy());
    expect(vi.mocked(invoke).mock.calls.some((c) => c[0] === "next_week")).toBe(true);
  });

  describe("after a race has been run", () => {
    const courses = {
      get_race_card: (args: unknown) =>
        (args as { race?: unknown }).race
          ? { ...freeCard, distance: 1600, course: { name: "京都", turn: "右回り", straight: 400, lap: 1894, after_goal: 93, corners: 2 } }
          : { ...freeCard, course: { name: "東京", turn: "左回り", straight: 525, lap: 2083, after_goal: 33, corners: 4 } },
    };
    const runFreeRace = async (extra: Record<string, unknown> = {}) => {
      calendarBackend({ ...raceOutcome, race_name: null }, { ...courses, ...extra });
      render(<App />);
      await ready();
      await waitFor(() => expect((screen.getByLabelText("レース") as HTMLSelectElement).options.length).toBe(2));
      fireEvent.click(screen.getByText("レース開始"));
      fireEvent.click(await screen.findByText("スキップ"));
      await screen.findByText("結果");
    };

    it("shows the course of the race chosen next, not of the one that was run", async () => {
      await runFreeRace();
      expect(screen.getByLabelText("コース情報").textContent).toContain("東京");
      fireEvent.change(screen.getByLabelText("レース"), { target: { value: "sat:kyoto:11" } });
      await waitFor(() => expect(screen.getByLabelText("コース情報").textContent).toContain("京都"));
      expect(screen.queryByText("結果")).toBeNull();
    });

    it("clears the race that was run when the week moves on", async () => {
      await runFreeRace({ next_week: { ...state, week: 2 } });
      fireEvent.click(screen.getByText("次の週へ"));
      await screen.findByText("第2週(1月)・1年目");
      expect(screen.queryByText("結果")).toBeNull();
    });
  });

  describe("where messages appear", () => {
    const followed = (first: Element, then: Element) =>
      (first.compareDocumentPosition(then) & Node.DOCUMENT_POSITION_FOLLOWING) !== 0;

    it("keeps the week bar still by showing them below it", async () => {
      calendarBackend(undefined, { next_week: { ...state, week: 2, news: ["お知らせです。"] } });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByText("次の週へ"));
      const news = await screen.findByText("お知らせです。");
      expect(followed(screen.getByText("次の週へ"), news)).toBe(true);
    });

    it("shows them below the tabs on the other tabs", async () => {
      calendarBackend(undefined, { save_game: undefined });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "育成" }));
      fireEvent.click(screen.getByText("セーブ"));
      const saved = await screen.findByText("セーブしました。");
      expect(followed(screen.getByRole("tablist"), saved)).toBe(true);
    });
  });

  it("cannot change race or skip the week during a replay", async () => {
    calendarBackend({ ...raceOutcome, race_name: null });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByText("レース開始"));
    await screen.findByText("スキップ");
    expect((screen.getByText("次の週へ") as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByLabelText("レース") as HTMLSelectElement).disabled).toBe(true);
  });

  it("names the race in the history", async () => {
    mockBackend();
    render(<App />);
    await ready();
    vi.mocked(invoke).mockResolvedValueOnce({
      ...state,
      round: 1,
      history: [
        { kind: "win", race: "初春マイル賞", round: 1, horse: "Fast", stake: 100, won: true, payout: 320, winner: "Fast" },
      ],
    });
    fireEvent.click(screen.getByText("ロード"));
    await waitFor(() =>
      expect(screen.getByText("第1R 初春マイル賞 単勝 Fast 100 → 的中 +220 (優勝: Fast)")).toBeTruthy()
    );
  });

  it("shows age and sex, and dirt aptitude, for each runner", async () => {
    calendarBackend();
    render(<App />);
    await ready();
    await waitFor(() => expect((screen.getByLabelText("レース") as HTMLSelectElement).options.length).toBe(2));
    fireEvent.change(screen.getByLabelText("レース"), { target: { value: "sat:kyoto:11" } });
    await screen.findByText("ライバルA", { selector: "td.name" });
    expect(screen.getByText("牡5")).toBeTruthy();
    expect(screen.getAllByText("牝4").length).toBe(2);
    expect(screen.getAllByLabelText("ダート適性: 普通").length).toBe(3);
  });

  it("switches the pedigree tab between the stable and the pedigree database", async () => {
    mockBackend();
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_game_state") return state;
      if (cmd === "get_calendar") return { meetings: [], races: [] };
      if (cmd === "get_bloodline")
        return [
          {
            id: 1, name: "イグニス", sire_id: null, dam_id: null, gender: "male", lineage: "Ignis",
            family: "Sire", gen: 1, coat_color: "鹿毛", genes: { G: "gg", E: "EE", A: "AA" },
            growth_factor: 0.5, growth_type: "普通", peak_age: 4, aptitude_min: 1600, aptitude_max: 2400,
            running_style: "逃げ", personality: "狂気", inbreed_vol: 0,
            stats: { speed: 85, stamina: 60, temper: 30, power: 60, turf: 60, dirt: 60 },
          },
        ];
      throw new Error("unexpected " + cmd);
    });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "血統" }));
    // the stable's pedigrees come first, and the database is not loaded yet
    expect(screen.getByText("血統不明")).toBeTruthy();
    expect(vi.mocked(invoke).mock.calls.some((c) => c[0] === "get_bloodline")).toBe(false);
    fireEvent.click(screen.getByRole("button", { name: "血統データベース" }));
    await screen.findByText("全1頭");
    expect(screen.queryByText("血統不明")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "厩舎の馬" }));
    expect(screen.getByText("血統不明")).toBeTruthy();
  });

  // ---- track condition (馬場状態), power and bad-track aptitude ----

  const chooseRace = async () => {
    await waitFor(() => expect((screen.getByLabelText("レース") as HTMLSelectElement).options.length).toBe(2));
    fireEvent.change(screen.getByLabelText("レース"), { target: { value: "sat:kyoto:11" } });
    await screen.findByText("ライバルA", { selector: "td.name" });
  };

  it("shows the track condition of a calendar race and what it means", async () => {
    calendarBackend();
    render(<App />);
    await ready();
    await chooseRace();
    expect(screen.getByLabelText("馬場状態: 重").textContent).toBe("重");
    expect(screen.getByText("道悪適性の高い馬が有利です。")).toBeTruthy();
  });

  it("says nothing special about a firm track", async () => {
    calendarBackend(undefined, { get_race_card: cardFor({ ...card, track: "good" }) });
    render(<App />);
    await ready();
    await chooseRace();
    expect(screen.getByLabelText("馬場状態: 良").textContent).toBe("良");
    expect(screen.queryByText("道悪適性の高い馬が有利です。")).toBeNull();
  });

  it("shows a slightly soft track in its own words", async () => {
    calendarBackend(undefined, { get_race_card: cardFor({ ...card, track: "soft" }) });
    render(<App />);
    await ready();
    await chooseRace();
    expect(screen.getByLabelText("馬場状態: 稍重").textContent).toBe("稍重");
  });

  it("shows a heavy track in its own words", async () => {
    calendarBackend(undefined, { get_race_card: cardFor({ ...card, track: "heavy" }) });
    render(<App />);
    await ready();
    await chooseRace();
    expect(screen.getByLabelText("馬場状態: 不良").textContent).toBe("不良");
  });

  it("has no track badge for the free race", async () => {
    calendarBackend();
    render(<App />);
    await ready();
    expect(screen.queryByLabelText(/馬場状態/)).toBeNull();
  });

  it("shows power and bad-track aptitude for every runner of a calendar race", async () => {
    calendarBackend();
    render(<App />);
    await ready();
    await chooseRace();
    const fast = screen.getByText("Fast", { selector: "td.name" }).closest("tr")!;
    expect(within(fast).getByText("85")).toBeTruthy(); // power
    expect(within(fast).getByLabelText("道悪適性: 82").textContent).toBe("◎");
    expect(screen.getByLabelText("道悪適性: 62.5").textContent).toBe("○");
    expect(screen.getByLabelText("道悪適性: 20").textContent).toBe("×");
    expect(screen.getAllByLabelText(/^道悪適性: /).length).toBe(3);
    expect(screen.getByText("パワー", { selector: "th" })).toBeTruthy();
    expect(screen.getByText("道悪", { selector: "th" })).toBeTruthy();
  });

  it("shows growth stage and distance fit for every runner of a calendar race", async () => {
    calendarBackend();
    render(<App />);
    await ready();
    await chooseRace();
    expect(screen.getByText("成長", { selector: "th" })).toBeTruthy();
    expect(screen.getByText("距離", { selector: "th" })).toBeTruthy();
    const marks = screen.getAllByLabelText(/^成長期: /);
    expect(marks.length).toBe(3);
    expect(screen.getAllByLabelText(/^距離適性: /).length).toBe(3);
  });

  it("shows running style and temperament for every runner of a calendar race", async () => {
    calendarBackend();
    render(<App />);
    await ready();
    await chooseRace();
    expect(screen.getByText("脚質", { selector: "th" })).toBeTruthy();
    expect(screen.getByText("気性", { selector: "th" })).toBeTruthy();
    expect(screen.getAllByLabelText("脚質: 先行").length).toBe(3);
    expect(screen.getAllByLabelText("気性: 55").length).toBe(3);
  });

  it("shows power and bad-track aptitude for the stable in the free race", async () => {
    calendarBackend();
    render(<App />);
    await ready();
    expect(screen.getByLabelText("道悪適性: 70").textContent).toBe("○");
    expect(screen.getByLabelText("道悪適性: 47").textContent).toBe("△");
  });

  it("trains power from the training tab", async () => {
    const trained = {
      ...state,
      money: 900,
      field: [{ ...state.field[0], power: 78, bad_track: 72.1, train_cost: 150 }, state.field[1]],
    };
    mockBackend(undefined, trained);
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "育成" }));
    expect(screen.getByText("パワー", { selector: "th" })).toBeTruthy();
    workout("Fast", "hill");
    await waitFor(() => expect(screen.getByText("所持金: 900")).toBeTruthy());
    expect(screen.getByText("78")).toBeTruthy();
    const call = vi.mocked(invoke).mock.calls.find((c) => c[0] === "train_horse")!;
    expect(call[1]).toEqual({ horse: "Fast", workout: "hill" });
  });

  it("keeps each horse's chosen menu separately", async () => {
    mockBackend();
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "育成" }));
    fireEvent.change(screen.getByLabelText("Fastの調教メニュー"), { target: { value: "walk" } });
    expect((screen.getByLabelText("Fastの調教メニュー") as HTMLSelectElement).value).toBe("walk");
    expect((screen.getByLabelText("Slowの調教メニュー") as HTMLSelectElement).value).toBe("wood");
    // what a menu does is explained once above the table, not on every row
    const fast = within(screen.getByRole("tabpanel", { name: "育成" })).getByText("Fast").closest("tr")!;
    expect(within(fast).queryByText("気性+3")).toBeNull();
  });

  it("offers every training menu for every horse, explained once above the table", async () => {
    mockBackend();
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "育成" }));
    const menus = Array.from((screen.getByLabelText("Slowの調教メニュー") as HTMLSelectElement).options);
    expect(menus.map((o) => o.textContent)).toEqual(["ウッドコース追い", "坂路", "プール", "併せ馬", "曳き運動", "ダートコース"]);
    expect(menus.map((o) => o.value)).toEqual(["wood", "hill", "pool", "pair", "walk", "dirt"]);
    expect((screen.getByLabelText("Slowの調教メニュー") as HTMLSelectElement).value).toBe("wood");
    expect(screen.getByLabelText("Slowを調教").textContent).toBe("調教");
    const menu = screen.getByRole("list", { name: "調教メニュー" });
    expect(within(menu).getByText("併せ馬")).toBeTruthy();
    expect(within(menu).getByText("スピード+2 パワー+2 気性-2 疲労+12")).toBeTruthy();
    expect(within(menu).getByText("ダート適性+1/3 パワー+1")).toBeTruthy();
    expect(within(menu).getByText("気が立ちます。")).toBeTruthy();
  });

  it("shows temperament and dirt progress and a coat icon instead of a gate number when training", async () => {
    const horses = {
      ...state,
      field: [
        { ...state.field[0], temper: 42, dirt: 1, dirt_progress: 2, blood: { coat_color: "栗毛" } },
        state.field[1],
      ],
    };
    mockBackend(undefined, undefined, undefined, { get_game_state: horses });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "育成" }));
    const train = screen.getByRole("tabpanel", { name: "育成" });
    expect(within(train).queryByLabelText(/の枠番/)).toBeNull();
    expect(within(train).getByLabelText("Fastの毛色: 栗毛")).toBeTruthy();
    expect(within(train).getByLabelText("Slowの毛色: 鹿毛")).toBeTruthy(); // no record: a bay
    expect(within(train).getByText("気性", { selector: "th" })).toBeTruthy();
    const fast = within(train).getByText("Fast").closest("tr")!;
    expect(within(fast).getByText("42")).toBeTruthy();
    expect(within(fast).getByLabelText("ダート適性: 得意(次の段階まで 2/3)")).toBeTruthy();
  });

  it("shows each horse's fatigue on the training tab", async () => {
    const horses = { ...state, field: [{ ...state.field[0], fatigue: 62 }, { ...state.field[1], fatigue: 0 }] };
    mockBackend(undefined, undefined, undefined, { get_game_state: horses });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "育成" }));
    const train = screen.getByRole("tabpanel", { name: "育成" });
    expect(within(train).getByText("疲労", { selector: "th" })).toBeTruthy();
    const meter = within(train).getByRole("meter", { name: "Fastの疲労度" });
    expect(meter.getAttribute("aria-valuenow")).toBe("62");
    expect(meter.className).toContain("tired"); // too tired to race
    expect(within(train).getByRole("meter", { name: "Slowの疲労度" }).getAttribute("aria-valuenow")).toBe("0");
  });

  it("rests a horse for the week", async () => {
    const rested = { ...state, field: [{ ...state.field[0], fatigue: 20, resting: true }, state.field[1]] };
    const rest = vi.fn(() => rested);
    mockBackend(undefined, undefined, undefined, { rest_horse: rest });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "育成" }));
    fireEvent.click(screen.getByLabelText("Fastを休養させる"));
    await waitFor(() => expect(rest).toHaveBeenCalledWith({ horse: "Fast" }));
    await screen.findByText("Fastを今週は休養させます。");
    const fast = within(screen.getByRole("tabpanel", { name: "育成" })).getByText("Fast").closest("tr")!;
    expect(within(fast).getByText("休養中")).toBeTruthy();
    expect((within(fast).getByLabelText("Fastを休養させる") as HTMLButtonElement).disabled).toBe(true);
    expect((within(fast).getByLabelText("Fastを調教") as HTMLButtonElement).disabled).toBe(true);
  });

  it("cannot work a horse that is too tired", async () => {
    const horses = { ...state, field: [{ ...state.field[0], fatigue: 90 }, state.field[1]] };
    mockBackend(undefined, undefined, undefined, { get_game_state: horses });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "育成" }));
    expect((screen.getByLabelText("Fastを調教") as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByLabelText("Slowを調教") as HTMLButtonElement).disabled).toBe(false);
  });

  it("leaves a horse's training to the trainer and takes it back", async () => {
    const left = { ...state, auto_keep_money: 500, field: [{ ...state.field[0], auto_training: true }, state.field[1]] };
    const setAuto = vi.fn((a: { on: boolean }) => (a.on ? left : { ...state, auto_keep_money: 500 }));
    mockBackend(undefined, undefined, undefined, {
      get_game_state: { ...state, auto_keep_money: 500 },
      set_auto_training: setAuto,
    });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "育成" }));
    const box = () => screen.getByLabelText("Fastをお任せ調教") as HTMLInputElement;
    expect(box().checked).toBe(false);
    expect(screen.getByText(/所持金を500残して/)).toBeTruthy();
    fireEvent.click(box());
    await waitFor(() => expect(setAuto).toHaveBeenCalledWith({ horse: "Fast", on: true }));
    await waitFor(() => expect(box().checked).toBe(true));
    // the horse can still be worked by hand
    expect((screen.getByLabelText("Fastを調教") as HTMLButtonElement).disabled).toBe(false);
    fireEvent.click(box());
    await waitFor(() => expect(setAuto).toHaveBeenCalledWith({ horse: "Fast", on: false }));
    await waitFor(() => expect(box().checked).toBe(false));
  });

  it("reports what the trainer did when the week moves on", async () => {
    const turned = { ...state, week: 2, news: ["Fastはお任せで坂路をしました。"] };
    mockBackend(undefined, undefined, undefined, { next_week: turned });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByText("次の週へ"));
    await screen.findByText("Fastはお任せで坂路をしました。");
  });

  it("enters a horse automatically from the training tab", async () => {
    const on = { ...state, field: state.field.map((h) => (h.name === "Fast" ? { ...h, auto_entry: true } : h)) };
    const setEntry = vi.fn((a: { on: boolean }) => (a.on ? on : state));
    mockBackend(undefined, undefined, undefined, { set_auto_entry: setEntry });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "育成" }));
    const box = () => screen.getByLabelText("Fastを自動出走") as HTMLInputElement;
    expect(box().checked).toBe(false);
    fireEvent.click(box());
    await waitFor(() => expect(setEntry).toHaveBeenCalledWith({ horse: "Fast", on: true }));
    await waitFor(() => expect(box().checked).toBe(true));
    fireEvent.click(box());
    await waitFor(() => expect(setEntry).toHaveBeenCalledWith({ horse: "Fast", on: false }));
    await waitFor(() => expect(box().checked).toBe(false));
  });

  it("writes an automatic run in the results as a run, not a bet", async () => {
    const ran = {
      kind: "win",
      race: "未勝利",
      round: 1,
      horse: "Fast 1着",
      stake: 0,
      won: true,
      payout: 0,
      prize: 11,
      ticket_count: 0,
      winner: "Fast",
    };
    mockBackend(undefined, undefined, undefined, { get_game_state: { ...state, history: [ran] } });
    render(<App />);
    await ready();
    expect(screen.getByText("第1R 未勝利 自動出走 Fast 1着 賞金+11 (優勝: Fast)")).toBeTruthy();
    expect(screen.getByText("収支: +11")).toBeTruthy();
  });

  describe("leaving the whole stable to its trainers", () => {
    const withAuto = (fn: (i: number) => boolean) => ({
      ...state,
      field: state.field.map((h, i) => ({ ...h, auto_training: fn(i), auto_entry: fn(i) })),
    });

    it("turns everything on from any tab", async () => {
      const set = vi.fn(() => withAuto(() => true));
      mockBackend(undefined, undefined, undefined, { get_game_state: withAuto(() => false), set_all_auto: set });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "実績" }));
      fireEvent.click(screen.getByRole("button", { name: "全頭を自動にする" }));
      await waitFor(() => expect(set).toHaveBeenCalledWith({ on: true }));
      await screen.findByRole("button", { name: "全頭の自動を解除" });
      expect(screen.queryByRole("button", { name: "全頭を自動にする" })).toBeNull();
    });

    it("turns everything off again", async () => {
      const set = vi.fn(() => withAuto(() => false));
      mockBackend(undefined, undefined, undefined, { get_game_state: withAuto(() => true), set_all_auto: set });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("button", { name: "全頭の自動を解除" }));
      await waitFor(() => expect(set).toHaveBeenCalledWith({ on: false }));
      await screen.findByRole("button", { name: "全頭を自動にする" });
    });

    it("offers both when only some horses are automatic", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: withAuto((i) => i === 0) });
      render(<App />);
      await ready();
      expect(screen.getByRole("button", { name: "全頭を自動にする" })).toBeTruthy();
      expect(screen.getByRole("button", { name: "全頭の自動を解除" })).toBeTruthy();
    });

    it("can be used while autoplay runs", async () => {
      let week = 1;
      const nextWeek = vi.fn(() => ({ ...withAuto(() => false), week: ++week }));
      mockBackend(undefined, undefined, undefined, { get_game_state: withAuto(() => false), next_week: nextWeek });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("button", { name: "オートプレイ開始" }));
      expect((screen.getByRole("button", { name: "全頭を自動にする" }) as HTMLButtonElement).disabled).toBe(false);
      fireEvent.click(screen.getByRole("button", { name: "オートプレイ停止" }));
    });

    it("reports a failure", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: withAuto(() => false), set_all_auto: new Error("だめです") });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("button", { name: "全頭を自動にする" }));
      await screen.findByText("だめです");
    });
  });

  describe("the layout of the training tab", () => {
    const open = async () => {
      mockBackend(undefined, undefined, undefined, {
        get_game_state: {
          ...state,
          trainers: [{ id: 0, name: "鬼塚 剛志", training: "スパルタ", distance: "短距離", rotation: "連闘も辞さない", aim: "堅実に賞金", description: "叩きます。" }],
          field: [{ ...state.field[0], trainer: 0, age: 4, record: { starts: 3, wins: 1 } }, state.field[1]],
        },
      });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "育成" }));
    };
    const panel = () => within(screen.getByRole("tabpanel", { name: "育成" }));

    it("keeps the table few enough columns to fit the window", async () => {
      await open();
      const heads = within(panel().getByRole("table")).getAllByRole("columnheader").map((h) => h.textContent);
      expect(heads).toEqual(["馬", "スピード", "スタミナ", "パワー", "気性", "ダート", "脚元", "疲労", "調子", "調教"]);
    });

    it("shows age, stable and record under the horse's name", async () => {
      await open();
      const row = panel().getByText("Fast").closest("tr")!;
      const who = row.querySelector("td")!;
      expect(within(who).getByText("牡4")).toBeTruthy();
      expect(within(who).getByText("鬼塚 剛志")).toBeTruthy();
      expect(within(who).getByText(/3戦1勝/)).toBeTruthy();
    });

    it("gathers every control for a horse in the last column", async () => {
      await open();
      const row = panel().getByText("Fast").closest("tr")!;
      const cells = row.querySelectorAll("td");
      const last = within(cells[cells.length - 1]);
      for (const label of [
        "Fastの調教メニュー", "Fastを調教", "Fastを休養させる", "Fastを放牧に出す", "Fastをお任せ調教",
        "Fastを自動出走", "Fastのメンコ", "Fastのシャドーロール", "Fastの調教費用",
      ]) {
        expect(last.getByLabelText(label)).toBeTruthy();
      }
    });

    it("folds the explanations away until asked for", async () => {
      await open();
      const help = panel().getByText("調教・疲労・お任せの説明").closest("details")!;
      expect(help.open).toBe(false);
      expect(within(help).getByText(/メンコは周りの音を遮り/)).toBeTruthy();
    });
  });

  it("names the jockey and stable lists on the stables tab", async () => {
    const trainers = [{ id: 0, name: "鬼塚 剛志", training: "スパルタ", distance: "短距離", rotation: "連闘も辞さない", aim: "堅実に賞金", description: "叩きます。" }];
    mockBackend(undefined, undefined, undefined, {
      get_game_state: { ...state, trainers, jockeys: [], field: [{ ...state.field[0], trainer: 0 }, state.field[1]] },
    });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "厩舎" }));
    const jockey = screen.getByLabelText("Fastの騎手");
    expect(jockey.closest("label")!.textContent).toMatch(/^騎手/);
    const stable = screen.getByLabelText("Fastの厩舎");
    expect(stable.closest("label")!.textContent).toMatch(/^厩舎/);
  });

  describe("the honours tab", () => {
    const star = {
      name: "ライバルオー", sex: "male", age: 4, speed: 90, stamina: 85, power: 70, dirt: 0, temper: 50, style: "sashi",
      coat_color: "鹿毛", gear: { menko: false, shadow_roll: false }, peak_age: 4, apt_min: 1600, apt_max: 2400,
      starts: 12, wins: 7, g1_wins: 3, earnings: 4200, year_earnings: 0, year_g1: 0, met: 5, beaten: 2, last_run: 0,
    };
    const career = {
      earnings: 3000,
      stakes_wins: [{ year: 1, race: "疾風賞", grade: "g1" }, { year: 1, race: "武蔵野ハンデキャップ", grade: "g3" }],
      titles: ["三冠"],
      year_earnings: 0,
      year_g1: 0,
    };
    const trainers = [
      { id: 0, name: "鬼塚 剛志", training: "スパルタ", distance: "短距離", rotation: "連闘も辞さない", aim: "堅実に賞金", description: "叩きます。" },
      { id: 1, name: "森田 穏", training: "優しい", distance: "長距離", rotation: "間隔を空ける", aim: "バランス", description: "じっくり。" },
    ];
    const honoured = {
      ...state,
      stars: [star],
      trainers,
      trainer_records: [{ starts: 20, wins: 5, g1_wins: 1, earnings: 3000 }, { starts: 0, wins: 0, g1_wins: 0, earnings: 0 }],
      awards: [
        { year: 1, title: "年度代表馬", horse: "ライバルオー", mine: false },
        { year: 2, title: "三冠", horse: "Fast", mine: true },
      ],
      hall: [{ name: "レジェンド", year: 3, wins: 9, g1_wins: 4, earnings: 8000, titles: ["三冠"] }],
      field: [{ ...state.field[0], career }, { ...state.field[1] }],
    };
    const open = async (backend: unknown = honoured) => {
      mockBackend(undefined, undefined, undefined, { get_game_state: backend });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "実績" }));
    };
    const region = (name: string) => within(screen.getByRole("region", { name }));

    it("lists the rivals with their records and how they have fared against the stable", async () => {
      await open();
      const r = region("ライバル");
      expect(r.getByText("ライバルオー")).toBeTruthy();
      expect(r.getByText("牡4")).toBeTruthy();
      expect(r.getByText("12戦7勝(G1 3勝)")).toBeTruthy();
      expect(r.getByText("4200")).toBeTruthy();
      expect(r.getByText("対戦5回・先着2回")).toBeTruthy();
    });

    it("shows the honours, newest first, and whose they were", async () => {
      await open();
      const items = region("受賞").getAllByRole("listitem");
      expect(items[0].textContent).toBe("2年目 三冠: Fast");
      expect(items[1].textContent).toBe("1年目 年度代表馬: ライバルオー(ライバル)");
    });

    it("shows the hall of fame", async () => {
      await open();
      const h = region("殿堂");
      expect(h.getByText("レジェンド")).toBeTruthy();
      expect(h.getByText("9勝(G1 4勝)・獲得賞金 8000・三冠")).toBeTruthy();
    });

    it("shows what the stable's horses have won", async () => {
      await open();
      const c = region("現役馬の実績");
      expect(c.getByText("Fast")).toBeTruthy();
      expect(c.getByText("獲得賞金 3000")).toBeTruthy();
      expect(c.getByText("G1 疾風賞(1年目)・G3 武蔵野ハンデキャップ(1年目)")).toBeTruthy();
      expect(c.getByText("三冠")).toBeTruthy();
    });

    it("shows how the trainers have done", async () => {
      await open();
      const row = region("調教師の成績").getByText("鬼塚 剛志").closest("tr")!;
      expect(within(row).getByText("20")).toBeTruthy();
      expect(within(row).getByText("5")).toBeTruthy();
      expect(within(row).getByText("25.0%")).toBeTruthy();
      const quiet = region("調教師の成績").getByText("森田 穏").closest("tr")!;
      expect(within(quiet).getByText("-")).toBeTruthy();
    });

    it("says when there is nothing to show yet", async () => {
      await open(state);
      expect(region("受賞").getByText("まだありません。")).toBeTruthy();
      expect(region("殿堂").getByText("まだいません。")).toBeTruthy();
    });

    it("marks a rival on the race card", async () => {
      const entrants = freeCard.entrants.map((e, i) => ({ ...e, star: i === 0 }));
      mockBackend(undefined, undefined, undefined, {
        get_game_state: { ...state, stars: [{ ...star, name: "Fast" }] },
        get_race_card: { ...freeCard, entrants },
      });
      render(<App />);
      await ready();
      const badge = screen.getByLabelText("Fastはライバル");
      expect(badge.getAttribute("title")).toBe("12戦7勝(G1 3勝)");
      expect(screen.queryByLabelText("Slowはライバル")).toBeNull();
    });
  });

  describe("weights", () => {
    it("shows what each runner carries", async () => {
      const entrants = freeCard.entrants.map((e, i) => ({ ...e, weight: i === 0 ? 58 : null }));
      mockBackend(undefined, undefined, undefined, { get_race_card: { ...freeCard, entrants } });
      render(<App />);
      await ready();
      expect(screen.getByLabelText("Fastの斤量: 58kg").textContent).toBe("58");
      expect(screen.queryByLabelText(/Slowの斤量/)).toBeNull();
    });

    it("explains the weight on the column", async () => {
      mockBackend();
      render(<App />);
      await ready();
      expect(screen.getByRole("columnheader", { name: "斤量" }).getAttribute("title")).toMatch(/重いほど/);
    });
  });

  describe("jockeys", () => {
    const jockeys = [
      { id: 0, name: "神崎 颯", skill: 92, style: "sashi", turn: "left", tact: 8, fee: 20, description: "名手です。" },
      { id: 1, name: "若葉 翔", skill: 52, style: "nige", turn: "right", tact: 2, fee: 3, description: "新人です。" },
    ];
    const trainers = [{ id: 0, name: "鬼塚 剛志", training: "スパルタ", distance: "短距離", rotation: "連闘も辞さない", aim: "堅実に賞金", description: "叩きます。" }];
    const staffed = (jockey: number | null) => ({
      ...state,
      jockeys,
      trainers,
      stable_capacity: 10,
      field: [{ ...state.field[0], trainer: 0, jockey }, { ...state.field[1], trainer: 0, jockey: null }],
    });
    const openStables = async () => {
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "厩舎" }));
    };
    const box = () => screen.getByLabelText("Fastの騎手") as HTMLSelectElement;

    it("leaves the choice of jockey to the trainer until one is chosen", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: staffed(null) });
      await openStables();
      expect(box().value).toBe("");
      expect(within(box()).getByRole("option", { name: "調教師におまかせ" })).toBeTruthy();
      expect(within(box()).getByRole("option", { name: "神崎 颯(騎乗料 20)" })).toBeTruthy();
    });

    it("books a jockey for a horse", async () => {
      const set = vi.fn(() => staffed(1));
      mockBackend(undefined, undefined, undefined, { get_game_state: staffed(null), set_jockey: set });
      await openStables();
      fireEvent.change(box(), { target: { value: "1" } });
      await waitFor(() => expect(set).toHaveBeenCalledWith({ horse: "Fast", jockey: 1 }));
      await waitFor(() => expect(box().value).toBe("1"));
    });

    it("hands the choice back to the trainer", async () => {
      const set = vi.fn(() => staffed(null));
      mockBackend(undefined, undefined, undefined, { get_game_state: staffed(1), set_jockey: set });
      await openStables();
      fireEvent.change(box(), { target: { value: "" } });
      await waitFor(() => expect(set).toHaveBeenCalledWith({ horse: "Fast", jockey: null }));
    });

    it("names the riders on the race card", async () => {
      const entrants = freeCard.entrants.map((e, i) => ({ ...e, jockey: i === 0 ? "神崎 颯" : null }));
      mockBackend(undefined, undefined, undefined, { get_race_card: { ...freeCard, entrants } });
      render(<App />);
      await ready();
      expect(screen.getByLabelText("Fastの騎手: 神崎 颯").textContent).toBe("神崎 颯");
    });

    it("lists the jockeys on the information tab", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: staffed(null), get_courses: [], get_course_fits: [] });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "情報" }));
      const list = within(await screen.findByRole("region", { name: "騎手" }));
      expect(list.getByText("神崎 颯")).toBeTruthy();
      expect(list.getByText("騎乗料 20")).toBeTruthy();
      expect(list.getByText("技術 92")).toBeTruthy();
      expect(list.getByText("差し")).toBeTruthy();
      expect(list.getByText("左回り")).toBeTruthy();
      expect(list.getByText("名手です。")).toBeTruthy();
    });
  });

  describe("injuries and pasture", () => {
    const pastured = { kind: "pasture", label: "放牧", weeks: 3 };
    const injured = { kind: "injured", label: "骨折", weeks: 12 };
    const withAway = (away: unknown) => ({
      ...state,
      field: [{ ...state.field[0], legs: 72, away }, { ...state.field[1], legs: 45, away: null }],
    });
    const openTraining = async () => {
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "育成" }));
    };

    it("shows the strength of each horse's legs", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: withAway(null) });
      await openTraining();
      expect(screen.getByLabelText("Fastの脚元: 72").textContent).toBe("72");
    });

    it("sends a horse to pasture", async () => {
      const send = vi.fn(() => withAway(pastured));
      mockBackend(undefined, undefined, undefined, { get_game_state: withAway(null), pasture_horse: send });
      await openTraining();
      fireEvent.click(screen.getByRole("button", { name: "Fastを放牧に出す" }));
      await waitFor(() => expect(send).toHaveBeenCalledWith({ name: "Fast" }));
      await screen.findByText("放牧中(残り3週)");
    });

    it("keeps a horse at pasture out of training and lets it be called back", async () => {
      const recall = vi.fn(() => withAway(null));
      mockBackend(undefined, undefined, undefined, { get_game_state: withAway(pastured), recall_horse: recall });
      await openTraining();
      expect((screen.getByLabelText("Fastを調教") as HTMLButtonElement).disabled).toBe(true);
      expect((screen.getByLabelText("Fastを休養させる") as HTMLButtonElement).disabled).toBe(true);
      expect(screen.queryByRole("button", { name: "Fastを放牧に出す" })).toBeNull();
      fireEvent.click(screen.getByRole("button", { name: "Fastを帰厩させる" }));
      await waitFor(() => expect(recall).toHaveBeenCalledWith({ name: "Fast" }));
      await screen.findByRole("button", { name: "Fastを放牧に出す" });
    });

    it("cannot call back an injured horse", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: withAway(injured) });
      await openTraining();
      expect(screen.getByText("骨折で休養中(残り12週)")).toBeTruthy();
      expect(screen.queryByRole("button", { name: "Fastを帰厩させる" })).toBeNull();
      expect(screen.queryByRole("button", { name: "Fastを放牧に出す" })).toBeNull();
      expect((screen.getByLabelText("Fastを調教") as HTMLButtonElement).disabled).toBe(true);
    });

    it("says so when a horse breaks down in training", async () => {
      const hurt = { ...withAway(injured), news: ["Fastが故障しました(骨折、復帰まで16週)。"] };
      mockBackend(undefined, () => hurt, undefined, { get_game_state: withAway(null), train_horse: hurt });
      await openTraining();
      fireEvent.click(screen.getByLabelText("Fastを調教"));
      await screen.findByText("Fastが故障しました(骨折、復帰まで16週)。");
    });

    it("says nothing after a session that goes well", async () => {
      const fine = { ...withAway(null), news: ["前の週の話"] };
      mockBackend(undefined, undefined, undefined, { get_game_state: withAway(null), train_horse: fine });
      await openTraining();
      fireEvent.click(screen.getByLabelText("Fastを調教"));
      await waitFor(() => expect(vi.mocked(invoke).mock.calls.some((c) => c[0] === "train_horse")).toBe(true));
      await act(async () => {
        await new Promise((r) => setTimeout(r, 0));
      });
      expect(screen.queryByText("前の週の話")).toBeNull();
    });

    it("says why a horse can't be entered in a race", async () => {
      const card = {
        ...freeCard,
        my_horses: [
          { name: "Fast", eligible: false, entered: false, away: injured },
          { name: "Slow", eligible: false, entered: false, away: pastured },
        ],
      };
      mockBackend(undefined, undefined, undefined, { get_race_card: card });
      render(<App />);
      await ready();
      expect(screen.getByText("(骨折で休養中)")).toBeTruthy();
      expect(screen.getByText("(放牧中)")).toBeTruthy();
    });

    it("shows the legs of the horses in the stud book", async () => {
      const stud = {
        name: "ロウバイ", sex: "female", speed: 70, stamina: 72, power: 60, dirt: 0, legs: 81,
        retired_age: 6, retired_year: 2, pedigree: null, blood: null,
      };
      mockBackend(undefined, undefined, undefined, { get_game_state: { ...state, studs: [stud], max_studs: 6 } });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "繁殖" }));
      expect(within(screen.getByRole("region", { name: "繁殖馬" })).getByText("脚元 81")).toBeTruthy();
    });
  });

  describe("the stable's money", () => {
    const staffed = {
      ...state,
      weekly_upkeep: 25,
      trainers: [{ id: 0, name: "鬼塚 剛志", training: "スパルタ", distance: "短距離", rotation: "連闘も辞さない", aim: "堅実に賞金", description: "叩きます。" }],
      stable_capacity: 10,
      field: [
        { ...state.field[0], trainer: 0, sale_price: 640 },
        { ...state.field[1], trainer: 0, sale_price: 100 },
      ],
    };
    const offer = {
      blood_id: 5, name: "ガイブノオー", sex: "male", speed: 80, stamina: 80, power: 80, dirt: 0, temper: 50,
      price: 1500, stud_fee: 300,
      blood: { lineage: "Aqua", family: "f1", gen: 9, coat_color: "鹿毛", growth_type: "普通", peak_age: 4,
        aptitude_min: 1600, aptitude_max: 2400, running_style: "差し", personality: "冷静", inbreed_vol: 0, mutations: [] },
    };

    it("shows what the stable pays each week", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: staffed });
      render(<App />);
      await ready();
      expect(screen.getByText("週の預託料: 25")).toBeTruthy();
    });

    it("sells a horse from the stables tab once confirmed", async () => {
      const sold = { ...staffed, money: 1640, field: [staffed.field[1]] };
      const sell = vi.fn(() => sold);
      const confirm = vi.spyOn(window, "confirm").mockReturnValue(true);
      mockBackend(undefined, undefined, undefined, { get_game_state: staffed, sell_horse: sell });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "厩舎" }));
      fireEvent.click(screen.getByRole("button", { name: "Fastを640で売却" }));
      await waitFor(() => expect(sell).toHaveBeenCalledWith({ name: "Fast" }));
      expect(confirm).toHaveBeenCalled();
      await screen.findByText("Fastを640で売却しました。");
      expect(screen.queryByRole("button", { name: /^Fastを/ })).toBeNull();
      confirm.mockRestore();
    });

    it("keeps the horse if the sale is called off", async () => {
      const sell = vi.fn(() => staffed);
      const confirm = vi.spyOn(window, "confirm").mockReturnValue(false);
      mockBackend(undefined, undefined, undefined, { get_game_state: staffed, sell_horse: sell });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "厩舎" }));
      fireEvent.click(screen.getByRole("button", { name: "Fastを640で売却" }));
      expect(sell).not.toHaveBeenCalled();
      confirm.mockRestore();
    });

    it("cannot sell the last horse", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: { ...staffed, field: [staffed.field[0]] } });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "厩舎" }));
      expect((screen.getByRole("button", { name: "Fastを640で売却" }) as HTMLButtonElement).disabled).toBe(true);
    });

    it("offers a market stallion for its fee and adds that to the cost of the mating", async () => {
      const outlook = vi.fn(() => ({ inbreed_vol: 0, penalty: 0, nick: null }));
      mockBackend(undefined, undefined, undefined, {
        get_game_state: staffed,
        get_stud_market: [offer],
        breeding_outlook: outlook,
      });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "繁殖" }));
      await screen.findByRole("option", { name: "ガイブノオー(種付け料 300)" });
      expect(screen.getByRole("button", { name: "配合 (500)" })).toBeTruthy();
      fireEvent.change(screen.getByLabelText("父馬(牡)"), { target: { value: "ガイブノオー" } });
      await screen.findByRole("button", { name: "配合 (800)" });
      await waitFor(() => expect(outlook).toHaveBeenCalledWith({ sire: "ガイブノオー", dam: "ロウバイ" }));
    });

    it("will not mate with a market stallion that cannot be paid for", async () => {
      mockBackend(undefined, undefined, undefined, {
        get_game_state: { ...staffed, money: 600 },
        get_stud_market: [offer],
      });
      render(<App />);
      await screen.findByText("所持金: 600");
      fireEvent.click(screen.getByRole("tab", { name: "繁殖" }));
      await screen.findByRole("option", { name: "ガイブノオー(種付け料 300)" });
      fireEvent.change(screen.getByLabelText("父馬(牡)"), { target: { value: "ガイブノオー" } });
      fireEvent.change(screen.getByLabelText("仔馬の名前"), { target: { value: "ハナミズキ" } });
      expect((await screen.findByRole("button", { name: "配合 (800)" }) as HTMLButtonElement).disabled).toBe(true);
    });
  });

  describe("the information tab", () => {
    const plain = { turn: "even", straight: "even", hill: "even", turf: "even", percent: 0, rating: "fair" };
    const best = { turn: "plus", straight: "plus", hill: "plus", turf: "even", percent: 2.75, rating: "great" };
    const worst = { turn: "minus", straight: "minus", hill: "minus", turf: "minus", percent: -3.5, rating: "poor" };
    const courses = [
      {
        venue: "tokyo", name: "東京", place: "東京都府中市", turn: "left", grass: "野芝(洋芝のオーバーシードあり)",
        turf: { lap: "2,083.1m", straight: "525.9m", elevation: "2.7m" },
        dirt: { lap: "1,899m", straight: "501.6m", elevation: "2.5m" },
        features: "直線が長いです。", suits: "差し馬",
      },
      {
        venue: "hakodate", name: "函館", place: "北海道函館市", turn: "right", grass: "洋芝(3種混合)",
        turf: { lap: "1,626.6m", straight: "262.1m", elevation: "3.5m" },
        dirt: { lap: "1,475.8m", straight: "260.3m", elevation: "3.5m" },
        features: "直線が短いです。", suits: "先行馬",
      },
    ];
    const fits = vi.fn((a: { horse: string }) =>
      a.horse === "Fast"
        ? [{ venue: "tokyo", turf: best, dirt: plain }, { venue: "hakodate", turf: worst, dirt: plain }]
        : [{ venue: "tokyo", turf: plain, dirt: plain }, { venue: "hakodate", turf: plain, dirt: plain }]
    );
    const open = async () => {
      mockBackend(undefined, undefined, undefined, { get_courses: courses, get_course_fits: fits });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "情報" }));
      await screen.findByRole("region", { name: "東京競馬場" });
    };
    const course = (name: string) => within(screen.getByRole("region", { name: `${name}競馬場` }));

    it("shows each course's layout", async () => {
      await open();
      const tokyo = course("東京");
      expect(tokyo.getByText("左回り")).toBeTruthy();
      expect(tokyo.getByText("東京都府中市")).toBeTruthy();
      expect(tokyo.getByText("野芝(洋芝のオーバーシードあり)")).toBeTruthy();
      expect(tokyo.getByText("525.9m")).toBeTruthy();
      expect(tokyo.getByText("2.5m")).toBeTruthy();
      expect(tokyo.getByText("直線が長いです。")).toBeTruthy();
      expect(tokyo.getByText("向いている馬: 差し馬")).toBeTruthy();
      expect(course("函館").getByText("右回り")).toBeTruthy();
    });

    it("says where the figures come from", async () => {
      await open();
      expect(screen.getByText(/JRA公式/)).toBeTruthy();
    });

    it("shows how the courses suit the chosen horse", async () => {
      await open();
      await waitFor(() => expect(fits).toHaveBeenCalledWith({ horse: "Fast" }));
      const tokyo = await waitFor(() => {
        const t = course("東京");
        t.getByLabelText("芝の相性: ◎");
        return t;
      });
      expect(tokyo.getByLabelText("ダートの相性: △")).toBeTruthy();
      expect(tokyo.getByText("+2.75%")).toBeTruthy();
      expect(tokyo.getByText("回りが合う・直線が合う・坂に強い")).toBeTruthy();
      expect(course("函館").getByLabelText("芝の相性: ×")).toBeTruthy();
      expect(course("函館").getByText("回りが合わない・直線が合わない・坂に弱い・洋芝が苦手")).toBeTruthy();
    });

    it("looks at another horse when asked", async () => {
      await open();
      await course("東京").findByLabelText("芝の相性: ◎");
      fireEvent.change(screen.getByLabelText("相性を見る馬"), { target: { value: "Slow" } });
      await waitFor(() => expect(fits).toHaveBeenCalledWith({ horse: "Slow" }));
      await course("東京").findByLabelText("芝の相性: △");
    });

    it("asks for nothing until the tab is opened", async () => {
      mockBackend(undefined, undefined, undefined, { get_courses: courses, get_course_fits: fits });
      fits.mockClear();
      render(<App />);
      await ready();
      expect(vi.mocked(invoke).mock.calls.some((c) => c[0] === "get_courses")).toBe(false);
      expect(fits).not.toHaveBeenCalled();
    });

    it("marks how the course suits each runner on the race card", async () => {
      const entrants = freeCard.entrants.map((e, i) => ({
        ...e,
        course_fit: i === 0 ? { ...best } : { ...worst },
      }));
      mockBackend(undefined, undefined, undefined, { get_race_card: { ...freeCard, entrants } });
      render(<App />);
      await ready();
      expect(screen.getByLabelText("Fastと競馬場の相性: ◎")).toBeTruthy();
      expect(screen.getByLabelText("Slowと競馬場の相性: ×")).toBeTruthy();
    });

    it("has nothing to say of the course in the free race", async () => {
      mockBackend();
      render(<App />);
      await ready();
      expect(screen.queryByLabelText(/と競馬場の相性/)).toBeNull();
    });
  });

  describe("the stables tab", () => {
    const trainers = [
      { id: 0, name: "鬼塚 剛志", training: "スパルタ", distance: "短距離", rotation: "連闘も辞さない", aim: "堅実に賞金", description: "叩いて強くします。" },
      { id: 1, name: "森田 穏", training: "優しい", distance: "長距離", rotation: "間隔を空ける", aim: "バランス", description: "じっくり育てます。" },
      { id: 2, name: "花村 ゆかり", training: "優しめ", distance: "マイル", rotation: "間隔を空ける", aim: "G1を目指す", description: "大舞台へ。" },
    ];
    const staffed = {
      ...state,
      trainers,
      stable_capacity: 2,
      field: [
        { ...state.field[0], trainer: 0, age: 4, record: { starts: 3, wins: 1 } },
        { ...state.field[1], trainer: 2, age: 3 },
      ],
    };
    const open = async () => {
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "厩舎" }));
    };
    const stable = (name: string) => screen.getByRole("region", { name: `${name}厩舎` });

    it("shows each trainer's ways and how full their stable is", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: staffed });
      await open();
      const s0 = within(stable("鬼塚 剛志"));
      expect(s0.getByText("1/2頭")).toBeTruthy();
      expect(s0.getByText("調教: スパルタ")).toBeTruthy();
      expect(s0.getByText("距離: 短距離")).toBeTruthy();
      expect(s0.getByText("間隔: 連闘も辞さない")).toBeTruthy();
      expect(s0.getByText("目標: 堅実に賞金")).toBeTruthy();
      expect(s0.getByText("叩いて強くします。")).toBeTruthy();
      expect(within(stable("森田 穏")).getByText("0/2頭")).toBeTruthy();
      expect(within(stable("森田 穏")).getByText("預けている馬はいません。")).toBeTruthy();
    });

    it("lists the horses in the stable they are in", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: staffed });
      await open();
      const s0 = within(stable("鬼塚 剛志"));
      expect(s0.getByText("Fast")).toBeTruthy();
      expect(s0.queryByText("Slow")).toBeNull();
      expect(s0.getByText("牡4")).toBeTruthy();
      expect(s0.getByText("3戦1勝・1勝クラス")).toBeTruthy();
      expect(within(stable("花村 ゆかり")).getByText("Slow")).toBeTruthy();
    });

    it("moves a horse to another stable", async () => {
      const moved = { ...staffed, field: [{ ...staffed.field[0], trainer: 1 }, staffed.field[1]] };
      const setTrainer = vi.fn(() => moved);
      mockBackend(undefined, undefined, undefined, { get_game_state: staffed, set_trainer: setTrainer });
      await open();
      fireEvent.change(screen.getByLabelText("Fastの厩舎"), { target: { value: "1" } });
      await waitFor(() => expect(setTrainer).toHaveBeenCalledWith({ horse: "Fast", trainer: 1 }));
      await waitFor(() => expect(within(stable("森田 穏")).getByText("Fast")).toBeTruthy());
      expect(screen.getByText("Fastを森田 穏厩舎に移しました。")).toBeTruthy();
    });

    it("cannot send a horse to a full stable", async () => {
      const full = {
        ...staffed,
        field: [
          { ...staffed.field[0], trainer: 1 },
          { ...staffed.field[1], trainer: 1 },
          { ...staffed.field[1], name: "Third", trainer: 0 },
        ],
      };
      mockBackend(undefined, undefined, undefined, { get_game_state: full });
      await open();
      const options = within(screen.getByLabelText("Thirdの厩舎")).getAllByRole("option") as HTMLOptionElement[];
      expect(options.find((o) => o.value === "1")!.disabled).toBe(true);
      expect(options.find((o) => o.value === "2")!.disabled).toBe(false);
      // its own stable stays selectable even when full
      const own = within(screen.getByLabelText("Fastの厩舎")).getAllByRole("option") as HTMLOptionElement[];
      expect(own.find((o) => o.value === "1")!.disabled).toBe(false);
    });

    it("reports a move the backend refuses", async () => {
      mockBackend(undefined, undefined, undefined, {
        get_game_state: staffed,
        set_trainer: new Error("その調教師の厩舎は満員です"),
      });
      await open();
      fireEvent.change(screen.getByLabelText("Fastの厩舎"), { target: { value: "1" } });
      await screen.findByText("その調教師の厩舎は満員です");
    });

    it("says what a trainer's distance is worth", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: { ...staffed, trainer_bonus_percent: 3 } });
      await open();
      expect(screen.getByText(/得意距離のレースではスピードとスタミナが3%上がります/)).toBeTruthy();
    });

    it("marks a runner over its trainer's distance on the race card", async () => {
      const entrants = freeCard.entrants.map((e, i) => ({ ...e, distance_fit: "fit", trainer_fit: i === 0 }));
      mockBackend(undefined, undefined, undefined, { get_race_card: { ...freeCard, entrants } });
      render(<App />);
      await ready();
      expect(screen.getAllByLabelText("調教師の得意距離").length).toBe(1);
    });

    it("names the stable on the training tab", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: staffed });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "育成" }));
      const row = within(screen.getByRole("tabpanel", { name: "育成" })).getByText("Fast").closest("tr")!;
      expect(within(row).getByText("鬼塚 剛志")).toBeTruthy();
    });
  });

  describe("what the race card says", () => {
    const entrants = freeCard.entrants;
    const cardWith = (extra: Record<string, unknown>) => ({
      ...freeCard,
      entrants: entrants.map((e, i) => ({ ...e, popularity: i + 1 })),
      ...extra,
    });

    it("shows how popular each horse is", async () => {
      mockBackend(undefined, undefined, undefined, { get_race_card: cardWith({}) });
      render(<App />);
      await ready();
      await waitFor(() => expect(screen.getByRole("columnheader", { name: "人気" })).toBeTruthy());
      expect(screen.getByLabelText(`${entrants[0].name}: 1番人気`)).toBeTruthy();
      expect(screen.getByLabelText(`${entrants[1].name}: 2番人気`)).toBeTruthy();
    });

    it("offers a wide ticket that picks two horses", async () => {
      mockBackend(undefined, undefined, undefined, { get_race_card: cardWith({}), quote_bet: 3.4 });
      render(<App />);
      await ready();
      fireEvent.change(await screen.findByLabelText("賭け方"), { target: { value: "wide" } });
      expect(within(screen.getByLabelText("賭け方")).getByRole("option", { name: "ワイド(2頭が3着以内)" })).toBeTruthy();
      expect(screen.getByLabelText("1頭目")).toBeTruthy();
      expect(screen.getByLabelText("2頭目")).toBeTruthy();
      expect(screen.queryByLabelText("3頭目")).toBeNull();
    });

    it("gives the weather with the course", async () => {
      const course = { name: "東京", turn: "左回り" };
      mockBackend(undefined, undefined, undefined, { get_race_card: cardWith({ course, weather: "light_rain", track: "soft" }) });
      render(<App />);
      await ready();
      await waitFor(() => expect(screen.getByLabelText("コース情報").textContent).toContain("小雨・稍重"));
    });
  });

  describe("new parts of the stable and the record", () => {
    const row = (name: string, wins: number, own_wins = 0) => ({ name, wins, own_wins });
    const withLeading = {
      ...state,
      leading: { jockeys: [row("神崎 颯", 80, 2)], trainers: [row("鬼塚", 40)], sires: [row("ナギサ", 30)] },
      leading_years: [{ year: 1, jockey: "神崎 颯", trainer: "鬼塚", sire: "ナギサ" }],
    };

    it("shows the leading tables with the stable's own wins, and past leaders", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: withLeading });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "実績" }));
      const jockeys = screen.getByRole("table", { name: "騎手リーディング" });
      expect(within(jockeys).getByText("80(2)")).toBeTruthy();
      expect(within(screen.getByRole("table", { name: "種牡馬リーディング" })).getByText("ナギサ")).toBeTruthy();
      expect(within(screen.getByRole("list", { name: "過去のリーディング" })).getByText(/1年目: 騎手 神崎 颯/)).toBeTruthy();
    });

    it("shows a gelding as セ and offers to geld only colts", async () => {
      const trainers = [{ id: 0, name: "鬼塚", region: "美浦", training: "標準", distance: "万能", rotation: "標準", aim: "バランス", description: "" }];
      const [first, second, third] = [state.field[0], state.field[1], state.field[0]];
      const colt = { ...first, sex: "male", trainer: 0, sale_price: 100 };
      const gelding = { ...second, sex: "male", gelded: true, trainer: 0, sale_price: 100, age: 4 };
      const filly = { ...third, name: "Filly", sex: "female", trainer: 0, sale_price: 100 };
      mockBackend(undefined, undefined, undefined, { get_game_state: { ...state, trainers, stable_capacity: 10, gelding_cost: 300, field: [colt, gelding, filly] } });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "厩舎" }));
      expect(screen.getByRole("button", { name: `${colt.name}を去勢` })).toBeTruthy();
      expect(screen.queryByRole("button", { name: `${gelding.name}を去勢` })).toBeNull();
      expect(screen.queryByRole("button", { name: "Fillyを去勢" })).toBeNull();
      expect(screen.getByText("セ4")).toBeTruthy();
    });

    it("gelds a colt once asked and told yes", async () => {
      const trainers = [{ id: 0, name: "鬼塚", region: "美浦", training: "標準", distance: "万能", rotation: "標準", aim: "バランス", description: "" }];
      const colt = { ...state.field[0], sex: "male", trainer: 0, sale_price: 100 };
      const geld = vi.fn(() => ({ ...state, trainers, field: [{ ...colt, gelded: true }] }));
      vi.spyOn(window, "confirm").mockReturnValue(true);
      mockBackend(undefined, undefined, undefined, { get_game_state: { ...state, trainers, stable_capacity: 10, gelding_cost: 300, field: [colt] }, geld_horse: geld });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "厩舎" }));
      fireEvent.click(screen.getByRole("button", { name: `${colt.name}を去勢` }));
      await waitFor(() => expect(geld).toHaveBeenCalledWith({ name: colt.name }));
    });

    it("sends the blinkers with the rest of the gear", async () => {
      const setGear = vi.fn(() => state);
      mockBackend(undefined, undefined, undefined, { set_gear: setGear });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "育成" }));
      fireEvent.click(screen.getByLabelText(`${state.field[0].name}のブリンカー`));
      await waitFor(() => expect(setGear).toHaveBeenCalledWith({ horse: state.field[0].name, menko: false, shadowRoll: false, blinkers: true }));
    });

    it("says what the season is doing to a runner", async () => {
      const entrants = freeCard.entrants.map((e, i) => ({ ...e, season: i === 0 ? "夏負け" : null }));
      mockBackend(undefined, undefined, undefined, { get_race_card: { ...freeCard, entrants } });
      render(<App />);
      await ready();
      expect(await screen.findByText("夏負け")).toBeTruthy();
    });

    it("writes the grades of the local tracks as Jpn", () => {
      expect(gradeLabel({ grade: "g1", venue: "oi" })).toBe("JpnI");
      expect(gradeLabel({ grade: "g3", venue: "sonoda" })).toBe("JpnIII");
      expect(gradeLabel({ grade: "g1", venue: "tokyo" })).toBe("G1");
      expect(gradeLabel({ grade: "open", venue: "oi" })).toBe("OP");
    });

    it("gives the odds of the foal's growth with the outlook", async () => {
      const outlook = { inbreed_vol: 0, penalty: 0, nick: null, growth: { early: 20, normal: 60, late: 20 } };
      const studs = [
        { name: "チチ", sex: "male", speed: 1, stamina: 1, power: 1, wins: 0, starts: 0, age: 9 },
        { name: "ハハ", sex: "female", speed: 1, stamina: 1, power: 1, wins: 0, starts: 0, age: 9 },
      ];
      mockBackend(undefined, undefined, undefined, { get_game_state: { ...state, studs }, breeding_outlook: outlook });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "繁殖" }));
      expect(await screen.findByText("仔の成長型の見込み: 早熟20% / 普通60% / 晩成20%")).toBeTruthy();
    });
  });

  describe("over fences", () => {
    const trainers = [{ id: 0, name: "鬼塚", region: "美浦", training: "標準", distance: "万能", rotation: "標準", aim: "バランス", description: "" }];
    const horse = (extra: Record<string, unknown>) => ({ ...state.field[0], trainer: 0, sale_price: 100, age: 4, ...extra });
    const game = (h: Record<string, unknown>) => ({ ...state, trainers, stable_capacity: 10, jump_exam_level: 30, jump_exam_cost: 100, field: [h] });

    it("offers a horse the chance to go over to jumps", async () => {
      const go = vi.fn(() => game(horse({ jumper: true, jump: 10 })));
      mockBackend(undefined, undefined, undefined, { get_game_state: game(horse({})), go_over_to_jumps: go });
      vi.spyOn(window, "confirm").mockReturnValue(true);
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "厩舎" }));
      fireEvent.click(screen.getByRole("button", { name: "Fastを障害転向" }));
      await waitFor(() => expect(go).toHaveBeenCalledWith({ name: "Fast" }));
    });

    it("lets a jumper sit the exam once it jumps well enough", async () => {
      const exam = vi.fn(() => game(horse({ jumper: true, jump: 30, jump_license: true })));
      mockBackend(undefined, undefined, undefined, { get_game_state: game(horse({ jumper: true, jump: 30 })), take_jump_exam: exam });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "厩舎" }));
      expect(screen.queryByRole("button", { name: "Fastを障害転向" })).toBeNull();
      fireEvent.click(screen.getByRole("button", { name: "Fastの障害試験" }));
      await waitFor(() => expect(exam).toHaveBeenCalledWith({ name: "Fast" }));
    });

    it("holds the exam back until the horse jumps well enough", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: game(horse({ jumper: true, jump: 20 })) });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "厩舎" }));
      expect((screen.getByRole("button", { name: "Fastの障害試験" }) as HTMLButtonElement).disabled).toBe(true);
    });

    it("shows schooling only for a jumper, with its jumping", async () => {
      const workouts = [
        { workout: "wood", name: "ウッドコース追い", description: "", gains: [{ ability: "speed", amount: 3 }], fatigue: 8 },
        { workout: "jump", name: "障害練習", description: "", gains: [{ ability: "jump", amount: 4 }], fatigue: 10 },
      ];
      mockBackend(undefined, undefined, undefined, { get_game_state: { ...game(horse({})), workouts } });
      const { unmount } = render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "育成" }));
      const menu = () => within(screen.getByLabelText("Fastの調教メニュー")).queryByRole("option", { name: "障害練習" });
      expect(menu()).toBeNull();
      unmount();
      mockBackend(undefined, undefined, undefined, { get_game_state: { ...game(horse({ jumper: true, jump: 25 })), workouts } });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "育成" }));
      expect(menu()).toBeTruthy();
      expect(screen.getByLabelText("Fastの飛越").textContent).toBe("飛越 25");
    });

    it("shows the fences and each runner's jumping on a jump race's card", async () => {
      const entrants = freeCard.entrants.map((e) => ({ ...e, jump: 55 }));
      const course = { name: "中山", turn: "右回り", lap: 1800, straight: 300 };
      mockBackend(undefined, undefined, undefined, { get_race_card: { ...freeCard, course, surface: "jump", distance: 3200, fences: 9, entrants } });
      render(<App />);
      await ready();
      await waitFor(() => expect(screen.getByRole("columnheader", { name: "飛越" })).toBeTruthy());
      expect(screen.getAllByText("55").length).toBe(entrants.length);
      expect(screen.getByLabelText("コース情報").textContent).toContain("障害3200m");
      expect(screen.getByLabelText("コース情報").textContent).toContain("障害9回");
    });

    it("leaves the jumping out on the flat", async () => {
      mockBackend();
      render(<App />);
      await ready();
      expect(screen.queryByRole("columnheader", { name: "飛越" })).toBeNull();
    });

    it("lists those who did not finish with the result", async () => {
      const race = {
        results: [{ rank: 1, name: "Fast", finish_time: 60 }, { rank: 2, name: "Third", finish_time: 60.5 }],
        stopped: ["Slow"],
        frames: [[500, 400, 300], [1000, 500, 900], [1000, 500, 1000]],
        commentary: [],
        runners: ["Fast", "Slow", "Third"],
        race_name: null,
        distance: 1000,
        won: false,
        payout: 0,
        tickets: [],
        prizes: [],
        prize_total: 0,
        money: 1000,
      };
      mockBackend(undefined, undefined, undefined, { watch_race: race });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByText("レース開始"));
      fireEvent.click(await screen.findByText("スキップ"));
      await waitFor(() => expect(screen.getByText("競走中止: Slow")).toBeTruthy());
    });

    it("writes the grades over fences as J-G", () => {
      expect(gradeLabel({ grade: "g1", venue: "nakayama", surface: "jump" })).toBe("J・G1");
      expect(gradeLabel({ grade: "g3", venue: "kokura", surface: "jump" })).toBe("J・G3");
      expect(gradeLabel({ grade: "maiden", venue: "kokura", surface: "jump" })).toBe("未勝利");
    });

    it("lets only a licensed jumper count on a jump race", async () => {
      const r = { surface: "jump", grade: "open", eligibility: { age: "three_up", fillies_only: false, weight: "fixed" } };
      expect(mayRunOver(r as never, { jump_license: true } as never)).toBe(true);
      expect(mayRunOver(r as never, {} as never)).toBe(false);
    });
  });

  describe("running out of horses", () => {
    const warning = () => screen.queryByRole("status", { name: "後継の馬" });

    it("says nothing while the stable has horses enough", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: { ...state, racers: 8, successors: 6, min_racers: 4, short_of_racers: false, short_of_successors: false } });
      render(<App />);
      await ready();
      expect(warning()).toBeNull();
    });

    it("says how many successors there are", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: { ...state, racers: 8, successors: 2, min_racers: 4, short_of_racers: false, short_of_successors: true } });
      render(<App />);
      await ready();
      expect(warning()?.textContent).toBe("後継の馬が2頭しかいません。種付けかセリで用意してください。");
    });

    it("says how many can race once those run short too", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: { ...state, racers: 3, successors: 0, min_racers: 4, short_of_racers: true, short_of_successors: true } });
      render(<App />);
      await ready();
      expect(warning()?.textContent).toBe("出走できる馬が3頭、後継の馬が0頭です。種付けかセリで用意してください。");
    });

    it("stays on whichever tab is open", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: { ...state, racers: 8, successors: 2, min_racers: 4, short_of_racers: false, short_of_successors: true } });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "繁殖" }));
      expect(warning()).toBeTruthy();
    });
  });

  describe("autoplay", () => {
    /** A backend whose weeks go on when asked, and what it was asked. */
    const weeks = (turn: (week: number) => Record<string, unknown> = () => ({})) => {
      let week = 1;
      const nextWeek = vi.fn(() => {
        week += 1;
        return { ...state, week, ...turn(week) };
      });
      mockBackend(undefined, undefined, undefined, { next_week: nextWeek });
      return nextWeek;
    };
    const start = () => fireEvent.click(screen.getByRole("button", { name: "オートプレイ開始" }));
    const fastest = () =>
      fireEvent.change(screen.getByLabelText("オートプレイの速さ"), { target: { value: "200" } });

    it("moves the weeks on by itself until it is stopped", async () => {
      const nextWeek = weeks();
      render(<App />);
      await ready();
      fastest();
      start();
      await waitFor(() => expect(nextWeek.mock.calls.length).toBeGreaterThanOrEqual(2), { timeout: 3000 });
      fireEvent.click(screen.getByRole("button", { name: "オートプレイ停止" }));
      await act(async () => {
        await new Promise((r) => setTimeout(r, 0));
      });
      const stoppedAt = nextWeek.mock.calls.length;
      await act(async () => {
        await new Promise((r) => setTimeout(r, 350));
      });
      expect(nextWeek.mock.calls.length).toBe(stoppedAt);
      expect(screen.getByRole("button", { name: "オートプレイ開始" })).toBeTruthy();
    });

    it("keeps a log of what happened each week, newest first", async () => {
      weeks((w) => ({ news: [`出来事${w}`] }));
      render(<App />);
      await ready();
      fastest();
      start();
      await waitFor(
        () => expect(within(screen.getByLabelText("オートプレイの記録")).getAllByRole("listitem").length).toBeGreaterThanOrEqual(2),
        { timeout: 3000 }
      );
      fireEvent.click(screen.getByRole("button", { name: "オートプレイ停止" }));
      const lines = within(screen.getByLabelText("オートプレイの記録")).getAllByRole("listitem");
      expect(lines[0].textContent).toBe("1年目 第3週: 出来事3");
      expect(lines[1].textContent).toBe("1年目 第2週: 出来事2");
    });

    it("runs while another tab is open", async () => {
      const nextWeek = weeks();
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "育成" }));
      fastest();
      start();
      await waitFor(() => expect(nextWeek).toHaveBeenCalled(), { timeout: 3000 });
      fireEvent.click(screen.getByRole("button", { name: "オートプレイ停止" }));
    });

    it("stops when the game is over", async () => {
      const nextWeek = weeks(() => ({ game_over: true, money: 0 }));
      render(<App />);
      await ready();
      fastest();
      start();
      await screen.findByText("ゲームオーバーのためオートプレイを止めました。", undefined, { timeout: 3000 });
      expect(nextWeek).toHaveBeenCalledTimes(1);
      expect((screen.getByRole("button", { name: "オートプレイ開始" }) as HTMLButtonElement).disabled).toBe(true);
    });

    it("stops when the stable is empty", async () => {
      const nextWeek = weeks(() => ({ field: [] }));
      render(<App />);
      await ready();
      fastest();
      start();
      await screen.findByText("厩舎に馬がいなくなったのでオートプレイを止めました。", undefined, { timeout: 3000 });
      expect(nextWeek).toHaveBeenCalledTimes(1);
    });

    it("stops before the money runs out and horses have to be sold", async () => {
      const nextWeek = weeks(() => ({ money: 119, weekly_upkeep: 30 }));
      render(<App />);
      await ready();
      fastest();
      start();
      await screen.findByText("所持金が少なくなったのでオートプレイを止めました。賞金が入るよう「自動出走」を確認してください。", undefined, { timeout: 3000 });
      expect(nextWeek).toHaveBeenCalledTimes(1);
    });

    it("does not point to automatic entry when every horse already has it", async () => {
      weeks(() => ({ money: 119, weekly_upkeep: 30, field: state.field.map((h) => ({ ...h, auto_entry: true })) }));
      render(<App />);
      await ready();
      fastest();
      start();
      await screen.findByText("所持金が少なくなったのでオートプレイを止めました。", undefined, { timeout: 3000 });
    });

    it("carries on while there is money enough", async () => {
      const nextWeek = weeks(() => ({ money: 120, weekly_upkeep: 30 }));
      render(<App />);
      await ready();
      fastest();
      start();
      await waitFor(() => expect(nextWeek.mock.calls.length).toBeGreaterThanOrEqual(2), { timeout: 3000 });
      fireEvent.click(screen.getByRole("button", { name: "オートプレイ停止" }));
    });

    it("stops when the racing horses run short, saying how many are left", async () => {
      const nextWeek = weeks(() => ({ racers: 3, min_racers: 4, short_of_racers: true }));
      render(<App />);
      await ready();
      fastest();
      start();
      await screen.findByText(/出走できる馬が3頭になったのでオートプレイを止めました。/, undefined, { timeout: 3000 });
      expect(nextWeek).toHaveBeenCalledTimes(1);
    });

    it("does not stop again and again for a stable already short of racing horses", async () => {
      const short = { racers: 3, min_racers: 4, short_of_racers: true };
      let week = 1;
      const nextWeek = vi.fn(() => {
        week += 1;
        return { ...state, ...short, week };
      });
      mockBackend(undefined, undefined, undefined, { get_game_state: { ...state, ...short }, next_week: nextWeek });
      render(<App />);
      await ready();
      fastest();
      start();
      await waitFor(() => expect(nextWeek.mock.calls.length).toBeGreaterThanOrEqual(2), { timeout: 3000 });
      expect(screen.queryByText(/オートプレイを止めました/)).toBeNull();
      fireEvent.click(screen.getByRole("button", { name: "オートプレイ停止" }));
    });

    it("stops at the turn of the year unless told not to", async () => {
      const nextWeek = weeks((w) => ({ year: w >= 2 ? 2 : 1 }));
      render(<App />);
      await ready();
      expect((screen.getByLabelText("年が変わったら止める") as HTMLInputElement).checked).toBe(true);
      fastest();
      start();
      await screen.findByText("2年目に入ったのでオートプレイを止めました。", undefined, { timeout: 3000 });
      expect(nextWeek).toHaveBeenCalledTimes(1);
    });

    it("goes on through the turn of the year when told to", async () => {
      const nextWeek = weeks((w) => ({ year: w >= 2 ? 2 : 1 }));
      render(<App />);
      await ready();
      fireEvent.click(screen.getByLabelText("年が変わったら止める"));
      fastest();
      start();
      await waitFor(() => expect(nextWeek.mock.calls.length).toBeGreaterThanOrEqual(2), { timeout: 3000 });
      fireEvent.click(screen.getByRole("button", { name: "オートプレイ停止" }));
    });

    it("keeps the race, the week and the save buttons out of reach while it runs", async () => {
      weeks();
      render(<App />);
      await ready();
      start();
      for (const name of ["レース開始", "次の週へ", "セーブ", "ロード"]) {
        expect((screen.getByRole("button", { name }) as HTMLButtonElement).disabled).toBe(true);
      }
      fireEvent.click(screen.getByRole("button", { name: "オートプレイ停止" }));
      await waitFor(() => expect((screen.getByRole("button", { name: "次の週へ" }) as HTMLButtonElement).disabled).toBe(false));
    });

    it("does not ask for the race card every week while it runs", async () => {
      weeks();
      render(<App />);
      await ready();
      const cards = () => vi.mocked(invoke).mock.calls.filter((c) => c[0] === "get_race_card").length;
      const before = cards();
      fastest();
      start();
      await act(async () => {
        await new Promise((r) => setTimeout(r, 500));
      });
      expect(cards()).toBe(before);
      fireEvent.click(screen.getByRole("button", { name: "オートプレイ停止" }));
      await waitFor(() => expect(cards()).toBeGreaterThan(before));
    });

    it("says which horses it will look after", async () => {
      mockBackend();
      render(<App />);
      await ready();
      expect(screen.getByText(/オンにした馬だけが/)).toBeTruthy();
    });

    it("says nothing about that once a horse is automatic", async () => {
      const on = { ...state, field: state.field.map((h) => ({ ...h, auto_entry: true })) };
      mockBackend(undefined, undefined, undefined, { get_game_state: on });
      render(<App />);
      await ready();
      expect(screen.queryByText(/オンにした馬だけが/)).toBeNull();
    });
  });

  it("names the cost of a session once per horse", async () => {
    mockBackend();
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "育成" }));
    const fast = within(screen.getByRole("tabpanel", { name: "育成" })).getByText("Fast").closest("tr")!;
    expect(within(fast).getByLabelText("Fastの調教費用").textContent).toBe("100");
  });

  it("puts a horse in its coat colour at the head of each lane", async () => {
    mockBackend({
      results: [{ rank: 1, name: "Fast", finish_time: 60 }],
      frames: [[600, 300], [2400, 2000]],
      runners: ["Fast", "Slow"],
      coats: ["芦毛", "青毛"],
      distance: 2400,
      commentary: [],
      won: true,
      payout: 320,
      money: 1220,
    });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByText("レース開始"));
    await screen.findByText("スキップ");
    const lane = screen.getByRole("progressbar", { name: "Fast" }).closest(".lane")!;
    expect(within(lane as HTMLElement).getByLabelText("Fastの毛色: 芦毛")).toBeTruthy();
    expect(screen.getByLabelText("Slowの毛色: 青毛")).toBeTruthy();
  });

  // ---- class races, racing records and equipment ----

  it("marks the races one of my horses can enter", async () => {
    const maiden = {
      ...g3, race_no: 1, name: "3歳未勝利", grade: "maiden",
      eligibility: { age: "three", fillies_only: false, weight: "age" }, prize: 550,
    };
    const debut = { ...maiden, race_no: 3, name: "3歳新馬", grade: "debut" };
    const horses = {
      ...state,
      field: [
        { ...state.field[0], age: 3, record: { starts: 2, wins: 0 } },
        { ...state.field[1], age: 4, record: { starts: 5, wins: 2 } },
      ],
    };
    calendarBackend(undefined, { get_game_state: horses, get_calendar: { meetings: [], races: [maiden, debut, g3] } });
    render(<App />);
    await ready();
    const options = await waitFor(() => {
      const o = Array.from((screen.getByLabelText("レース") as HTMLSelectElement).options).map((x) => x.textContent);
      expect(o.length).toBe(4);
      return o;
    });
    expect(options[1]).toBe("1R 未勝利 3歳未勝利(芝1600m) ★出走可");
    expect(options[2]).toBe("3R 新馬 3歳新馬(芝1600m)"); // raced already: no debut
    expect(options[3]).toBe("11R G3 初春マイル賞(芝1600m) ★出走可");
  });

  it("shows each of my horses' record and class when entering", async () => {
    const horses = {
      ...state,
      field: [
        { ...state.field[0], record: { starts: 12, wins: 4 } },
        { ...state.field[1], record: { starts: 0, wins: 0 } },
      ],
    };
    calendarBackend(undefined, { get_game_state: horses });
    render(<App />);
    await ready();
    await chooseRace();
    expect(screen.getByText("12戦4勝・オープン")).toBeTruthy();
    expect(screen.getByText("0戦0勝・新馬")).toBeTruthy();
  });

  it("reports what happened at the turn of the week after a calendar race", async () => {
    let raced = false;
    calendarBackend(raceOutcome, {
      get_game_state: () => (raced ? { ...state, week: 2, news: ["Fastはお任せで坂路をしました。"] } : state),
      place_bets: () => {
        raced = true;
        return raceOutcome;
      },
    });
    render(<App />);
    await ready();
    await chooseRace();
    fireEvent.click(screen.getByText("買い目に追加"));
    fireEvent.click(screen.getByText("レース開始"));
    fireEvent.click(await screen.findByText("スキップ"));
    await screen.findByText("Fastはお任せで坂路をしました。");
  });

  it("shows the fatigue from a free race on the training tab", async () => {
    let raced = false;
    const tired = { ...state, field: [{ ...state.field[0], fatigue: 25 }, state.field[1]] };
    const outcome = {
      results: [{ rank: 1, name: "Fast", finish_time: 60 }],
      frames: [[600, 300], [2400, 2000]],
      distance: 2400,
      commentary: [],
      won: true,
      payout: 320,
      money: 1220,
    };
    mockBackend(undefined, undefined, undefined, {
      get_game_state: () => (raced ? { ...tired, money: 1220 } : state),
      place_bets: () => {
        raced = true;
        return outcome;
      },
    });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByText("買い目に追加"));
    fireEvent.click(screen.getByText("レース開始"));
    fireEvent.click(await screen.findByText("スキップ"));
    await screen.findByText("的中! 配当 320");
    fireEvent.click(screen.getByRole("tab", { name: "育成" }));
    await waitFor(() =>
      expect(screen.getByRole("meter", { name: "Fastの疲労度" }).getAttribute("aria-valuenow")).toBe("25")
    );
  });

  it("says why a horse cannot run: too tired or resting", async () => {
    const tiredCard = {
      ...card,
      my_horses: [
        { name: "Fast", eligible: false, entered: false, fatigue: 55, resting: false },
        { name: "Slow", eligible: false, entered: false, fatigue: 10, resting: true },
      ],
    };
    const horses = { ...state, field: [{ ...state.field[0], fatigue: 55 }, { ...state.field[1], fatigue: 10, resting: true }] };
    mockBackend(undefined, undefined, undefined, { get_game_state: horses, get_calendar: calendarData, get_race_card: cardFor(tiredCard) });
    render(<App />);
    await ready();
    await chooseRace();
    expect(screen.getByText("(疲労 55)")).toBeTruthy();
    expect(screen.getByText("(休養中)")).toBeTruthy();
  });

  it("puts a menko or a shadow roll on one of my horses from the training tab", async () => {
    const hooded = { ...state, field: [{ ...state.field[0], gear: { menko: true, shadow_roll: false } }, state.field[1]] };
    const setGear = vi.fn(() => hooded);
    calendarBackend(undefined, { set_gear: setGear });
    render(<App />);
    await ready();
    const cards = () => vi.mocked(invoke).mock.calls.filter((c) => c[0] === "get_race_card").length;
    const before = cards();
    fireEvent.click(screen.getByRole("tab", { name: "育成" }));
    const train = screen.getByRole("tabpanel", { name: "育成" });
    fireEvent.click(within(train).getByLabelText("Fastのメンコ"));
    await waitFor(() => expect(setGear).toHaveBeenCalledWith({ horse: "Fast", menko: true, shadowRoll: false, blinkers: false }));
    await waitFor(() => expect((within(train).getByLabelText("Fastのメンコ") as HTMLInputElement).checked).toBe(true));
    await waitFor(() => expect(cards()).toBeGreaterThan(before)); // the odds are asked for again
    fireEvent.click(within(train).getByLabelText("Fastのシャドーロール"));
    await waitFor(() => expect(setGear).toHaveBeenCalledWith({ horse: "Fast", menko: true, shadowRoll: true, blinkers: false }));
  });

  it("sets gear only on the training tab, not when entering a race", async () => {
    calendarBackend();
    render(<App />);
    await ready();
    await chooseRace();
    expect(screen.queryByLabelText("Fastのメンコ")).toBeNull();
    expect(screen.queryByLabelText("Fastのシャドーロール")).toBeNull();
  });

  it("shows each horse's sex, age and record on the training tab", async () => {
    const horses = {
      ...state,
      field: [
        { ...state.field[0], age: 5, record: { starts: 12, wins: 4 } },
        { ...state.field[1], age: 3, record: { starts: 0, wins: 0 } },
      ],
    };
    mockBackend(undefined, undefined, undefined, { get_game_state: horses });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByRole("tab", { name: "育成" }));
    const train = screen.getByRole("tabpanel", { name: "育成" });
    const fast = within(train).getByText("Fast").closest("tr")!;
    expect(within(fast).getByText("牡5")).toBeTruthy();
    expect(within(fast).getByText("12戦4勝・オープン")).toBeTruthy();
    const slow = within(train).getByText("Slow").closest("tr")!;
    expect(within(slow).getByText("牝3")).toBeTruthy();
    expect(within(slow).getByText("0戦0勝・新馬")).toBeTruthy();
  });

  it("shows the gear every runner races in", async () => {
    const geared = {
      ...card,
      my_horses: [{ name: "Fast", eligible: true, entered: true, gear: { menko: true, shadow_roll: false } }, card.my_horses[1]],
      entrants: [
        { ...card.entrants[0], gear: { menko: true, shadow_roll: false } },
        { ...card.entrants[1], gear: { menko: true, shadow_roll: true } },
        { ...card.entrants[2], gear: { menko: false, shadow_roll: false } },
      ],
    };
    mockBackend(undefined, undefined, undefined, { get_calendar: calendarData, get_race_card: cardFor(geared) });
    render(<App />);
    await ready();
    await chooseRace();
    expect(screen.getByLabelText("馬具: メンコ・シャドーロール")).toBeTruthy();
    expect(screen.getAllByLabelText("馬具: メンコ").length).toBe(1);
  });

  // ---- betting tickets: 単勝 複勝 馬単 馬連 三連単 三連複 ----

  const kindSelect = () => screen.getByLabelText("賭け方") as HTMLSelectElement;
  const chooseKind = (kind: string) => fireEvent.change(kindSelect(), { target: { value: kind } });
  const horseSelect = (label: string) => screen.getByLabelText(label) as HTMLSelectElement;
  const runButton = () => screen.getByText("レース開始") as HTMLButtonElement;
  const quoteCalls = () => vi.mocked(invoke).mock.calls.filter((c) => c[0] === "quote_bet");

  it("offers all seven kinds of ticket", async () => {
    mockBackend();
    render(<App />);
    await ready();
    const options = Array.from(kindSelect().options).map((o) => [o.value, o.textContent]);
    expect(options).toEqual([
      ["win", "単勝(1着)"],
      ["place", "複勝(3着以内)"],
      ["wide", "ワイド(2頭が3着以内)"],
      ["exacta", "馬単(1着→2着の順)"],
      ["quinella", "馬連(1・2着、順不問)"],
      ["trifecta", "三連単(1→2→3着の順)"],
      ["trio", "三連複(1〜3着、順不問)"],
    ]);
  });

  it("asks for one horse for 単勝 and 複勝", async () => {
    mockBackend();
    render(<App />);
    await ready();
    for (const kind of ["win", "place"]) {
      chooseKind(kind);
      expect(horseSelect("馬")).toBeTruthy();
      expect(screen.queryByLabelText("2着")).toBeNull();
      expect(screen.queryByLabelText("2頭目")).toBeNull();
    }
  });

  it("asks for two horses in order for 馬単", async () => {
    mockBackend();
    render(<App />);
    await ready();
    chooseKind("exacta");
    expect(horseSelect("1着")).toBeTruthy();
    expect(horseSelect("2着")).toBeTruthy();
    expect(screen.queryByLabelText("3着")).toBeNull();
    expect(screen.queryByLabelText("馬")).toBeNull();
  });

  it("asks for two horses in any order for 馬連", async () => {
    mockBackend();
    render(<App />);
    await ready();
    chooseKind("quinella");
    expect(horseSelect("1頭目")).toBeTruthy();
    expect(horseSelect("2頭目")).toBeTruthy();
    expect(screen.queryByLabelText("3頭目")).toBeNull();
  });

  it("asks for three horses in order for 三連単 and in any order for 三連複", async () => {
    calendarBackend();
    render(<App />);
    await ready();
    await chooseRace();
    chooseKind("trifecta");
    for (const l of ["1着", "2着", "3着"]) expect(horseSelect(l)).toBeTruthy();
    chooseKind("trio");
    for (const l of ["1頭目", "2頭目", "3頭目"]) expect(horseSelect(l)).toBeTruthy();
    expect(screen.queryByLabelText("1着")).toBeNull();
  });

  it("starts with different horses chosen, in field order", async () => {
    calendarBackend();
    render(<App />);
    await ready();
    await chooseRace();
    chooseKind("trifecta");
    expect([horseSelect("1着").value, horseSelect("2着").value, horseSelect("3着").value]).toEqual([
      "Fast",
      "ライバルA",
      "ライバルB",
    ]);
  });

  it("every select offers every runner", async () => {
    calendarBackend();
    render(<App />);
    await ready();
    await chooseRace();
    chooseKind("exacta");
    const values = (l: string) => Array.from(horseSelect(l).options).map((o) => o.value);
    expect(values("1着")).toEqual(["Fast", "ライバルA", "ライバルB"]);
    expect(values("2着")).toEqual(["Fast", "ライバルA", "ライバルB"]);
  });

  it("refuses the same horse twice and says so", async () => {
    mockBackend();
    render(<App />);
    await ready();
    chooseKind("exacta");
    expect((screen.getByText("買い目に追加") as HTMLButtonElement).disabled).toBe(false);
    fireEvent.change(horseSelect("2着"), { target: { value: "Fast" } }); // 1着 is Fast too
    expect(screen.getByText("同じ馬は選べません。")).toBeTruthy();
    expect((screen.getByText("買い目に追加") as HTMLButtonElement).disabled).toBe(true);
    fireEvent.change(horseSelect("2着"), { target: { value: "Slow" } });
    expect(screen.queryByText("同じ馬は選べません。")).toBeNull();
    expect((screen.getByText("買い目に追加") as HTMLButtonElement).disabled).toBe(false);
  });

  it("says when the field is too small for the ticket", async () => {
    mockBackend(); // the free race has only two horses in this test
    render(<App />);
    await ready();
    chooseKind("trifecta");
    expect(screen.getByText("この馬券には3頭以上の出走馬が必要です。")).toBeTruthy();
    expect((screen.getByText("買い目に追加") as HTMLButtonElement).disabled).toBe(true);
    chooseKind("exacta");
    expect(screen.queryByText(/出走馬が必要です/)).toBeNull();
    expect((screen.getByText("買い目に追加") as HTMLButtonElement).disabled).toBe(false);
  });

  it("shows the odds of a single-horse ticket without asking the backend", async () => {
    mockBackend();
    render(<App />);
    await ready();
    expect(screen.getByText("オッズ: x3.2")).toBeTruthy();
    chooseKind("place");
    expect(screen.getByText("オッズ: x1.5")).toBeTruthy();
    expect(quoteCalls().length).toBe(0);
  });

  it("asks the backend for the odds of a combination and shows the payout it would make", async () => {
    mockBackend(undefined, undefined, undefined, { quote_bet: 45.6 });
    render(<App />);
    await ready();
    chooseKind("exacta");
    await screen.findByText("オッズ: x45.6");
    expect(screen.getByText("的中時の払い戻し: 4560")).toBeTruthy(); // 100 x 45.6
    expect(quoteCalls()[0][1]).toEqual({ horses: ["Fast", "Slow"], kind: "exacta", race: null });
    fireEvent.change(screen.getByLabelText("賭け金"), { target: { value: "200" } });
    await waitFor(() => expect(screen.getByText("的中時の払い戻し: 9120")).toBeTruthy());
  });

  it("asks again when the selection or the kind changes", async () => {
    mockBackend(undefined, undefined, undefined, {
      quote_bet: (a: { kind: string; horses: string[] }) => (a.kind === "exacta" ? 5.5 : 7.7),
    });
    render(<App />);
    await ready();
    chooseKind("exacta");
    await screen.findByText("オッズ: x5.5");
    // swap the two horses (the first change makes a duplicate, which is not asked about)
    fireEvent.change(horseSelect("1着"), { target: { value: "Slow" } });
    fireEvent.change(horseSelect("2着"), { target: { value: "Fast" } });
    await waitFor(() => expect(quoteCalls().length).toBe(2));
    expect((quoteCalls()[1][1] as { horses: string[] }).horses).toEqual(["Slow", "Fast"]);
    chooseKind("quinella");
    await screen.findByText("オッズ: x7.7");
  });

  it("does not ask for the odds of a ticket that cannot be bought", async () => {
    mockBackend();
    render(<App />);
    await ready();
    chooseKind("exacta");
    await waitFor(() => expect(quoteCalls().length).toBe(1));
    fireEvent.change(horseSelect("2着"), { target: { value: "Fast" } }); // duplicate
    chooseKind("trifecta"); // too few horses
    await new Promise((r) => setTimeout(r, 30));
    expect(quoteCalls().length).toBe(1);
    expect(screen.queryByText(/^オッズ: /)).toBeNull();
  });

  it("includes the race in the quote for a calendar race", async () => {
    calendarBackend(undefined, { quote_bet: 99.9 });
    render(<App />);
    await ready();
    await chooseRace();
    chooseKind("trio");
    await screen.findByText("オッズ: x99.9");
    const call = quoteCalls().pop()!;
    expect(call[1]).toEqual({
      horses: ["Fast", "ライバルA", "ライバルB"],
      kind: "trio",
      race: { day: "sat", venue: "kyoto", race_no: 11 },
    });
  });

  it("shows why a quote failed", async () => {
    mockBackend(undefined, undefined, undefined, { quote_bet: new Error("そのレースは今週は開催されません") });
    render(<App />);
    await ready();
    chooseKind("quinella");
    await screen.findByText("そのレースは今週は開催されません");
  });

  it("buys an ordered ticket with the horses in the order chosen", async () => {
    mockBackend({
      results: [{ rank: 1, name: "Fast", finish_time: 60 }, { rank: 2, name: "Slow", finish_time: 61 }],
      frames: [[2400, 2300]],
      distance: 2400,
      commentary: [],
      won: false,
      payout: 0,
      money: 900,
    });
    render(<App />);
    await ready();
    chooseKind("exacta");
    fireEvent.change(horseSelect("1着"), { target: { value: "Slow" } });
    fireEvent.change(horseSelect("2着"), { target: { value: "Fast" } });
    fireEvent.click(screen.getByText("買い目に追加"));
    fireEvent.click(runButton());
    fireEvent.click(await screen.findByText("スキップ"));
    const bet = vi.mocked(invoke).mock.calls.find((c) => c[0] === "place_bets")!;
    expect(bet[1]).toMatchObject({ tickets: [{ horses: ["Slow", "Fast"], kind: "exacta", amount: 100 }], race: null });
    // the history shows the order with an arrow
    await waitFor(() => expect(screen.getByText("第1R 馬単 Slow→Fast 100 → はずれ -100 (優勝: Fast)")).toBeTruthy());
  });

  it("writes unordered tickets with hyphens in the history", async () => {
    mockBackend({
      results: [{ rank: 1, name: "Fast", finish_time: 60 }, { rank: 2, name: "Slow", finish_time: 61 }],
      frames: [[2400, 2300]],
      distance: 2400,
      commentary: [],
      won: true,
      payout: 500,
      money: 1400,
    });
    render(<App />);
    await ready();
    chooseKind("quinella");
    fireEvent.click(screen.getByText("買い目に追加"));
    fireEvent.click(runButton());
    fireEvent.click(await screen.findByText("スキップ"));
    await waitFor(() => expect(screen.getByText("第1R 馬連 Fast-Slow 100 → 的中 +400 (優勝: Fast)")).toBeTruthy());
    expect(screen.getByText("的中! 配当 500")).toBeTruthy();
  });

  it("shows every kind of ticket in a loaded history", async () => {
    mockBackend();
    render(<App />);
    await ready();
    const entry = (kind: string, horse: string, round: number) => ({
      kind, round, horse, horses: horse.split(/[→-]/), stake: 100, won: false, payout: 0, winner: "Fast",
    });
    vi.mocked(invoke).mockResolvedValueOnce({
      ...state,
      round: 4,
      history: [
        entry("quinella", "Fast-Slow", 1),
        entry("trifecta", "Fast→Slow→A", 2),
        entry("trio", "Fast-Slow-A", 3),
        entry("exacta", "Slow→Fast", 4),
      ],
    });
    fireEvent.click(screen.getByText("ロード"));
    for (const text of [
      "第1R 馬連 Fast-Slow 100 → はずれ -100 (優勝: Fast)",
      "第2R 三連単 Fast→Slow→A 100 → はずれ -100 (優勝: Fast)",
      "第3R 三連複 Fast-Slow-A 100 → はずれ -100 (優勝: Fast)",
      "第4R 馬単 Slow→Fast 100 → はずれ -100 (優勝: Fast)",
    ]) {
      await waitFor(() => expect(screen.getByText(text)).toBeTruthy());
    }
  });

  it("buys a three-horse ticket in a calendar race and refreshes from the server", async () => {
    let stateCalls = 0;
    calendarBackend(raceOutcome, {
      get_game_state: () => (++stateCalls === 1 ? state : { ...state, week: 2, money: 1140, round: 1 }),
    });
    render(<App />);
    await ready();
    await chooseRace();
    chooseKind("trifecta");
    fireEvent.click(screen.getByText("買い目に追加"));
    fireEvent.click(runButton());
    fireEvent.click(await screen.findByText("スキップ"));
    const bet = vi.mocked(invoke).mock.calls.find((c) => c[0] === "place_bets")!;
    expect(bet[1]).toMatchObject({
      tickets: [{ horses: ["Fast", "ライバルA", "ライバルB"], kind: "trifecta" }],
      race: { day: "sat", venue: "kyoto", race_no: 11 },
    });
    await waitFor(() => expect(screen.getByText("第2週(1月)・1年目")).toBeTruthy());
  });

  // ---- which of my horses run (出走登録), and the rivals that fill the field ----

  const STABLE = ["アルファ", "ベータ", "ガンマ", "デルタ", "イプシロン"];
  const rivalRunner = (name: string, odds: number) => ({
    name, speed: 80, stamina: 75, power: 60, bad_track: 60, condition: "normal", sex: "male", age: 5,
    dirt: 0, mine: false, odds, place_odds: 2,
  });
  /** A free-race card in which `entered` of my horses run among two rivals. */
  const entryCard = (entered: string[], ineligible: string[] = []) => ({
    ...freeCard,
    my_horses: STABLE.map((name) => ({ name, eligible: !ineligible.includes(name), entered: entered.includes(name) })),
    entrants: [
      ...entered.map((name) => ({ ...rivalRunner(name, 3), mine: true })),
      rivalRunner("ライバルX", 5.5),
      rivalRunner("ライバルY", 8.8),
    ],
  });
  const entryBackend = (entered: string[], ineligible: string[] = []) => {
    const current = { entered };
    mockBackend(undefined, undefined, undefined, {
      get_race_card: () => entryCard(current.entered, ineligible),
      set_entries: (a: { horses: string[] }) => {
        current.entered = a.horses;
      },
    });
  };
  const entryBox = (name: string) => screen.getByLabelText(`出走登録: ${name}`) as HTMLInputElement;

  it("asks for the free race's field when the game starts", async () => {
    mockBackend();
    render(<App />);
    await ready();
    const call = vi.mocked(invoke).mock.calls.find((c) => c[0] === "get_race_card")!;
    expect(call[1]).toEqual({ race: null });
  });

  it("shows rivals alongside my horses in the free race, marking mine", async () => {
    entryBackend(["アルファ", "ベータ"]);
    render(<App />);
    await ready();
    expect(screen.getByText("ライバルX", { selector: "td.name" })).toBeTruthy();
    expect(screen.getByText("ライバルY", { selector: "td.name" })).toBeTruthy();
    expect(screen.getAllByLabelText("自馬").length).toBe(2);
    const horses = Array.from((screen.getByLabelText("馬") as HTMLSelectElement).options).map((o) => o.value);
    expect(horses).toEqual(["アルファ", "ベータ", "ライバルX", "ライバルY"]);
  });

  it("lists every horse of the stable on the entry list with the limit", async () => {
    entryBackend(["アルファ"]);
    render(<App />);
    await ready();
    expect(screen.getByText("出走登録(最大3頭)")).toBeTruthy();
    expect(screen.getByText("自分の馬は最大3頭まで出走できます。残りは相手馬です。")).toBeTruthy();
    for (const name of STABLE) expect(entryBox(name)).toBeTruthy();
    expect(entryBox("アルファ").checked).toBe(true);
    expect(entryBox("ベータ").checked).toBe(false);
  });

  it("stops offering more horses once the limit is reached", async () => {
    entryBackend(["アルファ", "ベータ", "ガンマ"]);
    render(<App />);
    await ready();
    for (const name of ["アルファ", "ベータ", "ガンマ"]) expect(entryBox(name).disabled).toBe(false); // can be taken out
    for (const name of ["デルタ", "イプシロン"]) expect(entryBox(name).disabled).toBe(true);
  });

  it("allows more horses while under the limit", async () => {
    entryBackend(["アルファ"]);
    render(<App />);
    await ready();
    for (const name of STABLE) expect(entryBox(name).disabled).toBe(false);
  });

  it("disables horses that do not meet the race's conditions and says so", async () => {
    entryBackend(["アルファ"], ["ガンマ", "デルタ"]);
    render(<App />);
    await ready();
    expect(entryBox("ガンマ").disabled).toBe(true);
    expect(entryBox("デルタ").disabled).toBe(true);
    expect(entryBox("ベータ").disabled).toBe(false);
    expect(screen.getAllByText("(条件外)").length).toBe(2);
  });

  it("enters a horse and reloads the field", async () => {
    entryBackend(["アルファ"]);
    render(<App />);
    await ready();
    fireEvent.click(entryBox("デルタ"));
    await waitFor(() => expect(entryBox("デルタ").checked).toBe(true));
    const call = vi.mocked(invoke).mock.calls.find((c) => c[0] === "set_entries")!;
    expect(call[1]).toEqual({ race: null, horses: ["アルファ", "デルタ"] });
    // the field now has the new horse, marked as mine
    expect(screen.getByText("デルタ", { selector: "td.name" })).toBeTruthy();
    expect(screen.getAllByLabelText("自馬").length).toBe(2);
  });

  it("takes a horse out of the race", async () => {
    entryBackend(["アルファ", "ベータ"]);
    render(<App />);
    await ready();
    fireEvent.click(entryBox("アルファ"));
    await waitFor(() => expect(entryBox("アルファ").checked).toBe(false));
    const call = vi.mocked(invoke).mock.calls.find((c) => c[0] === "set_entries")!;
    expect(call[1]).toEqual({ race: null, horses: ["ベータ"] });
    expect(screen.queryByText("アルファ", { selector: "td.name" })).toBeNull();
  });

  it("can enter none of my horses at all", async () => {
    entryBackend(["アルファ"]);
    render(<App />);
    await ready();
    fireEvent.click(entryBox("アルファ"));
    await waitFor(() => expect(entryBox("アルファ").checked).toBe(false));
    expect(screen.queryAllByLabelText("自馬").length).toBe(0);
    const call = vi.mocked(invoke).mock.calls.find((c) => c[0] === "set_entries")!;
    expect(call[1]).toEqual({ race: null, horses: [] });
  });

  it("enters horses for a calendar race under that race's key", async () => {
    const current = { entered: ["アルファ"] };
    mockBackend(undefined, undefined, undefined, {
      get_calendar: calendarData,
      get_race_card: (a: { race?: unknown }) =>
        a.race ? { ...entryCard(current.entered), race: g3, track: "yielding" } : entryCard(current.entered),
      set_entries: (a: { horses: string[] }) => {
        current.entered = a.horses;
      },
    });
    render(<App />);
    await ready();
    await waitFor(() => expect((screen.getByLabelText("レース") as HTMLSelectElement).options.length).toBe(2));
    fireEvent.change(screen.getByLabelText("レース"), { target: { value: "sat:kyoto:11" } });
    await screen.findByLabelText("馬場状態: 重");
    fireEvent.click(entryBox("ベータ"));
    await waitFor(() => expect(entryBox("ベータ").checked).toBe(true));
    const call = vi.mocked(invoke).mock.calls.find((c) => c[0] === "set_entries")!;
    expect(call[1]).toEqual({ race: { day: "sat", venue: "kyoto", race_no: 11 }, horses: ["アルファ", "ベータ"] });
  });

  it("shows why an entry was refused", async () => {
    mockBackend(undefined, undefined, undefined, {
      get_race_card: () => entryCard(["アルファ"]),
      set_entries: new Error("出走させられる自分の馬は最大3頭までです"),
    });
    render(<App />);
    await ready();
    fireEvent.click(entryBox("ベータ"));
    await screen.findByText("出走させられる自分の馬は最大3頭までです");
    expect(entryBox("ベータ").checked).toBe(false);
  });

  it("cannot change the entries while a race is being replayed", async () => {
    mockBackend(
      {
        results: [{ rank: 1, name: "アルファ", finish_time: 60 }],
        frames: [[600, 300, 200, 100], [2400, 2000, 1900, 1800]],
        distance: 2400,
        commentary: [],
        won: false,
        payout: 0,
        money: 900,
      },
      undefined,
      undefined,
      { get_race_card: () => entryCard(["アルファ"]) }
    );
    render(<App />);
    await ready();
    fireEvent.click(screen.getByText("レース開始"));
    await screen.findByText("スキップ");
    for (const name of STABLE) expect(entryBox(name).disabled).toBe(true);
  });

  it("does not offer a bet until the field has loaded", async () => {
    let release: (v: unknown) => void = () => {};
    const pending = new Promise((r) => {
      release = r;
    });
    mockBackend(undefined, undefined, undefined, { get_race_card: () => pending });
    render(<App />);
    await waitFor(() => screen.getByText("所持金: 1000"));
    expect((screen.getByText("レース開始") as HTMLButtonElement).disabled).toBe(true);
    await act(async () => {
      release(entryCard(["アルファ"]));
    });
    await waitFor(() => expect((screen.getByText("レース開始") as HTMLButtonElement).disabled).toBe(false));
  });

  // ---- prize money for the owner (賞金) ----

  const prizeOutcome = (prizes: { name: string; rank: number; amount: number }[]) => ({
    results: [
      { rank: 1, name: "ライバルA", finish_time: 90.1 },
      { rank: 2, name: "Fast", finish_time: 90.5 },
      { rank: 3, name: "ライバルB", finish_time: 91 },
    ],
    frames: [[2300, 2400, 2000]],
    commentary: [],
    runners: ["Fast", "ライバルA", "ライバルB"],
    race_name: null,
    distance: 1600,
    won: false,
    payout: 0,
    prizes,
    prize_total: prizes.reduce((t, p) => t + p.amount, 0),
    money: 1000 - 100 + prizes.reduce((t, p) => t + p.amount, 0),
  });

  const finishRace = async () => {
    fireEvent.click(screen.getByText("買い目に追加"));
    fireEvent.click(screen.getByText("レース開始"));
    fireEvent.click(await screen.findByText("スキップ"));
  };

  it("shows the prize money my horses earned, with each horse's place", async () => {
    mockBackend(prizeOutcome([{ name: "Fast", rank: 2, amount: 34 }, { name: "Slow", rank: 5, amount: 9 }]));
    render(<App />);
    await ready();
    await finishRace();
    await waitFor(() => expect(screen.getByText("賞金: +43")).toBeTruthy());
    expect(screen.getByText("Fast 2着 +34")).toBeTruthy();
    expect(screen.getByText("Slow 5着 +9")).toBeTruthy();
  });

  it("shows no prize section when none of my horses placed", async () => {
    mockBackend(prizeOutcome([]));
    render(<App />);
    await ready();
    await finishRace();
    await waitFor(() => screen.getByText("はずれ。-100"));
    expect(screen.queryByText(/^賞金: /)).toBeNull();
  });

  it("copes with outcomes that carry no prize information", async () => {
    mockBackend({ ...prizeOutcome([]), prizes: undefined, prize_total: undefined });
    render(<App />);
    await ready();
    await finishRace();
    await waitFor(() => screen.getByText("はずれ。-100"));
    expect(screen.queryByText(/^賞金: /)).toBeNull();
  });

  it("shows the prize in the history and counts it in the net result", async () => {
    mockBackend();
    render(<App />);
    await ready();
    vi.mocked(invoke).mockResolvedValueOnce({
      ...state,
      round: 1,
      history: [
        { kind: "win", race: "初春マイル賞", round: 1, horse: "Fast", horses: ["Fast"], stake: 100, won: false, payout: 0, prize: 86, winner: "ライバルA" },
      ],
    });
    fireEvent.click(screen.getByText("ロード"));
    await waitFor(() =>
      expect(screen.getByText("第1R 初春マイル賞 単勝 Fast 100 → はずれ -100 賞金+86 (優勝: ライバルA)")).toBeTruthy()
    );
    expect(screen.getByText("収支: -14")).toBeTruthy(); // -100 stake + 86 prize
  });

  it("leaves old history entries without a prize as they were", async () => {
    mockBackend();
    render(<App />);
    await ready();
    vi.mocked(invoke).mockResolvedValueOnce({
      ...state,
      round: 1,
      history: [{ kind: "win", round: 1, horse: "Fast", stake: 100, won: true, payout: 320, winner: "Fast" }],
    });
    fireEvent.click(screen.getByText("ロード"));
    await waitFor(() => expect(screen.getByText("第1R 単勝 Fast 100 → 的中 +220 (優勝: Fast)")).toBeTruthy());
    expect(screen.getByText("収支: +220")).toBeTruthy();
  });

  it("adds the prize of a finished free race to the displayed balance", async () => {
    mockBackend({ ...prizeOutcome([{ name: "Fast", rank: 1, amount: 50 }]), money: 950 });
    render(<App />);
    await ready();
    await finishRace();
    await waitFor(() => expect(screen.getByText("所持金: 950")).toBeTruthy());
  });

  it("tells how much a win is worth to the owner", async () => {
    calendarBackend();
    render(<App />);
    await ready();
    await chooseRace();
    expect(screen.getByText("1着の馬主賞金: 86(2〜5着にも順位に応じた賞金)")).toBeTruthy();
  });

  it("says the free race pays no prize", async () => {
    mockBackend();
    render(<App />);
    await ready();
    expect(screen.getByText("このレースに賞金はありません。")).toBeTruthy();
    expect(screen.queryByText(/馬主賞金/)).toBeNull();
  });

  // ---- several tickets at once (買い目リスト, ボックス, 流し) ----

  const addTicket = () => screen.getByText("買い目に追加") as HTMLButtonElement;
  const slipText = (text: string) => screen.getByText(text);
  const ticketCalls = (cmd: string) => vi.mocked(invoke).mock.calls.filter((c) => c[0] === cmd);
  const slipBackend = (extra: Record<string, unknown> = {}) =>
    mockBackend(undefined, undefined, undefined, extra);

  it("adds a ticket to the list with its stake and odds", async () => {
    slipBackend({ quote_tickets: () => [12.5] });
    render(<App />);
    await ready();
    chooseKind("exacta");
    fireEvent.change(screen.getByLabelText("賭け金"), { target: { value: "50" } });
    fireEvent.click(addTicket());
    expect(slipText("馬単 Fast→Slow 50")).toBeTruthy();
    expect(slipText("合計 1点 / 50")).toBeTruthy();
    await screen.findByText("x12.5");
    expect(ticketCalls("quote_tickets")[0][1]).toEqual({
      tickets: [{ kind: "exacta", horses: ["Fast", "Slow"], amount: 50 }],
      race: null,
    });
  });

  it("builds up a list of tickets of different kinds and stakes", async () => {
    slipBackend();
    render(<App />);
    await ready();
    fireEvent.click(addTicket()); // 単勝 Fast 100
    chooseKind("quinella");
    fireEvent.change(screen.getByLabelText("賭け金"), { target: { value: "30" } });
    fireEvent.click(addTicket()); // 馬連 Fast-Slow 30
    expect(slipText("単勝 Fast 100")).toBeTruthy();
    expect(slipText("馬連 Fast-Slow 30")).toBeTruthy();
    expect(slipText("合計 2点 / 130")).toBeTruthy();
  });

  it("removes one ticket or clears the whole list", async () => {
    slipBackend();
    render(<App />);
    await ready();
    fireEvent.click(addTicket());
    chooseKind("quinella");
    fireEvent.click(addTicket());
    fireEvent.click(screen.getByLabelText("削除: 単勝 Fast"));
    expect(screen.queryByText("単勝 Fast 100")).toBeNull();
    expect(slipText("合計 1点 / 100")).toBeTruthy();
    fireEvent.click(addTicket());
    fireEvent.click(screen.getByText("買い目をすべて削除"));
    expect(screen.queryByText(/^合計 /)).toBeNull();
  });

  it("does not offer the list until there is something in it", async () => {
    slipBackend();
    render(<App />);
    await ready();
    expect(screen.queryByText("買い目をすべて削除")).toBeNull();
    expect(screen.queryByText(/^合計 /)).toBeNull();
  });

  it("will not add a ticket that cannot be bought", async () => {
    slipBackend();
    render(<App />);
    await ready();
    chooseKind("exacta");
    fireEvent.change(horseSelect("2着"), { target: { value: "Fast" } }); // the same horse twice
    expect(addTicket().disabled).toBe(true);
    chooseKind("trifecta"); // too few horses in this field
    expect(addTicket().disabled).toBe(true);
  });

  it("buys the whole list at once and shows how each ticket turned out", async () => {
    slipBackend({
      place_bets: {
        results: [{ rank: 1, name: "Fast", finish_time: 60 }, { rank: 2, name: "Slow", finish_time: 61 }],
        frames: [[2400, 2300]],
        distance: 2400,
        commentary: [],
        won: true,
        payout: 500,
        tickets: [
          { kind: "win", horses: ["Fast"], label: "Fast", amount: 50, odds: 10, won: true, payout: 500 },
          { kind: "quinella", horses: ["Slow", "Fast"], label: "Slow-Fast", amount: 100, odds: 2, won: false, payout: 0 },
        ],
        prizes: [],
        prize_total: 0,
        money: 1350,
      },
    });
    render(<App />);
    await ready();
    fireEvent.change(screen.getByLabelText("賭け金"), { target: { value: "50" } });
    fireEvent.click(addTicket()); // 単勝 Fast 50
    chooseKind("quinella");
    fireEvent.change(screen.getByLabelText("賭け金"), { target: { value: "100" } });
    fireEvent.click(addTicket());
    fireEvent.click(screen.getByText("レース開始"));
    fireEvent.click(await screen.findByText("スキップ"));
    const call = ticketCalls("place_bets")[0];
    expect(call[1]).toMatchObject({
      tickets: [
        { kind: "win", horses: ["Fast"], amount: 50 },
        { kind: "quinella", horses: ["Fast", "Slow"], amount: 100 },
      ],
      race: null,
    });
    expect(ticketCalls("watch_race").length).toBe(0);
    await waitFor(() => expect(screen.getByText("所持金: 1350")).toBeTruthy());
    expect(screen.getByText("単勝 Fast 50 → 的中 配当 500")).toBeTruthy();
    expect(screen.getByText("馬連 Slow-Fast 100 → はずれ")).toBeTruthy();
    // the list is spent
    expect(screen.queryByText(/^合計 /)).toBeNull();
  });

  it("writes a multiple purchase as one history line", async () => {
    slipBackend({
      place_bets: {
        results: [{ rank: 1, name: "Fast", finish_time: 60 }, { rank: 2, name: "Slow", finish_time: 61 }],
        frames: [[2400, 2300]],
        distance: 2400,
        commentary: [],
        won: false,
        payout: 0,
        tickets: [
          { kind: "win", horses: ["Slow"], label: "Slow", amount: 50, odds: 10, won: false, payout: 0 },
          { kind: "quinella", horses: ["Slow", "Fast"], label: "Slow-Fast", amount: 100, odds: 2, won: false, payout: 0 },
        ],
        prizes: [],
        prize_total: 0,
        money: 850,
      },
    });
    render(<App />);
    await ready();
    fireEvent.click(addTicket());
    chooseKind("quinella");
    fireEvent.click(addTicket());
    fireEvent.click(screen.getByText("レース開始"));
    fireEvent.click(await screen.findByText("スキップ"));
    await waitFor(() => expect(screen.getByText("第1R 複数券 Fast ほか1点 200 → はずれ -200 (優勝: Fast)")).toBeTruthy());
  });

  it("buys nothing when no ticket has been added", async () => {
    slipBackend({
      watch_race: {
        results: [{ rank: 1, name: "Fast", finish_time: 60 }],
        frames: [[2400, 2300]], distance: 2400, commentary: [], won: false, payout: 0, tickets: [], money: 1000,
      },
    });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByText("レース開始"));
    fireEvent.click(await screen.findByText("スキップ"));
    await waitFor(() => expect(screen.getByText("馬券は買っていません。")).toBeTruthy());
    expect(ticketCalls("watch_race").length).toBe(1);
    expect(ticketCalls("place_bets").length).toBe(0);
    expect(screen.queryByText(/はずれ/)).toBeNull();
    expect(screen.getByText("第1R 馬券なし (優勝: Fast)")).toBeTruthy();
  });

  it("says that a race starts without a ticket until one is added", async () => {
    slipBackend();
    render(<App />);
    await ready();
    const hint = "馬券を買うときは「買い目に追加」を押してください。押さずに始めると馬券なしで走ります。";
    expect(screen.getByText(hint)).toBeTruthy();
    fireEvent.click(addTicket());
    expect(screen.queryByText(hint)).toBeNull();
  });

  it("will not buy a list whose total the player cannot pay", async () => {
    slipBackend();
    render(<App />);
    await ready();
    fireEvent.change(screen.getByLabelText("賭け金"), { target: { value: "600" } });
    fireEvent.click(addTicket());
    fireEvent.click(addTicket()); // 1200 in all, against 1000
    expect(screen.getByText("所持金が足りません。")).toBeTruthy();
    expect((screen.getByText("レース開始") as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(screen.getAllByLabelText("削除: 単勝 Fast")[0]);
    expect(screen.queryByText("所持金が足りません。")).toBeNull();
    expect((screen.getByText("レース開始") as HTMLButtonElement).disabled).toBe(false);
  });

  it("shows why a purchase was refused and keeps the list", async () => {
    slipBackend({ place_bets: new Error("一度に買える馬券は200点までです") });
    render(<App />);
    await ready();
    fireEvent.click(addTicket());
    fireEvent.click(screen.getByText("レース開始"));
    await screen.findByText("一度に買える馬券は200点までです");
    expect(slipText("単勝 Fast 100")).toBeTruthy();
  });

  it("clears the list when another race is chosen, since tickets belong to one race", async () => {
    calendarBackend();
    render(<App />);
    await ready();
    fireEvent.click(addTicket());
    expect(slipText("合計 1点 / 100")).toBeTruthy();
    await chooseRace();
    expect(screen.queryByText(/^合計 /)).toBeNull();
  });

  it("shows the odds of every ticket in the list, asked for together", async () => {
    slipBackend({ quote_tickets: (a: { tickets: unknown[] }) => a.tickets.map((_, i) => 3.5 + i) });
    render(<App />);
    await ready();
    fireEvent.click(addTicket());
    chooseKind("quinella");
    fireEvent.click(addTicket());
    await screen.findByText("x3.5");
    expect(screen.getByText("x4.5")).toBeTruthy();
    expect(ticketCalls("quote_tickets").pop()![1]).toMatchObject({ tickets: [{ kind: "win" }, { kind: "quinella" }] });
  });

  it("shows why the odds of the list could not be worked out", async () => {
    slipBackend({ quote_tickets: new Error("そのレースは今週は開催されません") });
    render(<App />);
    await ready();
    fireEvent.click(addTicket());
    await screen.findByText("そのレースは今週は開催されません");
  });

  // ---- ボックス ----

  it("offers box buying for tickets of two or three horses only", async () => {
    slipBackend();
    render(<App />);
    await ready();
    const style = screen.getByLabelText("買い方") as HTMLSelectElement;
    expect(style.disabled).toBe(true); // 単勝
    chooseKind("place");
    expect(style.disabled).toBe(true);
    chooseKind("quinella");
    expect(style.disabled).toBe(false);
    expect(Array.from(style.options).map((o) => o.value)).toEqual(["normal", "box", "flow"]);
  });

  it("boxes the chosen horses into tickets", async () => {
    slipBackend({ expand_tickets: [["Fast", "Slow"]] });
    render(<App />);
    await ready();
    chooseKind("quinella");
    fireEvent.change(screen.getByLabelText("買い方"), { target: { value: "box" } });
    expect(screen.queryByLabelText("1頭目")).toBeNull(); // the pickers give way to the box list
    expect(addTicket().disabled).toBe(true); // nothing chosen yet
    fireEvent.click(screen.getByLabelText("ボックス: Fast"));
    expect(addTicket().disabled).toBe(true); // a quinella needs two
    fireEvent.click(screen.getByLabelText("ボックス: Slow"));
    expect(addTicket().disabled).toBe(false);
    fireEvent.change(screen.getByLabelText("賭け金"), { target: { value: "20" } });
    fireEvent.click(addTicket());
    await screen.findByText("馬連 Fast-Slow 20");
    expect(ticketCalls("expand_tickets")[0][1]).toEqual({
      style: "box", kind: "quinella", axis: null, horses: ["Fast", "Slow"], race: null,
    });
  });

  it("puts every ticket of a box in the list with the same stake", async () => {
    calendarBackend(undefined, {
      expand_tickets: [["Fast", "ライバルA"], ["Fast", "ライバルB"], ["ライバルA", "ライバルB"]],
    });
    render(<App />);
    await ready();
    await chooseRace();
    chooseKind("quinella");
    fireEvent.change(screen.getByLabelText("買い方"), { target: { value: "box" } });
    for (const name of ["Fast", "ライバルA", "ライバルB"]) fireEvent.click(screen.getByLabelText(`ボックス: ${name}`));
    fireEvent.change(screen.getByLabelText("賭け金"), { target: { value: "20" } });
    fireEvent.click(addTicket());
    await screen.findByText("合計 3点 / 60");
    for (const t of ["馬連 Fast-ライバルA 20", "馬連 Fast-ライバルB 20", "馬連 ライバルA-ライバルB 20"]) {
      expect(screen.getByText(t)).toBeTruthy();
    }
    expect(ticketCalls("expand_tickets")[0][1]).toMatchObject({
      style: "box", horses: ["Fast", "ライバルA", "ライバルB"],
    });
  });

  it("explains a refused box", async () => {
    slipBackend({ expand_tickets: new Error("一度に買える馬券は200点までです") });
    render(<App />);
    await ready();
    chooseKind("quinella");
    fireEvent.change(screen.getByLabelText("買い方"), { target: { value: "box" } });
    fireEvent.click(screen.getByLabelText("ボックス: Fast"));
    fireEvent.click(screen.getByLabelText("ボックス: Slow"));
    fireEvent.click(addTicket());
    await screen.findByText("一度に買える馬券は200点までです");
    expect(screen.queryByText(/^合計 /)).toBeNull();
  });

  it("stops the list at 200 tickets", async () => {
    const many = Array.from({ length: 150 }, (_, i) => ["Fast", `X${i}`]);
    slipBackend({ expand_tickets: many });
    render(<App />);
    await ready();
    chooseKind("quinella");
    fireEvent.change(screen.getByLabelText("買い方"), { target: { value: "box" } });
    fireEvent.click(screen.getByLabelText("ボックス: Fast"));
    fireEvent.click(screen.getByLabelText("ボックス: Slow"));
    fireEvent.change(screen.getByLabelText("賭け金"), { target: { value: "1" } });
    fireEvent.click(addTicket());
    await screen.findByText("合計 150点 / 150");
    fireEvent.click(addTicket()); // another 150 would make 300
    await screen.findByText("一度に買える馬券は200点までです。");
    expect(screen.getByText("合計 150点 / 150")).toBeTruthy();
  });

  // ---- 流し ----

  it("flows an axis horse over its partners", async () => {
    calendarBackend(undefined, { expand_tickets: [["Fast", "ライバルA"], ["Fast", "ライバルB"]] });
    render(<App />);
    await ready();
    await chooseRace();
    chooseKind("exacta");
    fireEvent.change(screen.getByLabelText("買い方"), { target: { value: "flow" } });
    expect(screen.getByLabelText("軸")).toBeTruthy();
    expect(addTicket().disabled).toBe(true); // no partners yet
    // the axis is not offered as a partner
    expect(screen.queryByLabelText("相手: Fast")).toBeNull();
    fireEvent.click(screen.getByLabelText("相手: ライバルA"));
    fireEvent.click(screen.getByLabelText("相手: ライバルB"));
    expect(addTicket().disabled).toBe(false);
    fireEvent.click(addTicket());
    await screen.findByText("馬単 Fast→ライバルA 100");
    expect(screen.getByText("馬単 Fast→ライバルB 100")).toBeTruthy();
    expect(ticketCalls("expand_tickets")[0][1]).toEqual({
      style: "flow", kind: "exacta", axis: "Fast", horses: ["ライバルA", "ライバルB"],
      race: { day: "sat", venue: "kyoto", race_no: 11 },
    });
  });

  it("needs enough partners for a three-horse flow", async () => {
    calendarBackend(undefined, { expand_tickets: [["Fast", "ライバルA", "ライバルB"]] });
    render(<App />);
    await ready();
    await chooseRace();
    chooseKind("trio");
    fireEvent.change(screen.getByLabelText("買い方"), { target: { value: "flow" } });
    fireEvent.click(screen.getByLabelText("相手: ライバルA"));
    expect(addTicket().disabled).toBe(true); // a trio flow needs two partners
    fireEvent.click(screen.getByLabelText("相手: ライバルB"));
    expect(addTicket().disabled).toBe(false);
  });

  it("changing the axis takes it out of the partners", async () => {
    calendarBackend();
    render(<App />);
    await ready();
    await chooseRace();
    chooseKind("quinella");
    fireEvent.change(screen.getByLabelText("買い方"), { target: { value: "flow" } });
    fireEvent.click(screen.getByLabelText("相手: ライバルA"));
    fireEvent.change(screen.getByLabelText("軸"), { target: { value: "ライバルA" } });
    expect(screen.queryByLabelText("相手: ライバルA")).toBeNull();
    expect(screen.getByLabelText("相手: Fast")).toBeTruthy();
    expect(addTicket().disabled).toBe(true); // the old partner was dropped with the axis change
  });

  it("goes back to ordinary buying when a single-horse ticket is chosen again", async () => {
    slipBackend();
    render(<App />);
    await ready();
    chooseKind("quinella");
    fireEvent.change(screen.getByLabelText("買い方"), { target: { value: "box" } });
    chooseKind("win");
    expect(horseSelect("馬")).toBeTruthy();
    expect(screen.queryByLabelText("ボックス: Fast")).toBeNull();
    expect((screen.getByLabelText("買い方") as HTMLSelectElement).value).toBe("normal");
  });

  // ---- history from the server ----

  it("reads a multiple purchase in the loaded history", async () => {
    slipBackend();
    render(<App />);
    await ready();
    const entry = (round: number, kind: string, mixed: boolean) => ({
      kind, round, horse: `A-B ほか2点`, horses: ["A", "B"], stake: 300, won: false, payout: 0,
      winner: "Fast", ticket_count: 3, mixed_kinds: mixed,
    });
    vi.mocked(invoke).mockResolvedValueOnce({
      ...state,
      round: 2,
      history: [entry(1, "quinella", false), entry(2, "win", true)],
    });
    fireEvent.click(screen.getByText("ロード"));
    await waitFor(() => expect(screen.getByText("第1R 馬連 A-B ほか2点 300 → はずれ -300 (優勝: Fast)")).toBeTruthy());
    expect(screen.getByText("第2R 複数券 A-B ほか2点 300 → はずれ -300 (優勝: Fast)")).toBeTruthy();
  });

  // ---- breeding from the blood database: traits, outlook, mutations ----

  const traits = (over: Record<string, unknown> = {}) => ({
    lineage: "Ignis", family: "F1", gen: 10, coat_color: "鹿毛", growth_type: "晩成", peak_age: 6,
    aptitude_min: 1600, aptitude_max: 2400, running_style: "差し", personality: "冷静", temper: 55,
    turf: 60, inbreed_vol: 0, mutations: [], ...over,
  });
  const withBlood = (blood: unknown) => ({
    ...state,
    field: [{ ...state.field[0], blood }, state.field[1]],
  });
  const openPedigree = () => fireEvent.click(screen.getByRole("tab", { name: "血統" }));
  const openBreeding = () => fireEvent.click(screen.getByRole("tab", { name: "繁殖" }));

  it("shows each horse's traits from the blood database on its pedigree card", async () => {
    mockBackend(undefined, undefined, undefined, { get_game_state: withBlood(traits()) });
    render(<App />);
    await ready();
    openPedigree();
    for (const t of [
      "毛色: 鹿毛",
      "系統: Ignis",
      "成長型: 晩成(6歳ごろがピーク)",
      "脚質: 差し",
      "性格: 冷静",
      "距離適性: 1600m〜2400m",
      "近親度: なし",
    ]) {
      expect(screen.getByText(t)).toBeTruthy();
    }
    expect(screen.queryByText(/^突然変異:/)).toBeNull();
  });

  it("shows how inbred a horse is and which stats mutated", async () => {
    mockBackend(undefined, undefined, undefined, {
      get_game_state: withBlood(traits({ inbreed_vol: 0.125, mutations: ["speed", "power"] })),
    });
    render(<App />);
    await ready();
    openPedigree();
    expect(screen.getByText("近親度: 0.125(やや強い)")).toBeTruthy();
    expect(screen.getByText("突然変異: スピード・パワー")).toBeTruthy();
  });

  it("copes with horses that have no blood information", async () => {
    mockBackend();
    render(<App />);
    await ready();
    openPedigree();
    expect(screen.queryByText(/^毛色: /)).toBeNull();
    expect(screen.getByText("血統不明")).toBeTruthy();
  });

  const outlookCalls = () => vi.mocked(invoke).mock.calls.filter((c) => c[0] === "breeding_outlook");

  it("previews the inbreeding of the pair chosen for breeding", async () => {
    mockBackend(undefined, undefined, undefined, {
      breeding_outlook: { inbreed_vol: 0.125, penalty: 6.7, nick: null },
    });
    render(<App />);
    await ready();
    openBreeding();
    await screen.findByText("近親度(配合): 0.125(やや強い)");
    expect(screen.getByText("気性への影響: -6.7")).toBeTruthy();
    expect(outlookCalls()[0][1]).toEqual({ sire: "タイセイオー", dam: "ロウバイ" });
  });

  it("shows a clean pairing plainly", async () => {
    mockBackend();
    render(<App />);
    await ready();
    openBreeding();
    await screen.findByText("近親度(配合): なし");
    expect(screen.queryByText(/^気性への影響/)).toBeNull();
    expect(screen.getByText("相性(ニック): なし")).toBeTruthy();
  });

  it("announces a nick with the stat it lifts", async () => {
    mockBackend(undefined, undefined, undefined, {
      breeding_outlook: { inbreed_vol: 0, penalty: 0, nick: { stat: "speed", bonus: 7 } },
    });
    render(<App />);
    await ready();
    openBreeding();
    await screen.findByText("相性(ニック): スピード+7");
  });

  it("asks again when another parent is chosen", async () => {
    const stableWithTwoSires = {
      ...state,
      field: [state.field[0], { ...state.field[1], name: "Other", sex: "male" }, { ...state.field[1], name: "Mare", sex: "female" }],
    };
    mockBackend(undefined, undefined, undefined, {
      get_game_state: { ...stableWithTwoSires, studs: [studSire, { ...studSire, name: "Other" }, studMare] },
      breeding_outlook: (a: { sire: string }) => ({
        inbreed_vol: a.sire === "Other" ? 0.25 : 0,
        penalty: a.sire === "Other" ? 17.8 : 0,
        nick: null,
      }),
    });
    render(<App />);
    await ready();
    openBreeding();
    await screen.findByText("近親度(配合): なし");
    fireEvent.change(screen.getByLabelText("父馬(牡)"), { target: { value: "Other" } });
    await screen.findByText("近親度(配合): 0.250(強い)");
    expect(screen.getByText("気性への影響: -17.8")).toBeTruthy();
  });

  it("shows why an outlook could not be worked out", async () => {
    mockBackend(undefined, undefined, undefined, { breeding_outlook: new Error("父は牡馬、母は牝馬を選んでください") });
    render(<App />);
    await ready();
    openBreeding();
    await screen.findByText("父は牡馬、母は牝馬を選んでください");
  });

  it("shows the birth, with any mutation, when the week brings the foal", async () => {
    const news = ["「ニューホープ」が誕生しました(父 Fast・母 Slow)。 ✦突然変異"];
    mockBackend(undefined, undefined, undefined, { next_week: { ...state, week: 2, news } });
    render(<App />);
    await ready();
    fireEvent.click(screen.getByText("次の週へ"));
    await screen.findByText(news[0]);
  });

  describe("retirement and the stud book", () => {
    const stud = {
      name: "ロウバイ", sex: "female", speed: 70, stamina: 72, power: 60, dirt: 0,
      retired_age: 6, retired_year: 2, pedigree: null,
      blood: { lineage: "Aqua", family: "f1", gen: 10, coat_color: "鹿毛", growth_type: "普通", peak_age: 4,
        aptitude_min: 1600, aptitude_max: 2400, running_style: "差し", personality: "冷静", temper: 60,
        turf: 60, inbreed_vol: 0, mutations: [] },
    };
    const withRoom = { ...state, stable_size: 8, max_studs: 6, studs: [stud], news: [] };

    const openStable = async () => {
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "繁殖" }));
    };
    const toPedigree = () => fireEvent.click(screen.getByRole("tab", { name: "血統" }));

    it("lists the studs with their stats and when they retired", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: withRoom });
      await openStable();
      const book = screen.getByRole("region", { name: "繁殖馬" });
      expect(within(book).getByText("繁殖馬 (1/6)")).toBeTruthy();
      expect(within(book).getByText("ロウバイ")).toBeTruthy();
      expect(within(book).getByText("2年目に6歳で引退")).toBeTruthy();
      expect(within(book).getByText("スピード 70 / スタミナ 72 / パワー 60")).toBeTruthy();
    });

    it("offers studs as parents and lets a foal fill an empty place", async () => {
      const bred = { ...withRoom, money: 500 };
      mockBackend(undefined, undefined, bred, { get_game_state: { ...withRoom, studs: [studSire, stud] } });
      await openStable();
      const options = (label: string) =>
        Array.from((screen.getByLabelText(label) as HTMLSelectElement).options).map((o) => [o.value, o.text]);
      expect(options("母馬(牝)")).toEqual([["ロウバイ", "ロウバイ(繁殖)"]]);
      expect(options("入れ替える馬")[0]).toEqual(["", "空き枠に入れる"]);
      fireEvent.change(screen.getByLabelText("母馬(牝)"), { target: { value: "ロウバイ" } });
      fireEvent.change(screen.getByLabelText("仔馬の名前"), { target: { value: "ニューホープ" } });
      fireEvent.click(screen.getByText("配合 (500)"));
      await waitFor(() => expect(screen.getByText("所持金: 500")).toBeTruthy());
      const call = vi.mocked(invoke).mock.calls.find((c) => c[0] === "breed_horses")!;
      expect(call[1]).toEqual({ sire: "タイセイオー", dam: "ロウバイ", replace: "", name: "ニューホープ" });
    });

    it("retires a racehorse to stud after asking", async () => {
      const after = { ...withRoom, field: [state.field[1]], studs: [stud, { ...stud, name: "Fast", sex: "male" }] };
      const retire = vi.fn(() => after);
      mockBackend(undefined, undefined, undefined, { get_game_state: withRoom, retire_horse: retire });
      const ask = vi.spyOn(window, "confirm").mockReturnValueOnce(false).mockReturnValueOnce(true);
      await openStable();
      toPedigree();
      fireEvent.click(screen.getByRole("button", { name: "Fastを引退させる" }));
      expect(retire).not.toHaveBeenCalled();
      fireEvent.click(screen.getByRole("button", { name: "Fastを引退させる" }));
      await screen.findByText("Fastが引退し、繁殖入りしました。");
      expect(ask).toHaveBeenCalledWith("Fastを引退させて繁殖入りさせますか? 引退した馬はレースに戻れません。");
      expect(retire).toHaveBeenCalledWith({ name: "Fast" });
      fireEvent.click(screen.getByRole("tab", { name: "繁殖" }));
      expect(within(screen.getByRole("region", { name: "繁殖馬" })).getByText("繁殖馬 (2/6)")).toBeTruthy();
      ask.mockRestore();
    });

    it("cannot retire the last racehorse or into a full stud book", async () => {
      const full = { ...withRoom, studs: Array.from({ length: 6 }, (_, i) => ({ ...stud, name: `ハンショク${"アイウエオカ"[i]}` })) };
      mockBackend(undefined, undefined, undefined, { get_game_state: full });
      await openStable();
      toPedigree();
      expect((screen.getByRole("button", { name: "Fastを引退させる" }) as HTMLButtonElement).disabled).toBe(true);
    });

    it("cannot retire the only racehorse", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: { ...withRoom, field: [state.field[0]] } });
      await openStable();
      toPedigree();
      expect((screen.getByRole("button", { name: "Fastを引退させる" }) as HTMLButtonElement).disabled).toBe(true);
    });

    it("releases a stud after asking", async () => {
      const release = vi.fn(() => ({ ...withRoom, studs: [] }));
      mockBackend(undefined, undefined, undefined, { get_game_state: withRoom, release_stud: release });
      const ask = vi.spyOn(window, "confirm").mockReturnValue(true);
      await openStable();
      fireEvent.click(screen.getByRole("button", { name: "ロウバイを繁殖から外す" }));
      await waitFor(() => expect(release).toHaveBeenCalledWith({ name: "ロウバイ" }));
      await screen.findByText("繁殖馬 (0/6)");
      expect(screen.getByText("繁殖馬はいません。")).toBeTruthy();
      ask.mockRestore();
    });

    it("shows the turn-of-the-year retirements as news", async () => {
      const turned = { ...withRoom, week: 1, year: 3, news: ["Fastが引退し、繁殖入りしました。"] };
      mockBackend(undefined, undefined, undefined, { get_game_state: withRoom, next_week: turned });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByText("次の週へ"));
      await screen.findByText("Fastが引退し、繁殖入りしました。");
    });

    it("lists stallions and mares apart, saying which were bought", async () => {
      const stallion = { ...stud, name: "カイバ", sex: "male", bought: true, retired_age: 0, retired_year: 3 };
      mockBackend(undefined, undefined, undefined, { get_game_state: { ...withRoom, studs: [stud, stallion] } });
      await openStable();
      const sires = screen.getByRole("list", { name: "種牡馬" });
      const mares = screen.getByRole("list", { name: "繁殖牝馬" });
      expect(within(sires).getByText("カイバ")).toBeTruthy();
      expect(within(sires).getByText("3年目に購入")).toBeTruthy();
      expect(within(mares).getByText("ロウバイ")).toBeTruthy();
      expect(within(mares).queryByText("カイバ")).toBeNull();
    });

    it("shows the stallion and mare a new game starts with as such", async () => {
      const starter = { ...stud, name: "ハジマリ", starter: true, retired_age: 0, retired_year: 1, record: { starts: 0, wins: 0 } };
      mockBackend(undefined, undefined, undefined, { get_game_state: { ...withRoom, studs: [starter] } });
      await openStable();
      const mares = screen.getByRole("list", { name: "繁殖牝馬" });
      expect(within(mares).getByText("初期の繁殖馬")).toBeTruthy();
      expect(within(mares).queryByText(/歳で引退/)).toBeNull();
      expect(within(mares).queryByText("0戦0勝")).toBeNull();
    });

    const offer = {
      blood_id: 812, name: "ミカヅキエース", sex: "female", speed: 66, stamina: 61, power: 64, dirt: 0, temper: 40,
      price: 860, blood: stud.blood,
    };

    it("lists the stallions and mares on sale with their price", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: withRoom, get_stud_market: [offer] });
      await openStable();
      const market = screen.getByRole("region", { name: "繁殖馬の購入" });
      await within(market).findByText("ミカヅキエース");
      expect(within(market).getByText("スピード 66 / スタミナ 61 / パワー 64")).toBeTruthy();
      expect(within(market).getByRole("button", { name: "ミカヅキエースを購入" }).textContent).toBe("購入 (860)");
    });

    it("lists the stallions and the mares on sale apart", async () => {
      const stallion = { ...offer, blood_id: 813, name: "ライトドラゴン", sex: "male" };
      mockBackend(undefined, undefined, undefined, { get_game_state: withRoom, get_stud_market: [stallion, offer] });
      await openStable();
      const sires = await screen.findByRole("list", { name: "販売中の種牡馬" });
      const mares = screen.getByRole("list", { name: "販売中の繁殖牝馬" });
      expect(within(sires).getByText("ライトドラゴン")).toBeTruthy();
      expect(within(sires).queryByText("ミカヅキエース")).toBeNull();
      expect(within(mares).getByText("ミカヅキエース")).toBeTruthy();
      const market = screen.getByRole("region", { name: "繁殖馬の購入" });
      expect(within(market).getByText("種牡馬 (1)")).toBeTruthy();
      expect(within(market).getByText("繁殖牝馬 (1)")).toBeTruthy();
    });

    it("buys a mare after asking, and asks the market again", async () => {
      const bought = { ...withRoom, money: 140, studs: [stud, { ...stud, name: "ミカヅキエース", bought: true }] };
      const buy = vi.fn(() => bought);
      const market = vi.fn(() => [offer]);
      mockBackend(undefined, undefined, undefined, { get_game_state: withRoom, get_stud_market: market, buy_stud: buy });
      const ask = vi.spyOn(window, "confirm").mockReturnValue(true);
      await openStable();
      fireEvent.click(await screen.findByRole("button", { name: "ミカヅキエースを購入" }));
      await waitFor(() => expect(buy).toHaveBeenCalledWith({ bloodId: 812 }));
      expect(ask).toHaveBeenCalledWith("ミカヅキエースを860で購入しますか?");
      await screen.findByText("ミカヅキエースを購入しました。");
      expect(screen.getByText("所持金: 140")).toBeTruthy();
      await waitFor(() => expect(market.mock.calls.length).toBeGreaterThan(1));
      ask.mockRestore();
    });

    it("cannot buy without the money or with a full stud book", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: { ...withRoom, money: 500 }, get_stud_market: [offer] });
      const { unmount } = render(<App />);
      await screen.findByText("所持金: 500");
      fireEvent.click(screen.getByRole("tab", { name: "繁殖" }));
      const poor = (await screen.findByRole("button", { name: "ミカヅキエースを購入" })) as HTMLButtonElement;
      expect(poor.disabled).toBe(true);
      unmount();
      const full = { ...withRoom, studs: Array.from({ length: 6 }, (_, i) => ({ ...stud, name: `ハンショク${"アイウエオカ"[i]}` })) };
      mockBackend(undefined, undefined, undefined, { get_game_state: full, get_stud_market: [offer] });
      await openStable();
      const crowded = (await screen.findByRole("button", { name: "ミカヅキエースを購入" })) as HTMLButtonElement;
      expect(crowded.disabled).toBe(true);
    });

    it("keeps breeding off the pedigree tab", async () => {
      mockBackend(undefined, undefined, undefined, { get_game_state: withRoom });
      render(<App />);
      await ready();
      fireEvent.click(screen.getByRole("tab", { name: "血統" }));
      expect(screen.queryByLabelText("仔馬の名前")).toBeNull();
      expect(screen.queryByRole("region", { name: "繁殖馬" })).toBeNull();
    });
  });
});
