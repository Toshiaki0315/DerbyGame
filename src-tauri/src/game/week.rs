//! The end of a week, with the races the trainers send their horses to.

use super::*;

impl Game {
    /// Moves on one week: the horses' conditions are rolled again, and after
    /// week 52 a new year begins and every horse in the stable is a year older.
    pub fn advance_week(&mut self) {
        self.finish_week(None);
    }

    /// Ends the week: `played` is the race the player has just run in it, if any.
    /// The horses entered automatically run first, then the new week begins.
    pub(super) fn finish_week(&mut self, played: Option<(&RaceSpec, &[RaceResult])>) {
        self.news.clear();
        if let Some((spec, results)) = played {
            self.roll_race_injuries(spec);
            self.note_results(spec, results);
        }
        let played = played.map(|p| p.0);
        self.run_expeditions();
        let mut done = self.run_auto_entries(played);
        self.note_the_draw();
        done.extend(played.and_then(|s| s.race.as_ref()).map(race_place));
        self.run_star_races(&done);
        self.pay_upkeep();
        for h in &mut self.field {
            h.since_race = h.since_race.saturating_add(1);
            let rest = if h.away.is_some() { REST_RECOVERY } else { 0 };
            h.fatigue = h.fatigue.saturating_sub(WEEKLY_RECOVERY + rest);
            h.resting = false;
        }
        self.week += 1;
        if self.week > calendar::WEEKS_PER_YEAR {
            self.week = 1;
            self.choose_horse_of_the_year();
            self.close_leading();
            self.year += 1;
            self.change_staff();
            self.trial_rights.retain(|t| t.year == self.year);
            self.age_the_stars();
            for h in &mut self.field {
                h.age = (h.age + 1).min(MAX_AGE);
            }
            self.retire_the_old();
        }
        self.entries.clear();
        self.roll_conditions();
        self.bring_back_horses();
        self.deliver_foals();
        self.warn_of_succession();
        self.run_auto_training();
        self.note_goals();
    }

    /// This week's races a horse could be sent to: it meets their conditions, the
    /// distance is in its range, and it can act on the surface. The race the
    /// player has just run is over.
    pub(super) fn auto_choices(&self, i: usize, played: Option<&RaceSpec>) -> Vec<CalendarRace> {
        self.choices_in(i, self.week, played)
    }

    pub(super) fn choices_in(
        &self,
        i: usize,
        week: u8,
        played: Option<&RaceSpec>,
    ) -> Vec<CalendarRace> {
        let e = self.stable_entrant(i);
        let over = played.and_then(|s| s.race.as_ref());
        calendar::shared()
            .races
            .iter()
            .filter(|r| {
                r.week == week
                    && r.eligibility
                        .allows(self.field[i].age, self.field[i].sex == Sex::Female)
                    && self.admitted(i, r)
                    && (r.surface == Surface::Jump
                        || form::distance_effect(r.distance, e.apt_min, e.apt_max).fit
                            == form::DistanceFit::Fit)
                    && (r.surface != Surface::Dirt || self.field[i].dirt >= 0)
                    && !self.short_of_earnings(i, r)
                    && !over
                        .is_some_and(|o| (o.day, o.venue, o.race_no) == (r.day, r.venue, r.race_no))
            })
            .cloned()
            .collect()
    }

    /// The speed a horse can show this week, to set against a class's rivals.
    pub(super) fn auto_strength(&self, i: usize) -> f64 {
        let e = self.stable_entrant(i);
        e.speed as f64 * form::age_form(e.age, e.peak_age) * e.condition.multiplier()
    }

    /// Whether the horse's trainer, set on the G1, is keeping it for one in the
    /// coming weeks that it has a chance in, with none for it this week.
    pub(super) fn saving_for_g1(&self, i: usize) -> bool {
        let t = self.trainer_of_horse(i);
        if t.aim != trainer::Aim::Glory {
            return false;
        }
        let strength = self.auto_strength(i) + t.aim.margin();
        let in_reach = |r: &CalendarRace| {
            r.grade == Grade::G1 && rivals::speed_centre(r.grade) as f64 <= strength
        };
        !self.auto_choices(i, None).iter().any(in_reach)
            && (self.week + 1..=self.week + AUTO_G1_LEAD)
                .filter(|&w| w <= calendar::WEEKS_PER_YEAR)
                .any(|w| self.choices_in(i, w, None).iter().any(in_reach))
    }

    /// The race for a horse, as its trainer sees it: the richest of those within
    /// the horse's reach (their ambition sets how far that is), over the
    /// trainer's distance if there is one, or failing that the easiest. Races
    /// already holding as many of the stable as may run are passed over.
    pub(super) fn auto_race(
        &self,
        i: usize,
        taken: &[(CalendarRace, Vec<String>)],
        played: Option<&RaceSpec>,
    ) -> Option<CalendarRace> {
        let t = self.trainer_of_horse(i);
        let reach = self.auto_strength(i) + t.aim.margin();
        let same = |a: &CalendarRace, b: &CalendarRace| {
            (a.day, a.venue, a.race_no) == (b.day, b.venue, b.race_no)
        };
        let open: Vec<CalendarRace> = self
            .auto_choices(i, played)
            .into_iter()
            .filter(|r| {
                !taken
                    .iter()
                    .any(|(t, names)| same(t, r) && names.len() >= MAX_OWN_ENTRIES)
            })
            .collect();
        // a horse that has gone over to jumps goes over fences when it can
        let over_fences = open
            .iter()
            .rev()
            .filter(|r| r.surface == Surface::Jump)
            .max_by_key(|r| r.prize);
        if let Some(r) = over_fences {
            return Some(r.clone());
        }
        let within = |r: &&CalendarRace| rivals::speed_centre(r.grade) as f64 <= reach;
        // among equal prizes, the course that suits the horse best
        let richest = |r: &CalendarRace| (r.prize, (self.fit_at(i, r).percent * 100.0) as i64);
        // max_by_key takes the last of equals: reversed, the first on the card
        open.iter()
            .rev()
            .filter(within)
            .filter(|r| t.distance.covers(r.distance))
            .max_by_key(|r| richest(r))
            .or_else(|| open.iter().rev().filter(within).max_by_key(|r| richest(r)))
            .or_else(|| {
                open.iter()
                    .filter(|r| t.distance.covers(r.distance))
                    .min_by_key(|r| rivals::speed_centre(r.grade))
            })
            .or_else(|| open.iter().min_by_key(|r| rivals::speed_centre(r.grade)))
            .cloned()
    }

    /// Which of the horses entered automatically run where this week: their
    /// trainers decide, by the time since the horse last ran, how tired it is,
    /// and what they are aiming at.
    pub(super) fn auto_entries(
        &self,
        played: Option<&RaceSpec>,
    ) -> Vec<(CalendarRace, Vec<String>)> {
        let mut entries: Vec<(CalendarRace, Vec<String>)> = Vec::new();
        for i in 0..self.field.len() {
            let name = &self.field[i].name;
            let ran = played.is_some_and(|s| s.entrants.iter().any(|e| e.mine && &e.name == name));
            let rotation = self.trainer_of_horse(i).rotation;
            if !self.field[i].auto_entry
                || ran
                || self.field[i].resting
                || self.field[i].away.is_some()
                || self.field[i].fatigue > rotation.fatigue_limit()
                || self.field[i].since_race < rotation.gap(self.field[i].condition)
                || self.saving_for_g1(i)
            {
                continue;
            }
            let Some(race) = self.auto_race(i, &entries, played) else {
                continue;
            };
            let held = entries
                .iter_mut()
                .find(|(r, _)| (r.day, r.venue, r.race_no) == (race.day, race.venue, race.race_no));
            match held {
                Some((_, names)) => names.push(name.clone()),
                None => entries.push((race, vec![name.clone()])),
            }
        }
        entries
    }

    /// Runs the week's automatic entries. There is no bet on them: the horses
    /// earn their prize money and their records, and tire as in any race.
    pub(super) fn run_auto_entries(&mut self, played: Option<&RaceSpec>) -> Vec<(Day, Venue, u8)> {
        let mut ran = Vec::new();
        if self.is_game_over() {
            return ran;
        }
        for (race, names) in self.auto_entries(played) {
            let key = RaceKey::of(&race);
            let (_, slot) = match self.race_and_slot(Some(&key)) {
                Ok(found) => found,
                Err(_) => continue,
            };
            self.entries.insert(slot, names);
            let Ok(spec) = self.race_spec(Some(&key)) else {
                continue;
            };
            // the draw went against every one of them
            if !spec.entrants.iter().any(|e| e.mine) {
                continue;
            }
            ran.push(race_place(&race));
            let seed = self.race_seed(&race, 0xA070);
            let results = Self::finish_order(&spec, seed);
            let prizes = Self::owner_prizes(&spec, &results, race.prize);
            let prize_total: u32 = prizes.iter().map(|p| p.amount).sum();
            self.earn(prize_total as i64);
            self.add_to_records(&spec, &results);
            self.note_results(&spec, &results);
            self.pay_jockeys(&spec);
            self.tire_runners(&spec, race_fatigue(race.distance));
            self.roll_race_injuries(&spec);
            self.round += 1;

            let mine: Vec<(String, usize)> = results
                .iter()
                .enumerate()
                .filter(|(_, r)| spec.entrants.iter().any(|e| e.mine && e.name == r.name))
                .map(|(rank, r)| (r.name.clone(), rank + 1))
                .collect();
            let surface = match race.surface {
                Surface::Turf => "芝",
                Surface::Dirt => "ダート",
                Surface::Jump => "障害",
            };
            for (name, rank) in &mine {
                let won = prizes.iter().find(|p| &p.name == name);
                self.news.push(format!(
                    "{name}は自動出走で{}({surface}{}m)に出て{rank}着でした{}。",
                    race.name,
                    race.distance,
                    won.map_or(String::new(), |p| format!("(賞金+{})", p.amount)),
                ));
            }
            self.remember(HistoryEntry {
                kind: BetKind::Win,
                race: Some(race.name.clone()),
                round: self.round,
                horse: mine
                    .iter()
                    .map(|(name, rank)| format!("{name} {rank}着"))
                    .chain(
                        Self::did_not_finish(&spec, &results)
                            .into_iter()
                            .filter(|n| spec.entrants.iter().any(|e| e.mine && &e.name == n))
                            .map(|n| format!("{n} 競走中止")),
                    )
                    .collect::<Vec<_>>()
                    .join("、"),
                horses: mine.iter().map(|(name, _)| name.clone()).collect(),
                stake: 0,
                won: mine.iter().any(|(_, rank)| *rank == 1),
                payout: 0,
                prize: prize_total,
                ticket_count: 0,
                watched: false,
                mixed_kinds: false,
                winner: results.first().map(|r| r.name.clone()).unwrap_or_default(),
            });
        }
        ran
    }
}
