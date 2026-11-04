//! The game: a stable of horses, week by week.

use crate::bets;
use crate::bloodline::{self, BloodHorse};
use crate::calendar::{
    self, AgeRule, CalendarRace, Day, Eligibility, Grade, Surface, Venue, WeightRule,
};
use crate::commentary::{self, CommentLine};
use crate::course::{self, CourseFit};
use crate::entries;
use crate::form;
use crate::horse::{Horse, Pedigree};
use crate::jockey::{self, JockeyInfo, Ride, JOCKEYS};
use crate::race::{self, Gear, Race, RaceResult, Style, Traits};
use crate::rivals::{self, Entrant, Star};
use crate::track::{self, TrackCondition, Weather};
use crate::trainer::{self, TrainerInfo, STABLE_CAPACITY, TRAINERS};
use crate::weight;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

mod abroad;
mod betting;
mod bloodlines;
mod breeding;
pub mod commands;
mod consts;
mod honours;
mod leading;
mod racing;
mod save;
mod stable;
mod staff;
mod state;
#[cfg(test)]
mod tests;
mod training;
mod types;
mod week;

pub use consts::*;
use save::SAVE_VERSION;
pub use types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    /// Which format a save is in: see `save.rs`.
    #[serde(default)]
    save_version: u32,
    pub money: u32,
    pub round: u32,
    /// The racing stable.
    pub field: Vec<StableHorse>,
    #[serde(default)]
    history: Vec<HistoryEntry>,
    #[serde(default)]
    rng: u64,
    /// Records the player added to the database: their own foals (and older horses that had none).
    #[serde(default)]
    bred: Vec<BloodHorse>,
    /// The names the stable's founding horses carry in the database.
    #[serde(default)]
    blood_names: std::collections::HashMap<u32, String>,
    /// Week of the year, 1..=52.
    #[serde(default = "default_week")]
    pub week: u8,
    #[serde(default = "default_year")]
    pub year: u32,
    /// Seeds the rivals of calendar races.
    #[serde(default)]
    world_seed: u64,
    /// Which stable horses are entered in each race of this week (kept only until the week ends).
    #[serde(default)]
    entries: std::collections::HashMap<String, Vec<String>>,
    /// Retired horses kept for breeding, longest-serving first.
    #[serde(default)]
    studs: Vec<Stud>,
    /// The foals on the way.
    #[serde(default)]
    pregnancies: Vec<Pregnancy>,
    /// The farm's facilities.
    #[serde(default)]
    farm: Farm,
    /// Horses sent abroad and waiting for their race.
    #[serde(default)]
    expeditions: Vec<Expedition>,
    /// What each sire the player has used has got, by its blood record.
    #[serde(default)]
    sire_records: std::collections::HashMap<u32, SireRecord>,
    /// Whether a horse can be kept out of a race by the entry rules. Off for tests about
    /// other things, which should not find a horse suddenly drawn out.
    #[serde(default = "entry_rules_on")]
    entry_rules: bool,
    /// The horses whose last race earned them priority in the draw.
    #[serde(default)]
    priority: Vec<String>,
    /// The rights to run in a G1 earned in its trial.
    #[serde(default)]
    trial_rights: Vec<TrialRight>,
    /// Orders to the jockey (作戦), by horse.
    #[serde(default)]
    orders: std::collections::HashMap<String, Style>,
    /// The goals reached and announced so far.
    #[serde(default)]
    goals_done: Vec<String>,
    /// The lots already sold or bid on, by year.
    #[serde(default)]
    auction_closed: Vec<(u32, u32)>,
    /// What happened at the turn of this week.
    #[serde(default)]
    news: Vec<String>,
    /// The rivals who race on year after year (ライバル).
    #[serde(default)]
    stars: Vec<Star>,
    #[serde(default)]
    awards: Vec<Award>,
    /// The stable's hall of fame (殿堂).
    #[serde(default)]
    hall: Vec<HallEntry>,
    /// How each trainer has done with the stable's horses, by trainer id.
    #[serde(default)]
    trainer_records: Vec<TrainerRecord>,
    /// Everyone who has ridden in the game's time, retired or not.
    #[serde(default)]
    jockey_staff: Vec<Staff>,
    /// Everyone who has had a stable in the game's time, retired or not.
    #[serde(default)]
    trainer_staff: Vec<Staff>,
    /// The stable's own wins this year.
    #[serde(default)]
    year_wins: YearWins,
    /// Who led the years gone by.
    #[serde(default)]
    leading_years: Vec<LeadingYear>,
}

pub struct GameStore(pub Mutex<Game>);

impl Game {
    #[cfg(test)]
    pub fn new() -> Self {
        Self::with_seed(DEFAULT_SEED)
    }

    pub fn with_seed(seed: u64) -> Self {
        let starters = [
            ("アサギリライズ", 82, 75),
            ("ギンセイストーム", 88, 58),
            ("グレンキング", 78, 85),
            ("オウゴンオーシャン", 76, 90),
            ("ヘキクウブレイブ", 74, 86),
            ("サクラフブキ", 68, 72),
            ("ホクトセイバー", 80, 72),
            ("ミナモムーン", 72, 80),
        ];
        let trainers = starting_trainers(seed, starters.len());
        let field: Vec<StableHorse> = starters
            .iter()
            .zip(trainers)
            .map(|(&(name, speed, stamina), trainer)| {
                StableHorse::starter(name, speed, stamina, trainer)
            })
            .collect();
        debug_assert!(field.iter().all(|h| crate::horse::is_valid_name(&h.name)));
        let mut game = Self {
            save_version: SAVE_VERSION,
            money: START_MONEY,
            round: 0,
            history: vec![],
            rng: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1,
            bred: vec![],
            blood_names: std::collections::HashMap::new(),
            week: 1,
            year: 1,
            world_seed: seed.wrapping_mul(0xA24B_AED4_963E_E407) | 1,
            entries: std::collections::HashMap::new(),
            studs: vec![],
            pregnancies: vec![],
            farm: Farm::default(),
            expeditions: vec![],
            sire_records: std::collections::HashMap::new(),
            entry_rules: cfg!(not(test)),
            priority: vec![],
            trial_rights: vec![],
            orders: std::collections::HashMap::new(),
            goals_done: vec![],
            auction_closed: vec![],
            news: vec![],
            stars: vec![],
            awards: vec![],
            hall: vec![],
            trainer_records: vec![TrainerRecord::default(); trainer::ALL_TRAINERS],
            jockey_staff: vec![],
            trainer_staff: vec![],
            year_wins: YearWins::default(),
            leading_years: vec![],
            field,
        };
        game.first_staff();
        game.link_blood();
        game.set_tempers_from_records();
        game.set_legs_from_blood();
        game.studs = game.starting_studs();
        game.stars = game.first_stars();
        game.roll_conditions();
        game
    }

    /// Starts over, carrying the random stream forward so each new game rolls fresh conditions.
    pub fn reset(&mut self) {
        let seed = self.next_u64();
        *self = Self::with_seed(seed);
    }

    fn next_u64(&mut self) -> u64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        self.rng
    }

    fn roll_condition(&mut self) -> Condition {
        Condition::from_roll((self.next_u64() >> 11) as f64 / (1u64 << 53) as f64)
    }

    fn roll_conditions(&mut self) {
        for i in 0..self.field.len() {
            self.field[i].condition = self.roll_condition();
        }
    }

    /// Speed after applying the horse's current condition.
    #[cfg(test)]
    pub fn effective_speed(&self, i: usize) -> u32 {
        (self.field[i].speed as f64 * self.field[i].condition.multiplier()).round() as u32
    }

    pub fn is_game_over(&self) -> bool {
        self.money == 0
    }
}
