// The words and small workings-out the screens share.

import { STAT_LABELS } from "./Bloodline";
import { CalendarRace } from "./Calendar";
import { Ability, BetKind, Condition, CourseFit, Gear, RaceKey, RaceRecord, RaceResult, RunStyle, RunningStyle, Sex, Star, TabId, Track, Weather, WorkoutInfo } from "./types";

/** The class a record puts a horse in. */
export const classOf = (r: RaceRecord) =>
  r.starts === 0 ? "新馬" : r.wins === 0 ? "未勝利" : r.wins <= 3 ? `${r.wins}勝クラス` : "オープン";

export const recordText = (r: RaceRecord) => `${r.starts}戦${r.wins}勝・${classOf(r)}`;

/** The gear a runner wears, in words ("メンコ・シャドーロール"), or "" for none. */
export const gearWords = (g?: Gear | null) =>
  [g?.menko ? "メンコ" : null, g?.shadow_roll ? "シャドーロール" : null, g?.blinkers ? "ブリンカー" : null].filter(Boolean).join("・");

export const statLabel = (stat: string) => STAT_LABELS.find((x) => x.key === stat)?.label ?? stat;

/** The age from which a horse can race, retire or stand as a sire. */
export const DEBUT_AGE = 2;

export const isYoung = (h?: { age?: number }) => (h?.age ?? DEBUT_AGE) < DEBUT_AGE;

export const mutationText = (mutations: string[]) =>
  STAT_LABELS.filter(({ key }) => mutations.includes(key))
    .map(({ label }) => label)
    .join("・");

export const ORDER_LABELS: [RunningStyle, string][] = [["nige", "逃げ"], ["senko", "先行"], ["sashi", "差し"], ["oikomi", "追込"]];

export const starRecord = (s: Star) => `${s.starts}戦${s.wins}勝(G1 ${s.g1_wins}勝)`;

export const FIT_RATING_MARK: Record<CourseFit["rating"], string> = { great: "◎", good: "○", fair: "△", poor: "×" };

export const TURN_LABEL = { left: "左回り", right: "右回り" } as const;

/** Why a course suits a horse or not, in words. */
export const fitReasons = (f: CourseFit) =>
  [
    f.turn === "plus" ? "回りが合う" : f.turn === "minus" ? "回りが合わない" : null,
    f.straight === "plus" ? "直線が合う" : f.straight === "minus" ? "直線が合わない" : null,
    f.hill === "plus" ? "坂に強い" : f.hill === "minus" ? "坂に弱い" : null,
    f.turf === "plus" ? "洋芝が得意" : f.turf === "minus" ? "洋芝が苦手" : null,
  ]
    .filter(Boolean)
    .join("・");

/** The most tickets that can be bought at once. */
export const MAX_TICKETS = 200;

export const FREE_RACE = "free";

export const raceValue = (r: CalendarRace) => `${r.day}:${r.venue}:${r.race_no}`;

export const raceKey = (r: CalendarRace): RaceKey => ({ day: r.day, venue: r.venue, race_no: r.race_no });

export const DIRT_LABEL: Record<number, string> = { [-2]: "×", [-1]: "△", 0: "○", 1: "◎", 2: "★" };

export const DIRT_WORDS: Record<number, string> = { [-2]: "苦手", [-1]: "やや苦手", 0: "普通", 1: "得意", 2: "大得意" };

export const FRAME_MS = 100;

/** The final straight is shown more slowly, to draw it out. */
export const STRETCH_FRAME_MS = 160;

/** What counts as the final stretch on a straight course, in metres. */
export const STRAIGHT_COURSE_FINISH = 400;

export const TABS: { id: TabId; label: string }[] = [
  { id: "race", label: "レース" },
  { id: "train", label: "育成" },
  { id: "stable", label: "厩舎" },
  { id: "pedigree", label: "血統" },
  { id: "breeding", label: "繁殖" },
  { id: "calendar", label: "カレンダー" },
  { id: "info", label: "情報" },
  { id: "honours", label: "実績" },
];

export const SEX_LABEL: Record<Sex, string> = { male: "牡", female: "牝" };

/** 牡, 牝 or セ for a gelding. */
export const sexLabel = (h: { sex: Sex; gelded?: boolean }) => (h.gelded ? "セ" : SEX_LABEL[h.sex]);

export const KIND_LABEL: Record<BetKind, string> = {
  win: "単勝",
  place: "複勝",
  wide: "ワイド",
  exacta: "馬単",
  quinella: "馬連",
  trifecta: "三連単",
  trio: "三連複",
};

export const KIND_OPTIONS: { value: BetKind; label: string }[] = [
  { value: "win", label: "単勝(1着)" },
  { value: "place", label: "複勝" },
  { value: "wide", label: "ワイド(2頭が3着以内)" },
  { value: "exacta", label: "馬単(1着→2着の順)" },
  { value: "quinella", label: "馬連(1・2着、順不問)" },
  { value: "trifecta", label: "三連単(1→2→3着の順)" },
  { value: "trio", label: "三連複(1〜3着、順不問)" },
];

/** How many horses a ticket picks, and whether the order matters. */
export const KIND_PICKS: Record<BetKind, number> = { win: 1, place: 1, wide: 2, exacta: 2, quinella: 2, trifecta: 3, trio: 3 };

export const KIND_ORDERED: Record<BetKind, boolean> = {
  win: false, place: false, wide: false, exacta: true, quinella: false, trifecta: true, trio: false,
};

export const ORDERED_SLOTS = ["1着", "2着", "3着"];

/** The two training centres, east first. */
export const SIDES = ["美浦", "栗東"];

/** People by where they are based; anyone without a side comes last, under no heading. */
export const bySide = <T extends { region?: string }>(people: T[]): [string, T[]][] =>
  ([...SIDES, ""] as string[])
    .map((side): [string, T[]] => [side, people.filter((p) => (p.region ?? "") === side)])
    .filter(([, here]) => here.length > 0);

export const UNORDERED_SLOTS = ["1頭目", "2頭目", "3頭目"];

/** A selection as the history writes it: ordered picks joined with →, unordered with -. */
export const selectionLabel = (kind: BetKind, names: string[]) => names.join(KIND_ORDERED[kind] ? "→" : "-");

export const CONDITION_LABEL: Record<Condition, string> = {
  great: "絶好調",
  good: "好調",
  normal: "普通",
  poor: "不調",
};

export const ABILITY_LABEL: Record<Ability, string> = {
  speed: "スピード",
  stamina: "スタミナ",
  power: "パワー",
  temper: "気性",
  dirt: "ダート適性",
  jump: "飛越",
};

/** What a session of a workout changes, e.g. "スピード+2 パワー+2 気性-2". */
export const gainText = (w: WorkoutInfo, dirtSessions: number) =>
  [
    ...w.gains.map((g) => `${ABILITY_LABEL[g.ability]}${g.ability === "dirt" ? `+${g.amount}/${dirtSessions}` : signed(g.amount)}`),
    w.fatigue ? `疲労+${w.fatigue}` : null,
  ]
    .filter(Boolean)
    .join(" ");

/** Body and mane colours of each coat (毛色). */
export const COAT_COLOURS: Record<string, { body: string; mane: string }> = {
  鹿毛: { body: "#8a5530", mane: "#1d1410" },
  栗毛: { body: "#b8682f", mane: "#d9935a" },
  青毛: { body: "#22222a", mane: "#050507" },
  芦毛: { body: "#c9c9c9", mane: "#8d8d8d" },
};

export const DEFAULT_COAT = "鹿毛";

export const TRACK_LABEL: Record<Track, string> = { good: "良", soft: "稍重", yielding: "重", heavy: "不良" };

export const WEATHER_LABEL: Record<Weather, string> = { sunny: "晴", cloudy: "曇", light_rain: "小雨", rain: "雨", snow: "雪" };

/** A line of the result: "2着 名前 クビ (60.50秒・3番人気)". */
export const resultLine = (r: RaceResult, popularity: number | undefined) =>
  `${r.rank}着 ${r.name}${r.margin ? ` ${r.margin}` : ""} (${r.finish_time.toFixed(2)}秒${popularity ? `・${popularity}番人気` : ""})`;

/** A symbol for how well a horse copes with a heavy track (0-100). */
/** Age at which horses retire to stud at the turn of the year (as the backend's RETIRE_AGE). */
export const RETIRE_AGE = 9;

export const STYLE_WORDS: Record<RunStyle, string> = { nige: "逃げ", senko: "先行", sashi: "差し", oikomi: "追込" };

/** Calm horses run to form; hot ones are erratic and may pull for their head. */
export const temperMark = (v: number) => (v >= 65 ? "◎" : v >= 50 ? "○" : v >= 30 ? "△" : "×");

export const GROWTH_WORDS = { rising: "成長途上", peak: "充実", declining: "下降" } as const;

export const GROWTH_MARK = { rising: "↗", peak: "◎", declining: "↘" } as const;

export const FIT_WORDS = { fit: "適", too_short: "短すぎ", too_long: "長すぎ" } as const;

export const FIT_MARK = { fit: "◎", too_short: "短", too_long: "長" } as const;

export const badTrackMark = (v: number) => (v >= 75 ? "◎" : v >= 60 ? "○" : v >= 45 ? "△" : "×");

export const HISTORY_LIMIT = 50;

export const HISTORY_SHOWN = 10;

/** How long オートプレイ spends on a week, in milliseconds. */
export const AUTOPLAY_SPEEDS = [
  { ms: 1000, label: "ゆっくり" },
  { ms: 500, label: "ふつう" },
  { ms: 200, label: "速い" },
];

export const AUTOPLAY_LOG_LIMIT = 100;

/** オートプレイ stops with less than this many weeks of 預託料 in hand, before horses have to go. */
export const AUTOPLAY_RESERVE_WEEKS = 4;

export const signed = (n: number) => (n > 0 ? `+${n}` : `${n}`);
