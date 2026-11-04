import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export type Venue =
  | "sapporo" | "hakodate" | "fukushima" | "niigata" | "tokyo"
  | "nakayama" | "chukyo" | "kyoto" | "hanshin" | "kokura"
  | "oi" | "kawasaki" | "funabashi" | "sonoda";
export type Grade = "g1" | "g2" | "g3" | "listed" | "open" | "win3" | "win2" | "win1" | "maiden" | "debut";
export type AgeRule = "two" | "three" | "three_up" | "four_up";
export type WeightRule = "age" | "fixed" | "allowance" | "handicap";

interface Meeting {
  venue: Venue;
  round: number;
  start_week: number;
  end_week: number;
}

export interface CalendarRace {
  week: number;
  day: "sat" | "sun";
  race_no: number;
  venue: Venue;
  name: string;
  grade: Grade;
  surface: "turf" | "dirt";
  distance: number;
  eligibility: { age: AgeRule; fillies_only: boolean; weight: WeightRule };
  prize: number;
}

export interface CalendarData {
  meetings: Meeting[];
  races: CalendarRace[];
}

const WEEKS = 52;
export const VENUE_LABEL: Record<Venue, string> = {
  sapporo: "札幌", hakodate: "函館", fukushima: "福島", niigata: "新潟", tokyo: "東京",
  nakayama: "中山", chukyo: "中京", kyoto: "京都", hanshin: "阪神", kokura: "小倉",
  oi: "大井", kawasaki: "川崎", funabashi: "船橋", sonoda: "園田",
};

const LOCAL_VENUES: Venue[] = ["oi", "kawasaki", "funabashi", "sonoda"];
const LOCAL_GRADE: Partial<Record<Grade, string>> = { g1: "JpnI", g2: "JpnII", g3: "JpnIII" };

/** A race's grade as it is written: races on the local tracks are Jpn grades. */
export const gradeLabel = (r: { grade: Grade; venue: Venue }) =>
  (LOCAL_VENUES.includes(r.venue) && LOCAL_GRADE[r.grade]) || GRADE_LABEL[r.grade];
export const GRADE_LABEL: Record<Grade, string> = {
  g1: "G1", g2: "G2", g3: "G3", listed: "L", open: "OP",
  win3: "3勝", win2: "2勝", win1: "1勝", maiden: "未勝利", debut: "新馬",
};
/** Stakes (重賞・リステッド・オープン) are open to any record; the classes below are not. */
export const isStakes = (g: Grade) => g === "g1" || g === "g2" || g === "g3" || g === "listed" || g === "open";

/** Whether a horse with this record may run in a race of this grade (as the backend's `Grade::admits`). */
export const admits = (g: Grade, starts: number, wins: number) =>
  g === "debut" ? starts === 0
  : g === "maiden" ? wins === 0
  : g === "win1" ? wins <= 1
  : g === "win2" ? wins <= 2
  : g === "win3" ? wins <= 3
  : true;

/** Whether a horse of this age and sex meets a race's conditions. */
export const ageAllows = (r: CalendarRace, age: number, female: boolean) => {
  const a = r.eligibility.age;
  const ok = a === "two" ? age === 2 : a === "three" ? age === 3 : a === "three_up" ? age >= 3 : age >= 4;
  return ok && (female || !r.eligibility.fillies_only);
};

type GradeFilter = Grade | "all" | "stakes" | "class";
const AGE_LABEL: Record<AgeRule, string> = { two: "2歳", three: "3歳", three_up: "3歳以上", four_up: "4歳以上" };
const WEIGHT_LABEL: Record<WeightRule, string> = { age: "馬齢", fixed: "定量", allowance: "別定", handicap: "ハンデ" };
export const SURFACE_LABEL = { turf: "芝", dirt: "ダート" } as const;
const DAY_LABEL = { sat: "土", sun: "日" } as const;

/** Month the given week (week 1 = first week of January) mostly falls in. */
export const monthOfWeek = (week: number) => Math.min(12, Math.floor(((week - 1) * 7 + 3) / 30.4) + 1);

export const conditionText = (r: CalendarRace) =>
  [AGE_LABEL[r.eligibility.age], r.eligibility.fillies_only ? "牝馬限定" : null, WEIGHT_LABEL[r.eligibility.weight]]
    .filter(Boolean)
    .join("・");

/** A week's races in the order of the card: Saturday first, then by race number. */
export const racesOfWeek = (races: CalendarRace[], week: number | undefined) =>
  races
    .filter((r) => r.week === week)
    .sort((a, b) => (a.day === b.day ? a.race_no - b.race_no : a.day === "sat" ? -1 : 1));

/** `data` can be passed in when the caller has already loaded the calendar. */
function Calendar({ data: provided }: { data?: CalendarData }) {
  const [fetched, setFetched] = useState<CalendarData | null>(null);
  const [error, setError] = useState("");
  const [week, setWeek] = useState(1);
  const [grade, setGrade] = useState<GradeFilter>("all");

  useEffect(() => {
    if (provided) return;
    invoke<CalendarData>("get_calendar")
      .then(setFetched)
      .catch((e) => setError(String(e)));
  }, [provided]);

  const data = provided ?? fetched;

  if (error) return <p className="error">{error}</p>;
  if (!data) return <p>読み込み中...</p>;

  const g1Weeks = new Set(data.races.filter((r) => r.grade === "g1").map((r) => r.week));
  const meetings = data.meetings.filter((m) => m.start_week <= week && week <= m.end_week);
  const weekRaces = racesOfWeek(data.races, week);
  const races = weekRaces.filter(
    (r) =>
      grade === "all" ||
      (grade === "stakes" && isStakes(r.grade)) ||
      (grade === "class" && !isStakes(r.grade)) ||
      r.grade === grade
  );

  return (
    <main>
      <section className="panel calendar">
        <div className="calendar-nav">
          <button disabled={week <= 1} onClick={() => setWeek(week - 1)}>前の週</button>
          <select aria-label="週" value={week} onChange={(e) => setWeek(Number(e.target.value))}>
            {Array.from({ length: WEEKS }, (_, i) => i + 1).map((w) => (
              <option key={w} value={w}>
                {`${w}週目・${monthOfWeek(w)}月${g1Weeks.has(w) ? " ★" : ""}`}
              </option>
            ))}
          </select>
          <button disabled={week >= WEEKS} onClick={() => setWeek(week + 1)}>次の週</button>
          <label className="grade-filter">
            グレード
            <select aria-label="グレード" value={grade} onChange={(e) => setGrade(e.target.value as GradeFilter)}>
              <option value="all">すべて</option>
              <option value="stakes">特別・重賞のみ</option>
              <option value="class">条件戦のみ</option>
              <option value="g1">G1</option>
              <option value="g2">G2</option>
              <option value="g3">G3</option>
              <option value="listed">L(リステッド)</option>
              <option value="open">OP(オープン)</option>
              <option value="win3">3勝クラス</option>
              <option value="win2">2勝クラス</option>
              <option value="win1">1勝クラス</option>
              <option value="maiden">未勝利</option>
              <option value="debut">新馬</option>
            </select>
          </label>
        </div>

        <h2>{`第${week}週(${monthOfWeek(week)}月)`}</h2>
        <p className="muted">★ はG1のある週です。架空のカレンダーで、重賞・リステッド・オープン特別のほか、新馬・未勝利・1〜3勝クラスの条件戦があります。</p>

        <ul className="meetings">
          {meetings.map((m) => (
            <li key={`${m.venue}-${m.round}`}>
              {`${VENUE_LABEL[m.venue]} ${m.round}回(${week - m.start_week + 1}/${m.end_week - m.start_week + 1}週目)`}
            </li>
          ))}
        </ul>

        {races.length === 0 ? (
          <p className="muted">
            {weekRaces.length === 0 ? "この週に掲載のレースはありません。" : "該当するレースはありません。"}
          </p>
        ) : (
          <table className="race-table">
            <thead>
              <tr>
                <th align="left">日・R</th>
                <th>グレード</th>
                <th align="left">レース名</th>
                <th>競馬場</th>
                <th>コース</th>
                <th align="left">出走条件</th>
                <th align="right">1着賞金</th>
              </tr>
            </thead>
            <tbody>
              {races.map((r) => (
                <tr key={`${r.day}-${r.venue}-${r.race_no}`}>
                  <td>{`${DAY_LABEL[r.day]} ${r.race_no}R`}</td>
                  <td align="center"><span className={`grade ${r.grade}`}>{gradeLabel(r)}</span></td>
                  <td className="name">{r.name}</td>
                  <td align="center">{VENUE_LABEL[r.venue]}</td>
                  <td align="center">{`${SURFACE_LABEL[r.surface]}${r.distance}m`}</td>
                  <td>{conditionText(r)}</td>
                  <td align="right">{`${r.prize.toLocaleString("en-US")}万円`}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </section>
    </main>
  );
}

export default Calendar;
