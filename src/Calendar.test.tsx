import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor, within } from "@testing-library/react";
import Calendar from "./Calendar";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
import { invoke } from "@tauri-apps/api/core";

const calendar = {
  meetings: [
    { venue: "kyoto", round: 1, start_week: 1, end_week: 4 },
    { venue: "nakayama", round: 1, start_week: 1, end_week: 4 },
    { venue: "tokyo", round: 2, start_week: 19, end_week: 22 },
    { venue: "kyoto", round: 3, start_week: 17, end_week: 20 },
    { venue: "niigata", round: 1, start_week: 17, end_week: 20 },
    { venue: "tokyo", round: 3, start_week: 23, end_week: 25 },
  ],
  races: [
    {
      week: 1, day: "sat", race_no: 11, venue: "kyoto", name: "初春マイル賞", grade: "g3",
      surface: "turf", distance: 1600,
      eligibility: { age: "three_up", fillies_only: false, weight: "handicap" }, prize: 4300,
    },
    {
      week: 20, day: "sun", race_no: 11, venue: "tokyo", name: "薫風ヴィーナスマイル", grade: "g1",
      surface: "turf", distance: 1600,
      eligibility: { age: "four_up", fillies_only: true, weight: "fixed" }, prize: 12000,
    },
    {
      week: 20, day: "sat", race_no: 11, venue: "tokyo", name: "初夏スプリントカップ", grade: "g2",
      surface: "turf", distance: 1400,
      eligibility: { age: "three_up", fillies_only: false, weight: "allowance" }, prize: 6000,
    },
    {
      week: 20, day: "sat", race_no: 9, venue: "niigata", name: "越後新緑特別", grade: "open",
      surface: "dirt", distance: 1200,
      eligibility: { age: "four_up", fillies_only: false, weight: "allowance" }, prize: 2500,
    },
    {
      week: 1, day: "sat", race_no: 1, venue: "kyoto", name: "3歳未勝利", grade: "maiden",
      surface: "dirt", distance: 1200,
      eligibility: { age: "three", fillies_only: false, weight: "age" }, prize: 550,
    },
    {
      week: 1, day: "sun", race_no: 5, venue: "kyoto", name: "4歳以上1勝クラス", grade: "win1",
      surface: "turf", distance: 1800,
      eligibility: { age: "four_up", fillies_only: false, weight: "fixed" }, prize: 800,
    },
    {
      week: 16, day: "sun", race_no: 11, venue: "nakayama", name: "疾風賞", grade: "g1",
      surface: "turf", distance: 2000,
      eligibility: { age: "three", fillies_only: false, weight: "age" }, prize: 20000,
    },
  ],
};

async function open() {
  vi.mocked(invoke).mockResolvedValue(calendar);
  render(<Calendar />);
  await screen.findByText("初春マイル賞");
}

describe("Calendar", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("loads the calendar once and starts on week 1", async () => {
    await open();
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledWith("get_calendar");
    expect((screen.getByLabelText("週") as HTMLSelectElement).value).toBe("1");
    expect(screen.getByText("第1週(1月)")).toBeTruthy();
  });

  it("shows the meetings on that week with their round and progress", async () => {
    await open();
    expect(screen.getByText("京都 1回(1/4週目)")).toBeTruthy();
    expect(screen.getByText("中山 1回(1/4週目)")).toBeTruthy();
    expect(screen.queryByText(/東京/)).toBeNull();
  });

  it("describes each race: grade, course, distance, entry conditions and prize", async () => {
    await open();
    const row = screen.getByText("初春マイル賞").closest("tr")!;
    const cells = within(row);
    expect(cells.getByText("G3")).toBeTruthy();
    expect(cells.getByText("京都")).toBeTruthy();
    expect(cells.getByText("芝1600m")).toBeTruthy();
    expect(cells.getByText("3歳以上・ハンデ")).toBeTruthy();
    expect(cells.getByText("4,300万円")).toBeTruthy();
    expect(cells.getByText("土 11R")).toBeTruthy();
  });

  it("jumps to another week and lists races by day then race number", async () => {
    await open();
    fireEvent.change(screen.getByLabelText("週"), { target: { value: "20" } });
    expect(screen.getByText("第20週(5月)")).toBeTruthy();
    const names = screen.getAllByRole("row").slice(1).map((r) => r.textContent ?? "");
    const order = ["越後新緑特別", "初夏スプリントカップ", "薫風ヴィーナスマイル"].map((n) =>
      names.findIndex((t) => t.includes(n))
    );
    expect(order.every((i) => i >= 0)).toBe(true);
    expect(order).toEqual([...order].sort((a, b) => a - b)); // Sat R9, Sat R11, Sun R11
  });

  it("shows fillies-only and dirt conditions", async () => {
    await open();
    fireEvent.change(screen.getByLabelText("週"), { target: { value: "20" } });
    expect(screen.getByText("4歳以上・牝馬限定・定量")).toBeTruthy();
    expect(screen.getByText("ダート1200m")).toBeTruthy();
    expect(screen.getByText("12,000万円")).toBeTruthy();
  });

  it("steps through weeks with the previous and next buttons", async () => {
    await open();
    const prev = screen.getByText("前の週") as HTMLButtonElement;
    const next = screen.getByText("次の週") as HTMLButtonElement;
    expect(prev.disabled).toBe(true);
    fireEvent.click(next);
    expect(screen.getByText("第2週(1月)")).toBeTruthy();
    expect(prev.disabled).toBe(false);
    fireEvent.click(prev);
    expect(screen.getByText("第1週(1月)")).toBeTruthy();
    fireEvent.change(screen.getByLabelText("週"), { target: { value: "52" } });
    expect((screen.getByText("次の週") as HTMLButtonElement).disabled).toBe(true);
  });

  it("filters the week by grade", async () => {
    await open();
    fireEvent.change(screen.getByLabelText("週"), { target: { value: "20" } });
    fireEvent.change(screen.getByLabelText("グレード"), { target: { value: "g1" } });
    expect(screen.getByText("薫風ヴィーナスマイル")).toBeTruthy();
    expect(screen.queryByText("初夏スプリントカップ")).toBeNull();
    expect(screen.queryByText("越後新緑特別")).toBeNull();
    fireEvent.change(screen.getByLabelText("グレード"), { target: { value: "all" } });
    expect(screen.getByText("初夏スプリントカップ")).toBeTruthy();
  });

  it("says so when a week has meetings but no listed races", async () => {
    await open();
    fireEvent.change(screen.getByLabelText("週"), { target: { value: "2" } });
    expect(screen.getByText("この週に掲載のレースはありません。")).toBeTruthy();
    expect(screen.getByText("京都 1回(2/4週目)")).toBeTruthy();
  });

  it("marks weeks that have a G1 in the week list", async () => {
    await open();
    const option = (w: number) =>
      Array.from((screen.getByLabelText("週") as HTMLSelectElement).options).find((o) => o.value === String(w))!;
    expect(option(20).textContent).toContain("★");
    expect(option(16).textContent).toContain("★");
    expect(option(1).textContent).not.toContain("★");
    expect((screen.getByLabelText("週") as HTMLSelectElement).options.length).toBe(52);
  });

  it("shows an error when the calendar cannot be loaded", async () => {
    vi.mocked(invoke).mockRejectedValue("読み込めませんでした");
    render(<Calendar />);
    await waitFor(() => expect(screen.getByText("読み込めませんでした")).toBeTruthy());
  });

  it("lists the class races with their class and can show only them or only the stakes", async () => {
    await open();
    const row = screen.getByText("3歳未勝利").closest("tr")!;
    expect(row.textContent).toContain("未勝利");
    expect(screen.getByText("4歳以上1勝クラス").closest("tr")!.textContent).toContain("1勝");
    fireEvent.change(screen.getByLabelText("グレード"), { target: { value: "class" } });
    expect(screen.getByText("3歳未勝利")).toBeTruthy();
    expect(screen.queryByText("初春マイル賞")).toBeNull();
    fireEvent.change(screen.getByLabelText("グレード"), { target: { value: "stakes" } });
    expect(screen.queryByText("3歳未勝利")).toBeNull();
    expect(screen.getByText("初春マイル賞")).toBeTruthy();
    fireEvent.change(screen.getByLabelText("グレード"), { target: { value: "win1" } });
    expect(screen.getByText("4歳以上1勝クラス")).toBeTruthy();
    expect(screen.queryByText("3歳未勝利")).toBeNull();
  });
});
