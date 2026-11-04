//! A fictional race calendar modelled on the structure of the JRA's ten
//! racecourses: meetings (開催), graded races and open stakes with their
//! distances and entry conditions. Every race name is original.

use crate::rivals;
use serde::{Deserialize, Serialize};

pub const WEEKS_PER_YEAR: u8 = 52;
#[cfg(test)]
pub const WEEKS: u8 = WEEKS_PER_YEAR;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Venue {
    Sapporo,
    Hakodate,
    Fukushima,
    Niigata,
    Tokyo,
    Nakayama,
    Chukyo,
    Kyoto,
    Hanshin,
    Kokura,
    /// The local tracks, where the races open to all are run (交流重賞).
    Oi,
    Kawasaki,
    Funabashi,
    Sonoda,
}

impl Venue {
    /// Whether this is a local track rather than one of the ten national courses.
    #[cfg(test)]
    pub fn is_local(self) -> bool {
        matches!(
            self,
            Venue::Oi | Venue::Kawasaki | Venue::Funabashi | Venue::Sonoda
        )
    }
}

#[cfg(test)]
pub const ALL_VENUES: [Venue; 10] = [
    Venue::Sapporo,
    Venue::Hakodate,
    Venue::Fukushima,
    Venue::Niigata,
    Venue::Tokyo,
    Venue::Nakayama,
    Venue::Chukyo,
    Venue::Kyoto,
    Venue::Hanshin,
    Venue::Kokura,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Surface {
    Turf,
    Dirt,
    /// Over fences (障害), on the turf.
    Jump,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Grade {
    G1,
    G2,
    G3,
    Listed,
    Open,
    /// 3勝クラス: for horses with three wins or fewer.
    Win3,
    /// 2勝クラス.
    Win2,
    /// 1勝クラス.
    Win1,
    /// 未勝利: for horses that have yet to win.
    Maiden,
    /// 新馬: for horses that have never raced.
    Debut,
}

impl Grade {
    /// Stakes races (重賞, リステッド, オープン) are open to horses of any record;
    /// the classes below them are for horses that have not won too often.
    pub fn is_stakes(self) -> bool {
        matches!(
            self,
            Grade::G1 | Grade::G2 | Grade::G3 | Grade::Listed | Grade::Open
        )
    }

    /// Whether a horse with this record may run. A horse may always try a class
    /// above its own (格上挑戦), never one below.
    pub fn admits(self, starts: u32, wins: u32) -> bool {
        if self.is_stakes() {
            return true;
        }
        match self {
            Grade::Debut => starts == 0,
            Grade::Maiden => wins == 0,
            Grade::Win1 => wins <= 1,
            Grade::Win2 => wins <= 2,
            Grade::Win3 => wins <= 3,
            _ => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Day {
    Sat,
    Sun,
}

/// Age restriction of a race.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgeRule {
    Two,
    Three,
    ThreeUp,
    FourUp,
}

/// How weights are set: 馬齢 / 定量 / 別定 / ハンデ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WeightRule {
    Age,
    Fixed,
    Allowance,
    Handicap,
}

/// Entry conditions (出走条件).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Eligibility {
    pub age: AgeRule,
    pub fillies_only: bool,
    pub weight: WeightRule,
}

impl Eligibility {
    /// Whether a horse of this age and sex may enter.
    pub fn allows(&self, age: u8, female: bool) -> bool {
        let age_ok = match self.age {
            AgeRule::Two => age == 2,
            AgeRule::Three => age == 3,
            AgeRule::ThreeUp => age >= 3,
            AgeRule::FourUp => age >= 4,
        };
        age_ok && (female || !self.fillies_only)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalendarRace {
    pub week: u8,
    pub day: Day,
    pub race_no: u8,
    pub venue: Venue,
    pub name: String,
    pub grade: Grade,
    pub surface: Surface,
    /// Metres.
    pub distance: u32,
    pub eligibility: Eligibility,
    /// First-place prize in units of 10,000 yen.
    pub prize: u32,
}

/// One meeting (開催): a run of consecutive race weeks at a course.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Meeting {
    pub venue: Venue,
    /// Which meeting of the year this is at the venue (第N回).
    pub round: u8,
    pub start_week: u8,
    pub end_week: u8,
}

impl Meeting {
    /// Racing days: Saturday and Sunday of every week.
    #[cfg(test)]
    pub fn days(&self) -> u32 {
        (self.end_week - self.start_week + 1) as u32 * 2
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Calendar {
    pub meetings: Vec<Meeting>,
    pub races: Vec<CalendarRace>,
}

impl Calendar {
    #[cfg(test)]
    pub fn venues_in_week(&self, week: u8) -> Vec<Venue> {
        self.meetings
            .iter()
            .filter(|m| (m.start_week..=m.end_week).contains(&week))
            .map(|m| m.venue)
            .collect()
    }
}

/// Distances (metres) that actually exist at each course, per surface.
pub fn course_distances(venue: Venue, surface: Surface) -> &'static [u32] {
    use Surface::*;
    use Venue::*;
    match (venue, surface) {
        (_, Jump) => &[2800, 3000, 3200, 3400, 3600, 4000, 4200],
        (Sapporo, Turf) => &[1200, 1500, 1800, 2000, 2600],
        (Sapporo, Dirt) => &[1000, 1700, 2400],
        (Hakodate, Turf) => &[1000, 1200, 1800, 2000, 2600],
        (Hakodate, Dirt) => &[1000, 1700, 2400],
        (Fukushima, Turf) => &[1200, 1800, 2000, 2600],
        (Fukushima, Dirt) => &[1150, 1700, 2400],
        (Niigata, Turf) => &[1000, 1200, 1400, 1600, 1800, 2000, 2200, 2400],
        (Niigata, Dirt) => &[1200, 1800, 2500],
        (Tokyo, Turf) => &[1400, 1600, 1800, 2000, 2400, 2500, 3400],
        (Tokyo, Dirt) => &[1300, 1400, 1600, 2100, 2400],
        (Nakayama, Turf) => &[1200, 1600, 1800, 2000, 2200, 2500, 3200, 3600],
        (Nakayama, Dirt) => &[1200, 1800, 2400, 2500],
        (Chukyo, Turf) => &[1200, 1400, 1600, 2000, 2200],
        (Chukyo, Dirt) => &[1200, 1400, 1800, 1900],
        (Kyoto, Turf) => &[1200, 1400, 1600, 1800, 2000, 2200, 2400, 3000, 3200],
        (Kyoto, Dirt) => &[1200, 1400, 1800, 1900],
        (Hanshin, Turf) => &[1200, 1400, 1600, 1800, 2000, 2200, 2400, 2600, 3000],
        (Hanshin, Dirt) => &[1200, 1400, 1800, 2000],
        (Kokura, Turf) => &[1200, 1800, 2000, 2600],
        (Kokura, Dirt) => &[1000, 1700, 2400],
        (Oi, _) => &[1200, 1600, 1800, 2000],
        (Kawasaki, _) => &[1400, 1600, 2000],
        (Funabashi, _) => &[1200, 1600, 1800],
        (Sonoda, _) => &[1400, 1700],
    }
}

// Short names used only by the data tables below.
const T: Surface = Surface::Turf;
const D: Surface = Surface::Dirt;
const J: Surface = Surface::Jump;
const A2: AgeRule = AgeRule::Two;
const A3: AgeRule = AgeRule::Three;
const U3: AgeRule = AgeRule::ThreeUp;
const U4: AgeRule = AgeRule::FourUp;
const WA: WeightRule = WeightRule::Age;
const WF: WeightRule = WeightRule::Fixed;
const WS: WeightRule = WeightRule::Allowance;
const WH: WeightRule = WeightRule::Handicap;

/// (venue, round, first week, last week). Roughly follows the real year:
/// winter at Nakayama/Kyoto, spring at Tokyo/Hanshin, summer at the northern
/// and Kyushu courses, autumn back at the big courses.
const MEETINGS: [(Venue, u8, u8, u8); 38] = [
    (Venue::Kyoto, 1, 1, 4),
    (Venue::Kyoto, 2, 5, 8),
    (Venue::Kyoto, 3, 17, 20),
    (Venue::Kyoto, 4, 40, 43),
    (Venue::Kyoto, 5, 44, 47),
    (Venue::Nakayama, 1, 1, 4),
    (Venue::Nakayama, 2, 9, 12),
    (Venue::Nakayama, 3, 13, 16),
    (Venue::Nakayama, 4, 36, 39),
    (Venue::Nakayama, 5, 49, 52),
    (Venue::Tokyo, 1, 5, 8),
    (Venue::Tokyo, 2, 19, 22),
    (Venue::Tokyo, 3, 23, 25),
    (Venue::Tokyo, 4, 41, 44),
    (Venue::Tokyo, 5, 45, 48),
    (Venue::Hanshin, 1, 9, 12),
    (Venue::Hanshin, 2, 13, 16),
    (Venue::Hanshin, 3, 23, 26),
    (Venue::Hanshin, 4, 36, 39),
    (Venue::Hanshin, 5, 48, 51),
    (Venue::Chukyo, 1, 3, 4),
    (Venue::Chukyo, 2, 10, 13),
    (Venue::Chukyo, 3, 37, 39),
    (Venue::Chukyo, 4, 49, 52),
    (Venue::Kokura, 1, 5, 8),
    (Venue::Kokura, 2, 27, 30),
    (Venue::Kokura, 3, 31, 34),
    (Venue::Fukushima, 1, 14, 17),
    (Venue::Fukushima, 2, 21, 22),
    (Venue::Fukushima, 3, 27, 30),
    (Venue::Fukushima, 4, 44, 47),
    (Venue::Niigata, 1, 17, 20),
    (Venue::Niigata, 2, 31, 35),
    (Venue::Niigata, 3, 40, 43),
    (Venue::Hakodate, 1, 25, 27),
    (Venue::Hakodate, 2, 28, 30),
    (Venue::Sapporo, 1, 31, 33),
    (Venue::Sapporo, 2, 34, 36),
];

type Row = (
    u8,
    Venue,
    Surface,
    u32,
    AgeRule,
    bool,
    WeightRule,
    Grade,
    &'static str,
);

/// (week, venue, surface, metres, age rule, fillies only, weight rule, grade, name)
#[rustfmt::skip]
fn race_rows() -> Vec<Row> {
    use Grade::*;
    use Venue::*;
    vec![
        // ---- G1 (24), patterned on the JRA's G1 programme
        (8, Tokyo, D, 1600, U4, false, WF, G1, "銀嶺ダートクラシック"),
        (13, Chukyo, T, 1200, U4, false, WF, G1, "春雷スプリント"),
        (14, Hanshin, T, 2000, U4, false, WF, G1, "浪速中距離ステークス"),
        (15, Hanshin, T, 1600, A3, true, WA, G1, "白梅賞"),
        (16, Nakayama, T, 2000, A3, false, WA, G1, "疾風賞"),
        (18, Kyoto, T, 3200, U4, false, WF, G1, "春嵐記念"),
        (19, Tokyo, T, 1600, A3, false, WA, G1, "新緑マイルカップ"),
        (20, Tokyo, T, 1600, U4, true, WF, G1, "薫風ヴィーナスマイル"),
        (21, Tokyo, T, 2400, A3, true, WA, G1, "月下美人賞"),
        (22, Tokyo, T, 2400, A3, false, WA, G1, "王冠ステークス"),
        (23, Tokyo, T, 1600, U3, false, WF, G1, "夏至マイルチャンピオン"),
        (25, Hanshin, T, 2200, U3, false, WF, G1, "夏越グランプリ"),
        (39, Nakayama, T, 1200, U3, false, WF, G1, "秋雷スプリントS"),
        (41, Kyoto, T, 2000, A3, true, WA, G1, "秋桜ティアラ"),
        (42, Kyoto, T, 3000, A3, false, WA, G1, "錦秋ロングステークス"),
        (44, Tokyo, T, 2000, U3, false, WF, G1, "天高馬肥賞"),
        (45, Kyoto, T, 2200, U3, true, WF, G1, "麗人記念"),
        (46, Kyoto, T, 1600, U3, false, WF, G1, "紅葉マイルチャンピオン"),
        (47, Tokyo, T, 2400, U3, false, WF, G1, "国際友好ステークス"),
        (49, Chukyo, D, 1800, U3, false, WF, G1, "師走ダート王決定戦"),
        (50, Hanshin, T, 1600, A2, true, WA, G1, "ジュニアティアラ"),
        (51, Hanshin, T, 1600, A2, false, WA, G1, "ジュニアマイルステークス"),
        (52, Nakayama, T, 2500, U3, false, WF, G1, "グランドフィナーレ記念"),
        (52, Nakayama, T, 2000, A2, false, WA, G1, "未来希望賞"),
        // ---- 交流重賞, on the dirt of the local tracks
        (6, Kawasaki, D, 2000, U4, false, WF, G1, "京浜ダート記念"),
        (9, Funabashi, D, 1800, U4, false, WS, G3, "葛南ダートカップ"),
        (18, Funabashi, D, 1600, U4, false, WF, G1, "新緑砂王マイル"),
        (20, Kawasaki, D, 1600, U4, true, WS, G2, "京浜レディースカップ"),
        (26, Oi, D, 2000, U4, false, WF, G1, "夏至ダート帝王賞"),
        (28, Oi, D, 2000, A3, false, WA, G1, "砂の頂上決戦"),
        (34, Sonoda, D, 1700, U3, false, WS, G3, "播磨ダートステークス"),
        (40, Oi, D, 1200, U3, false, WS, G2, "湾岸スプリント"),
        (51, Oi, D, 2000, U3, false, WF, G1, "帝都ダート大賞典"),
        // ---- over fences (障害重賞)
        (16, Nakayama, J, 4200, U4, false, WF, G1, "春の大障害"),
        (52, Nakayama, J, 4000, U3, false, WF, G1, "暮れの大障害"),
        (10, Hanshin, J, 3000, U4, false, WS, G2, "阪神ジャンプステークス"),
        (19, Tokyo, J, 3200, U4, false, WS, G2, "東京ハイジャンプ"),
        (37, Nakayama, J, 3200, U3, false, WS, G2, "秋の障害ステークス"),
        (6, Kokura, J, 2800, U4, false, WH, G3, "小倉冬ジャンプ"),
        (20, Kyoto, J, 3000, U4, false, WS, G3, "京都ジャンプステークス"),
        (30, Fukushima, J, 2800, U3, false, WH, G3, "夏の障害ハンデ"),
        (42, Niigata, J, 2800, U3, false, WS, G3, "越後ジャンプステークス"),
        // ---- G2
        (2, Kyoto, T, 2400, U4, false, WH, G2, "新年ハンデキャップ"),
        (4, Nakayama, T, 2200, U4, false, WS, G2, "冬晴ゴールドカップ"),
        (6, Kyoto, T, 2200, U4, false, WS, G2, "洛北記念"),
        (9, Nakayama, T, 1800, U4, false, WS, G2, "春先取りカップ"),
        (10, Nakayama, T, 2000, A3, false, WA, G2, "桜前線ステークス"),
        (11, Chukyo, T, 2000, U4, false, WS, G2, "黄金鯱記念"),
        (12, Hanshin, T, 3000, U4, false, WS, G2, "浪速ロングクラシック"),
        (13, Nakayama, T, 1800, A3, false, WA, G2, "春疾風ステークス"),
        (15, Hanshin, T, 1400, U4, true, WS, G2, "春風レディースマイル"),
        (20, Tokyo, T, 2400, A3, false, WA, G2, "若葉の森ステークス"),
        (20, Tokyo, T, 1400, U3, false, WS, G2, "初夏スプリントカップ"),
        (22, Tokyo, T, 2500, U4, false, WH, G2, "麦秋ハンデキャップ"),
        (28, Hakodate, T, 2000, U3, false, WS, G2, "函館湾記念"),
        (32, Niigata, T, 1600, U3, false, WS, G2, "越後マイル記念"),
        (33, Sapporo, T, 2000, U3, false, WS, G2, "北の大地記念"),
        (35, Sapporo, T, 1800, A2, false, WA, G2, "北国ジュニアステークス"),
        (36, Nakayama, T, 2000, A3, true, WA, G2, "薄紅トライアル"),
        (37, Nakayama, T, 2200, U3, false, WS, G2, "秋晴れ大賞"),
        (38, Hanshin, T, 2400, A3, false, WA, G2, "錦秋トライアル"),
        (40, Kyoto, T, 1400, U3, false, WS, G2, "白鷺スプリントカップ"),
        (41, Tokyo, T, 1800, U3, false, WS, G2, "秋麗王冠"),
        (41, Kyoto, T, 2400, U3, false, WS, G2, "古都大賞典"),
        (42, Kyoto, D, 1800, U3, false, WS, G2, "秋の砂王決定ステークス"),
        (43, Kyoto, T, 1400, U3, false, WS, G2, "白鳥スプリントS"),
        (44, Tokyo, T, 2500, U3, false, WH, G2, "木枯らしハンデキャップ"),
        (48, Hanshin, T, 1800, U3, true, WS, G2, "冬薔薇レディース"),
        // ---- G3
        (1, Kyoto, T, 1600, U3, false, WH, G3, "初春マイル賞"),
        (1, Nakayama, T, 2000, U3, false, WH, G3, "中山初春ハンデキャップ"),
        (2, Nakayama, T, 1600, A3, false, WA, G3, "冬芽賞"),
        (3, Chukyo, T, 1400, U3, false, WH, G3, "尾張ハンデキャップ"),
        (3, Kyoto, T, 1800, A3, false, WA, G3, "寒椿ステークス"),
        (4, Chukyo, T, 2000, U3, false, WS, G3, "鳳凰記念"),
        (5, Tokyo, T, 1600, U4, false, WS, G3, "梅香マイル"),
        (5, Kokura, T, 2000, U3, false, WH, G3, "豊前ハンデキャップ"),
        (6, Tokyo, T, 1800, A3, false, WA, G3, "春萌ステークス"),
        (7, Tokyo, T, 1400, U4, false, WS, G3, "梅花スプリント"),
        (7, Kyoto, T, 1400, A3, true, WA, G3, "紅梅ティアラ"),
        (8, Kokura, T, 1800, U4, false, WS, G3, "豊国記念"),
        (9, Hanshin, T, 1400, U4, false, WS, G3, "春一番スプリント"),
        (10, Hanshin, T, 1800, A3, false, WA, G3, "桃花賞"),
        (10, Chukyo, T, 1200, U4, false, WS, G3, "伊吹スプリント"),
        (11, Nakayama, T, 1200, U4, false, WH, G3, "春雷トライアル"),
        (12, Hanshin, T, 1600, A3, true, WA, G3, "白梅トライアル"),
        (13, Chukyo, T, 2000, U4, false, WS, G3, "尾張春待ち賞"),
        (14, Fukushima, T, 1800, U4, false, WH, G3, "みちのく春ハンデ"),
        (15, Fukushima, T, 1800, A3, false, WA, G3, "若駒ステークス"),
        (16, Nakayama, T, 1600, A3, false, WA, G3, "皐月前哨マイルS"),
        (17, Niigata, T, 1800, U3, false, WH, G3, "越後ハンデキャップ"),
        (17, Kyoto, T, 1600, U4, true, WS, G3, "新緑レディースカップ"),
        (18, Niigata, T, 2000, U3, false, WS, G3, "信濃川記念"),
        (19, Kyoto, T, 1200, U4, false, WS, G3, "淀スプリント"),
        (21, Fukushima, T, 2000, U4, false, WH, G3, "阿武隈ハンデキャップ"),
        (22, Fukushima, T, 1200, U3, false, WS, G3, "奥羽スプリント"),
        (23, Hanshin, T, 1400, U3, false, WS, G3, "初夏の浪速賞"),
        (24, Tokyo, T, 1800, U3, false, WH, G3, "武蔵野ハンデキャップ"),
        (25, Hakodate, T, 1200, U3, false, WS, G3, "渡島スプリント"),
        (26, Hanshin, T, 1600, U3, false, WS, G3, "梅雨明けマイル"),
        (26, Hakodate, T, 1200, A2, false, WA, G3, "道南ジュニアカップ"),
        (27, Fukushima, T, 1200, U3, false, WH, G3, "織姫ハンデスプリント"),
        (27, Kokura, T, 1800, U3, false, WH, G3, "筑紫サマーハンデ"),
        (28, Hakodate, T, 1800, U3, false, WH, G3, "津軽海峡ハンデキャップ"),
        (28, Kokura, T, 1200, U3, false, WS, G3, "玄界スプリント"),
        (29, Fukushima, T, 2000, U4, false, WH, G3, "夏の福島ハンデ"),
        (30, Kokura, T, 2000, U3, false, WH, G3, "夏空記念"),
        (31, Sapporo, T, 1200, U3, false, WS, G3, "大雪スプリント"),
        (31, Niigata, T, 2000, U3, false, WH, G3, "越後湯沢ハンデ"),
        (32, Niigata, T, 1000, U3, false, WH, G3, "韋駄天ダッシュ"),
        (32, Kokura, T, 1200, U3, false, WH, G3, "九州ハンデキャップ"),
        (33, Niigata, T, 2000, U3, false, WH, G3, "信濃サマーハンデ"),
        (34, Niigata, T, 1600, A2, false, WA, G3, "越後ジュニアカップ"),
        (34, Sapporo, T, 1800, U3, false, WH, G3, "石狩ハンデキャップ"),
        (35, Niigata, T, 2200, U3, false, WH, G3, "佐渡サマー記念"),
        (36, Sapporo, T, 1800, A2, false, WA, G3, "北斗ジュニア"),
        (36, Nakayama, T, 1600, U3, false, WS, G3, "秋立つマイル"),
        (37, Chukyo, T, 1200, U3, false, WS, G3, "錦城スプリント"),
        (37, Hanshin, T, 1600, U3, false, WS, G3, "神戸マイル記念"),
        (38, Chukyo, T, 1400, U3, false, WS, G3, "伊勢湾スプリント"),
        (39, Hanshin, T, 1800, U3, true, WS, G3, "秋の淑女カップ"),
        (40, Niigata, T, 1800, U3, false, WS, G3, "越後秋彩賞"),
        (41, Tokyo, T, 1800, A2, false, WA, G3, "武蔵野ジュニア賞"),
        (43, Kyoto, T, 1800, A2, false, WA, G3, "洛北ジュニアカップ"),
        (44, Fukushima, T, 2000, U3, false, WH, G3, "福島秋ハンデ"),
        (45, Fukushima, T, 1200, U3, false, WS, G3, "会津スプリント"),
        (48, Tokyo, T, 1800, U4, false, WS, G3, "霜月ステークス"),
        (49, Nakayama, T, 1800, U3, false, WH, G3, "年の瀬ハンデキャップ"),
        (50, Nakayama, T, 1200, U3, false, WS, G3, "師走スプリント"),
        (50, Chukyo, T, 1600, U3, false, WH, G3, "中京年末ハンデ"),
        (51, Hanshin, T, 1400, U3, false, WS, G3, "年末スプリント"),
        (52, Chukyo, T, 2000, U3, false, WS, G3, "賀正記念"),
        // ---- Listed
        (2, Kyoto, D, 1800, U4, false, WS, Listed, "寒月ダートステークス"),
        (4, Kyoto, T, 2000, U4, false, WS, Listed, "氷室ステークス"),
        (6, Tokyo, D, 1600, U4, false, WS, Listed, "風信子ダートマイル"),
        (8, Kokura, D, 1700, U4, false, WS, Listed, "筑豊ダートステークス"),
        (11, Nakayama, D, 1800, U4, false, WH, Listed, "春霞ハンデキャップ"),
        (12, Chukyo, D, 1800, U4, false, WS, Listed, "名古屋城ステークス"),
        (14, Fukushima, D, 1700, U4, false, WH, Listed, "磐梯ダートハンデ"),
        (16, Hanshin, D, 1800, U4, false, WS, Listed, "花冷えダートステークス"),
        (19, Niigata, D, 1800, U4, false, WS, Listed, "弥彦ダートステークス"),
        (22, Tokyo, D, 1600, U3, false, WS, Listed, "麦秋ダートカップ"),
        (26, Hanshin, D, 1800, U3, false, WH, Listed, "梅雨ダートハンデ"),
        (29, Kokura, D, 1700, U3, false, WH, Listed, "夏の砂ハンデ"),
        (33, Niigata, D, 1800, U3, false, WS, Listed, "信濃川ダートステークス"),
        (35, Sapporo, T, 2600, U3, false, WS, Listed, "天北ロングステークス"),
        (38, Chukyo, D, 1800, U3, false, WS, Listed, "伊勢湾ダートカップ"),
        (42, Tokyo, D, 1600, U3, false, WS, Listed, "武蔵野ダートマイル"),
        (46, Kyoto, D, 1800, U3, false, WS, Listed, "洛南ダートステークス"),
        (51, Nakayama, D, 1800, U3, false, WH, Listed, "年忘れダートハンデ"),
        // ---- Open specials
        (2, Kyoto, T, 1800, A3, false, WA, Open, "寒紅特別"),
        (3, Nakayama, D, 1800, U4, false, WS, Open, "冬霞特別"),
        (5, Kokura, D, 1700, U4, false, WH, Open, "豊前ダート特別"),
        (7, Tokyo, D, 2100, U4, false, WS, Open, "武蔵野ダート特別"),
        (9, Hanshin, D, 1400, U4, false, WS, Open, "浪速ダート特別"),
        (11, Chukyo, D, 1400, U4, false, WH, Open, "木曽路特別"),
        (15, Fukushima, D, 1150, U4, false, WS, Open, "信夫山特別"),
        (17, Niigata, D, 1200, U4, false, WS, Open, "越後新緑特別"),
        (21, Tokyo, D, 1400, U4, false, WS, Open, "初夏の砂特別"),
        (24, Hanshin, D, 1400, U3, false, WH, Open, "夏越ダート特別"),
        (28, Hakodate, D, 1000, U3, false, WS, Open, "渡島ダート特別"),
        (31, Sapporo, D, 1700, U3, false, WS, Open, "石狩ダート特別"),
        (32, Niigata, D, 1200, U3, false, WS, Open, "佐渡ダート特別"),
        (36, Sapporo, D, 1700, U3, false, WH, Open, "天北ダート特別"),
        (39, Chukyo, D, 1400, U3, false, WS, Open, "尾張ダート特別"),
        (43, Niigata, D, 1800, U3, false, WS, Open, "越後秋ダート特別"),
        (45, Fukushima, D, 1700, U3, false, WH, Open, "会津ダート特別"),
        (48, Hanshin, D, 1400, U3, false, WH, Open, "年末浪速特別"),
    ]
}

/// First-place prize (units of 10,000 yen). Money rises strictly with grade.
fn prize_for(grade: Grade, age: AgeRule, fillies: bool, surface: Surface, week: u8) -> u32 {
    if surface == Surface::Jump {
        return match grade {
            Grade::G1 => 6000,
            Grade::G2 => 2500,
            Grade::G3 => 1800,
            Grade::Open => 1000,
            _ => 700,
        };
    }
    let young = matches!(age, AgeRule::Two | AgeRule::Three);
    match grade {
        Grade::G1 if age == AgeRule::Two => 7000,
        Grade::G1 if week == 52 && age == AgeRule::ThreeUp => 50000, // year-end grand prix
        Grade::G1 if week == 47 => 30000,                            // international
        Grade::G1 if fillies || surface == Surface::Dirt => 12000,
        Grade::G1 => 20000,
        Grade::G2 => {
            if young {
                5200
            } else {
                6000
            }
        }
        Grade::G3 => {
            if young {
                3800
            } else {
                4300
            }
        }
        Grade::Listed => {
            if young {
                2700
            } else {
                3300
            }
        }
        Grade::Open => {
            if young {
                1800
            } else {
                2500
            }
        }
        Grade::Win3 => 1500,
        Grade::Win2 => 1100,
        Grade::Win1 => 800,
        Grade::Maiden if age == AgeRule::Two => 650,
        Grade::Maiden => 550,
        Grade::Debut if age == AgeRule::Two => 720,
        Grade::Debut => 600,
    }
}

/// Weeks in which two-year-olds make their debut (新馬戦), from June.
const DEBUT_TWO: std::ops::RangeInclusive<u8> = 23..=52;
/// Weeks in which unraced three-year-olds still have debut races, until early March.
const DEBUT_THREE: std::ops::RangeInclusive<u8> = 1..=9;
/// Maiden races for two-year-olds start a few weeks after their debuts.
const MAIDEN_TWO: std::ops::RangeInclusive<u8> = 26..=52;
/// Three-year-old maiden races end in September.
const MAIDEN_THREE: std::ops::RangeInclusive<u8> = 1..=36;
/// From June, horses of three race against their elders in the classes.
const CLASSES_MERGE: u8 = 23;

/// The class races of a racing day at a course (races 1-8, before the stakes):
/// (race number, class, age rule).
fn class_card(week: u8, day: Day) -> Vec<(u8, Grade, AgeRule)> {
    use AgeRule::*;
    use Grade::*;
    let sat = day == Day::Sat;
    let maiden_age = |second: bool| {
        let two = MAIDEN_TWO.contains(&week);
        let three = MAIDEN_THREE.contains(&week);
        match (two, three) {
            (true, true) => {
                if sat ^ second {
                    Two
                } else {
                    Three
                }
            }
            (true, false) => Two,
            _ => Three,
        }
    };
    let debut = if DEBUT_TWO.contains(&week) {
        (Debut, Two)
    } else if DEBUT_THREE.contains(&week) {
        (Debut, Three)
    } else {
        (Maiden, maiden_age(true))
    };
    let win1 = if week < CLASSES_MERGE {
        if sat {
            Three
        } else {
            FourUp
        }
    } else if sat && week >= 36 {
        Two
    } else {
        ThreeUp
    };
    let older = if week < CLASSES_MERGE {
        FourUp
    } else {
        ThreeUp
    };
    vec![
        (1, Maiden, maiden_age(false)),
        (3, debut.0, debut.1),
        (5, Win1, win1),
        (7, Win2, older),
        (8, Win3, older),
    ]
}

/// A class race's name, as the JRA writes them: "2歳新馬", "3歳以上1勝クラス".
fn class_name(grade: Grade, age: AgeRule) -> String {
    let ages = match age {
        AgeRule::Two => "2歳",
        AgeRule::Three => "3歳",
        AgeRule::ThreeUp => "3歳以上",
        AgeRule::FourUp => "4歳以上",
    };
    let class = match grade {
        Grade::Debut => "新馬",
        Grade::Maiden => "未勝利",
        Grade::Win1 => "1勝クラス",
        Grade::Win2 => "2勝クラス",
        Grade::Win3 => "3勝クラス",
        _ => unreachable!("not a class race"),
    };
    format!("{ages}{class}")
}

/// Every class race of the year, at every course racing that week.
fn class_races(meetings: &[Meeting]) -> Vec<CalendarRace> {
    let mut races = vec![];
    for m in meetings {
        for week in m.start_week..=m.end_week {
            for day in [Day::Sat, Day::Sun] {
                for (race_no, grade, age) in class_card(week, day) {
                    // turf and dirt in turn, at a distance the course has
                    let pick = rivals::mix_seed(&[
                        week as u64,
                        m.venue as u64,
                        day as u64,
                        race_no as u64,
                    ]);
                    let surface = if (week as u64 + race_no as u64 + day as u64).is_multiple_of(2) {
                        Surface::Turf
                    } else {
                        Surface::Dirt
                    };
                    let longest = if age == AgeRule::Two { 2000 } else { 2600 };
                    let options: Vec<u32> = course_distances(m.venue, surface)
                        .iter()
                        .copied()
                        .filter(|&d| (1000..=longest).contains(&d))
                        .collect();
                    let distance = options[(pick % options.len() as u64) as usize];
                    let young = matches!(age, AgeRule::Two | AgeRule::Three);
                    // a few older class races are handicaps, as in the real programme
                    let weight = if young {
                        WeightRule::Age
                    } else if grade == Grade::Win3 && pick.is_multiple_of(3) {
                        WeightRule::Handicap
                    } else {
                        WeightRule::Fixed
                    };
                    races.push(CalendarRace {
                        week,
                        day,
                        race_no,
                        venue: m.venue,
                        name: class_name(grade, age),
                        grade,
                        surface,
                        distance,
                        eligibility: Eligibility {
                            age,
                            fillies_only: false,
                            weight,
                        },
                        prize: prize_for(grade, age, false, surface, week),
                    });
                }
            }
        }
    }
    races
}

/// The courses with fences.
const JUMP_COURSES: [Venue; 8] = [
    Venue::Nakayama,
    Venue::Tokyo,
    Venue::Kyoto,
    Venue::Hanshin,
    Venue::Kokura,
    Venue::Niigata,
    Venue::Fukushima,
    Venue::Chukyo,
];
/// Where a jump race goes on the card, clear of the flat races.
const JUMP_MAIDEN_RACE: u8 = 2;
const JUMP_OPEN_RACE: u8 = 4;

/// The everyday races over fences: a maiden or an open each Saturday at the courses
/// that have fences, turn about.
fn jump_races(meetings: &[Meeting]) -> Vec<CalendarRace> {
    let mut races = vec![];
    for m in meetings.iter().filter(|m| JUMP_COURSES.contains(&m.venue)) {
        for week in m.start_week..=m.end_week {
            // the longest are kept for the big races
            let distances = &course_distances(m.venue, Surface::Jump)[..5];
            let distance = distances[week as usize % distances.len()];
            let (grade, race_no, name) = if week % 2 == 0 {
                (Grade::Maiden, JUMP_MAIDEN_RACE, "障害未勝利")
            } else {
                (Grade::Open, JUMP_OPEN_RACE, "障害オープン")
            };
            let age = AgeRule::ThreeUp;
            races.push(CalendarRace {
                week,
                day: Day::Sat,
                race_no,
                venue: m.venue,
                name: name.to_string(),
                grade,
                surface: Surface::Jump,
                distance,
                eligibility: Eligibility {
                    age,
                    fillies_only: false,
                    weight: WeightRule::Fixed,
                },
                prize: prize_for(grade, age, false, Surface::Jump, week),
            });
        }
    }
    races
}

/// Where in the weekend card a race goes, in order of preference.
fn slot_preferences(grade: Grade) -> &'static [(Day, u8)] {
    use Day::*;
    match grade {
        Grade::G1 => &[(Sun, 11), (Sat, 11)],
        Grade::G2 => &[(Sun, 11), (Sat, 11), (Sun, 10)],
        Grade::G3 => &[(Sat, 11), (Sun, 11), (Sat, 10), (Sun, 10)],
        Grade::Listed => &[
            (Sun, 10),
            (Sat, 10),
            (Sun, 11),
            (Sat, 11),
            (Sun, 9),
            (Sat, 9),
        ],
        Grade::Open => &[
            (Sat, 9),
            (Sun, 9),
            (Sat, 10),
            (Sun, 10),
            (Sat, 12),
            (Sun, 12),
        ],
        // class races have their own fixed places on the card (see `class_card`)
        _ => &[],
    }
}

/// The fictional calendar for one year.
pub fn fictional_calendar() -> Calendar {
    let meetings: Vec<Meeting> = MEETINGS
        .iter()
        .map(|&(venue, round, start_week, end_week)| Meeting {
            venue,
            round,
            start_week,
            end_week,
        })
        .collect();

    let mut taken = std::collections::HashSet::new();
    let mut races: Vec<CalendarRace> = race_rows()
        .into_iter()
        .map(
            |(week, venue, surface, distance, age, fillies_only, weight, grade, name)| {
                // a stakes race over fences goes before the day's big race on the flat
                let slots: &[(Day, u8)] = if surface == Surface::Jump {
                    &[(Day::Sat, 9), (Day::Sun, 9)]
                } else {
                    slot_preferences(grade)
                };
                let (day, race_no) = slots
                    .iter()
                    .copied()
                    .find(|&(d, n)| taken.insert((week, d, venue, n)))
                    .unwrap_or_else(|| panic!("no free slot for {name}"));
                CalendarRace {
                    week,
                    day,
                    race_no,
                    venue,
                    name: name.to_string(),
                    grade,
                    surface,
                    distance,
                    eligibility: Eligibility {
                        age,
                        fillies_only,
                        weight,
                    },
                    prize: prize_for(grade, age, fillies_only, surface, week),
                }
            },
        )
        .collect();
    races.extend(class_races(&meetings));
    races.extend(jump_races(&meetings));
    // in card order: week, day, course, race number
    races.sort_by_key(|r| (r.week, r.day as u8, r.venue as u8, r.race_no));

    Calendar { meetings, races }
}

/// The calendar, built once and shared.
pub fn shared() -> &'static Calendar {
    static CALENDAR: std::sync::OnceLock<Calendar> = std::sync::OnceLock::new();
    CALENDAR.get_or_init(fictional_calendar)
}

#[tauri::command]
pub fn get_calendar() -> Calendar {
    shared().clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    fn cal() -> Calendar {
        // the jump races have rules of their own, tried with the game's
        let mut c = fictional_calendar();
        c.races.retain(|r| r.surface != Surface::Jump);
        c
    }

    fn count(c: &Calendar, g: Grade) -> usize {
        c.races.iter().filter(|r| r.grade == g).count()
    }

    #[test]
    fn all_ten_racecourses_hold_meetings() {
        let c = cal();
        let venues: HashSet<Venue> = c.meetings.iter().map(|m| m.venue).collect();
        assert_eq!(venues.len(), 10);
        for v in ALL_VENUES {
            assert!(venues.contains(&v), "{v:?} has no meeting");
        }
    }

    #[test]
    fn every_week_has_two_or_three_courses_racing() {
        let c = cal();
        for week in 1..=WEEKS {
            let n = c.venues_in_week(week).len();
            assert!((2..=3).contains(&n), "week {week} has {n} courses");
        }
    }

    #[test]
    fn meetings_are_ordered_numbered_and_never_overlap_at_a_course() {
        let c = cal();
        for v in ALL_VENUES {
            let ms: Vec<&Meeting> = c.meetings.iter().filter(|m| m.venue == v).collect();
            for (i, m) in ms.iter().enumerate() {
                assert_eq!(m.round as usize, i + 1, "{v:?} round numbering");
                assert!(m.start_week >= 1 && m.end_week <= WEEKS && m.start_week <= m.end_week);
                assert!(
                    m.end_week - m.start_week < 5,
                    "{v:?} round {} too long",
                    m.round
                );
                if i > 0 {
                    assert!(m.start_week > ms[i - 1].end_week, "{v:?} meetings overlap");
                }
            }
        }
    }

    #[test]
    fn the_year_has_about_as_many_racing_days_as_the_real_one() {
        let days: u32 = cal().meetings.iter().map(|m| m.days()).sum();
        assert!((270..=300).contains(&days), "{days} racing days");
    }

    #[test]
    fn every_race_is_held_during_a_meeting_at_its_course() {
        let c = cal();
        for r in c.races.iter().filter(|r| !r.venue.is_local()) {
            assert!(
                c.venues_in_week(r.week).contains(&r.venue),
                "{} is on week {} at {:?} but no meeting is on",
                r.name,
                r.week,
                r.venue
            );
        }
    }

    #[test]
    fn every_race_uses_a_distance_that_exists_at_its_course() {
        for r in &cal().races {
            assert!(
                course_distances(r.venue, r.surface).contains(&r.distance),
                "{}: {:?} {:?} {}m does not exist",
                r.name,
                r.venue,
                r.surface,
                r.distance
            );
        }
    }

    #[test]
    fn races_do_not_share_a_slot_and_use_the_afternoon_card() {
        let mut seen = HashSet::new();
        for r in &cal().races {
            assert!(
                !r.grade.is_stakes() || (9..=12).contains(&r.race_no),
                "{}: race {}",
                r.name,
                r.race_no
            );
            assert!(
                seen.insert((r.week, r.day, r.venue, r.race_no)),
                "{}: slot already taken",
                r.name
            );
        }
    }

    #[test]
    fn grade_counts_match_the_shape_of_the_real_programme() {
        let c = cal();
        assert_eq!(count(&c, Grade::G1), 24 + 5); // and the 交流 G1s
        assert!(count(&c, Grade::G2) >= 25, "G2: {}", count(&c, Grade::G2));
        assert!(count(&c, Grade::G3) >= 50, "G3: {}", count(&c, Grade::G3));
        assert!(count(&c, Grade::Listed) >= 12);
        assert!(count(&c, Grade::Open) >= 12);
    }

    #[test]
    fn g1_races_are_the_main_race_and_almost_always_on_sunday() {
        let c = cal();
        let g1: Vec<_> = c.races.iter().filter(|r| r.grade == Grade::G1).collect();
        assert!(g1.iter().all(|r| r.race_no == 11));
        // only the year-end double header puts a second G1 on Saturday
        assert!(g1.iter().filter(|r| r.day == Day::Sat).count() <= 1);
    }

    #[test]
    fn g1_line_up_matches_the_jra_pattern() {
        let c = cal();
        let g1: Vec<&CalendarRace> = c
            .races
            .iter()
            .filter(|r| r.grade == Grade::G1 && !r.venue.is_local())
            .collect();
        let by_age = |a: AgeRule| g1.iter().filter(|r| r.eligibility.age == a).count();
        assert_eq!(by_age(AgeRule::Three), 7); // fillies' and colts' classics + mile
        assert_eq!(by_age(AgeRule::Two), 3);
        assert_eq!(g1.iter().filter(|r| r.eligibility.fillies_only).count(), 6);
        // the dirt G1s
        let dirt: Vec<_> = g1.iter().filter(|r| r.surface == Surface::Dirt).collect();
        assert_eq!(dirt.len(), 2);
        // the classic distances exist: 1200m sprint, 2400m derby-type, 3000m+ stayers
        assert!(g1
            .iter()
            .any(|r| r.distance == 1200 && r.eligibility.age == AgeRule::ThreeUp));
        assert!(g1.iter().any(|r| r.distance == 2400
            && r.eligibility.age == AgeRule::Three
            && !r.eligibility.fillies_only));
        assert!(g1
            .iter()
            .any(|r| r.distance >= 3000 && r.eligibility.age == AgeRule::Three));
        assert!(g1
            .iter()
            .any(|r| r.distance >= 3000 && r.eligibility.age == AgeRule::FourUp));
        // all four spring/autumn weeks at Tokyo-type big courses are covered
        let venues: HashSet<Venue> = g1.iter().map(|r| r.venue).collect();
        for v in [
            Venue::Tokyo,
            Venue::Nakayama,
            Venue::Kyoto,
            Venue::Hanshin,
            Venue::Chukyo,
        ] {
            assert!(venues.contains(&v), "no G1 at {v:?}");
        }
    }

    #[test]
    fn two_year_olds_only_race_from_summer_on() {
        for r in cal()
            .races
            .iter()
            .filter(|r| r.eligibility.age == AgeRule::Two)
        {
            if !r.grade.is_stakes() {
                assert!(r.week >= 23, "{} is a 2yo race in week {}", r.name, r.week);
                continue;
            }
            assert!(r.week >= 25, "{} is a 2yo race in week {}", r.name, r.week);
        }
    }

    #[test]
    fn weight_rules_follow_age_rules() {
        for r in &cal().races {
            if matches!(r.eligibility.age, AgeRule::Two | AgeRule::Three)
                && r.eligibility.weight == WeightRule::Fixed
            {
                panic!("{}: 定量 is for older horses", r.name);
            }
            if r.eligibility.weight == WeightRule::Age {
                assert!(
                    matches!(r.eligibility.age, AgeRule::Two | AgeRule::Three),
                    "{}: 馬齢 is for young horses",
                    r.name
                );
            }
            if r.grade == Grade::G1 {
                assert!(
                    matches!(r.eligibility.weight, WeightRule::Age | WeightRule::Fixed),
                    "{}",
                    r.name
                );
            }
        }
    }

    #[test]
    fn handicaps_are_never_g1() {
        assert!(cal()
            .races
            .iter()
            .all(|r| !(r.grade == Grade::G1 && r.eligibility.weight == WeightRule::Handicap)));
        assert!(cal()
            .races
            .iter()
            .any(|r| r.eligibility.weight == WeightRule::Handicap));
    }

    #[test]
    fn prize_money_rises_with_grade() {
        let c = cal();
        let range = |g: Grade| {
            let p: Vec<u32> = c
                .races
                .iter()
                .filter(|r| r.grade == g)
                .map(|r| r.prize)
                .collect();
            (*p.iter().min().unwrap(), *p.iter().max().unwrap())
        };
        let order = [
            Grade::G1,
            Grade::G2,
            Grade::G3,
            Grade::Listed,
            Grade::Open,
            Grade::Win3,
            Grade::Win2,
            Grade::Win1,
        ];
        for w in order.windows(2) {
            assert!(
                range(w[0]).0 > range(w[1]).1,
                "{:?} min must exceed {:?} max",
                w[0],
                w[1]
            );
        }
    }

    #[test]
    fn race_names_are_unique_and_original() {
        let c = cal();
        let mut seen = HashSet::new();
        for r in &c.races {
            assert!(
                !r.name.is_empty() && r.name.chars().count() <= 16,
                "{}",
                r.name
            );
            // class races go by their class, like "3歳未勝利"; only stakes have names of their own
            if r.grade.is_stakes() {
                assert!(seen.insert(r.name.clone()), "duplicate name {}", r.name);
            }
        }
        // none of these real race names (or anything containing them) may appear
        let real = [
            "有馬記念",
            "日本ダービー",
            "東京優駿",
            "皐月賞",
            "菊花賞",
            "桜花賞",
            "優駿牝馬",
            "オークス",
            "天皇賞",
            "宝塚記念",
            "安田記念",
            "ジャパンカップ",
            "フェブラリー",
            "高松宮記念",
            "大阪杯",
            "秋華賞",
            "エリザベス女王杯",
            "マイルチャンピオンシップ",
            "スプリンターズ",
            "チャンピオンズカップ",
            "朝日杯",
            "ジュベナイル",
            "ホープフル",
            "ヴィクトリアマイル",
            "NHKマイル",
            "日経賞",
            "弥生賞",
            "札幌記念",
            "小倉記念",
            "京都記念",
            "中山記念",
            "金鯱賞",
            "オールカマー",
            "セントライト",
            "神戸新聞杯",
            "阪神大賞典",
            "スプリングステークス",
            "青葉賞",
            "京王杯",
            "目黒記念",
            "紫苑",
            "毎日王冠",
            "京都大賞典",
            "アルゼンチン",
            "スワンステークス",
            "阪急杯",
            "共同通信杯",
            "きさらぎ賞",
            "シンザン記念",
            "東京新聞杯",
            "ダイヤモンドステークス",
            "新潟記念",
            "関屋記念",
            "函館記念",
            "七夕賞",
            "小倉サマー",
            "アイビスサマー",
            "北九州記念",
            "エルムステークス",
            "新潟2歳",
            "札幌2歳",
            "デイリー杯",
            "東スポ杯",
            "京都2歳",
            "サウジアラビア",
            "ユニコーン",
            "レパードステークス",
        ];
        for r in &c.races {
            for bad in real {
                assert!(
                    !r.name.contains(bad),
                    "{} contains the real race name {bad}",
                    r.name
                );
            }
        }
    }

    #[test]
    fn races_in_a_week_are_listed_by_the_calendar() {
        let c = cal();
        let mut per_week: HashMap<u8, usize> = HashMap::new();
        for r in &c.races {
            *per_week.entry(r.week).or_default() += 1;
        }
        assert!(per_week.keys().all(|w| (1..=WEEKS).contains(w)));
        // a graded race somewhere in at least 45 of the 52 weeks
        let graded_weeks: HashSet<u8> = c
            .races
            .iter()
            .filter(|r| matches!(r.grade, Grade::G1 | Grade::G2 | Grade::G3))
            .map(|r| r.week)
            .collect();
        assert!(
            graded_weeks.len() >= 45,
            "graded races in only {} weeks",
            graded_weeks.len()
        );
    }

    fn rule(age: AgeRule, fillies_only: bool) -> Eligibility {
        Eligibility {
            age,
            fillies_only,
            weight: WeightRule::Allowance,
        }
    }

    #[test]
    fn age_rules_decide_who_may_enter() {
        let two = rule(AgeRule::Two, false);
        assert!(two.allows(2, false) && !two.allows(3, false));
        let three = rule(AgeRule::Three, false);
        assert!(three.allows(3, true) && !three.allows(2, true) && !three.allows(4, true));
        let three_up = rule(AgeRule::ThreeUp, false);
        assert!(
            !three_up.allows(2, false) && three_up.allows(3, false) && three_up.allows(7, false)
        );
        let four_up = rule(AgeRule::FourUp, false);
        assert!(!four_up.allows(3, false) && four_up.allows(4, false) && four_up.allows(9, false));
    }

    #[test]
    fn fillies_only_races_exclude_colts_and_geldings() {
        let r = rule(AgeRule::ThreeUp, true);
        assert!(r.allows(4, true));
        assert!(!r.allows(4, false));
        assert!(rule(AgeRule::ThreeUp, false).allows(4, false));
    }

    #[test]
    fn calendar_is_deterministic_and_serializable() {
        assert_eq!(cal(), cal());
        let json = serde_json::to_string(&cal()).unwrap();
        let back: Calendar = serde_json::from_str(&json).unwrap();
        assert_eq!(back, cal());
    }

    // ---- class races: 新馬, 未勝利, 1勝・2勝・3勝クラス ----

    fn class_count(c: &Calendar, g: Grade) -> usize {
        c.races.iter().filter(|r| r.grade == g).count()
    }

    #[test]
    fn every_racing_day_has_its_class_races_before_the_stakes() {
        let c = cal();
        for m in &c.meetings {
            for week in m.start_week..=m.end_week {
                for day in [Day::Sat, Day::Sun] {
                    let card: Vec<&CalendarRace> = c
                        .races
                        .iter()
                        .filter(|r| {
                            r.week == week
                                && r.day == day
                                && r.venue == m.venue
                                && !r.grade.is_stakes()
                        })
                        .collect();
                    assert_eq!(card.len(), 5, "{:?} week {week} {day:?}", m.venue);
                    assert!(card.iter().all(|r| r.race_no <= 8));
                }
            }
        }
        for g in [
            Grade::Debut,
            Grade::Maiden,
            Grade::Win1,
            Grade::Win2,
            Grade::Win3,
        ] {
            assert!(class_count(&c, g) > 50, "{g:?}: {}", class_count(&c, g));
        }
    }

    #[test]
    fn class_races_are_named_by_age_and_class() {
        let c = cal();
        let names: HashSet<&str> = c
            .races
            .iter()
            .filter(|r| !r.grade.is_stakes())
            .map(|r| r.name.as_str())
            .collect();
        for n in [
            "2歳新馬",
            "3歳新馬",
            "2歳未勝利",
            "3歳未勝利",
            "3歳1勝クラス",
            "4歳以上1勝クラス",
            "3歳以上1勝クラス",
            "2歳1勝クラス",
            "4歳以上2勝クラス",
            "3歳以上3勝クラス",
        ] {
            assert!(names.contains(n), "{n} missing");
        }
    }

    #[test]
    fn debut_races_run_from_june_for_two_year_olds_and_until_march_for_three_year_olds() {
        for r in cal().races.iter().filter(|r| r.grade == Grade::Debut) {
            match r.eligibility.age {
                AgeRule::Two => assert!(r.week >= 23, "{} week {}", r.name, r.week),
                AgeRule::Three => assert!(r.week <= 9, "{} week {}", r.name, r.week),
                a => panic!("debut race for {a:?}"),
            }
        }
    }

    #[test]
    fn three_year_old_maidens_end_in_september() {
        for r in cal().races.iter().filter(|r| r.grade == Grade::Maiden) {
            match r.eligibility.age {
                AgeRule::Two => assert!(r.week >= 26),
                AgeRule::Three => assert!(r.week <= 36, "week {}", r.week),
                a => panic!("maiden race for {a:?}"),
            }
        }
    }

    #[test]
    fn three_year_olds_meet_their_elders_in_the_classes_from_june() {
        for r in cal()
            .races
            .iter()
            .filter(|r| matches!(r.grade, Grade::Win1 | Grade::Win2 | Grade::Win3))
        {
            if r.week < 23 {
                assert_ne!(
                    r.eligibility.age,
                    AgeRule::ThreeUp,
                    "{} week {}",
                    r.name,
                    r.week
                );
            } else {
                assert_ne!(
                    r.eligibility.age,
                    AgeRule::FourUp,
                    "{} week {}",
                    r.name,
                    r.week
                );
            }
        }
    }

    #[test]
    fn class_races_pay_less_than_stakes_and_more_the_higher_the_class() {
        let c = cal();
        let max = |g: Grade| {
            c.races
                .iter()
                .filter(|r| r.grade == g)
                .map(|r| r.prize)
                .max()
                .unwrap()
        };
        let min = |g: Grade| {
            c.races
                .iter()
                .filter(|r| r.grade == g)
                .map(|r| r.prize)
                .min()
                .unwrap()
        };
        assert!(min(Grade::Open) > max(Grade::Win3));
        assert!(min(Grade::Win1) > max(Grade::Maiden));
        assert!(max(Grade::Debut) > max(Grade::Maiden));
    }

    #[test]
    fn a_horse_may_try_a_class_above_its_own_but_never_one_below() {
        use Grade::*;
        assert!(Debut.admits(0, 0) && !Debut.admits(1, 0));
        assert!(Maiden.admits(0, 0) && Maiden.admits(5, 0) && !Maiden.admits(5, 1));
        assert!(Win1.admits(3, 0) && Win1.admits(4, 1) && !Win1.admits(6, 2));
        assert!(Win2.admits(6, 2) && !Win2.admits(9, 3));
        assert!(Win3.admits(9, 3) && !Win3.admits(12, 4));
        for g in [G1, G2, G3, Listed, Open] {
            assert!(g.is_stakes() && g.admits(0, 0) && g.admits(30, 12));
        }
        assert!(!Win1.is_stakes() && !Debut.is_stakes());
    }

    #[test]
    fn the_calendar_is_listed_in_card_order() {
        let c = cal();
        assert!(c.races.windows(2).all(|w| (
            w[0].week,
            w[0].day as u8,
            w[0].venue as u8,
            w[0].race_no
        ) < (
            w[1].week,
            w[1].day as u8,
            w[1].venue as u8,
            w[1].race_no
        )));
    }

    #[test]
    fn class_grades_serialise_as_the_ui_expects() {
        for (g, s) in [
            (Grade::Win3, "win3"),
            (Grade::Win1, "win1"),
            (Grade::Maiden, "maiden"),
            (Grade::Debut, "debut"),
        ] {
            assert_eq!(serde_json::to_string(&g).unwrap(), format!("\"{s}\""));
        }
    }
}
