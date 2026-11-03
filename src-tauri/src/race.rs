use crate::horse::Horse;
use serde::{Deserialize, Serialize};

/// Seconds simulated per tick.
const TICK_SECONDS: f64 = 1.0;
/// Each runner's per-race pace varies by up to +/- this fraction.
const PACE_SPREAD: f64 = 0.07;
/// Safety cap so a race can never loop forever.
const MAX_TICKS: u32 = 10_000;
/// Metres per second of a horse of average ability at an even pace, before the race's own pace.
const AVERAGE_PACE: f64 = 19.0;
/// How much of what sets the horses apart from the average shows on the track: ability
/// and the luck of the day decide the order, but real fields finish seconds apart.
const GAP_SHARE: f64 = 0.25;
/// The pace of an average horse over a distance: `PACE_FLOOR` + `PACE_SPRINT` / metres,
/// which makes 55s for 1000m, 92s for 1600m and 143s for 2400m.
const PACE_FLOOR: f64 = 15.8;
const PACE_SPRINT: f64 = 2400.0;

/// Share of the race run before the final stretch, where styles change gear.
const EARLY_SHARE: f64 = 0.6;
/// Late-stretch pace lost by each front-runner for every other one in the field.
const DUEL_LOSS: f64 = 0.015;
const MAX_DUEL_LOSS: f64 = 0.045;
/// Temperament (0-100) of a horse with nothing known about it.
pub const NEUTRAL_TEMPER: f64 = 50.0;
/// Chance a horse of no temperament at all pulls for its head (掛かる).
const MAX_KAKARI_CHANCE: f64 = 0.30;
/// A horse that pulls goes this much faster early and pays for it later.
const KAKARI_EARLY: f64 = 1.03;
const KAKARI_LATE: f64 = 0.95;
/// Where in the race a horse that pulls settles down.
const KAKARI_SHARE: f64 = 0.5;

/// Running style (脚質): where in the field a horse races.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Style {
    /// 逃げ: goes to the front.
    Nige,
    /// 先行: races just behind the leaders.
    Senko,
    /// 差し: mid-field, finishing strongly.
    Sashi,
    /// 追込: at the back, with one late run.
    Oikomi,
}

impl Style {
    pub fn from_label(s: &str) -> Option<Self> {
        match s {
            "逃げ" => Some(Self::Nige),
            "先行" => Some(Self::Senko),
            "差し" => Some(Self::Sashi),
            "追込" => Some(Self::Oikomi),
            _ => None,
        }
    }

    /// Pace before the final stretch. The late pace is set so that over a whole
    /// race every style averages the same pace: no style is better in itself.
    fn early(self) -> f64 {
        match self {
            Self::Nige => 1.04,
            Self::Senko => 1.02,
            Self::Sashi => 0.98,
            Self::Oikomi => 0.96,
        }
    }

    fn late(self) -> f64 {
        // paces on the track differ too little for the harmonic mean to matter
        1.0 - (self.early() - 1.0) * EARLY_SHARE / (1.0 - EARLY_SHARE)
    }
}

/// Racing equipment (馬具).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Gear {
    /// メンコ (ear hood): shuts out the noise, so a hot horse pulls less often.
    pub menko: bool,
    /// シャドーロール: keeps the horse's eyes on the track, so it runs closer to its form.
    pub shadow_roll: bool,
}

/// メンコ halves the chance of pulling for its head.
const MENKO_KAKARI: f64 = 0.5;
/// シャドーロール narrows how much a horse's form varies.
const SHADOW_ROLL_SPREAD: f64 = 0.75;

/// What a horse brings to a race besides its speed and stamina.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Traits {
    pub style: Option<Style>,
    /// Temperament (気性), 0-100: a calm horse runs to form, a hot one is erratic.
    pub temper: f64,
    pub gear: Gear,
}

impl Default for Traits {
    fn default() -> Self {
        Self {
            style: None,
            temper: NEUTRAL_TEMPER,
            gear: Gear::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Runner {
    pub name: String,
    pub speed: u32,
    pub stamina: u32,
    /// Metres covered so far.
    pub position: f64,
    /// Pace multiplier drawn once per race: the unpredictable part of a horse's day.
    pub pace: f64,
    /// Seconds at which the runner crossed the line, if it has.
    pub finish_time: Option<f64>,
    pub style: Option<Style>,
    pub temper: f64,
    /// Pulled for its head early on (掛かった).
    pub kakari: bool,
    pub gear: Gear,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RaceResult {
    pub rank: u32,
    pub name: String,
    pub finish_time: f64,
}

#[derive(Debug, Clone)]
pub struct Race {
    pub runners: Vec<Runner>,
    pub distance: f64,
    pub elapsed: f64,
    rng: u64,
    ticks: u32,
    /// Runners whose style is 逃げ.
    front_runners: usize,
}

impl Race {
    #[cfg(test)]
    pub fn new(horses: Vec<Horse>, distance: f64, seed: u64) -> Self {
        let traits = vec![Traits::default(); horses.len()];
        Self::with_traits(horses, &traits, distance, seed)
    }

    /// A race whose runners have running styles and temperaments (in runner order).
    pub fn with_traits(horses: Vec<Horse>, traits: &[Traits], distance: f64, seed: u64) -> Self {
        let runners = horses
            .into_iter()
            .zip(traits)
            .map(|(h, t)| Runner {
                name: h.name,
                speed: h.speed,
                stamina: h.stamina,
                position: 0.0,
                pace: 1.0,
                finish_time: None,
                style: t.style,
                temper: t.temper,
                kakari: false,
                gear: t.gear,
            })
            .collect::<Vec<Runner>>();
        let front_runners = runners
            .iter()
            .filter(|r| r.style == Some(Style::Nige))
            .count();
        let mut race = Self {
            front_runners,
            runners,
            distance,
            elapsed: 0.0,
            // xorshift must not start at zero.
            rng: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1,
            ticks: 0,
        };
        for i in 0..race.runners.len() {
            let temper = race.runners[i].temper;
            let gear = race.runners[i].gear;
            let spread = PACE_SPREAD * Self::spread_factor(temper) * Self::gear_spread(gear);
            race.runners[i].pace = 1.0 - spread + race.next_f64() * 2.0 * spread;
            // only a hot-tempered horse can pull, so a neutral field draws nothing more
            let chance = Self::kakari_chance(temper) * Self::gear_kakari(gear);
            if chance > 0.0 {
                race.runners[i].kakari = race.next_f64() < chance;
            }
        }
        race
    }

    /// How much a horse's form varies from race to race: calm 0.6x, hot 1.4x.
    pub fn spread_factor(temper: f64) -> f64 {
        1.4 - 0.8 * temper.clamp(0.0, 100.0) / 100.0
    }

    /// How equipment narrows a horse's form (1.0 without a シャドーロール).
    pub fn gear_spread(gear: Gear) -> f64 {
        if gear.shadow_roll {
            SHADOW_ROLL_SPREAD
        } else {
            1.0
        }
    }

    /// How equipment changes the chance of pulling (1.0 without a メンコ).
    pub fn gear_kakari(gear: Gear) -> f64 {
        if gear.menko {
            MENKO_KAKARI
        } else {
            1.0
        }
    }

    /// Chance a horse pulls for its head: none from a neutral temperament up.
    pub fn kakari_chance(temper: f64) -> f64 {
        ((NEUTRAL_TEMPER - temper) / NEUTRAL_TEMPER).clamp(0.0, 1.0) * MAX_KAKARI_CHANCE
    }

    /// Pace from the runner's style and temperament at this point in the race.
    fn tactics(r: &Runner, distance: f64, front_runners: usize) -> f64 {
        let share = r.position / distance;
        let mut m = match r.style {
            Some(style) if share < EARLY_SHARE => style.early(),
            Some(style) => {
                let mut late = style.late();
                if style == Style::Nige {
                    // front-runners take each other on and tire themselves out
                    late -= (DUEL_LOSS * front_runners.saturating_sub(1) as f64).min(MAX_DUEL_LOSS);
                }
                late
            }
            None => 1.0,
        };
        if r.kakari {
            m *= if share < KAKARI_SHARE {
                KAKARI_EARLY
            } else {
                KAKARI_LATE
            };
        }
        m
    }

    /// Uniform f64 in [0, 1).
    fn next_f64(&mut self) -> f64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn is_finished(&self) -> bool {
        self.runners.iter().all(|r| r.finish_time.is_some())
    }

    pub fn tick(&mut self) {
        self.ticks += 1;
        let start = self.elapsed;
        self.elapsed += TICK_SECONDS;
        let front_runners = self.front_runners;
        let distance = self.distance;
        for i in 0..self.runners.len() {
            if self.runners[i].finish_time.is_some() {
                continue;
            }
            let noise = 0.95 + self.next_f64() * 0.10;
            let r = &mut self.runners[i];
            let velocity = Self::on_the_track(
                Self::velocity(r.speed, r.stamina, r.position)
                    * r.pace
                    * noise
                    * Self::tactics(r, distance, front_runners),
                distance,
            );
            let step = velocity * TICK_SECONDS;
            let new_pos = r.position + step;
            if new_pos >= self.distance {
                let frac = (self.distance - r.position) / step;
                r.finish_time = Some(start + frac * TICK_SECONDS);
                r.position = self.distance;
            } else {
                r.position = new_pos;
            }
        }
    }

    /// Metres per second: base pace from speed, reduced once the runner
    /// passes the distance its stamina can sustain.
    fn velocity(speed: u32, stamina: u32, position: f64) -> f64 {
        let base = 12.0 + speed as f64 * 0.10;
        let limit = stamina as f64 * 30.0;
        let fatigue = (1.0 - 0.0004 * (position - limit).max(0.0)).max(0.6);
        base * fatigue
    }

    /// The metres per second actually run, given what a horse's ability and form make it.
    fn on_the_track(velocity: f64, distance: f64) -> f64 {
        PACE_FLOOR + PACE_SPRINT / distance + (velocity - AVERAGE_PACE) * GAP_SHARE
    }

    pub fn run_to_finish(&mut self) {
        while !self.is_finished() && self.ticks < MAX_TICKS {
            self.tick();
        }
    }

    /// Runs to the finish, returning every runner's position after each tick
    /// (in runner order) so the UI can replay the race.
    pub fn run_recording(&mut self) -> Vec<Vec<f64>> {
        let mut frames = vec![];
        while !self.is_finished() && self.ticks < MAX_TICKS {
            self.tick();
            frames.push(self.runners.iter().map(|r| r.position).collect());
        }
        frames
    }

    /// Finished runners ranked by finish time (fastest first).
    pub fn results(&self) -> Vec<RaceResult> {
        let mut finished: Vec<(&Runner, f64)> = self
            .runners
            .iter()
            .filter_map(|r| r.finish_time.map(|t| (r, t)))
            .collect();
        finished.sort_by(|a, b| a.1.total_cmp(&b.1));
        finished
            .into_iter()
            .enumerate()
            .map(|(i, (r, t))| RaceResult {
                rank: i as u32 + 1,
                name: r.name.clone(),
                finish_time: t,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn horses() -> Vec<Horse> {
        vec![
            Horse::new("Fast", 90, 80),
            Horse::new("Slow", 40, 80),
            Horse::new("Tired", 90, 10),
        ]
    }

    #[test]
    fn new_race_starts_at_zero_and_not_finished() {
        let race = Race::new(horses(), 2000.0, 1);
        assert_eq!(race.runners.len(), 3);
        assert!(race.runners.iter().all(|r| r.position == 0.0));
        assert!(!race.is_finished());
    }

    #[test]
    fn tick_moves_runners_forward() {
        let mut race = Race::new(horses(), 2000.0, 1);
        race.tick();
        assert!(race.runners.iter().all(|r| r.position > 0.0));
    }

    #[test]
    fn recording_captures_positions_every_tick() {
        let mut race = Race::new(horses(), 1200.0, 3);
        let frames = race.run_recording();
        assert!(frames.len() > 1);
        assert!(frames.iter().all(|f| f.len() == 3));
        // monotonic non-decreasing per runner, ending at the finish line
        for i in 0..3 {
            assert!(frames.windows(2).all(|w| w[1][i] >= w[0][i]));
            assert_eq!(frames.last().unwrap()[i], 1200.0);
        }
    }

    #[test]
    fn equal_horses_each_win_a_fair_share() {
        let mut wins = [0u32; 2];
        for seed in 0..200u64 {
            let mut race = Race::new(
                vec![Horse::new("A", 70, 70), Horse::new("B", 70, 70)],
                2400.0,
                seed,
            );
            race.run_to_finish();
            let first = race.results()[0].name.clone();
            wins[(first == "B") as usize] += 1;
        }
        assert!(wins[0] >= 60 && wins[1] >= 60, "uneven: {wins:?}");
    }

    #[test]
    fn a_slightly_better_horse_does_not_always_win() {
        let upsets = (0..200u64)
            .filter(|seed| {
                let mut race = Race::new(
                    vec![Horse::new("Better", 80, 70), Horse::new("Worse", 74, 70)],
                    2400.0,
                    *seed,
                );
                race.run_to_finish();
                race.results()[0].name == "Worse"
            })
            .count();
        assert!(upsets >= 20, "upsets only {upsets}/200");
        assert!(
            upsets <= 100,
            "better horse should still be favoured, upsets {upsets}/200"
        );
    }

    #[test]
    fn same_seed_is_deterministic() {
        let mut a = Race::new(horses(), 1200.0, 42);
        let mut b = Race::new(horses(), 1200.0, 42);
        a.run_to_finish();
        b.run_to_finish();
        assert_eq!(a.results(), b.results());
    }

    #[test]
    fn race_finishes_and_ranks_all_horses() {
        let mut race = Race::new(horses(), 1200.0, 7);
        race.run_to_finish();
        assert!(race.is_finished());
        let results = race.results();
        assert_eq!(results.len(), 3);
        let mut ranks: Vec<u32> = results.iter().map(|r| r.rank).collect();
        ranks.sort();
        assert_eq!(ranks, vec![1, 2, 3]);
    }

    #[test]
    fn faster_horse_usually_beats_slower_horse() {
        let wins = (0..50)
            .filter(|seed| {
                let mut race = Race::new(
                    vec![Horse::new("Fast", 90, 80), Horse::new("Slow", 40, 80)],
                    1200.0,
                    *seed,
                );
                race.run_to_finish();
                race.results()[0].name == "Fast"
            })
            .count();
        assert!(wins >= 40, "fast horse won only {wins}/50");
    }

    #[test]
    fn low_stamina_hurts_over_long_distance() {
        let wins = (0..50)
            .filter(|seed| {
                let mut race = Race::new(
                    vec![Horse::new("Stayer", 70, 90), Horse::new("Tired", 70, 10)],
                    3200.0,
                    *seed,
                );
                race.run_to_finish();
                race.results()[0].name == "Stayer"
            })
            .count();
        assert!(wins >= 40, "stayer won only {wins}/50");
    }

    // ---- running style (脚質) and temperament (気性) ----

    fn styled(style: Option<Style>, temper: f64) -> Traits {
        Traits {
            style,
            temper,
            gear: Gear::default(),
        }
    }

    /// Runs equal horses with these traits, returning each race.
    fn run_equal(traits: &[Traits], distance: f64, seeds: u64) -> Vec<Race> {
        (0..seeds)
            .map(|seed| {
                let horses = (0..traits.len())
                    .map(|i| Horse::new(&format!("H{i}"), 75, 75))
                    .collect();
                let mut race = Race::with_traits(horses, traits, distance, seed);
                race.run_to_finish();
                race
            })
            .collect()
    }

    fn wins(races: &[Race], name: &str) -> usize {
        races.iter().filter(|r| r.results()[0].name == name).count()
    }

    #[test]
    fn every_style_takes_the_same_time_over_a_whole_race() {
        for style in [Style::Nige, Style::Senko, Style::Sashi, Style::Oikomi] {
            let pace = EARLY_SHARE * style.early() + (1.0 - EARLY_SHARE) * style.late();
            assert!((pace - 1.0).abs() < 1e-9, "{style:?}");
        }
        assert!(Style::Nige.late() < 1.0 && Style::Oikomi.late() > 1.0);
    }

    #[test]
    fn styles_read_from_their_japanese_names() {
        assert_eq!(Style::from_label("逃げ"), Some(Style::Nige));
        assert_eq!(Style::from_label("先行"), Some(Style::Senko));
        assert_eq!(Style::from_label("差し"), Some(Style::Sashi));
        assert_eq!(Style::from_label("追込"), Some(Style::Oikomi));
        assert_eq!(Style::from_label("自在"), None);
    }

    #[test]
    fn a_front_runner_leads_at_halfway_and_a_closer_runs_on_late() {
        let traits = [
            styled(Some(Style::Nige), 50.0),
            styled(Some(Style::Oikomi), 50.0),
        ];
        let (mut led, mut gained) = (0, 0);
        for seed in 0..100u64 {
            let horses = vec![Horse::new("逃", 75, 75), Horse::new("追", 75, 75)];
            let mut race = Race::with_traits(horses, &traits, 2000.0, seed);
            let frames = race.run_recording();
            let half = frames
                .iter()
                .position(|f| f[0].max(f[1]) >= 1000.0)
                .unwrap();
            let gap_half = frames[half][0] - frames[half][1];
            led += (gap_half > 0.0) as usize;
            let end = frames
                .iter()
                .position(|f| f[0] >= 2000.0 || f[1] >= 2000.0)
                .unwrap();
            let gap_end = frames[end][0] - frames[end][1];
            gained += (gap_end < gap_half) as usize;
        }
        assert!(led >= 80, "front-runner led at halfway {led}/100");
        assert!(gained >= 80, "closer gained in the stretch {gained}/100");
    }

    #[test]
    fn no_style_wins_more_than_its_share_of_races() {
        let races = run_equal(
            &[
                styled(Some(Style::Nige), 50.0),
                styled(Some(Style::Oikomi), 50.0),
            ],
            2400.0,
            300,
        );
        let nige = wins(&races, "H0");
        assert!((90..=210).contains(&nige), "front-runner won {nige}/300");
    }

    #[test]
    fn too_many_front_runners_set_it_up_for_the_closers() {
        let traits = [
            styled(Some(Style::Nige), 50.0),
            styled(Some(Style::Nige), 50.0),
            styled(Some(Style::Nige), 50.0),
            styled(Some(Style::Sashi), 50.0),
        ];
        let sashi = wins(&run_equal(&traits, 2000.0, 300), "H3");
        assert!(
            sashi > 100,
            "closer won {sashi}/300 against three front-runners"
        );
    }

    #[test]
    fn a_lone_front_runner_has_no_one_to_take_it_on() {
        let alone = Race::with_traits(
            vec![Horse::new("A", 75, 75)],
            &[styled(Some(Style::Nige), 50.0)],
            2000.0,
            1,
        );
        let r = &alone.runners[0];
        let late = Race::tactics(
            &Runner {
                position: 1900.0,
                ..r.clone()
            },
            2000.0,
            1,
        );
        assert!((late - Style::Nige.late()).abs() < 1e-12);
        let duel = Race::tactics(
            &Runner {
                position: 1900.0,
                ..r.clone()
            },
            2000.0,
            3,
        );
        assert!((late - duel - 2.0 * DUEL_LOSS).abs() < 1e-12);
        let crowd = Race::tactics(
            &Runner {
                position: 1900.0,
                ..r.clone()
            },
            2000.0,
            9,
        );
        assert!((late - crowd - MAX_DUEL_LOSS).abs() < 1e-12);
    }

    /// The average winning time and gap from first to last of a field of 18 of mixed ability.
    fn times_over(distance: f64) -> (f64, f64) {
        let (mut win, mut gap) = (0.0, 0.0);
        let races = 100;
        for seed in 0..races {
            let horses: Vec<Horse> = (0..18)
                .map(|i| Horse::new(&format!("H{i}"), 60 + i * 25 / 17, 90))
                .collect();
            let mut race = Race::with_traits(horses, &[Traits::default(); 18], distance, seed + 1);
            race.run_to_finish();
            let results = race.results();
            win += results[0].finish_time;
            gap += results[17].finish_time - results[0].finish_time;
        }
        (win / races as f64, gap / races as f64)
    }

    #[test]
    fn winning_times_are_like_those_of_real_races() {
        for (distance, low, high) in [
            (1000.0, 52.0, 58.0),
            (1600.0, 88.0, 97.0),
            (2000.0, 112.0, 122.0),
            (2400.0, 136.0, 150.0),
        ] {
            let (win, _) = times_over(distance);
            assert!(win > low && win < high, "{distance}m in {win}");
        }
    }

    #[test]
    fn a_field_finishes_within_seconds_of_the_winner() {
        for (distance, most) in [(1000.0, 5.0), (2000.0, 9.0), (2400.0, 11.0)] {
            let (_, gap) = times_over(distance);
            assert!(gap > 1.0 && gap < most, "{distance}m: {gap}");
        }
    }

    #[test]
    fn a_calm_horse_runs_closer_to_its_form_than_a_hot_one() {
        let spread = |temper: f64| {
            let times: Vec<f64> = run_equal(&[styled(None, temper)], 2000.0, 200)
                .iter()
                .map(|r| r.results()[0].finish_time)
                .collect();
            let mean = times.iter().sum::<f64>() / times.len() as f64;
            (times.iter().map(|t| (t - mean).powi(2)).sum::<f64>() / times.len() as f64).sqrt()
        };
        assert!(
            spread(90.0) < spread(10.0) * 0.7,
            "{} vs {}",
            spread(90.0),
            spread(10.0)
        );
        assert!((Race::spread_factor(50.0) - 1.0).abs() < 1e-12);
        assert!((Race::spread_factor(100.0) - 0.6).abs() < 1e-12);
        assert!((Race::spread_factor(0.0) - 1.4).abs() < 1e-12);
    }

    #[test]
    fn only_a_hot_tempered_horse_pulls_for_its_head() {
        assert_eq!(Race::kakari_chance(50.0), 0.0);
        assert_eq!(Race::kakari_chance(80.0), 0.0);
        assert!((Race::kakari_chance(25.0) - 0.15).abs() < 1e-12);
        assert!((Race::kakari_chance(0.0) - 0.30).abs() < 1e-12);
        let pulled = run_equal(&[styled(None, 0.0)], 2000.0, 300)
            .iter()
            .filter(|r| r.runners[0].kakari)
            .count();
        assert!((60..=120).contains(&pulled), "pulled {pulled}/300");
        let calm = run_equal(&[styled(None, 60.0)], 2000.0, 100);
        assert!(calm.iter().all(|r| !r.runners[0].kakari));
    }

    #[test]
    fn pulling_for_its_head_costs_a_horse_time() {
        let base = Race::with_traits(
            vec![Horse::new("A", 75, 75)],
            &[Traits::default()],
            2000.0,
            1,
        );
        let r = base.runners[0].clone();
        let pulled = Runner {
            kakari: true,
            ..r.clone()
        };
        let early = Race::tactics(
            &Runner {
                position: 100.0,
                ..pulled.clone()
            },
            2000.0,
            0,
        );
        let late = Race::tactics(
            &Runner {
                position: 1500.0,
                ..pulled
            },
            2000.0,
            0,
        );
        assert_eq!((early, late), (KAKARI_EARLY, KAKARI_LATE));
        const { assert!(KAKARI_SHARE / KAKARI_EARLY + (1.0 - KAKARI_SHARE) / KAKARI_LATE > 1.0) };
    }

    #[test]
    fn a_neutral_field_runs_exactly_as_before() {
        // Race::new and default traits draw the same numbers in the same order
        let mut a = Race::new(horses(), 2000.0, 9);
        let mut b = Race::with_traits(horses(), &[Traits::default(); 3], 2000.0, 9);
        a.run_to_finish();
        b.run_to_finish();
        assert_eq!(a.results(), b.results());
        assert!(a
            .runners
            .iter()
            .all(|r| r.pace >= 1.0 - PACE_SPREAD && r.pace <= 1.0 + PACE_SPREAD));
    }

    // ---- equipment (馬具) ----

    fn geared(temper: f64, menko: bool, shadow_roll: bool) -> Traits {
        Traits {
            style: None,
            temper,
            gear: Gear { menko, shadow_roll },
        }
    }

    #[test]
    fn a_menko_halves_how_often_a_hot_horse_pulls() {
        let pulled = |menko: bool| {
            run_equal(&[geared(0.0, menko, false)], 2000.0, 600)
                .iter()
                .filter(|r| r.runners[0].kakari)
                .count()
        };
        let (bare, hooded) = (pulled(false), pulled(true));
        assert!((140..=220).contains(&bare), "bare {bare}/600");
        assert!((60..=120).contains(&hooded), "with a menko {hooded}/600");
        assert_eq!(
            Race::gear_kakari(Gear {
                menko: true,
                shadow_roll: false
            }),
            MENKO_KAKARI
        );
    }

    #[test]
    fn a_shadow_roll_makes_a_horse_run_closer_to_its_form() {
        let spread = |shadow: bool| {
            let times: Vec<f64> = run_equal(&[geared(50.0, false, shadow)], 2000.0, 300)
                .iter()
                .map(|r| r.results()[0].finish_time)
                .collect();
            let mean = times.iter().sum::<f64>() / times.len() as f64;
            (times.iter().map(|t| (t - mean).powi(2)).sum::<f64>() / times.len() as f64).sqrt()
        };
        assert!(
            spread(true) < spread(false) * 0.9,
            "{} vs {}",
            spread(true),
            spread(false)
        );
        assert_eq!(
            Race::gear_spread(Gear {
                menko: false,
                shadow_roll: true
            }),
            SHADOW_ROLL_SPREAD
        );
    }

    #[test]
    fn a_menko_does_nothing_for_a_calm_horse() {
        // a calm horse never pulls, hood or not, and the hood draws no extra numbers
        let a = run_equal(&[geared(70.0, true, false)], 2000.0, 50);
        let b = run_equal(&[geared(70.0, false, false)], 2000.0, 50);
        for (x, y) in a.iter().zip(&b) {
            assert!(!x.runners[0].kakari);
            assert_eq!(x.results(), y.results());
        }
    }

    #[test]
    fn gear_serialises_with_snake_case_names() {
        let v = serde_json::to_value(Gear {
            menko: true,
            shadow_roll: false,
        })
        .unwrap();
        assert_eq!(v, serde_json::json!({"menko": true, "shadow_roll": false}));
    }
}
