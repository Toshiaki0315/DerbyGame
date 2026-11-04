//! The trainers (調教師) a horse can be sent to. Each has a stable and their own
//! ways: how hard they work a horse, what distances they like, how often they
//! run it, and what they run it for. Half have their stables in the east (美浦),
//! half in the west (栗東).

use crate::course::Region;
use crate::game::Condition;
use serde::{Deserialize, Serialize};

/// The most horses one stable can hold.
pub const STABLE_CAPACITY: usize = 10;
/// What a trainer brings out of a horse over their distance (speed and stamina).
pub const DISTANCE_BONUS: f64 = 1.03;

/// How hard a trainer works a horse: スパルタ down to 優しい.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Training {
    Sparta,
    Hard,
    Normal,
    Gentle,
    Kind,
}

impl Training {
    pub fn label(self) -> &'static str {
        match self {
            Training::Sparta => "スパルタ",
            Training::Hard => "厳しめ",
            Training::Normal => "標準",
            Training::Gentle => "優しめ",
            Training::Kind => "優しい",
        }
    }

    /// Workouts a week.
    pub fn sessions(self) -> u32 {
        match self {
            Training::Sparta => 2,
            _ => 1,
        }
    }

    /// A workout must leave the horse no more tired than this.
    pub fn work_ceiling(self) -> u32 {
        match self {
            Training::Sparta => 45,
            Training::Hard => 40,
            Training::Normal => 30,
            Training::Gentle => 20,
            Training::Kind => 10,
        }
    }

    /// From this fatigue the horse is rested.
    pub fn rest_from(self) -> u32 {
        match self {
            Training::Sparta => 60,
            Training::Hard | Training::Normal => 50,
            Training::Gentle => 40,
            Training::Kind => 30,
        }
    }
}

/// The distances a trainer likes to run their horses over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Distance {
    Sprint,
    Mile,
    Middle,
    Stayer,
    Any,
}

impl Distance {
    pub fn label(self) -> &'static str {
        match self {
            Distance::Sprint => "短距離",
            Distance::Mile => "マイル",
            Distance::Middle => "中距離",
            Distance::Stayer => "長距離",
            Distance::Any => "万能",
        }
    }

    pub fn covers(self, metres: u32) -> bool {
        match self {
            Distance::Sprint => metres <= 1400,
            Distance::Mile => (1401..=1800).contains(&metres),
            Distance::Middle => (1801..=2400).contains(&metres),
            Distance::Stayer => metres >= 2401,
            Distance::Any => true,
        }
    }

    /// Whether this is the trainer's own distance (an all-rounder has none).
    pub fn favours(self, metres: u32) -> bool {
        self != Distance::Any && self.covers(metres)
    }

    pub fn bonus(self, metres: u32) -> f64 {
        if self.favours(metres) {
            DISTANCE_BONUS
        } else {
            1.0
        }
    }
}

/// How often a trainer runs a horse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    /// Runs it whenever it can stand up to it (連闘も辞さない).
    Busy,
    /// Runs a horse in form again the next week, gives one out of form time.
    ByForm,
    Regular,
    /// Gives it plenty of time between races.
    Spaced,
}

impl Rotation {
    pub fn label(self) -> &'static str {
        match self {
            Rotation::Busy => "連闘も辞さない",
            Rotation::ByForm => "調子を見て連闘",
            Rotation::Regular => "標準",
            Rotation::Spaced => "間隔を空ける",
        }
    }

    /// Weeks that must have gone by since the last race (1: 連闘).
    pub fn gap(self, condition: Condition) -> u32 {
        match self {
            Rotation::Busy => 1,
            Rotation::ByForm => match condition {
                Condition::Great | Condition::Good => 1,
                _ => 4,
            },
            Rotation::Regular => 3,
            Rotation::Spaced => 7,
        }
    }

    /// The most tired a horse may be and still be sent out (the race's own limit also applies).
    pub fn fatigue_limit(self) -> u32 {
        match self {
            Rotation::Busy => 49,
            Rotation::ByForm => 40,
            Rotation::Regular => 30,
            Rotation::Spaced => 15,
        }
    }
}

/// What a trainer runs a horse for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aim {
    /// Races it should win, for steady prize money.
    Steady,
    Balanced,
    /// The biggest races it has any chance in, and saves it for a coming G1.
    Glory,
}

impl Aim {
    pub fn label(self) -> &'static str {
        match self {
            Aim::Steady => "堅実に賞金",
            Aim::Balanced => "バランス",
            Aim::Glory => "G1を目指す",
        }
    }

    /// How far above (or below) a horse's speed the rivals' may be in a race it is sent to.
    pub fn margin(self) -> f64 {
        match self {
            Aim::Steady => -4.0,
            Aim::Balanced => 0.0,
            Aim::Glory => 8.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Trainer {
    pub id: u8,
    pub name: &'static str,
    /// Where their stable is (美浦 or 栗東).
    pub region: Region,
    pub training: Training,
    pub distance: Distance,
    pub rotation: Rotation,
    pub aim: Aim,
    pub description: &'static str,
}

/// A trainer as the screen shows them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrainerInfo {
    pub id: u8,
    pub name: String,
    /// Where their stable is: 美浦 or 栗東.
    pub region: String,
    pub training: String,
    pub distance: String,
    pub rotation: String,
    pub aim: String,
    pub description: String,
}

impl Trainer {
    pub fn info(&self) -> TrainerInfo {
        TrainerInfo {
            id: self.id,
            name: self.name.to_string(),
            region: self.region.label().to_string(),
            training: self.training.label().to_string(),
            distance: self.distance.label().to_string(),
            rotation: self.rotation.label().to_string(),
            aim: self.aim.label().to_string(),
            description: self.description.to_string(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
const fn trainer(
    id: u8,
    name: &'static str,
    region: Region,
    training: Training,
    distance: Distance,
    rotation: Rotation,
    aim: Aim,
    description: &'static str,
) -> Trainer {
    Trainer {
        id,
        name,
        region,
        training,
        distance,
        rotation,
        aim,
        description,
    }
}

/// Trainer with every tendency in the middle.
#[cfg(test)]
pub const STANDARD: u8 = 4;

pub const TRAINERS: [Trainer; 10] = [
    trainer(
        0,
        "鬼塚 剛志",
        Region::East,
        Training::Sparta,
        Distance::Sprint,
        Rotation::Busy,
        Aim::Steady,
        "叩いて強くする短距離の職人。休ませず走らせて賞金を積み上げます。",
    ),
    trainer(
        1,
        "大河内 鉄平",
        Region::West,
        Training::Sparta,
        Distance::Stayer,
        Rotation::Regular,
        Aim::Glory,
        "猛稽古でスタミナを鍛え上げ、長距離のG1を狙う名伯楽です。",
    ),
    trainer(
        2,
        "獅子堂 健",
        Region::East,
        Training::Hard,
        Distance::Middle,
        Rotation::ByForm,
        Aim::Glory,
        "勝負どころを見極める勝負師。調子が良ければ連闘でも大レースへ向かいます。",
    ),
    trainer(
        3,
        "早瀬 駿",
        Region::West,
        Training::Hard,
        Distance::Sprint,
        Rotation::ByForm,
        Aim::Balanced,
        "スピード自慢を育てる若手。馬の調子を見ながら積極的に使います。",
    ),
    trainer(
        4,
        "松原 誠一",
        Region::East,
        Training::Normal,
        Distance::Any,
        Rotation::Regular,
        Aim::Balanced,
        "どんな馬も無理なく育てる、堅実で癖のないベテランです。",
    ),
    trainer(
        5,
        "白石 玲子",
        Region::West,
        Training::Normal,
        Distance::Mile,
        Rotation::ByForm,
        Aim::Steady,
        "マイル戦を得意とし、馬の調子に合わせて確実に賞金を稼ぎます。",
    ),
    trainer(
        6,
        "桐生 宗一郎",
        Region::West,
        Training::Normal,
        Distance::Middle,
        Rotation::Spaced,
        Aim::Glory,
        "間隔を空けて仕上げ、中距離の大一番に照準を合わせる理論派です。",
    ),
    trainer(
        7,
        "花村 ゆかり",
        Region::East,
        Training::Gentle,
        Distance::Mile,
        Rotation::Spaced,
        Aim::Steady,
        "馬の気持ちを大切にする調教師。勝てる舞台を選んで使います。",
    ),
    trainer(
        8,
        "森田 穏",
        Region::East,
        Training::Kind,
        Distance::Stayer,
        Rotation::Spaced,
        Aim::Balanced,
        "時間をかけてじっくり育てる長距離の名人。晩成の馬と相性が良い人です。",
    ),
    trainer(
        9,
        "春日 のどか",
        Region::West,
        Training::Kind,
        Distance::Any,
        Rotation::Regular,
        Aim::Steady,
        "疲れを残さない優しい調教が持ち味。距離を問わず手堅く走らせます。",
    ),
];

/// The trainer with this id (an unknown id gets the first).
/// Those who take over a stable as others retire, in the order they do.
pub const SUCCESSORS: [Trainer; 15] = [
    trainer(
        10,
        "柏木 洋介",
        Region::East,
        Training::Hard,
        Distance::Middle,
        Rotation::Regular,
        Aim::Balanced,
        "中距離路線で着実に勝ち星を重ねます。",
    ),
    trainer(
        11,
        "三条 真理",
        Region::West,
        Training::Gentle,
        Distance::Mile,
        Rotation::Spaced,
        Aim::Balanced,
        "馬に無理をさせない丁寧な仕事が評判です。",
    ),
    trainer(
        12,
        "大門 勝",
        Region::East,
        Training::Sparta,
        Distance::Sprint,
        Rotation::Busy,
        Aim::Steady,
        "強い調教で短距離馬を鍛え上げます。",
    ),
    trainer(
        13,
        "志摩 恭介",
        Region::West,
        Training::Normal,
        Distance::Stayer,
        Rotation::ByForm,
        Aim::Glory,
        "長距離の大レースを狙う野心家です。",
    ),
    trainer(
        14,
        "若林 悠",
        Region::East,
        Training::Kind,
        Distance::Any,
        Rotation::Spaced,
        Aim::Glory,
        "馬の成長を待つ若手調教師です。",
    ),
    trainer(
        15,
        "浜口 剛士",
        Region::West,
        Training::Hard,
        Distance::Any,
        Rotation::Regular,
        Aim::Steady,
        "堅実に賞金を積み上げます。",
    ),
    trainer(
        16,
        "東堂 彩",
        Region::East,
        Training::Normal,
        Distance::Mile,
        Rotation::ByForm,
        Aim::Glory,
        "マイル路線に強い期待の新人です。",
    ),
    trainer(
        17,
        "鷹取 健",
        Region::West,
        Training::Sparta,
        Distance::Middle,
        Rotation::Regular,
        Aim::Glory,
        "厳しい調教で大舞台を目指します。",
    ),
    trainer(
        18,
        "芹沢 拓",
        Region::East,
        Training::Gentle,
        Distance::Stayer,
        Rotation::Spaced,
        Aim::Balanced,
        "長い距離をじっくり仕上げます。",
    ),
    trainer(
        19,
        "朝倉 律",
        Region::West,
        Training::Normal,
        Distance::Sprint,
        Rotation::ByForm,
        Aim::Steady,
        "短距離で堅実に稼ぎます。",
    ),
    trainer(
        20,
        "国見 亮",
        Region::East,
        Training::Hard,
        Distance::Middle,
        Rotation::Busy,
        Aim::Balanced,
        "使いながら仕上げるのが得意です。",
    ),
    trainer(
        21,
        "八雲 澄",
        Region::West,
        Training::Kind,
        Distance::Mile,
        Rotation::Spaced,
        Aim::Balanced,
        "馬を大切にする調教師です。",
    ),
    trainer(
        22,
        "望月 俊",
        Region::East,
        Training::Normal,
        Distance::Any,
        Rotation::Regular,
        Aim::Balanced,
        "どの距離でも結果を出します。",
    ),
    trainer(
        23,
        "蓮見 貴",
        Region::West,
        Training::Hard,
        Distance::Stayer,
        Rotation::Regular,
        Aim::Glory,
        "長距離のG1を狙います。",
    ),
    trainer(
        24,
        "結城 奏",
        Region::East,
        Training::Gentle,
        Distance::Sprint,
        Rotation::ByForm,
        Aim::Steady,
        "スピード馬の扱いが上手です。",
    ),
];

/// Every trainer there will ever be.
pub const ALL_TRAINERS: usize = TRAINERS.len() + SUCCESSORS.len();

pub fn trainer_by_id(id: u8) -> &'static Trainer {
    TRAINERS
        .iter()
        .chain(&SUCCESSORS)
        .find(|t| t.id == id)
        .unwrap_or(&TRAINERS[0])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn the_trainers_are_split_evenly_between_east_and_west_with_every_distance_on_each_side() {
        for region in [Region::East, Region::West] {
            let here: Vec<_> = TRAINERS.iter().filter(|t| t.region == region).collect();
            assert_eq!(here.len(), 5, "{region:?}");
            for distance in [
                Distance::Sprint,
                Distance::Mile,
                Distance::Middle,
                Distance::Stayer,
                Distance::Any,
            ] {
                assert!(
                    here.iter().any(|t| t.distance == distance),
                    "{region:?} {distance:?}"
                );
            }
            for aim in [Aim::Steady, Aim::Balanced, Aim::Glory] {
                assert!(here.iter().any(|t| t.aim == aim), "{region:?} {aim:?}");
            }
        }
    }

    #[test]
    fn a_trainer_is_shown_with_their_side() {
        let v = serde_json::to_value(trainer_by_id(0).info()).unwrap();
        assert_eq!(v["region"], "美浦");
        let v = serde_json::to_value(trainer_by_id(1).info()).unwrap();
        assert_eq!(v["region"], "栗東");
    }

    #[test]
    fn there_are_ten_trainers_with_their_own_ids_names_and_combinations() {
        assert_eq!(TRAINERS.len(), 10);
        for (i, t) in TRAINERS.iter().enumerate() {
            assert_eq!(t.id as usize, i);
            assert!(!t.description.is_empty());
        }
        let names: HashSet<_> = TRAINERS.iter().map(|t| t.name).collect();
        assert_eq!(names.len(), 10);
        let ways: HashSet<_> = TRAINERS
            .iter()
            .map(|t| {
                (
                    t.training.label(),
                    t.distance.label(),
                    t.rotation.label(),
                    t.aim.label(),
                )
            })
            .collect();
        assert_eq!(ways.len(), 10);
    }

    #[test]
    fn every_tendency_is_found_among_the_trainers() {
        for training in [
            Training::Sparta,
            Training::Hard,
            Training::Normal,
            Training::Gentle,
            Training::Kind,
        ] {
            assert!(TRAINERS.iter().any(|t| t.training == training));
        }
        for distance in [
            Distance::Sprint,
            Distance::Mile,
            Distance::Middle,
            Distance::Stayer,
            Distance::Any,
        ] {
            assert!(TRAINERS.iter().any(|t| t.distance == distance));
        }
        for rotation in [
            Rotation::Busy,
            Rotation::ByForm,
            Rotation::Regular,
            Rotation::Spaced,
        ] {
            assert!(TRAINERS.iter().any(|t| t.rotation == rotation));
        }
        for aim in [Aim::Steady, Aim::Balanced, Aim::Glory] {
            assert!(TRAINERS.iter().any(|t| t.aim == aim));
        }
    }

    #[test]
    fn the_standard_trainer_is_in_the_middle_of_everything() {
        let t = trainer_by_id(STANDARD);
        assert_eq!(t.training, Training::Normal);
        assert_eq!(t.distance, Distance::Any);
        assert_eq!(t.rotation, Rotation::Regular);
        assert_eq!(t.aim, Aim::Balanced);
    }

    #[test]
    fn the_harder_the_trainer_the_more_a_horse_is_worked() {
        let ladder = [
            Training::Sparta,
            Training::Hard,
            Training::Normal,
            Training::Gentle,
            Training::Kind,
        ];
        for pair in ladder.windows(2) {
            assert!(pair[0].work_ceiling() > pair[1].work_ceiling());
            assert!(pair[0].rest_from() >= pair[1].rest_from());
            assert!(pair[0].sessions() >= pair[1].sessions());
        }
        assert_eq!(Training::Sparta.sessions(), 2);
        assert_eq!(Training::Kind.sessions(), 1);
    }

    #[test]
    fn each_distance_has_its_band_and_every_race_falls_in_exactly_one() {
        let bands = [
            Distance::Sprint,
            Distance::Mile,
            Distance::Middle,
            Distance::Stayer,
        ];
        for m in (1000..=3600).step_by(100) {
            assert_eq!(bands.iter().filter(|d| d.covers(m)).count(), 1, "{m}m");
            assert!(Distance::Any.covers(m));
        }
        assert!(Distance::Sprint.covers(1400) && Distance::Mile.covers(1600));
        assert!(Distance::Middle.covers(2400) && Distance::Stayer.covers(3000));
    }

    #[test]
    fn a_trainer_brings_more_out_of_a_horse_over_their_distance() {
        assert_eq!(Distance::Sprint.bonus(1200), DISTANCE_BONUS);
        assert_eq!(Distance::Sprint.bonus(1600), 1.0);
        assert_eq!(Distance::Stayer.bonus(3000), DISTANCE_BONUS);
        assert_eq!(Distance::Stayer.bonus(2400), 1.0);
        assert_eq!(DISTANCE_BONUS, 1.03);
    }

    #[test]
    fn an_all_rounder_has_no_distance_of_their_own() {
        for m in [1200, 1600, 2000, 3000] {
            assert_eq!(Distance::Any.bonus(m), 1.0);
            assert!(!Distance::Any.favours(m));
        }
        assert!(Distance::Mile.favours(1600));
        assert!(!Distance::Mile.favours(2000));
    }

    #[test]
    fn a_busy_trainer_runs_a_horse_sooner_than_a_patient_one() {
        use Condition::*;
        assert_eq!(Rotation::Busy.gap(Poor), 1);
        assert_eq!(Rotation::Regular.gap(Great), 3);
        assert_eq!(Rotation::Spaced.gap(Great), 7);
        assert!(Rotation::Busy.fatigue_limit() > Rotation::Regular.fatigue_limit());
        assert!(Rotation::Regular.fatigue_limit() > Rotation::Spaced.fatigue_limit());
    }

    #[test]
    fn a_trainer_who_watches_form_only_runs_a_horse_in_form_back_to_back() {
        use Condition::*;
        assert_eq!(Rotation::ByForm.gap(Great), 1);
        assert_eq!(Rotation::ByForm.gap(Good), 1);
        assert_eq!(Rotation::ByForm.gap(Normal), 4);
        assert_eq!(Rotation::ByForm.gap(Poor), 4);
    }

    #[test]
    fn the_ambitious_aim_higher_than_the_careful() {
        assert!(Aim::Glory.margin() > Aim::Balanced.margin());
        assert!(Aim::Balanced.margin() > Aim::Steady.margin());
    }

    #[test]
    fn a_trainer_is_shown_in_words() {
        let v = serde_json::to_value(trainer_by_id(1).info()).unwrap();
        assert_eq!(v["name"], "大河内 鉄平");
        assert_eq!(v["training"], "スパルタ");
        assert_eq!(v["distance"], "長距離");
        assert_eq!(v["rotation"], "標準");
        assert_eq!(v["aim"], "G1を目指す");
    }

    #[test]
    fn an_unknown_id_gets_a_trainer_all_the_same() {
        assert_eq!(trainer_by_id(200).id, 0);
    }
}
