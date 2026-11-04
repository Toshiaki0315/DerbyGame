//! The ten JRA racecourses: their layouts, and how each suits a horse.
//!
//! The figures are from the JRA's course pages (https://www.jra.go.jp/facilities/race/),
//! for the A course where a track has several. A course's own character sets how
//! it treats a horse in up to four ways: which way it turns, how long its straight
//! is, how hilly it is, and (in Hokkaido) its heavy western turf.

use crate::calendar::{Surface, Venue};
use crate::race::Style;
use serde::{Deserialize, Serialize};

/// A straight at least this long lets a horse come from behind.
const LONG_STRAIGHT: f64 = 400.0;
/// A straight shorter than this favours a horse up with the pace.
const SHORT_STRAIGHT: f64 = 320.0;
/// A rise of this many metres makes a course hilly, as does a climb to the finish.
const HILLY: f64 = 3.0;
/// Power from which a horse relishes hills and heavy turf, and below which it does not.
const STRONG: u32 = 70;
const WEAK: u32 = 45;

/// Effect on a horse's ability of each thing in or against its favour, in percent.
const TURN_PERCENT: f64 = 1.0;
const STRAIGHT_PERCENT: f64 = 1.0;
const HILL_PERCENT: f64 = 0.75;
const TURF_PERCENT: f64 = 0.75;

/// Which way a course (or a horse) goes round.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Turn {
    Left,
    Right,
}

/// Which of the two training centres someone is based at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    /// 関東: 美浦トレーニングセンター.
    East,
    /// 関西: 栗東トレーニングセンター.
    West,
}

impl Region {
    pub fn label(self) -> &'static str {
        match self {
            Region::East => "美浦",
            Region::West => "栗東",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grass {
    /// 洋芝: three western grasses, deep and tiring.
    Western,
    /// 野芝, overseeded with Italian ryegrass in some seasons.
    Japanese,
}

/// The layout of a course's turf or dirt track.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Track {
    /// The straight and rise that decide how it suits a horse (the longer,
    /// outer course where there are two).
    pub straight: f64,
    pub elevation: f64,
    pub lap_text: &'static str,
    pub straight_text: &'static str,
    pub elevation_text: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Course {
    pub venue: Venue,
    pub name: &'static str,
    pub place: &'static str,
    pub turn: Turn,
    pub grass: Grass,
    /// A notable climb in the run to the finish on the turf.
    pub finish_climb: bool,
    pub turf: Track,
    pub dirt: Track,
    pub features: &'static str,
    pub suits: &'static str,
}

impl Track {
    /// The length of a lap in metres: of the inner course, where there are two.
    pub fn lap_metres(&self) -> f64 {
        let digits: String = self
            .lap_text
            .chars()
            .skip_while(|c| !c.is_ascii_digit())
            .take_while(|c| c.is_ascii_digit() || *c == ',' || *c == '.')
            .filter(|c| *c != ',')
            .collect();
        digits.parse().unwrap_or(0.0)
    }
}

/// How a race of a given distance uses a course: which course it is run on, how far it
/// goes round, and where its gate stands.
#[derive(Debug, Clone, PartialEq)]
pub struct CoursePlan {
    pub lap: f64,
    /// From the last corner to the goal.
    pub straight: f64,
    /// From the goal on to the first corner.
    pub after_goal: f64,
    /// 内回り, 外回り or 直線, where a course has more than one way round.
    pub name: Option<&'static str>,
    pub corners: u8,
    /// A dirt race started on the turf.
    pub turf_start: bool,
    pub straight_course: bool,
}

/// The distances run on the inner course, at the courses with two.
fn inner_distances(venue: Venue) -> Option<&'static [u32]> {
    match venue {
        Venue::Kyoto => Some(&[1200, 2000]),
        Venue::Hanshin => Some(&[1200, 1400, 2000, 2200, 3000]),
        Venue::Niigata => Some(&[1200, 1400, 2200, 2400]),
        Venue::Nakayama => Some(&[1800, 2000, 2500, 3600]),
        _ => None,
    }
}

/// The length of Niigata's straight course.
const STRAIGHT_COURSE_METRES: u32 = 1000;

/// From the goal on to the first corner, by venue (turf, dirt): what a gate in the home
/// straight has to the first corner, less what it has to the goal.
fn after_goal(venue: Venue, surface: Surface) -> f64 {
    let (turf, dirt) = match venue {
        Venue::Tokyo => (33.0, 35.0),
        Venue::Nakayama => (72.0, 68.0),
        Venue::Kyoto => (92.0, 93.0),
        Venue::Hanshin => (20.0, 16.0),
        Venue::Chukyo => (20.0, 22.0),
        Venue::Niigata => (59.0, 61.0),
        Venue::Fukushima => (105.0, 83.0),
        Venue::Kokura => (87.0, 88.0),
        Venue::Sapporo => (21.0, 27.0),
        Venue::Hakodate => (102.0, 104.0),
        Venue::Oi => (0.0, 40.0),
        Venue::Kawasaki => (0.0, 30.0),
        Venue::Funabashi => (0.0, 35.0),
        Venue::Sonoda => (0.0, 25.0),
    };
    match surface {
        Surface::Turf => turf,
        Surface::Dirt => dirt,
    }
}

/// The dirt races started on the turf.
const TURF_STARTS: &[(Venue, u32)] = &[
    (Venue::Chukyo, 1400),
    (Venue::Kyoto, 1400),
    (Venue::Hanshin, 1400),
    (Venue::Hanshin, 2000),
    (Venue::Niigata, 1200),
    (Venue::Nakayama, 1200),
    (Venue::Tokyo, 1600),
    (Venue::Fukushima, 1150),
];

/// Distances with the corners a race over them goes round.
type CornersBy = &'static [(u32, u8)];

/// The corners a race goes round, by course, surface and distance.
const CORNERS: &[(Venue, Surface, CornersBy)] = &[
    (
        Venue::Oi,
        Surface::Dirt,
        &[(1200, 2), (1600, 2), (1800, 4), (2000, 4)],
    ),
    (
        Venue::Kawasaki,
        Surface::Dirt,
        &[(1400, 4), (1600, 4), (2000, 6)],
    ),
    (
        Venue::Funabashi,
        Surface::Dirt,
        &[(1200, 2), (1600, 2), (1800, 4)],
    ),
    (Venue::Sonoda, Surface::Dirt, &[(1400, 4), (1700, 4)]),
    (
        Venue::Chukyo,
        Surface::Dirt,
        &[(1200, 2), (1400, 2), (1800, 4), (1900, 4)],
    ),
    (
        Venue::Chukyo,
        Surface::Turf,
        &[(1200, 2), (1400, 2), (1600, 3), (2000, 4), (2200, 4)],
    ),
    (
        Venue::Kyoto,
        Surface::Dirt,
        &[(1200, 2), (1400, 2), (1800, 4), (1900, 4)],
    ),
    (
        Venue::Kyoto,
        Surface::Turf,
        &[
            (1200, 2),
            (1400, 2),
            (1600, 2),
            (1800, 2),
            (2000, 4),
            (2200, 4),
            (2400, 4),
            (3000, 6),
            (3200, 6),
        ],
    ),
    (
        Venue::Hanshin,
        Surface::Dirt,
        &[(1200, 2), (1400, 2), (1800, 4), (2000, 4)],
    ),
    (
        Venue::Hanshin,
        Surface::Turf,
        &[
            (1200, 2),
            (1400, 2),
            (1600, 2),
            (1800, 2),
            (2000, 4),
            (2200, 4),
            (2400, 4),
            (2600, 6),
            (3000, 6),
        ],
    ),
    (
        Venue::Sapporo,
        Surface::Dirt,
        &[(1000, 2), (1700, 4), (2400, 6)],
    ),
    (
        Venue::Sapporo,
        Surface::Turf,
        &[(1200, 2), (1500, 3), (1800, 4), (2000, 4), (2600, 6)],
    ),
    (
        Venue::Kokura,
        Surface::Dirt,
        &[(1000, 2), (1700, 4), (2400, 6)],
    ),
    (
        Venue::Kokura,
        Surface::Turf,
        &[(1200, 2), (1800, 4), (2000, 4), (2600, 6)],
    ),
    (
        Venue::Niigata,
        Surface::Dirt,
        &[(1200, 2), (1800, 4), (2500, 6)],
    ),
    (
        Venue::Niigata,
        Surface::Turf,
        &[
            (1200, 2),
            (1400, 2),
            (1600, 2),
            (1800, 2),
            (2000, 2),
            (2200, 4),
            (2400, 4),
        ],
    ),
    (
        Venue::Nakayama,
        Surface::Dirt,
        &[(1200, 2), (1800, 4), (2400, 6), (2500, 6)],
    ),
    (
        Venue::Nakayama,
        Surface::Turf,
        &[
            (1200, 2),
            (1600, 3),
            (1800, 4),
            (2000, 4),
            (2200, 5),
            (2500, 5),
        ],
    ),
    (
        Venue::Tokyo,
        Surface::Dirt,
        &[(1300, 2), (1400, 2), (1600, 2), (2100, 4), (2400, 4)],
    ),
    (
        Venue::Tokyo,
        Surface::Turf,
        &[
            (1400, 2),
            (1600, 2),
            (1800, 3),
            (2000, 3),
            (2400, 4),
            (2500, 4),
        ],
    ),
    (
        Venue::Hakodate,
        Surface::Dirt,
        &[(1000, 2), (1700, 4), (2400, 6)],
    ),
    (
        Venue::Hakodate,
        Surface::Turf,
        &[(1000, 2), (1200, 2), (1800, 4), (2000, 4), (2600, 6)],
    ),
    (
        Venue::Fukushima,
        Surface::Dirt,
        &[(1150, 2), (1700, 4), (2400, 6)],
    ),
    (
        Venue::Fukushima,
        Surface::Turf,
        &[(1200, 2), (1800, 4), (2000, 4), (2600, 6)],
    ),
];

/// The numbers in a text like "内1,623m/外2,223m": the first is the inner course's.
fn metres_in(text: &str) -> Vec<f64> {
    text.split(|c: char| !(c.is_ascii_digit() || c == ',' || c == '.'))
        .filter_map(|n| n.replace(',', "").parse().ok())
        .collect()
}

/// How a race of this surface and distance uses a course.
pub fn plan_of(venue: Venue, surface: Surface, distance: u32) -> CoursePlan {
    let course = course_of(venue);
    let track = match surface {
        Surface::Turf => &course.turf,
        Surface::Dirt => &course.dirt,
    };
    let corners = CORNERS
        .iter()
        .find(|(v, s, _)| *v == venue && *s == surface)
        .and_then(|(_, _, by)| by.iter().find(|(d, _)| *d == distance))
        .map_or(0, |(_, n)| *n);
    let turf_start = surface == Surface::Dirt && TURF_STARTS.contains(&(venue, distance));
    if venue == Venue::Niigata && surface == Surface::Turf && distance == STRAIGHT_COURSE_METRES {
        return CoursePlan {
            lap: distance as f64,
            straight: distance as f64,
            after_goal: 0.0,
            name: Some("直線"),
            corners: 0,
            turf_start,
            straight_course: true,
        };
    }
    let inner = (surface == Surface::Turf)
        .then(|| inner_distances(venue))
        .flatten();
    let pick = |text: &str, fallback: f64| {
        let found = metres_in(text);
        match inner {
            Some(list) if list.contains(&distance) => found.first().copied(),
            Some(_) => found.last().copied(),
            None => found.first().copied(),
        }
        .unwrap_or(fallback)
    };
    CoursePlan {
        lap: pick(track.lap_text, track.lap_metres()),
        straight: pick(track.straight_text, track.straight),
        after_goal: after_goal(venue, surface),
        name: inner.map(|list| {
            if list.contains(&distance) {
                "内回り"
            } else {
                "外回り"
            }
        }),
        corners,
        turf_start,
        straight_course: false,
    }
}

const fn track(
    straight: f64,
    elevation: f64,
    lap_text: &'static str,
    straight_text: &'static str,
    elevation_text: &'static str,
) -> Track {
    Track {
        straight,
        elevation,
        lap_text,
        straight_text,
        elevation_text,
    }
}

pub const COURSES: [Course; 10] = [
    Course {
        venue: Venue::Sapporo,
        name: "札幌",
        place: "北海道札幌市",
        turn: Turn::Right,
        grass: Grass::Western,
        finish_climb: false,
        turf: track(266.1, 0.7, "1,640.9m", "266.1m", "0.7m"),
        dirt: track(264.3, 0.9, "1,487m", "264.3m", "0.9m"),
        features: "円に近い形でほぼ平坦です。直線は全10場でも短い部類で、洋芝は力を要します。",
        suits: "先行力があり、力の要る芝をこなせる馬",
    },
    Course {
        venue: Venue::Hakodate,
        name: "函館",
        place: "北海道函館市",
        turn: Turn::Right,
        grass: Grass::Western,
        finish_climb: false,
        turf: track(262.1, 3.5, "1,626.6m", "262.1m", "3.5m"),
        dirt: track(260.3, 3.5, "1,475.8m", "260.3m", "3.5m"),
        features: "直線が全10場で最も短く、ローカルの中では起伏があります。洋芝は力を要します。",
        suits: "前で粘れるパワーのある馬",
    },
    Course {
        venue: Venue::Fukushima,
        name: "福島",
        place: "福島県福島市",
        turn: Turn::Right,
        grass: Grass::Japanese,
        finish_climb: false,
        turf: track(292.0, 1.9, "1,600m", "292.0m", "1.9m"),
        dirt: track(295.7, 2.1, "1,444.6m", "295.7m", "2.1m"),
        features: "JRAで最もコンパクトな競馬場です。小回りで直線も短く、先に動ける馬が有利です。",
        suits: "器用に立ち回れる先行馬",
    },
    Course {
        venue: Venue::Niigata,
        name: "新潟",
        place: "新潟県新潟市",
        turn: Turn::Left,
        grass: Grass::Japanese,
        finish_climb: false,
        turf: track(
            658.7,
            2.2,
            "内1,623m/外2,223m",
            "内358.7m/外658.7m",
            "内0.8m/外2.2m",
        ),
        dirt: track(353.9, 0.6, "1,472.5m", "353.9m", "0.6m"),
        features: "外回りの直線はJRA最長です。芝1000mの直線コースもあります。起伏は穏やかです。",
        suits: "長く良い脚を使える差し馬",
    },
    Course {
        venue: Venue::Tokyo,
        name: "東京",
        place: "東京都府中市",
        turn: Turn::Left,
        grass: Grass::Japanese,
        finish_climb: false,
        turf: track(525.9, 2.7, "2,083.1m", "525.9m", "2.7m"),
        dirt: track(501.6, 2.5, "1,899m", "501.6m", "2.5m"),
        features: "幅の広い大きなコースで、直線は500mを超えます。実力が素直に出やすい舞台です。",
        suits: "末脚の切れる差し・追込馬",
    },
    Course {
        venue: Venue::Nakayama,
        name: "中山",
        place: "千葉県船橋市",
        turn: Turn::Right,
        grass: Grass::Japanese,
        finish_climb: true,
        turf: track(310.0, 5.3, "内1,667.1m/外1,839.7m", "310m", "5.3m"),
        dirt: track(308.0, 4.5, "1,493m", "308m", "4.5m"),
        features: "高低差は全10場で最大です。ゴール前に急坂があり、直線は短めです。",
        suits: "坂を苦にしないパワーのある先行馬",
    },
    Course {
        venue: Venue::Chukyo,
        name: "中京",
        place: "愛知県豊明市",
        turn: Turn::Left,
        grass: Grass::Japanese,
        finish_climb: true,
        turf: track(412.5, 3.5, "1,705.9m", "412.5m", "3.5m"),
        dirt: track(410.7, 3.4, "1,530m", "410.7m", "3.4m"),
        features: "直線が400mを超え、そこに坂が待ち受けます。スタミナと力の両方を問われます。",
        suits: "力強く長く脚を使える差し馬",
    },
    Course {
        venue: Venue::Kyoto,
        name: "京都",
        place: "京都府京都市",
        turn: Turn::Right,
        grass: Grass::Japanese,
        finish_climb: false,
        turf: track(
            403.7,
            4.3,
            "内1,782.8m/外1,894.3m",
            "内328.4m/外403.7m",
            "内3.1m/外4.3m",
        ),
        dirt: track(329.1, 3.0, "1,607.6m", "329.1m", "3.0m"),
        features: "3コーナーの坂を上って下り、平坦な直線に向かいます。下りの勢いが生きます。",
        suits: "坂をこなして直線で伸びる馬",
    },
    Course {
        venue: Venue::Hanshin,
        name: "阪神",
        place: "兵庫県宝塚市",
        turn: Turn::Right,
        grass: Grass::Japanese,
        finish_climb: true,
        turf: track(
            473.6,
            2.4,
            "内1,689m/外2,089m",
            "内356.5m/外473.6m",
            "内1.9m/外2.4m",
        ),
        dirt: track(352.7, 1.6, "1,517.6m", "352.7m", "1.6m"),
        features: "ゴール前に急な上りがあります。外回りは直線が長く、差しも届きます。",
        suits: "最後の坂でもうひと伸びできる馬",
    },
    Course {
        venue: Venue::Kokura,
        name: "小倉",
        place: "福岡県北九州市",
        turn: Turn::Right,
        grass: Grass::Japanese,
        finish_climb: false,
        turf: track(293.0, 3.0, "1,615.1m", "293m", "3.0m"),
        dirt: track(291.3, 2.9, "1,445.4m", "291.3m", "2.9m"),
        features: "小回りで直線が短いコースです。向正面から下り、勢いをつけて直線に入ります。",
        suits: "スピードのある先行馬",
    },
];

/// The local tracks, all dirt: their "turf" is the same dirt, and no race is run on it.
/// The figures are approximate, not taken from the tracks' own course guides.
pub const LOCAL_COURSES: [Course; 4] = [
    Course {
        venue: Venue::Oi,
        name: "大井",
        place: "東京都品川区",
        turn: Turn::Right,
        grass: Grass::Japanese,
        finish_climb: false,
        turf: track(386.0, 0.0, "1,600m", "386m", "0m"),
        dirt: track(386.0, 0.0, "1,600m", "386m", "0m"),
        features: "直線の長い右回りのダートコースです。",
        suits: "長く良い脚を使える馬",
    },
    Course {
        venue: Venue::Kawasaki,
        name: "川崎",
        place: "神奈川県川崎市",
        turn: Turn::Left,
        grass: Grass::Japanese,
        finish_climb: false,
        turf: track(300.0, 0.0, "1,200m", "300m", "0m"),
        dirt: track(300.0, 0.0, "1,200m", "300m", "0m"),
        features: "コーナーのきつい左回りの小回りコースです。",
        suits: "器用に立ち回れる先行馬",
    },
    Course {
        venue: Venue::Funabashi,
        name: "船橋",
        place: "千葉県船橋市",
        turn: Turn::Left,
        grass: Grass::Japanese,
        finish_climb: false,
        turf: track(308.0, 0.0, "1,400m", "308m", "0m"),
        dirt: track(308.0, 0.0, "1,400m", "308m", "0m"),
        features: "左回りで、スピードの出やすいコースです。",
        suits: "スピードのある馬",
    },
    Course {
        venue: Venue::Sonoda,
        name: "園田",
        place: "兵庫県尼崎市",
        turn: Turn::Right,
        grass: Grass::Japanese,
        finish_climb: false,
        turf: track(213.0, 0.0, "1,051m", "213m", "0m"),
        dirt: track(213.0, 0.0, "1,051m", "213m", "0m"),
        features: "直線の短い右回りの小回りコースです。",
        suits: "早めに動ける先行馬",
    },
];

pub fn course_of(venue: Venue) -> &'static Course {
    COURSES
        .iter()
        .chain(&LOCAL_COURSES)
        .find(|c| c.venue == venue)
        .unwrap_or(&COURSES[0])
}

/// Which way a horse likes to go, fixed for the horse. Many don't mind.
pub fn turn_preference(name: &str) -> Option<Turn> {
    // FNV-1a: the same name always gives the same answer
    let hash = name.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
        (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01B3)
    });
    match (hash >> 32) % 10 {
        0..=2 => Some(Turn::Left),
        3..=5 => Some(Turn::Right),
        _ => None,
    }
}

/// How one thing about a course sits with a horse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Factor {
    Plus,
    Even,
    Minus,
}

impl Factor {
    fn sign(self) -> f64 {
        match self {
            Factor::Plus => 1.0,
            Factor::Even => 0.0,
            Factor::Minus => -1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rating {
    Great,
    Good,
    Fair,
    Poor,
}

/// How a course suits a horse.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CourseFit {
    pub turn: Factor,
    pub straight: Factor,
    pub hill: Factor,
    pub turf: Factor,
    /// The whole effect on its speed and stamina, in percent.
    pub percent: f64,
    pub rating: Rating,
}

impl CourseFit {
    pub fn multiplier(&self) -> f64 {
        1.0 + self.percent / 100.0
    }
}

fn by_power(power: u32, strong: u32) -> Factor {
    if power >= strong {
        Factor::Plus
    } else if power <= WEAK {
        Factor::Minus
    } else {
        Factor::Even
    }
}

/// How a course suits a horse on this surface.
pub fn fit(
    venue: Venue,
    surface: Surface,
    name: &str,
    style: Option<Style>,
    power: u32,
) -> CourseFit {
    let course = course_of(venue);
    let track = match surface {
        Surface::Turf => &course.turf,
        Surface::Dirt => &course.dirt,
    };
    let turn = match turn_preference(name) {
        None => Factor::Even,
        Some(t) if t == course.turn => Factor::Plus,
        Some(_) => Factor::Minus,
    };
    let straight = if track.straight >= LONG_STRAIGHT {
        match style {
            Some(Style::Sashi | Style::Oikomi) => Factor::Plus,
            Some(Style::Nige) => Factor::Minus,
            _ => Factor::Even,
        }
    } else if track.straight < SHORT_STRAIGHT {
        match style {
            Some(Style::Nige | Style::Senko) => Factor::Plus,
            Some(Style::Oikomi) => Factor::Minus,
            _ => Factor::Even,
        }
    } else {
        Factor::Even
    };
    let hill = if track.elevation >= HILLY || course.finish_climb {
        by_power(power, STRONG)
    } else {
        Factor::Even
    };
    let turf = if surface == Surface::Turf && course.grass == Grass::Western {
        by_power(power, STRONG)
    } else {
        Factor::Even
    };
    let percent = turn.sign() * TURN_PERCENT
        + straight.sign() * STRAIGHT_PERCENT
        + hill.sign() * HILL_PERCENT
        + turf.sign() * TURF_PERCENT;
    let rating = if percent >= 1.5 {
        Rating::Great
    } else if percent >= 0.5 {
        Rating::Good
    } else if percent > -0.5 {
        Rating::Fair
    } else {
        Rating::Poor
    };
    CourseFit {
        turn,
        straight,
        hill,
        turf,
        percent,
        rating,
    }
}

/// One track of a course, as the screen shows it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackInfo {
    pub lap: String,
    pub straight: String,
    pub elevation: String,
}

/// What the screen needs to draw a course's map.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CourseMapInfo {
    pub label: String,
    pub surface: Surface,
    pub layout: Option<String>,
    pub lap: f64,
    pub straight: f64,
    pub after_goal: f64,
    pub straight_course: bool,
}

/// The maps of a course: its turf, inner and outer where it has both, and its dirt.
fn course_maps(venue: Venue) -> Vec<CourseMapInfo> {
    let map = |label: String, surface: Surface, distance: u32| {
        let plan = plan_of(venue, surface, distance);
        CourseMapInfo {
            label,
            surface,
            layout: plan.name.map(str::to_string),
            lap: plan.lap,
            straight: plan.straight,
            after_goal: plan.after_goal,
            straight_course: plan.straight_course,
        }
    };
    // a distance that is on no inner course's list is run on the outer one
    let mut maps = match inner_distances(venue) {
        Some(inner) => vec![
            map("芝 内回り".to_string(), Surface::Turf, inner[0]),
            map("芝 外回り".to_string(), Surface::Turf, 0),
        ],
        None => vec![map("芝".to_string(), Surface::Turf, 0)],
    };
    if venue == Venue::Niigata {
        maps.push(map(
            "芝 直線".to_string(),
            Surface::Turf,
            STRAIGHT_COURSE_METRES,
        ));
    }
    maps.push(map("ダート".to_string(), Surface::Dirt, 0));
    maps
}

/// A course, as the screen shows it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CourseInfo {
    /// A map for each way round the turf and for the dirt.
    pub maps: Vec<CourseMapInfo>,
    pub venue: Venue,
    pub name: String,
    pub place: String,
    pub turn: Turn,
    pub grass: String,
    pub turf: TrackInfo,
    pub dirt: TrackInfo,
    pub features: String,
    pub suits: String,
}

fn track_info(t: &Track) -> TrackInfo {
    TrackInfo {
        lap: t.lap_text.to_string(),
        straight: t.straight_text.to_string(),
        elevation: t.elevation_text.to_string(),
    }
}

pub fn course_infos() -> Vec<CourseInfo> {
    COURSES
        .iter()
        .map(|c| CourseInfo {
            maps: course_maps(c.venue),
            venue: c.venue,
            name: c.name.to_string(),
            place: c.place.to_string(),
            turn: c.turn,
            grass: match c.grass {
                Grass::Western => "洋芝(3種混合)".to_string(),
                Grass::Japanese => "野芝(洋芝のオーバーシードあり)".to_string(),
            },
            turf: track_info(&c.turf),
            dirt: track_info(&c.dirt),
            features: c.features.to_string(),
            suits: c.suits.to_string(),
        })
        .collect()
}

#[tauri::command]
pub fn get_courses() -> Vec<CourseInfo> {
    course_infos()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn each_course_lists_a_map_for_each_way_round_and_its_dirt() {
        let infos = course_infos();
        let maps = |name: &str| infos.iter().find(|c| c.name == name).unwrap().maps.clone();
        let labels = |name: &str| {
            maps(name)
                .iter()
                .map(|m| m.label.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(labels("東京"), ["芝", "ダート"]);
        assert_eq!(labels("京都"), ["芝 内回り", "芝 外回り", "ダート"]);
        assert_eq!(labels("中山"), ["芝 内回り", "芝 外回り", "ダート"]);
        assert_eq!(labels("阪神"), ["芝 内回り", "芝 外回り", "ダート"]);
        assert_eq!(
            labels("新潟"),
            ["芝 内回り", "芝 外回り", "芝 直線", "ダート"]
        );
        let straight = maps("新潟")[2].clone();
        assert!(straight.straight_course);
        assert_eq!(
            (straight.lap, straight.layout.as_deref()),
            (1000.0, Some("直線"))
        );
        assert!(maps("新潟").iter().filter(|m| m.straight_course).count() == 1);
        assert!(maps("東京").iter().all(|m| !m.straight_course));
        let kyoto = maps("京都");
        assert_eq!(
            (kyoto[0].lap, kyoto[0].layout.as_deref()),
            (1782.8, Some("内回り"))
        );
        assert_eq!((kyoto[1].lap, kyoto[1].straight), (1894.3, 403.7));
        assert_eq!(kyoto[2].surface, Surface::Dirt);
        assert_eq!(kyoto[2].after_goal, 93.0);
    }

    #[test]
    fn all_ten_courses_are_there_once() {
        let venues: HashSet<Venue> = COURSES.iter().map(|c| c.venue).collect();
        assert_eq!(venues.len(), 10);
        for c in &COURSES {
            assert_eq!(course_of(c.venue).name, c.name);
            assert!(!c.features.is_empty() && !c.suits.is_empty());
        }
    }

    #[test]
    fn three_courses_go_left_and_seven_right() {
        let left: Vec<&str> = COURSES
            .iter()
            .filter(|c| c.turn == Turn::Left)
            .map(|c| c.name)
            .collect();
        assert_eq!(left, vec!["新潟", "東京", "中京"]);
    }

    #[test]
    fn only_the_hokkaido_courses_have_western_turf() {
        let western: Vec<&str> = COURSES
            .iter()
            .filter(|c| c.grass == Grass::Western)
            .map(|c| c.name)
            .collect();
        assert_eq!(western, vec!["札幌", "函館"]);
    }

    #[test]
    fn the_figures_are_the_ones_the_jra_publishes() {
        assert_eq!(course_of(Venue::Tokyo).turf.straight, 525.9);
        assert_eq!(course_of(Venue::Nakayama).turf.elevation, 5.3);
        assert_eq!(course_of(Venue::Niigata).turf.straight, 658.7);
        assert_eq!(course_of(Venue::Hakodate).turf.straight, 262.1);
        assert_eq!(course_of(Venue::Fukushima).turf.lap_text, "1,600m");
        let turf = |v, d| plan_of(v, Surface::Turf, d);
        let dirt = |v, d| plan_of(v, Surface::Dirt, d);
        // the inner and outer courses, by the distance
        assert_eq!(turf(Venue::Kyoto, 2000).lap, 1782.8);
        assert_eq!(turf(Venue::Kyoto, 2000).name, Some("内回り"));
        assert_eq!(turf(Venue::Kyoto, 2400).lap, 1894.3);
        assert_eq!(turf(Venue::Kyoto, 2400).straight, 403.7);
        assert_eq!(turf(Venue::Kyoto, 2400).name, Some("外回り"));
        assert_eq!(turf(Venue::Hanshin, 1600).straight, 473.6);
        assert_eq!(turf(Venue::Hanshin, 2000).straight, 356.5);
        assert_eq!(turf(Venue::Nakayama, 1600).name, Some("外回り"));
        assert_eq!(turf(Venue::Nakayama, 2000).lap, 1667.1);
        assert_eq!(turf(Venue::Niigata, 1800).straight, 658.7);
        assert_eq!(turf(Venue::Tokyo, 2400).name, None);
        // the corners run and where the gate stands
        assert_eq!(turf(Venue::Tokyo, 2000).corners, 3);
        assert_eq!(turf(Venue::Hanshin, 1600).corners, 2);
        assert!(dirt(Venue::Tokyo, 1600).turf_start);
        assert!(!dirt(Venue::Tokyo, 2100).turf_start);
        assert!(!turf(Venue::Tokyo, 1600).turf_start);
        // the straight course
        let straight = turf(Venue::Niigata, 1000);
        assert!(straight.straight_course);
        assert_eq!(straight.corners, 0);
        assert!(!turf(Venue::Niigata, 1200).straight_course);
        // worked out from the distance from a gate in the home straight to the first corner;
        // the goal is one line, so turf and dirt should come out close at each course
        assert_eq!(turf(Venue::Tokyo, 2400).after_goal, 33.0);
        assert_eq!(dirt(Venue::Tokyo, 2100).after_goal, 35.0);
        assert_eq!(turf(Venue::Fukushima, 1800).after_goal, 105.0);
        for c in &COURSES {
            let gap = after_goal(c.venue, Surface::Turf) - after_goal(c.venue, Surface::Dirt);
            assert!(gap.abs() <= 22.0, "{}", c.name);
        }
        // the goal stands short of the first corner
        assert!(turf(Venue::Hakodate, 1800).after_goal > turf(Venue::Sapporo, 1800).after_goal);
        assert_eq!(course_of(Venue::Fukushima).turf.lap_metres(), 1600.0);
        assert_eq!(course_of(Venue::Tokyo).turf.lap_metres(), 2083.1);
        // every race of the calendar has a plan with room for its corners
        for r in &crate::calendar::shared().races {
            let p = plan_of(r.venue, r.surface, r.distance);
            assert!(
                p.straight_course || p.lap > 2.0 * (p.straight + p.after_goal),
                "{:?} {}",
                r.venue,
                r.distance
            );
            assert!(
                p.straight_course || p.corners > 0,
                "{:?} {}",
                r.venue,
                r.distance
            );
        }
        assert_eq!(course_of(Venue::Nakayama).turf.lap_metres(), 1667.1);
        for c in &COURSES {
            for t in [&c.turf, &c.dirt] {
                assert!(t.lap_metres() > t.straight * 2.0, "{}", c.name);
            }
        }
    }

    #[test]
    fn nakayama_is_the_hilliest_and_niigata_has_the_longest_straight() {
        let hilliest = COURSES
            .iter()
            .max_by(|a, b| a.turf.elevation.total_cmp(&b.turf.elevation))
            .unwrap();
        assert_eq!(hilliest.name, "中山");
        let longest = COURSES
            .iter()
            .max_by(|a, b| a.turf.straight.total_cmp(&b.turf.straight))
            .unwrap();
        assert_eq!(longest.name, "新潟");
    }

    /// A name with this turning preference.
    fn named(pref: Option<Turn>) -> String {
        (0..1000)
            .map(|i| format!("ウマ{i}"))
            .find(|n| turn_preference(n) == pref)
            .unwrap()
    }

    #[test]
    fn a_horse_keeps_its_preference_and_all_kinds_are_found() {
        assert_eq!(turn_preference("アオバ"), turn_preference("アオバ"));
        let mut counts = [0; 3];
        for i in 0..1000 {
            match turn_preference(&format!("ウマ{i}")) {
                Some(Turn::Left) => counts[0] += 1,
                Some(Turn::Right) => counts[1] += 1,
                None => counts[2] += 1,
            }
        }
        assert!(
            counts[0] > 200 && counts[1] > 200 && counts[2] > 300,
            "{counts:?}"
        );
    }

    #[test]
    fn a_horse_does_better_going_its_own_way() {
        let left = named(Some(Turn::Left));
        let at = |v| fit(v, Surface::Dirt, &left, None, 60).turn;
        assert_eq!(at(Venue::Tokyo), Factor::Plus);
        assert_eq!(at(Venue::Nakayama), Factor::Minus);
        let either = named(None);
        assert_eq!(
            fit(Venue::Tokyo, Surface::Turf, &either, None, 60).turn,
            Factor::Even
        );
    }

    #[test]
    fn a_long_straight_suits_a_closer_and_a_short_one_a_front_runner() {
        let n = named(None);
        let at = |v, s| fit(v, Surface::Turf, &n, Some(s), 60).straight;
        assert_eq!(at(Venue::Tokyo, Style::Oikomi), Factor::Plus);
        assert_eq!(at(Venue::Tokyo, Style::Sashi), Factor::Plus);
        assert_eq!(at(Venue::Tokyo, Style::Senko), Factor::Even);
        assert_eq!(at(Venue::Tokyo, Style::Nige), Factor::Minus);
        assert_eq!(at(Venue::Hakodate, Style::Nige), Factor::Plus);
        assert_eq!(at(Venue::Hakodate, Style::Senko), Factor::Plus);
        assert_eq!(at(Venue::Hakodate, Style::Sashi), Factor::Even);
        assert_eq!(at(Venue::Hakodate, Style::Oikomi), Factor::Minus);
    }

    #[test]
    fn a_middling_straight_favours_nobody_nor_does_an_unknown_style() {
        let n = named(None);
        for s in [Style::Nige, Style::Senko, Style::Sashi, Style::Oikomi] {
            assert_eq!(
                fit(Venue::Kyoto, Surface::Dirt, &n, Some(s), 60).straight,
                Factor::Even
            );
        }
        assert_eq!(
            fit(Venue::Tokyo, Surface::Turf, &n, None, 60).straight,
            Factor::Even
        );
    }

    #[test]
    fn hills_suit_a_powerful_horse_and_tell_on_a_weak_one() {
        let n = named(None);
        let hill = |v, p| fit(v, Surface::Turf, &n, None, p).hill;
        assert_eq!(hill(Venue::Nakayama, STRONG), Factor::Plus);
        assert_eq!(hill(Venue::Nakayama, STRONG - 1), Factor::Even);
        assert_eq!(hill(Venue::Nakayama, WEAK + 1), Factor::Even);
        assert_eq!(hill(Venue::Nakayama, WEAK), Factor::Minus);
        // a flat course makes no difference
        assert_eq!(hill(Venue::Sapporo, 100), Factor::Even);
        // Hanshin rises little overall, but climbs to the finish
        assert_eq!(hill(Venue::Hanshin, 100), Factor::Plus);
    }

    #[test]
    fn western_turf_suits_power_only_on_the_turf_in_hokkaido() {
        let n = named(None);
        assert_eq!(
            fit(Venue::Sapporo, Surface::Turf, &n, None, 80).turf,
            Factor::Plus
        );
        assert_eq!(
            fit(Venue::Sapporo, Surface::Turf, &n, None, 40).turf,
            Factor::Minus
        );
        assert_eq!(
            fit(Venue::Sapporo, Surface::Dirt, &n, None, 80).turf,
            Factor::Even
        );
        assert_eq!(
            fit(Venue::Tokyo, Surface::Turf, &n, None, 80).turf,
            Factor::Even
        );
    }

    #[test]
    fn the_effects_add_up_and_stay_small() {
        let right = named(Some(Turn::Right));
        // right-handed, short straight, hilly, western: everything in its favour
        let best = fit(
            Venue::Hakodate,
            Surface::Turf,
            &right,
            Some(Style::Nige),
            90,
        );
        assert_eq!(best.percent, 3.5);
        assert_eq!(best.rating, Rating::Great);
        assert_eq!(best.multiplier(), 1.035);
        let worst = fit(
            Venue::Hakodate,
            Surface::Turf,
            &right,
            Some(Style::Oikomi),
            20,
        );
        assert_eq!(worst.percent, 1.0 - 1.0 - 0.75 - 0.75);
        assert_eq!(worst.rating, Rating::Poor);
        for c in &COURSES {
            for s in [Surface::Turf, Surface::Dirt] {
                for p in [20, 60, 90] {
                    let f = fit(c.venue, s, &right, Some(Style::Sashi), p);
                    assert!(f.percent.abs() <= 3.5);
                }
            }
        }
    }

    #[test]
    fn the_rating_follows_the_percent() {
        let n = named(None);
        let plain = fit(Venue::Fukushima, Surface::Turf, &n, Some(Style::Sashi), 60);
        assert_eq!((plain.percent, plain.rating), (0.0, Rating::Fair));
        let good = fit(Venue::Fukushima, Surface::Turf, &n, Some(Style::Nige), 60);
        assert_eq!((good.percent, good.rating), (1.0, Rating::Good));
        let poor = fit(Venue::Fukushima, Surface::Turf, &n, Some(Style::Oikomi), 60);
        assert_eq!((poor.percent, poor.rating), (-1.0, Rating::Poor));
    }

    #[test]
    fn the_screen_gets_the_courses_in_words() {
        let infos = course_infos();
        assert_eq!(infos.len(), 10);
        let v = serde_json::to_value(&infos[4]).unwrap();
        assert_eq!(v["venue"], "tokyo");
        assert_eq!(v["name"], "東京");
        assert_eq!(v["turn"], "left");
        assert_eq!(v["turf"]["straight"], "525.9m");
        assert_eq!(v["dirt"]["elevation"], "2.5m");
        assert_eq!(
            serde_json::to_value(&infos[0]).unwrap()["grass"],
            "洋芝(3種混合)"
        );
    }
}
