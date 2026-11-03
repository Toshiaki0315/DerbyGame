//! The jockeys (騎手) who ride the stable's horses in calendar races. All are
//! fictional. A jockey brings out more of a horse the better they are, and more
//! still when the horse's running style and the way the course turns are their
//! own; a tactful one settles a hot horse. The better they are, the more they cost.
//! Half are based in the east (美浦), half in the west (栗東).

use crate::course::{Region, Turn};
use crate::race::Style;
use serde::{Deserialize, Serialize};

/// Skill at which a jockey neither adds to nor takes from a horse (the rivals' riders).
const AVERAGE_SKILL: f64 = 60.0;
/// Percent of ability per point of skill above or below the average.
const SKILL_PERCENT: f64 = 0.06;
/// Percent for a running style or a way round that is the jockey's own.
const STYLE_PERCENT: f64 = 0.5;
const TURN_PERCENT: f64 = 0.5;
/// Percent for each race in a row together (コンビ), up to the limit.
const PARTNER_PERCENT: f64 = 0.2;
const PARTNER_LIMIT: f64 = 1.0;

/// Who a trainer can afford to book, by what they are aiming for.
pub const BUDGET_STEADY: u32 = 6;
pub const BUDGET_BALANCED: u32 = 12;
pub const BUDGET_GLORY: u32 = u32::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Jockey {
    pub id: u8,
    pub name: &'static str,
    pub skill: u32,
    pub style: Style,
    pub turn: Turn,
    /// Where they are based (美浦 or 栗東).
    pub region: Region,
    /// How many points of temperament a horse behaves as if it had with them up.
    pub tact: u32,
    /// What the stable pays for each ride (騎乗料).
    pub fee: u32,
    pub description: &'static str,
}

impl Jockey {
    /// What they add to a horse with this style at a course turning this way, in
    /// percent, having ridden it this many races in a row.
    pub fn percent(&self, style: Option<Style>, turn: Turn, rides_together: u32) -> f64 {
        let own_style = if style == Some(self.style) {
            STYLE_PERCENT
        } else {
            0.0
        };
        let own_turn = if turn == self.turn { TURN_PERCENT } else { 0.0 };
        (self.skill as f64 - AVERAGE_SKILL) * SKILL_PERCENT
            + own_style
            + own_turn
            + (rides_together as f64 * PARTNER_PERCENT).min(PARTNER_LIMIT)
    }

    pub fn info(&self) -> JockeyInfo {
        JockeyInfo {
            id: self.id,
            name: self.name.to_string(),
            skill: self.skill,
            style: self.style,
            turn: self.turn,
            region: self.region.label().to_string(),
            tact: self.tact,
            fee: self.fee,
            description: self.description.to_string(),
        }
    }
}

/// A jockey as the screen shows them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JockeyInfo {
    pub id: u8,
    pub name: String,
    pub skill: u32,
    pub style: Style,
    pub turn: Turn,
    /// Where they are based: 美浦 or 栗東.
    pub region: String,
    pub tact: u32,
    pub fee: u32,
    pub description: String,
}

/// A jockey booked for a horse in a race.
#[derive(Debug, Clone, PartialEq)]
pub struct Ride {
    pub id: u8,
    pub name: String,
    pub percent: f64,
    pub tact: u32,
    pub fee: u32,
}

#[allow(clippy::too_many_arguments)]
const fn jockey(
    id: u8,
    name: &'static str,
    skill: u32,
    style: Style,
    turn: Turn,
    region: Region,
    tact: u32,
    fee: u32,
    description: &'static str,
) -> Jockey {
    Jockey {
        id,
        name,
        skill,
        style,
        turn,
        region,
        tact,
        fee,
        description,
    }
}

pub const JOCKEYS: [Jockey; 10] = [
    jockey(
        0,
        "神崎 颯",
        92,
        Style::Sashi,
        Turn::Left,
        Region::East,
        8,
        20,
        "当代随一の名手。直線の長い左回りで鋭い末脚を引き出します。",
    ),
    jockey(
        1,
        "獅童 蓮",
        88,
        Style::Nige,
        Turn::Right,
        Region::West,
        4,
        18,
        "果敢に先手を奪う勝負師。右回りの逃げは一級品です。",
    ),
    jockey(
        2,
        "水無月 凛",
        84,
        Style::Sashi,
        Turn::Right,
        Region::West,
        14,
        15,
        "折り合いの名人。気性の難しい馬でも落ち着かせます。",
    ),
    jockey(
        3,
        "高峰 岳",
        80,
        Style::Senko,
        Turn::Right,
        Region::East,
        8,
        12,
        "好位から確実に伸ばす堅実派です。",
    ),
    jockey(
        4,
        "遠野 楓",
        76,
        Style::Oikomi,
        Turn::Left,
        Region::West,
        6,
        10,
        "最後方からの一撃が持ち味の追込職人です。",
    ),
    jockey(
        5,
        "剣持 大地",
        72,
        Style::Senko,
        Turn::Left,
        Region::East,
        5,
        8,
        "手頃な騎乗料で左回りの先行馬を巧みに導きます。",
    ),
    jockey(
        6,
        "小鳥遊 葵",
        66,
        Style::Nige,
        Turn::Right,
        Region::West,
        10,
        6,
        "急成長中の若手。馬を落ち着かせて前へ運びます。",
    ),
    jockey(
        7,
        "結城 陸",
        60,
        Style::Sashi,
        Turn::Right,
        Region::East,
        4,
        5,
        "安定した中堅。平均的な騎乗を手頃な料金で提供します。",
    ),
    jockey(
        8,
        "久我 宗介",
        56,
        Style::Senko,
        Turn::Right,
        Region::West,
        15,
        4,
        "引退間近のベテラン。腕は落ちても馬をなだめる技は健在です。",
    ),
    jockey(
        9,
        "若葉 翔",
        52,
        Style::Nige,
        Turn::Left,
        Region::East,
        2,
        3,
        "デビューしたての新人。騎乗料が安いのが魅力です。",
    ),
];

pub fn jockey_by_id(id: u8) -> Option<&'static Jockey> {
    JOCKEYS.get(id as usize)
}

/// The jockey a trainer books: of those they can afford and who are not riding
/// another of the stable's horses in the race, the one who adds the most to this
/// horse, from the trainer's own side if anyone there is free. With nobody within
/// budget, the cheapest who is free.
pub fn trainer_pick(
    budget: u32,
    style: Option<Style>,
    turn: Turn,
    taken: &[u8],
    region: Option<Region>,
) -> Option<&'static Jockey> {
    let free = || JOCKEYS.iter().filter(|j| !taken.contains(&j.id));
    let best_of = |near: bool| {
        free()
            .rev()
            .filter(|j| j.fee <= budget && (!near || region.is_none_or(|r| j.region == r)))
            .max_by(|a, b| {
                a.percent(style, turn, 0)
                    .total_cmp(&b.percent(style, turn, 0))
            })
    };
    best_of(true)
        .or_else(|| best_of(false))
        .or_else(|| free().min_by_key(|j| j.fee))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn the_jockeys_are_split_evenly_between_east_and_west() {
        for region in [Region::East, Region::West] {
            let here: Vec<_> = JOCKEYS.iter().filter(|j| j.region == region).collect();
            assert_eq!(here.len(), 5, "{region:?}");
            // every trainer's budget finds someone on their own side
            assert!(here.iter().any(|j| j.fee <= BUDGET_STEADY), "{region:?}");
            assert!(
                here.iter()
                    .any(|j| j.fee > BUDGET_STEADY && j.fee <= BUDGET_BALANCED),
                "{region:?}"
            );
            assert!(here.iter().any(|j| j.fee > BUDGET_BALANCED), "{region:?}");
        }
    }

    #[test]
    fn a_trainer_books_from_their_own_side_while_anyone_there_is_free() {
        for region in [Region::East, Region::West] {
            for budget in [BUDGET_STEADY, BUDGET_BALANCED, BUDGET_GLORY] {
                let j = trainer_pick(budget, None, Turn::Left, &[], Some(region)).unwrap();
                assert_eq!(j.region, region);
                assert!(j.fee <= budget);
            }
            let everyone_here: Vec<u8> = JOCKEYS
                .iter()
                .filter(|j| j.region == region)
                .map(|j| j.id)
                .collect();
            let other =
                trainer_pick(BUDGET_GLORY, None, Turn::Left, &everyone_here, Some(region)).unwrap();
            assert_ne!(other.region, region);
        }
    }

    #[test]
    fn a_jockey_is_shown_with_their_side() {
        let v = serde_json::to_value(jockey_by_id(0).unwrap().info()).unwrap();
        assert_eq!(v["region"], "美浦");
        let v = serde_json::to_value(jockey_by_id(1).unwrap().info()).unwrap();
        assert_eq!(v["region"], "栗東");
    }

    #[test]
    fn there_are_ten_jockeys_with_their_own_ids_and_names() {
        for (i, j) in JOCKEYS.iter().enumerate() {
            assert_eq!(j.id as usize, i);
            assert!(!j.description.is_empty());
        }
        let names: HashSet<_> = JOCKEYS.iter().map(|j| j.name).collect();
        assert_eq!(names.len(), 10);
        assert!(jockey_by_id(0).is_some() && jockey_by_id(10).is_none());
    }

    #[test]
    fn the_better_the_jockey_the_more_they_cost() {
        let by_skill = {
            let mut v = JOCKEYS.to_vec();
            v.sort_by_key(|j| j.skill);
            v
        };
        for pair in by_skill.windows(2) {
            assert!(
                pair[0].fee <= pair[1].fee,
                "{} {}",
                pair[0].name,
                pair[1].name
            );
        }
    }

    #[test]
    fn every_style_and_way_round_has_its_jockeys() {
        for s in [Style::Nige, Style::Senko, Style::Sashi, Style::Oikomi] {
            assert!(JOCKEYS.iter().any(|j| j.style == s), "{s:?}");
        }
        for t in [Turn::Left, Turn::Right] {
            assert!(JOCKEYS.iter().any(|j| j.turn == t));
        }
    }

    fn other_turn(t: Turn) -> Turn {
        match t {
            Turn::Left => Turn::Right,
            Turn::Right => Turn::Left,
        }
    }

    #[test]
    fn a_jockey_adds_for_skill_for_their_style_their_way_round_and_time_together() {
        let j = jockey_by_id(0).unwrap(); // skill 92, 差し, 左
        let away = other_turn(j.turn);
        let base = (92.0 - 60.0) * SKILL_PERCENT;
        assert_eq!(j.percent(None, away, 0), base);
        assert_eq!(j.percent(Some(Style::Sashi), away, 0), base + STYLE_PERCENT);
        assert_eq!(j.percent(Some(Style::Nige), away, 0), base);
        assert_eq!(j.percent(None, Turn::Left, 0), base + TURN_PERCENT);
        assert_eq!(j.percent(None, away, 2), base + 2.0 * PARTNER_PERCENT);
        assert_eq!(j.percent(None, away, 100), base + PARTNER_LIMIT);
    }

    #[test]
    fn a_poor_jockey_can_cost_a_horse_something() {
        let j = jockey_by_id(9).unwrap();
        assert!(j.percent(None, other_turn(j.turn), 0) < 0.0);
    }

    #[test]
    fn a_trainer_books_the_jockey_who_helps_most_within_budget() {
        // the best on the left; on the right, the next best has the edge of their own way round
        assert_eq!(
            trainer_pick(BUDGET_GLORY, None, Turn::Left, &[], None)
                .unwrap()
                .id,
            0
        );
        assert_eq!(
            trainer_pick(BUDGET_GLORY, None, Turn::Right, &[], None)
                .unwrap()
                .id,
            1
        );
        let mid = trainer_pick(BUDGET_BALANCED, None, Turn::Right, &[], None).unwrap();
        assert!(mid.fee <= BUDGET_BALANCED);
        let low = trainer_pick(BUDGET_STEADY, None, Turn::Right, &[], None).unwrap();
        assert!(low.fee <= BUDGET_STEADY);
        assert!(mid.skill > low.skill);
    }

    #[test]
    fn a_trainer_takes_the_style_and_course_into_account() {
        let style =
            trainer_pick(BUDGET_STEADY, Some(Style::Senko), Turn::Right, &[], None).unwrap();
        let other = trainer_pick(BUDGET_STEADY, Some(Style::Nige), Turn::Right, &[], None).unwrap();
        assert_eq!(other.style, Style::Nige);
        assert!(
            style.percent(Some(Style::Senko), Turn::Right, 0)
                >= other.percent(Some(Style::Senko), Turn::Right, 0)
        );
    }

    #[test]
    fn a_jockey_can_only_ride_one_horse_in_a_race() {
        let first = trainer_pick(BUDGET_GLORY, None, Turn::Left, &[], None).unwrap();
        let second = trainer_pick(BUDGET_GLORY, None, Turn::Left, &[first.id], None).unwrap();
        assert_ne!(first.id, second.id);
    }

    #[test]
    fn with_nobody_in_budget_the_cheapest_free_jockey_rides() {
        let cheapest = trainer_pick(0, None, Turn::Left, &[], None).unwrap();
        assert_eq!(cheapest.fee, 3);
        let next = trainer_pick(0, None, Turn::Left, &[cheapest.id], None).unwrap();
        assert_eq!(next.fee, 4);
    }

    #[test]
    fn nobody_is_left_only_if_every_jockey_is_taken() {
        let all: Vec<u8> = JOCKEYS.iter().map(|j| j.id).collect();
        assert_eq!(
            trainer_pick(BUDGET_GLORY, None, Turn::Left, &all, None),
            None
        );
    }

    #[test]
    fn a_jockey_is_shown_with_their_ways() {
        let v = serde_json::to_value(JOCKEYS[2].info()).unwrap();
        assert_eq!(v["name"], "水無月 凛");
        assert_eq!(v["style"], "sashi");
        assert_eq!(v["turn"], "right");
        assert_eq!(v["fee"], 15);
    }
}
