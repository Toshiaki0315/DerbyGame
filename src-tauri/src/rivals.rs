//! Generates the other horses in a race (the rivals the player's stable meets),
//! scaled to the race's grade, distance, surface and entry conditions.

use crate::calendar::{AgeRule, CalendarRace, Grade, Surface};
use crate::course::CourseFit;
use crate::game::{Condition, Sex};
use crate::jockey::Ride;
use crate::race::{Gear, Style};
use crate::trainer::Distance;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// One runner in a race, whether from the player's stable or a generated rival.
#[derive(Debug, Clone, PartialEq)]
pub struct Entrant {
    pub name: String,
    pub speed: u32,
    pub stamina: u32,
    pub condition: Condition,
    pub sex: Sex,
    pub age: u8,
    /// Dirt aptitude, -2 (poor) to +2 (specialist).
    pub dirt: i8,
    /// Power (0-100): how well the horse copes with a heavy track.
    pub power: u32,
    /// The age its growth type peaks at (see [`crate::form`]).
    pub peak_age: u8,
    /// The range of distances (m) the horse is best over.
    pub apt_min: u32,
    pub apt_max: u32,
    /// Running style (脚質), if known.
    pub style: Option<Style>,
    /// Temperament (気性), 0-100.
    pub temper: u32,
    /// Coat colour (毛色).
    pub coat_color: String,
    /// Racing equipment (馬具).
    pub gear: Gear,
    /// Fatigue (疲労度, 0-100); rivals come to a race fresh.
    pub fatigue: u32,
    /// The distances its trainer favours; rivals' trainers are not modelled.
    pub trainer_distance: Option<Distance>,
    /// How the course suits it (set when a race's field is made up).
    pub course_fit: Option<CourseFit>,
    /// Who rides it, for the stable's own in a calendar race.
    pub jockey: Option<Ride>,
    /// What it carries, in kilos (set when a race's field is made up).
    pub weight: Option<u32>,
    /// One of the rivals who race on year after year.
    pub star: bool,
    /// From the player's own stable.
    pub mine: bool,
}

/// Small deterministic generator (xorshift64*), so a race's field never changes between looks.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Normally distributed value (Box-Muller), like Python's `random.gauss`.
    pub fn gauss(&mut self, mean: f64, sd: f64) -> f64 {
        let u1 = 1.0 - self.next_f64(); // (0, 1], so ln() is finite
        let u2 = self.next_f64();
        mean + sd * (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }

    /// Uniform float in `lo..=hi`.
    pub fn range_f64(&mut self, lo: f64, hi: f64) -> f64 {
        lo + self.next_f64() * (hi - lo)
    }

    /// True with probability `p`.
    pub fn chance(&mut self, p: f64) -> bool {
        self.next_f64() < p
    }

    /// A random element of a non-empty slice.
    pub fn choice<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.range(0, items.len() as i64 - 1) as usize]
    }

    /// Uniform integer in `lo..=hi`.
    pub fn range(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.next_u64() % (hi - lo + 1) as u64) as i64
    }
}

/// Folds several numbers into one seed.
pub fn mix_seed(parts: &[u64]) -> u64 {
    parts.iter().fold(0xCBF2_9CE4_8422_2325u64, |h, &p| {
        (h ^ p).wrapping_mul(0x0000_0100_0000_01B3).rotate_left(23)
    })
}

const PREFIXES: [&str; 24] = [
    "アカツキ",
    "アサヒ",
    "ウミ",
    "オオゾラ",
    "カゼ",
    "キリ",
    "クロ",
    "コガネ",
    "サクラ",
    "シロ",
    "スイ",
    "ソラ",
    "タイヨウ",
    "ツキ",
    "ハヤテ",
    "ヒカリ",
    "ホシ",
    "マツリ",
    "ミドリ",
    "ユキ",
    "ライト",
    "ルナ",
    "レッド",
    "ロイヤル",
];
const SUFFIXES: [&str; 24] = [
    "アロー",
    "ウイング",
    "エース",
    "オーラ",
    "カイザー",
    "キング",
    "クイーン",
    "ゴールド",
    "サンダー",
    "スター",
    "ストーム",
    "ダンサー",
    "チャンプ",
    "ドリーム",
    "ナイト",
    "ハート",
    "ファイター",
    "フェザー",
    "ブレイブ",
    "ブレード",
    "ホープ",
    "マスター",
    "ランナー",
    "レジェンド",
];

fn unique_name(rng: &mut Rng, taken: &HashSet<String>) -> String {
    loop {
        let name = format!(
            "{}{}",
            PREFIXES[rng.range(0, PREFIXES.len() as i64 - 1) as usize],
            SUFFIXES[rng.range(0, SUFFIXES.len() as i64 - 1) as usize]
        );
        if !taken.contains(&name) {
            return name;
        }
    }
}

/// Typical speed of a rival, by grade.
pub fn speed_centre(grade: Grade) -> i64 {
    match grade {
        Grade::G1 => 80,
        Grade::G2 => 77,
        Grade::G3 => 74,
        Grade::Listed => 71,
        Grade::Open => 68,
        Grade::Win3 => 66,
        Grade::Win2 => 64,
        Grade::Win1 => 62,
        Grade::Maiden => 58,
        Grade::Debut => 57,
    }
}

/// Stamina range a horse entered at this distance would plausibly have.
fn stamina_range(distance: u32) -> (i64, i64) {
    match distance {
        0..=1499 => (45, 75),
        1500..=1999 => (55, 85),
        2000..=2399 => (60, 90),
        _ => (70, 95),
    }
}

/// How many rivals make the core of a field, and how far below them the rest are.
const FIELD_CORE: usize = 9;
const FILLER_GAP: i64 = 12;

/// `count` rivals for `race`, with names that avoid `taken` and each other.
pub fn generate(
    race: &CalendarRace,
    count: usize,
    taken: &HashSet<String>,
    seed: u64,
) -> Vec<Entrant> {
    let mut rng = Rng::new(seed);
    let mut used = taken.clone();
    let centre = speed_centre(race.grade);
    let (lo, hi) = stamina_range(race.distance);
    (0..count)
        .map(|i| {
            let centre = if i < FIELD_CORE {
                centre
            } else {
                centre - FILLER_GAP
            };
            let name = unique_name(&mut rng, &used);
            used.insert(name.clone());
            let age = match race.eligibility.age {
                AgeRule::Two => 2,
                AgeRule::Three => 3,
                AgeRule::ThreeUp => rng.range(3, 6) as u8,
                AgeRule::FourUp => rng.range(4, 7) as u8,
            };
            let sex = if race.eligibility.fillies_only || rng.range(0, 99) >= 70 {
                Sex::Female
            } else {
                Sex::Male
            };
            // horses entered in a dirt race can run on dirt; turf horses may or may not
            let dirt = match race.surface {
                Surface::Dirt => rng.range(0, 2),
                Surface::Turf => rng.range(-2, 1),
            } as i8;
            let style = Some(rival_style(&mut rng));
            // spread like the blood DB's, where inbreeding has worn temperaments down
            let temper = rng.range(10, 70) as u32;
            let coat_color = rival_coat(&mut rng).to_string();
            let peak_age = rival_peak_age(&mut rng, age);
            let (apt_min, apt_max) = rival_aptitude(&mut rng, race.distance);
            Entrant {
                name,
                speed: (centre + rng.range(-9, 9)).clamp(40, 98) as u32,
                stamina: rng.range(lo, hi) as u32,
                condition: Condition::from_roll(rng.next_f64()),
                sex,
                age,
                dirt,
                power: rng.range(45, 85) as u32,
                peak_age,
                apt_min,
                apt_max,
                style,
                temper,
                coat_color,
                gear: rival_gear(&mut rng, temper),
                fatigue: 0,
                trainer_distance: None,
                course_fit: None,
                jockey: None,
                weight: None,
                star: false,
                mine: false,
            }
        })
        .collect()
}

/// A rival who races on year after year (ライバル), keeping a record of their own.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Star {
    pub name: String,
    pub sex: Sex,
    pub age: u8,
    pub speed: u32,
    pub stamina: u32,
    pub power: u32,
    pub dirt: i8,
    pub temper: u32,
    pub style: Style,
    pub coat_color: String,
    pub gear: Gear,
    pub peak_age: u8,
    pub apt_min: u32,
    pub apt_max: u32,
    pub starts: u32,
    pub wins: u32,
    pub g1_wins: u32,
    pub earnings: u32,
    /// So far this year, for 年度代表馬.
    pub year_earnings: u32,
    pub year_g1: u32,
    /// Races against the stable, and how many of those the stable got the better of.
    pub met: u32,
    pub beaten: u32,
    /// The week of its last race, counted across the years.
    pub last_run: u32,
}

impl Star {
    /// The rival as a runner, on the form it shows that day.
    pub fn entrant(&self, rng: &mut Rng) -> Entrant {
        Entrant {
            name: self.name.clone(),
            speed: self.speed,
            stamina: self.stamina,
            condition: Condition::from_roll(rng.next_f64()),
            sex: self.sex,
            age: self.age,
            dirt: self.dirt,
            power: self.power,
            peak_age: self.peak_age,
            apt_min: self.apt_min,
            apt_max: self.apt_max,
            style: Some(self.style),
            temper: self.temper,
            coat_color: self.coat_color.clone(),
            gear: self.gear,
            fatigue: 0,
            trainer_distance: None,
            course_fit: None,
            jockey: None,
            weight: None,
            star: true,
            mine: false,
        }
    }
}

/// A new rival: well above the common run of the field.
pub fn new_star(rng: &mut Rng, age: u8, taken: &HashSet<String>) -> Star {
    let name = unique_name(rng, taken);
    let temper = rng.range(30, 80) as u32;
    let mid = 200 * rng.range(6, 14) as u32;
    let width = 200 * rng.range(1, 3) as u32;
    Star {
        name,
        sex: if rng.range(0, 99) < 35 {
            Sex::Female
        } else {
            Sex::Male
        },
        age,
        speed: rng.range(76, 96) as u32,
        stamina: rng.range(65, 95) as u32,
        power: rng.range(50, 90) as u32,
        dirt: rng.range(-2, 2) as i8,
        temper,
        style: rival_style(rng),
        coat_color: rival_coat(rng).to_string(),
        gear: rival_gear(rng, temper),
        peak_age: rng.range(3, 6) as u8,
        apt_min: mid.saturating_sub(width).max(1000),
        apt_max: (mid + width).min(3600),
        starts: 0,
        wins: 0,
        g1_wins: 0,
        earnings: 0,
        year_earnings: 0,
        year_g1: 0,
        met: 0,
        beaten: 0,
        last_run: 0,
    }
}

/// Half the hot-tempered rivals race in a メンコ; one in five wears a シャドーロール.
fn rival_gear(rng: &mut Rng, temper: u32) -> Gear {
    Gear {
        menko: temper < 40 && rng.chance(0.5),
        shadow_roll: rng.chance(0.2),
    }
}

/// 鹿毛 45%, 栗毛 25%, 青毛 15%, 芦毛 15%: the four colours of the blood database.
fn rival_coat(rng: &mut Rng) -> &'static str {
    match rng.range(0, 99) {
        0..=44 => "鹿毛",
        45..=69 => "栗毛",
        70..=84 => "青毛",
        _ => "芦毛",
    }
}

/// 逃げ 15%, 先行 35%, 差し 35%, 追込 15%.
fn rival_style(rng: &mut Rng) -> Style {
    match rng.range(0, 99) {
        0..=14 => Style::Nige,
        15..=49 => Style::Senko,
        50..=84 => Style::Sashi,
        _ => Style::Oikomi,
    }
}

/// Most rivals are in their prime; some are still developing or past it.
fn rival_peak_age(rng: &mut Rng, age: u8) -> u8 {
    if rng.range(0, 99) < 80 {
        (age as i64 + rng.range(-1, 1)).clamp(2, 7) as u8
    } else {
        rng.range(2, 7) as u8
    }
}

/// Most rivals are placed in a race they suit; some are out of their distance.
fn rival_aptitude(rng: &mut Rng, distance: u32) -> (u32, u32) {
    let d = distance as i64;
    let (lo, hi) = if rng.range(0, 99) < 85 {
        (d - 200 * rng.range(0, 3), d + 200 * rng.range(0, 3))
    } else if rng.range(0, 1) == 0 {
        // better over shorter
        let top = d - 200 * rng.range(1, 3);
        (top - 200 * rng.range(1, 3), top)
    } else {
        // better over longer
        let bottom = d + 200 * rng.range(1, 3);
        (bottom, bottom + 200 * rng.range(1, 3))
    };
    (lo.clamp(1000, 3600) as u32, hi.clamp(1000, 3600) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::fictional_calendar;
    use crate::horse::is_valid_name;

    fn races(grade: Grade) -> Vec<CalendarRace> {
        fictional_calendar()
            .races
            .into_iter()
            .filter(|r| r.grade == grade)
            .collect()
    }

    fn mean_speed(grade: Grade) -> f64 {
        let mut total = 0.0;
        let mut n = 0.0;
        for race in races(grade) {
            for e in generate(&race, 10, &HashSet::new(), 7) {
                total += e.speed as f64;
                n += 1.0;
            }
        }
        total / n
    }

    #[test]
    fn rivals_have_valid_unique_names_that_avoid_taken_ones() {
        let race = &races(Grade::G1)[0];
        let taken: HashSet<String> = ["サクラブレイブ".to_string()].into();
        for seed in 0..50 {
            let rivals = generate(race, 12, &taken, seed);
            let mut names = HashSet::new();
            for r in &rivals {
                assert!(is_valid_name(&r.name), "{}", r.name);
                assert!(!taken.contains(&r.name));
                assert!(names.insert(r.name.clone()), "duplicate {}", r.name);
                assert!(!r.mine);
            }
            assert_eq!(rivals.len(), 12);
        }
    }

    #[test]
    fn rivals_meet_the_race_entry_conditions() {
        for race in fictional_calendar().races {
            for r in generate(&race, 12, &HashSet::new(), 3) {
                assert!(
                    race.eligibility.allows(r.age, r.sex == Sex::Female),
                    "{}: {} (age {}, {:?}) is not eligible",
                    race.name,
                    r.name,
                    r.age,
                    r.sex
                );
            }
        }
    }

    #[test]
    fn higher_grades_draw_stronger_fields() {
        let (g1, g3, op) = (
            mean_speed(Grade::G1),
            mean_speed(Grade::G3),
            mean_speed(Grade::Open),
        );
        assert!(g1 > g3 + 3.0 && g3 > op + 3.0, "{g1:.1} {g3:.1} {op:.1}");
    }

    #[test]
    fn stats_stay_in_playable_bounds() {
        for race in fictional_calendar().races {
            for r in generate(&race, 12, &HashSet::new(), 11) {
                assert!((40..=98).contains(&r.speed), "{}", r.speed);
                assert!((45..=95).contains(&r.stamina), "{}", r.stamina);
                assert!((-2..=2).contains(&r.dirt));
                assert!((45..=85).contains(&r.power), "{}", r.power);
            }
        }
    }

    #[test]
    fn dirt_races_draw_dirt_capable_horses() {
        for race in fictional_calendar()
            .races
            .iter()
            .filter(|r| r.surface == Surface::Dirt)
        {
            assert!(generate(race, 12, &HashSet::new(), 5)
                .iter()
                .all(|r| r.dirt >= 0));
        }
    }

    #[test]
    fn long_races_draw_stayers_and_sprints_draw_speedsters() {
        let cal = fictional_calendar();
        let mean_stamina = |pred: &dyn Fn(&CalendarRace) -> bool| {
            let (mut t, mut n) = (0.0, 0.0);
            for race in cal.races.iter().filter(|r| pred(r)) {
                for e in generate(race, 10, &HashSet::new(), 9) {
                    t += e.stamina as f64;
                    n += 1.0;
                }
            }
            t / n
        };
        let long = mean_stamina(&|r| r.distance >= 2400);
        let sprint = mean_stamina(&|r| r.distance <= 1400);
        assert!(long > sprint + 15.0, "long {long:.1} sprint {sprint:.1}");
    }

    #[test]
    fn same_seed_gives_the_same_rivals_and_other_seeds_differ() {
        let race = &races(Grade::G2)[0];
        let a = generate(race, 8, &HashSet::new(), 1);
        assert_eq!(a, generate(race, 8, &HashSet::new(), 1));
        assert_ne!(a, generate(race, 8, &HashSet::new(), 2));
    }

    #[test]
    fn rivals_come_in_both_sexes_in_open_races() {
        let race = races(Grade::G3)
            .into_iter()
            .find(|r| !r.eligibility.fillies_only)
            .unwrap();
        let rivals = generate(&race, 60, &HashSet::new(), 4);
        assert!(rivals.iter().any(|r| r.sex == Sex::Male));
        assert!(rivals.iter().any(|r| r.sex == Sex::Female));
    }

    #[test]
    fn gauss_has_the_requested_mean_and_spread() {
        let mut rng = Rng::new(42);
        let xs: Vec<f64> = (0..20000).map(|_| rng.gauss(70.0, 5.0)).collect();
        let mean = xs.iter().sum::<f64>() / xs.len() as f64;
        let var = xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / xs.len() as f64;
        assert!((mean - 70.0).abs() < 0.2, "mean {mean}");
        assert!((var.sqrt() - 5.0).abs() < 0.2, "sd {}", var.sqrt());
        assert!(xs.iter().all(|x| x.is_finite()));
    }

    #[test]
    fn range_f64_stays_inside_its_bounds_and_covers_them() {
        let mut rng = Rng::new(3);
        let xs: Vec<f64> = (0..5000).map(|_| rng.range_f64(15.0, 25.0)).collect();
        assert!(xs.iter().all(|x| (15.0..=25.0).contains(x)));
        assert!(xs.iter().any(|x| *x < 16.0) && xs.iter().any(|x| *x > 24.0));
        let mean = xs.iter().sum::<f64>() / xs.len() as f64;
        assert!((mean - 20.0).abs() < 0.2, "{mean}");
    }

    #[test]
    fn chance_and_choice_behave() {
        let mut rng = Rng::new(7);
        let hits = (0..10000).filter(|_| rng.chance(0.3)).count();
        assert!((2800..3200).contains(&hits), "{hits}");
        assert!(!(0..100).any(|_| rng.chance(0.0)));
        assert!((0..100).all(|_| rng.chance(1.0)));
        let items = ["a", "b", "c"];
        let picked: std::collections::HashSet<_> = (0..200).map(|_| *rng.choice(&items)).collect();
        assert_eq!(picked.len(), 3);
        assert_eq!(*rng.choice(&["only"]), "only");
    }

    #[test]
    fn mix_seed_depends_on_every_part_and_their_order() {
        assert_ne!(mix_seed(&[1, 2, 3]), mix_seed(&[1, 2, 4]));
        assert_ne!(mix_seed(&[1, 2, 3]), mix_seed(&[3, 2, 1]));
        assert_eq!(mix_seed(&[1, 2, 3]), mix_seed(&[1, 2, 3]));
    }

    #[test]
    fn rivals_have_a_growth_peak_and_a_distance_range() {
        for race in races(Grade::G1).into_iter().take(10) {
            for e in generate(&race, 12, &HashSet::new(), 7) {
                assert!((2..=7).contains(&e.peak_age), "{}", e.peak_age);
                assert!(e.apt_min >= 1000 && e.apt_max <= 3600 && e.apt_min <= e.apt_max);
                assert_eq!((e.apt_min % 200, e.apt_max % 200), (0, 0));
            }
        }
    }

    #[test]
    fn most_rivals_suit_the_race_but_some_do_not() {
        let (mut fit, mut off, mut prime, mut other) = (0, 0, 0, 0);
        for race in races(Grade::G1).into_iter().chain(races(Grade::G3)) {
            for seed in 0..10u64 {
                for e in generate(&race, 12, &HashSet::new(), seed) {
                    if (e.apt_min..=e.apt_max).contains(&race.distance) {
                        fit += 1
                    } else {
                        off += 1
                    }
                    if (e.age as i32 - e.peak_age as i32).abs() <= 1 {
                        prime += 1
                    } else {
                        other += 1
                    }
                }
            }
        }
        assert!(fit as f64 > 0.75 * (fit + off) as f64, "{fit} fit vs {off}");
        assert!(off > 0, "some rivals should be off their distance");
        assert!(
            prime as f64 > 0.7 * (prime + other) as f64,
            "{prime} prime vs {other}"
        );
        assert!(
            other > 0,
            "some rivals should be young or old for their type"
        );
    }

    #[test]
    fn rival_form_is_the_same_for_the_same_seed() {
        let race = &races(Grade::G1)[0];
        let a = generate(race, 12, &HashSet::new(), 3);
        let b = generate(race, 12, &HashSet::new(), 3);
        assert_eq!(a, b);
    }

    #[test]
    fn rivals_have_every_running_style_and_a_range_of_temperaments() {
        let race = &races(Grade::G1)[0];
        let all: Vec<Entrant> = (0..20u64)
            .flat_map(|s| generate(race, 12, &HashSet::new(), s))
            .collect();
        for style in [Style::Nige, Style::Senko, Style::Sashi, Style::Oikomi] {
            assert!(all.iter().any(|e| e.style == Some(style)), "{style:?}");
        }
        let nige = all.iter().filter(|e| e.style == Some(Style::Nige)).count();
        assert!(
            nige * 4 < all.len(),
            "front-runners should be the fewest: {nige}/{}",
            all.len()
        );
        assert!(all.iter().all(|e| (10..=70).contains(&e.temper)));
        assert!(all.iter().any(|e| e.temper < 30) && all.iter().any(|e| e.temper > 55));
    }

    #[test]
    fn rivals_come_in_every_coat_colour_bays_most_of_all() {
        let race = &races(Grade::G1)[0];
        let all: Vec<Entrant> = (0..20u64)
            .flat_map(|s| generate(race, 12, &HashSet::new(), s))
            .collect();
        let count = |c: &str| all.iter().filter(|e| e.coat_color == c).count();
        for c in ["鹿毛", "栗毛", "青毛", "芦毛"] {
            assert!(count(c) > 0, "{c}");
        }
        assert!(count("鹿毛") > count("栗毛") && count("栗毛") > count("芦毛"));
        assert_eq!(
            all.len(),
            ["鹿毛", "栗毛", "青毛", "芦毛"]
                .iter()
                .map(|c| count(c))
                .sum::<usize>()
        );
    }

    #[test]
    fn some_rivals_wear_a_menko_mostly_the_hot_tempered_ones_and_some_a_shadow_roll() {
        let race = &races(Grade::G1)[0];
        let all: Vec<Entrant> = (0..30u64)
            .flat_map(|s| generate(race, 12, &HashSet::new(), s))
            .collect();
        let hooded: Vec<&Entrant> = all.iter().filter(|e| e.gear.menko).collect();
        assert!(!hooded.is_empty() && hooded.iter().all(|e| e.temper < 40));
        let rolled = all.iter().filter(|e| e.gear.shadow_roll).count();
        assert!(
            rolled * 10 > all.len() && rolled * 3 < all.len(),
            "{rolled}/{}",
            all.len()
        );
    }

    #[test]
    fn the_runners_beyond_the_core_of_a_big_field_are_there_to_fill_it_out() {
        let race = crate::calendar::shared()
            .races
            .iter()
            .find(|r| r.grade == Grade::Win1)
            .unwrap();
        let (mut core, mut filler) = (0i64, 0i64);
        for seed in 0..200 {
            let rivals = generate(race, FIELD_CORE + 6, &HashSet::new(), seed);
            core += rivals[..FIELD_CORE]
                .iter()
                .map(|e| e.speed as i64)
                .sum::<i64>();
            filler += rivals[FIELD_CORE..]
                .iter()
                .map(|e| e.speed as i64)
                .sum::<i64>();
        }
        let (core, filler) = (core / (200 * FIELD_CORE as i64), filler / (200 * 6));
        assert!((core - filler - FILLER_GAP).abs() <= 2, "{core} {filler}");
    }
}
