//! Making up a race: who is entered, who gets a place, and how they run.

use super::*;

impl Game {
    /// A runner's speed for a race: base speed adjusted for condition and,
    /// on dirt, for dirt aptitude.
    pub fn run_speed(e: &Entrant, surface: Surface) -> u32 {
        let mut v = e.speed as f64 * e.condition.multiplier();
        if surface == Surface::Dirt {
            v *= 1.0 + DIRT_STEP * e.dirt as f64;
        }
        v.round() as u32
    }

    /// Speed and stamina a runner brings to a race, after its condition, dirt
    /// aptitude (on dirt) and how well it copes with the track.
    pub fn run_stats(
        e: &Entrant,
        surface: Surface,
        distance: u32,
        track: TrackCondition,
    ) -> (u32, u32) {
        let coached = e.trainer_distance.map_or(1.0, |d| d.bonus(distance));
        let at_course = e.course_fit.map_or(1.0, |f| f.multiplier());
        let ridden = e.jockey.as_ref().map_or(1.0, |j| 1.0 + j.percent / 100.0);
        let load = e.weight.map_or(1.0, weight::multiplier);
        let age = form::age_form(e.age, e.peak_age)
            * fatigue_form(e.fatigue)
            * coached
            * at_course
            * ridden
            * load
            * e.body_form;
        // over fences it is a different game, and flat form says little about the trip
        let dist = if surface == Surface::Jump {
            form::distance_effect(e.apt_min, e.apt_min, e.apt_max)
        } else {
            form::distance_effect(distance, e.apt_min, e.apt_max)
        };
        let speed = Self::run_speed(e, surface) as f64 * age * dist.speed;
        let (speed, stamina) = track::apply_track(
            speed,
            e.stamina as f64 * age * dist.stamina,
            e.power as f64,
            track::dirt_score(e.dirt),
            track,
        );
        (speed.round() as u32, stamina.round() as u32)
    }

    /// What a horse's character does for it in this race.
    fn character_form(e: &Entrant, spec: &RaceSpec) -> f64 {
        let Some(race) = &spec.race else {
            return 1.0;
        };
        let has = |s| e.specials.contains(&s);
        let mut form = 1.0;
        if has(Special::BigStage) && race.grade == Grade::G1 {
            form *= BIG_STAGE_FORM;
        }
        if has(Special::Fresh) && e.rested {
            form *= FRESH_FORM;
        }
        if has(Special::Mudlark) && spec.track != TrackCondition::Good {
            form *= MUDLARK_FORM;
        }
        form
    }

    pub(super) fn runners_for(spec: &RaceSpec) -> Vec<Horse> {
        spec.entrants
            .iter()
            .map(|e| {
                let (speed, stamina) = Self::run_stats(e, spec.surface, spec.distance, spec.track);
                let season = spec
                    .race
                    .as_ref()
                    .map_or(1.0, |r| form::season_form(&e.name, r.week));
                let scaled = |stat: u32| {
                    (stat as f64 * season * Self::character_form(e, spec)).round() as u32
                };
                Horse::new(&e.name, scaled(speed), scaled(stamina))
            })
            .collect()
    }

    /// Runs the race once, returning every runner's position per second and the finishing order.
    pub fn run_race(spec: &RaceSpec, seed: u64) -> (Vec<Vec<f64>>, Vec<RaceResult>) {
        let mut race = Self::start_race(spec, seed);
        let frames = race.run_recording();
        (frames, race.results())
    }

    /// A calendar race counts in the records of the stable's runners (the free race does not).
    pub(super) fn add_to_records(&mut self, spec: &RaceSpec, results: &[RaceResult]) {
        let winner = results.first().map(|r| r.name.as_str());
        let over_fences = spec.surface == Surface::Jump;
        for e in spec.entrants.iter().filter(|e| e.mine) {
            if let Some(i) = self.field.iter().position(|h| h.name == e.name) {
                if !over_fences {
                    self.field[i].record.starts += 1;
                }
                self.field[i].since_race = 0;
                if let Some(ride) = &e.jockey {
                    self.field[i].partner = Some(match self.field[i].partner {
                        Some((id, rides)) if id == ride.id => (id, rides + 1),
                        _ => (ride.id, 1),
                    });
                }
                if winner == Some(e.name.as_str()) {
                    if over_fences {
                        self.field[i].jump_wins += 1;
                        self.field[i].year_jump_wins += 1;
                    } else {
                        self.field[i].record.wins += 1;
                    }
                    if let Some(ride) = &e.jockey {
                        *self.year_wins.jockeys.entry(ride.id).or_default() += 1;
                    }
                    self.maybe_bring_out_big_stage(i, spec);
                    let trainer = self.field[i].trainer;
                    *self.year_wins.trainers.entry(trainer).or_default() += 1;
                    let sire = self
                        .blood_record(self.field[i].blood_id)
                        .and_then(|r| r.sire_id);
                    if let Some(sire) = sire {
                        *self.year_wins.sires.entry(sire).or_default() += 1;
                    }
                }
                if results.iter().all(|r| r.name != e.name) {
                    self.news
                        .push(format!("{}は競走中止となりました。", e.name));
                    self.roll_injury(i, STOP_INJURY, 0x57A1);
                }
            }
        }
    }

    /// A G1 win may show a horse to be one for the big occasion.
    fn maybe_bring_out_big_stage(&mut self, i: usize, spec: &RaceSpec) {
        let Some(race) = &spec.race else {
            return;
        };
        let roll = self.race_seed(race, 0xB165) % 100;
        if race.grade == Grade::G1
            && !self.field[i].specials.contains(&Special::BigStage)
            && roll < BIG_STAGE_PERCENT
        {
            self.field[i].specials.push(Special::BigStage);
            let name = self.field[i].name.clone();
            self.news.push(format!(
                "{name}が固有特性「{}」に目覚めました。",
                Special::BigStage.name()
            ));
        }
    }

    /// Runners that pull for their head (掛かる) in this race.
    pub(super) fn pulled(spec: &RaceSpec, seed: u64) -> Vec<String> {
        Self::start_race(spec, seed)
            .runners
            .into_iter()
            .filter(|r| r.kakari)
            .map(|r| r.name)
            .collect()
    }

    /// Adds a call for each runner that pulls, early in the race, keeping the lines in frame order.
    pub(super) fn call_pulling(lines: &mut Vec<CommentLine>, pulled: &[String], frames: usize) {
        if frames == 0 {
            return;
        }
        let frame = KAKARI_CALL_FRAME.min(frames - 1);
        for name in pulled {
            lines.push(CommentLine {
                frame,
                text: format!("{name}が掛かっている、行きたがっています!"),
            });
        }
        lines.sort_by_key(|l| l.frame); // stable: the start call stays first
    }

    /// Adds a call for each mistake at a fence, and for each horse that did not finish.
    pub(super) fn call_mishaps(
        lines: &mut Vec<CommentLine>,
        spec: &RaceSpec,
        seed: u64,
        frames: &[Vec<f64>],
        stopped: &[String],
    ) {
        if frames.is_empty() {
            return;
        }
        let mut race = Self::start_race(spec, seed);
        race.run_to_finish();
        for &(tick, runner) in &race.blunders {
            lines.push(CommentLine {
                frame: (tick as usize).min(frames.len() - 1),
                text: format!("{}が飛越でミス!", spec.entrants[runner].name),
            });
        }
        for name in stopped {
            let Some(at) = spec.entrants.iter().position(|e| &e.name == name) else {
                continue;
            };
            let last = frames[frames.len() - 1][at];
            let frame = frames.iter().position(|f| f[at] >= last).unwrap_or(0);
            lines.push(CommentLine {
                frame,
                text: format!("{name}が競走中止!"),
            });
        }
        lines.push(CommentLine {
            frame: frames.len() / 3,
            text: format!("{}で流れています。", race.pace.label()),
        });
        lines.sort_by_key(|l| l.frame);
    }

    /// The finishing order alone, without recording the race (for the odds).
    pub(super) fn finish_order(spec: &RaceSpec, seed: u64) -> Vec<RaceResult> {
        let mut race = Self::start_race(spec, seed);
        race.run_to_finish();
        race.results()
    }

    /// A horse's jumping with its jockey up.
    fn jump_with_jockey(e: &Entrant) -> u32 {
        let help = e.jockey.as_ref().map_or(0, |j| {
            (j.jump_skill as i64 - JUMP_JOCKEY_AVERAGE) / JUMP_JOCKEY_DIVISOR
        });
        (e.jump as i64 + help).clamp(0, 100) as u32
    }

    /// Those who started and did not finish (競走中止), in the order they started.
    pub(super) fn did_not_finish(spec: &RaceSpec, results: &[RaceResult]) -> Vec<String> {
        spec.entrants
            .iter()
            .filter(|e| results.iter().all(|r| r.name != e.name))
            .map(|e| e.name.clone())
            .collect()
    }

    pub(super) fn start_race(spec: &RaceSpec, seed: u64) -> Race {
        let traits: Vec<Traits> = spec
            .entrants
            .iter()
            .map(|e| Traits {
                style: e.style,
                temper: (e.temper + e.jockey.as_ref().map_or(0, |j| j.tact)).min(MAX_STAT) as f64,
                gear: e.gear,
                jump: (spec.surface == Surface::Jump).then(|| Self::jump_with_jockey(e)),
                kick: form::kick_of(&e.name, e.speed, e.stamina),
            })
            .collect();
        Race::with_traits(Self::runners_for(spec), &traits, spec.distance as f64, seed)
    }

    pub(super) fn stable_entrant(&self, i: usize) -> Entrant {
        let h = &self.field[i];
        // a horse without a family record has no growth or distance bias
        let (peak_age, apt_min, apt_max, style, coat_color) =
            match self.blood_record(self.field[i].blood_id) {
                Some(r) => (
                    r.peak_age,
                    r.aptitude_min,
                    r.aptitude_max,
                    Style::from_label(&r.running_style),
                    r.coat_color,
                ),
                None => (
                    self.field[i].age,
                    1000,
                    3600,
                    None,
                    DEFAULT_COAT.to_string(),
                ),
            };
        let order = self.orders.get(&h.name).copied();
        // a horse made to run against its natural way is harder to settle
        let against = order.is_some() && style.is_some() && order != style;
        Entrant {
            jump: self.field[i].jump,
            body_weight: self.field[i].body_weight,
            body_change: self.field[i].body_weight as i32 - self.field[i].last_race_weight as i32,
            body_form: body_form(
                self.field[i].body_weight,
                ideal_weight(self.field[i].blood_id),
            ),
            specials: self.field[i].specials.clone(),
            rested: self.field[i].since_race >= FRESH_WEEKS,
            temper: self.field[i].temper.saturating_sub(if against {
                ORDER_TEMPER_COST
            } else {
                0
            }),
            gear: self.field[i].gear,
            fatigue: self.field[i].fatigue,
            coat_color,
            style: order.or(style),
            peak_age,
            apt_min,
            apt_max,
            name: h.name.clone(),
            speed: h.speed,
            stamina: h.stamina,
            condition: match self.field[i].condition {
                Condition::Poor if self.field[i].specials.contains(&Special::Steady) => {
                    Condition::Normal
                }
                other => other,
            },
            sex: self.field[i].sex,
            age: self.field[i].age,
            dirt: self.field[i].dirt,
            power: self.field[i].power,
            trainer_distance: Some(self.trainer_of_horse(i).distance),
            course_fit: None,
            jockey: None,
            weight: None,
            star: false,
            mine: true,
        }
    }

    /// The free race is run like a race of the calendar, for rivals of a good
    /// open class over 2400m of turf, open to horses of three and over.
    pub(super) fn free_race() -> CalendarRace {
        CalendarRace {
            week: 0,
            day: Day::Sat,
            race_no: 11,
            venue: Venue::Tokyo,
            name: "フリーレース".to_string(),
            grade: Grade::Listed,
            surface: Surface::Turf,
            distance: FREE_DISTANCE,
            eligibility: Eligibility {
                age: AgeRule::ThreeUp,
                fillies_only: false,
                weight: WeightRule::Allowance,
            },
            prize: 0,
        }
    }

    /// The whole stable alone over 2400m of turf (the old free race), for tests of the odds machinery.
    #[cfg(test)]
    pub(super) fn practice_spec(&self) -> RaceSpec {
        RaceSpec {
            race: None,
            distance: FREE_DISTANCE,
            surface: Surface::Turf,
            entrants: (0..self.field.len())
                .map(|i| self.stable_entrant(i))
                .collect(),
            track: TrackCondition::Good,
        }
    }

    /// Track condition at a course on a day of the current week (the same for every race there).
    pub fn track_for(&self, day: Day, venue: Venue) -> TrackCondition {
        track::forecast(self.race_day_seed(day, venue), self.week)
    }

    /// The weather at a course on a day of the current week: it goes with the track
    /// and has no effect of its own.
    pub fn weather_for(&self, day: Day, venue: Venue) -> Weather {
        let seed = rivals::mix_seed(&[self.race_day_seed(day, venue), 0x5CA1]);
        track::weather_on(self.track_for(day, venue), seed, self.week)
    }

    fn race_day_seed(&self, day: Day, venue: Venue) -> u64 {
        rivals::mix_seed(&[
            self.world_seed,
            self.year as u64,
            self.week as u64,
            day as u64,
            venue as u64,
            0x7AC4,
        ])
    }

    /// The race on this week's card at this place in the programme.
    pub(super) fn calendar_race(&self, key: &RaceKey) -> Result<CalendarRace, GameError> {
        calendar::shared()
            .races
            .iter()
            .find(|r| {
                r.week == self.week
                    && r.day == key.day
                    && r.venue == key.venue
                    && r.race_no == key.race_no
            })
            .cloned()
            .ok_or(GameError::NoSuchRace)
    }

    /// The race (a calendar race, or the free race) and the key its entries are kept under.
    pub(super) fn race_and_slot(
        &self,
        key: Option<&RaceKey>,
    ) -> Result<(CalendarRace, String), GameError> {
        match key {
            None => Ok((Self::free_race(), "free".to_string())),
            Some(k) => Ok((
                self.calendar_race(k)?,
                format!("{:?}:{:?}:{}", k.day, k.venue, k.race_no),
            )),
        }
    }

    /// The stable horses that meet a race's age and sex conditions and whose record suits its class.
    pub(super) fn eligible_horses(&self, race: &CalendarRace) -> Vec<usize> {
        (0..self.field.len())
            .filter(|&i| {
                race.eligibility
                    .allows(self.field[i].age, self.field[i].sex == Sex::Female)
                    && self.admitted(i, race)
                    && !self.field[i].resting
                    && self.field[i].away.is_none()
                    && !self.short_of_earnings(i, race)
                    && !self.closed_to_gelding(i, race)
                    && self.field[i].fatigue < RACE_FATIGUE_LIMIT
            })
            .collect()
    }

    /// Whether a horse's class lets it into a race. Over fences it takes a horse that has
    /// passed the exam, and only its wins over fences count.
    pub(super) fn admitted(&self, i: usize, race: &CalendarRace) -> bool {
        let h = &self.field[i];
        if race.surface == Surface::Jump {
            return h.jump_license
                && match race.grade {
                    Grade::Maiden => h.jump_wins == 0,
                    Grade::G1 | Grade::G2 | Grade::G3 => h.jump_wins >= 1,
                    _ => true,
                };
        }
        race.grade.admits(h.record.starts, h.record.wins)
    }

    /// The Classics are for horses that could go on to stud.
    pub(super) fn closed_to_gelding(&self, i: usize, race: &CalendarRace) -> bool {
        self.field[i].gelded && TRIPLE_CROWN.contains(&race.name.as_str())
    }

    /// Whether a G1 is closed to a horse: it takes horses with enough earnings or a
    /// right from its trial.
    pub(super) fn short_of_earnings(&self, i: usize, race: &CalendarRace) -> bool {
        if !self.entry_rules
            || race.week == 0
            || race.grade != Grade::G1
            || race.surface == Surface::Jump
        {
            return false;
        }
        let needed = if race.eligibility.age == AgeRule::Two {
            G1_MIN_EARNINGS_TWO
        } else {
            G1_MIN_EARNINGS
        };
        let name = &self.field[i].name;
        let right = self
            .trial_rights
            .iter()
            .any(|t| &t.horse == name && t.year == self.year && t.race == race.name);
        self.field[i].career.earnings < needed && !right
    }

    /// How many horses other stables put forward for a race and how many it can
    /// start. The free race and a game without the entry rules always have a full field.
    pub(super) fn nominations(&self, race: &CalendarRace) -> Option<(usize, usize)> {
        (self.entry_rules && race.week != 0).then(|| {
            let seed = self.race_seed(race, 0xF1E1D);
            if race.surface == Surface::Jump {
                let (fewest, most) = JUMP_RIVALS;
                return ((fewest + seed % (most - fewest + 1)) as usize, JUMP_GATE);
            }
            (
                entries::nominated_rivals(race.grade, seed),
                entries::full_gate(race.venue),
            )
        })
    }

    /// A seed of this race, this year, for one of the things that are settled about it.
    pub(super) fn race_seed(&self, race: &CalendarRace, salt: u64) -> u64 {
        rivals::mix_seed(&[
            self.world_seed,
            self.year as u64,
            race.week as u64,
            race.day as u64,
            race.venue as u64,
            race.race_no as u64,
            salt,
        ])
    }

    /// Whether the draw goes against a horse put forward for an oversubscribed race.
    /// A G1 goes by earnings instead, and a horse with priority is sure of its place.
    pub(super) fn drawn_out(&self, i: usize, race: &CalendarRace) -> bool {
        let Some((rivals, gate)) = self.nominations(race) else {
            return false;
        };
        if race.grade == Grade::G1 || self.priority.contains(&self.field[i].name) {
            return false;
        }
        let seed = rivals::mix_seed(&[
            self.world_seed,
            self.year as u64,
            race.week as u64,
            race.day as u64,
            race.venue as u64,
            race.race_no as u64,
            self.field[i].blood_id as u64,
            0xE17,
        ]);
        entries::drawn_out(rivals, gate, seed)
    }

    /// The stable horses put forward for the race: the ones the player chose, or
    /// by default the strongest eligible ones, never more than the limit.
    pub(super) fn registered_horses(&self, slot: &str, race: &CalendarRace) -> Vec<usize> {
        let eligible = self.eligible_horses(race);
        match self.entries.get(slot) {
            Some(names) => names
                .iter()
                .filter_map(|n| self.field.iter().position(|h| &h.name == n))
                .filter(|i| eligible.contains(i))
                .take(MAX_OWN_ENTRIES)
                .collect(),
            None => {
                let mut by_strength = eligible;
                by_strength.sort_by(|&a, &b| {
                    let (sa, sb) = (
                        self.field[a].speed + self.field[a].stamina,
                        self.field[b].speed + self.field[b].stamina,
                    );
                    sb.cmp(&sa).then(a.cmp(&b))
                });
                by_strength.truncate(MAX_OWN_ENTRIES);
                by_strength
            }
        }
    }

    /// The stable horses that really run in the race: those put forward, less any
    /// the draw went against.
    pub(super) fn entered_horses(&self, slot: &str, race: &CalendarRace) -> Vec<usize> {
        self.registered_horses(slot, race)
            .into_iter()
            .filter(|&i| !self.drawn_out(i, race))
            .collect()
    }

    /// Chooses which of the stable's horses run in a race of this week (`None`: the
    /// free race). Up to three horses that meet the race's conditions; none is allowed.
    pub fn set_entries(
        &mut self,
        key: Option<&RaceKey>,
        names: &[String],
    ) -> Result<(), GameError> {
        let (race, slot) = self.race_and_slot(key)?;
        let mut seen = std::collections::HashSet::new();
        let mut idx = Vec::new();
        for n in names {
            idx.push(
                self.field
                    .iter()
                    .position(|h| &h.name == n)
                    .ok_or(GameError::UnknownHorse)?,
            );
            if !seen.insert(n) {
                return Err(GameError::DuplicateHorse);
            }
        }
        let eligible = self.eligible_horses(&race);
        if idx.iter().any(|i| !eligible.contains(i)) {
            return Err(GameError::NotEligible);
        }
        if names.len() > MAX_OWN_ENTRIES {
            return Err(GameError::TooManyEntries);
        }
        self.entries.insert(slot, names.to_vec());
        Ok(())
    }

    /// A race: the stable horses entered in it plus the rivals who got a place.
    pub(super) fn build_spec(
        &self,
        race: &CalendarRace,
        slot: &str,
        rival_seed: u64,
        track: TrackCondition,
        on_calendar: bool,
    ) -> RaceSpec {
        let mut taken_jockeys: Vec<u8> = Vec::new();
        let mut entrants: Vec<Entrant> = self
            .entered_horses(slot, race)
            .into_iter()
            .map(|i| {
                let mut e = self.stable_entrant(i);
                if on_calendar {
                    e.jockey = self.book_jockey(i, race, &e, &taken_jockeys);
                    taken_jockeys.extend(e.jockey.as_ref().map(|j| j.id));
                }
                e
            })
            .collect();
        let taken: std::collections::HashSet<String> =
            self.field.iter().map(|h| h.name.clone()).collect();
        let mut taken = taken;
        taken.extend(self.stars.iter().map(|s| s.name.clone()));
        if on_calendar {
            let mut rng = rivals::Rng::new(rival_seed ^ 0x57A2);
            entrants.extend(self.stars_in(race).into_iter().map(|s| s.entrant(&mut rng)));
        }
        let field = self
            .nominations(race)
            .map_or(entries::FULL_FIELD, |(rivals, gate)| {
                (rivals + entrants.len()).min(gate)
            });
        let rival_count = field.saturating_sub(entrants.len());
        entrants.extend(rivals::generate(race, rival_count, &taken, rival_seed));
        if on_calendar {
            for e in &mut entrants {
                e.course_fit = Some(course::fit(
                    race.venue,
                    race.surface,
                    &e.name,
                    e.style,
                    e.power,
                ));
                let class = rivals::speed_centre(race.grade);
                // a rival's wins are not known: reckon them from how it stands against its class
                let wins = match self.field.iter().position(|h| h.name == e.name) {
                    Some(i) if e.mine => self.field[i].record.wins,
                    _ => (e.speed as i64 - class).max(0) as u32 / RIVAL_SPEED_PER_WIN,
                };
                let fences = if race.surface == Surface::Jump {
                    JUMP_EXTRA_KG
                } else {
                    0
                };
                e.weight = Some(
                    weight::carried(
                        &race.eligibility,
                        race.week,
                        e.age,
                        e.sex == Sex::Female,
                        wins,
                        e.speed,
                        class,
                    ) + fences,
                );
            }
        }

        // deterministic shuffle so the player's horses don't always get the inside gates
        let mut rng = rivals::Rng::new(rival_seed ^ 0x5DEE_CE66);
        for i in (1..entrants.len()).rev() {
            let j = rng.range(0, i as i64) as usize;
            entrants.swap(i, j);
        }

        RaceSpec {
            track,
            distance: race.distance,
            surface: race.surface,
            race: if on_calendar {
                Some(race.clone())
            } else {
                None
            },
            entrants,
        }
    }

    pub(super) fn race_spec(&self, key: Option<&RaceKey>) -> Result<RaceSpec, GameError> {
        let (race, slot) = self.race_and_slot(key)?;
        Ok(match key {
            None => {
                let seed = rivals::mix_seed(&[
                    self.world_seed,
                    self.year as u64,
                    self.week as u64,
                    0xF4EE,
                ]);
                self.build_spec(&race, &slot, seed, TrackCondition::Good, false)
            }
            Some(k) => {
                let seed = rivals::mix_seed(&[
                    self.world_seed,
                    self.year as u64,
                    self.week as u64,
                    k.day as u64,
                    k.venue as u64,
                    k.race_no as u64,
                ]);
                let track = self.track_for(race.day, race.venue);
                self.build_spec(&race, &slot, seed, track, true)
            }
        })
    }

    /// The field and odds for a race of this week (or the free race when `key` is `None`).
    pub fn race_card(&self, key: Option<&RaceKey>) -> Result<RaceCard, GameError> {
        let (race, slot) = self.race_and_slot(key)?;
        let spec = self.race_spec(key)?;
        let table = Self::odds_table_for(&spec);
        let win_odds: Vec<f64> = table.iter().map(|t| t.0).collect();
        let popularity = bets::popularity(&win_odds);
        let eligible = self.eligible_horses(&race);
        let registered = self.registered_horses(&slot, &race);
        let (nominated, gate) = self.nominations(&race).map_or(
            (spec.entrants.len(), entries::FULL_FIELD),
            |(rivals, gate)| (rivals + registered.len(), gate),
        );
        Ok(RaceCard {
            distance: spec.distance,
            surface: spec.surface,
            track: spec.track,
            pace_outlook: race::pace_outlook(race::front_runners(
                spec.entrants.iter().map(|e| e.style),
            ))
            .to_string(),
            fences: if spec.surface == Surface::Jump {
                race::fences_in(spec.distance as f64)
            } else {
                0
            },
            weather: spec
                .race
                .as_ref()
                .map_or(Weather::Sunny, |r| self.weather_for(r.day, r.venue)),
            my_horses: self
                .field
                .iter()
                .enumerate()
                .map(|(i, h)| MyHorse {
                    name: h.name.clone(),
                    eligible: eligible.contains(&i),
                    registered: registered.contains(&i),
                    drawn_out: registered.contains(&i) && self.drawn_out(i, &race),
                    entered: spec.entrants.iter().any(|e| e.mine && e.name == h.name),
                    gear: self.field[i].gear,
                    fatigue: self.field[i].fatigue,
                    resting: self.field[i].resting,
                    away: self.field[i].away.clone(),
                    barred: if self.closed_to_gelding(i, &race) {
                        Some("セン馬不可".to_string())
                    } else {
                        self.short_of_earnings(i, &race)
                            .then(|| "賞金不足".to_string())
                    },
                })
                .collect(),
            trial_for: trials_for(&race),
            course: {
                let course = course::course_of(race.venue);
                let plan = course::plan_of(race.venue, race.surface, race.distance);
                RaceCourse {
                    name: course.name.to_string(),
                    turn: match course.turn {
                        course::Turn::Left => "左回り",
                        course::Turn::Right => "右回り",
                    }
                    .to_string(),
                    straight: plan.straight,
                    lap: plan.lap,
                    layout: plan.name.map(str::to_string),
                    after_goal: plan.after_goal,
                    corners: plan.corners,
                    turf_start: plan.turf_start,
                    straight_course: plan.straight_course,
                }
            },
            max_entries: MAX_OWN_ENTRIES,
            nominated,
            gate,
            place_slots: bets::place_slots(spec.entrants.len()),
            first_prize: spec
                .race
                .as_ref()
                .map_or(0, |r| (r.prize as f64 * PRIZE_RATE).round() as u32),
            entrants: spec
                .entrants
                .iter()
                .zip(table)
                .zip(popularity)
                .map(|((e, (odds, place_odds)), popularity)| CardEntry {
                    body_weight: e.body_weight,
                    body_change: e.body_change,
                    specials: e.specials.iter().map(|s| s.info()).collect(),
                    season: spec
                        .race
                        .as_ref()
                        .and_then(|r| form::season_note(&e.name, r.week))
                        .map(str::to_string),
                    gelded: e.mine && self.field.iter().any(|h| h.name == e.name && h.gelded),
                    jump: e.jump,
                    kick: form::kick_label(form::kick_of(&e.name, e.speed, e.stamina))
                        .map(str::to_string),
                    name: e.name.clone(),
                    speed: e.speed,
                    stamina: e.stamina,
                    condition: e.condition,
                    sex: e.sex,
                    age: e.age,
                    dirt: e.dirt,
                    power: e.power,
                    bad_track: bad_track_of(e.power, e.dirt),
                    growth: form::growth_stage(e.age, e.peak_age),
                    age_form: form::age_form(e.age, e.peak_age),
                    apt_min: e.apt_min,
                    apt_max: e.apt_max,
                    distance_fit: if spec.surface == Surface::Jump {
                        form::DistanceFit::Fit
                    } else {
                        form::distance_effect(spec.distance, e.apt_min, e.apt_max).fit
                    },
                    style: e.style,
                    temper: e.temper,
                    coat_color: e.coat_color.clone(),
                    gear: e.gear,
                    mine: e.mine,
                    course_fit: e.course_fit,
                    jockey: e.jockey.as_ref().map(|j| j.name.clone()),
                    weight: e.weight,
                    star: e.star,
                    trainer_fit: e.trainer_distance.is_some_and(|d| d.favours(spec.distance)),
                    odds,
                    place_odds,
                    popularity,
                })
                .collect(),
            race: spec.race,
        })
    }

    /// The horses the draw kept out of this week's races, unless they ran elsewhere,
    /// have priority next time.
    pub(super) fn note_the_draw(&mut self) {
        let week = self.week;
        let mut out = Vec::new();
        for race in calendar::shared().races.iter().filter(|r| r.week == week) {
            let key = RaceKey::of(race);
            let Ok((_, slot)) = self.race_and_slot(Some(&key)) else {
                continue;
            };
            for name in self.entries.get(&slot).into_iter().flatten() {
                let Some(i) = self.field.iter().position(|h| &h.name == name) else {
                    continue;
                };
                if self.field[i].since_race != 0 && self.drawn_out(i, race) {
                    out.push((name.clone(), race.name.clone()));
                }
            }
        }
        for (name, race) in out {
            self.news.push(format!(
                "{name}は{race}に登録しましたが、抽選で除外されました。次走は優先出走できます。"
            ));
            if !self.priority.contains(&name) {
                self.priority.push(name);
            }
        }
    }

    /// How a race's course suits one of the stable's horses.
    pub(super) fn fit_at(&self, i: usize, race: &CalendarRace) -> CourseFit {
        let e = self.stable_entrant(i);
        course::fit(race.venue, race.surface, &e.name, e.style, e.power)
    }

    /// How every course suits a horse, on turf and dirt.
    pub fn course_fits(&self, name: &str) -> Result<Vec<VenueFit>, GameError> {
        let i = self.slot(name)?;
        let e = self.stable_entrant(i);
        Ok(course::COURSES
            .iter()
            .map(|c| VenueFit {
                venue: c.venue,
                turf: course::fit(c.venue, Surface::Turf, name, e.style, e.power),
                dirt: course::fit(c.venue, Surface::Dirt, name, e.style, e.power),
            })
            .collect())
    }

    /// Who rides a horse in a race: its chosen jockey if they are free, otherwise
    /// whoever its trainer books.
    pub(super) fn book_jockey(
        &self,
        i: usize,
        race: &CalendarRace,
        e: &Entrant,
        taken: &[u8],
    ) -> Option<Ride> {
        let turn = course::course_of(race.venue).turn;
        let budget = match self.trainer_of_horse(i).aim {
            trainer::Aim::Steady => jockey::BUDGET_STEADY,
            trainer::Aim::Balanced => jockey::BUDGET_BALANCED,
            trainer::Aim::Glory => jockey::BUDGET_GLORY,
        };
        let roster = self.jockeys_now();
        // over fences a trainer looks for a jumps rider
        let over_fences = || {
            roster
                .iter()
                .filter(|j| {
                    race.surface == Surface::Jump && j.fee <= budget && !taken.contains(&j.id)
                })
                .max_by_key(|j| j.jump_skill())
                .copied()
        };
        let j = self.field[i]
            .jockey
            .and_then(|id| roster.iter().find(|j| j.id == id).copied())
            .filter(|j| !taken.contains(&j.id))
            .or_else(over_fences)
            .or_else(|| {
                jockey::pick_from(
                    &roster,
                    budget,
                    e.style,
                    turn,
                    taken,
                    Some(self.trainer_of_horse(i).region),
                )
            })?;
        let together = self.field[i]
            .partner
            .filter(|p| p.0 == j.id)
            .map_or(0, |p| p.1);
        Some(Ride {
            jump_skill: j.jump_skill(),
            id: j.id,
            name: j.name.to_string(),
            percent: j.percent(e.style, turn, together),
            tact: j.tact,
            fee: j.fee,
        })
    }

    /// Pays the jockeys who rode in a race. The bill never finishes a stable off.
    pub(super) fn pay_jockeys(&mut self, spec: &RaceSpec) {
        let fees: u32 = spec
            .entrants
            .iter()
            .filter_map(|e| e.jockey.as_ref().map(|j| j.fee))
            .sum();
        self.charge(fees);
    }
}
