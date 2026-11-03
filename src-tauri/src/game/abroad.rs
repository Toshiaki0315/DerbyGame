//! Expeditions to the big races abroad (海外遠征).

use super::*;

impl Game {
    /// Whether a horse is good enough, old enough and free to go abroad.
    pub(super) fn may_go_abroad(&self, i: usize) -> bool {
        self.field[i].age >= 3
            && self.field[i].away.is_none()
            && (self.field[i].career.g1_wins() > 0
                || self.field[i].career.earnings >= EXPEDITION_MIN_EARNINGS)
    }

    /// Sends a horse abroad for a race. It is away until it gets back.
    pub fn go_abroad(&mut self, name: &str, race: usize) -> Result<(), GameError> {
        let i = self.slot(name)?;
        let target = OVERSEAS.get(race).ok_or(GameError::UnknownRace)?;
        if self.field[i].away.is_some() {
            return Err(GameError::Away);
        }
        if !expedition_open(target, self.week) {
            return Err(GameError::SaleClosed);
        }
        if !self.may_go_abroad(i) {
            return Err(GameError::NotEligible);
        }
        // the whole of the money would end the game
        if EXPEDITION_COST >= self.money {
            return Err(GameError::InsufficientFunds);
        }
        self.money -= EXPEDITION_COST;
        self.field[i].away = Some(Away {
            kind: AwayKind::Expedition,
            label: "遠征".to_string(),
            weeks: (target.week - self.week) as u32 + EXPEDITION_RETURN_WEEKS,
        });
        self.field[i].resting = false;
        self.expeditions.push(Expedition {
            horse: name.to_string(),
            race,
            year: self.year,
        });
        Ok(())
    }

    /// Runs the races abroad of this week for the horses sent for them.
    pub(super) fn run_expeditions(&mut self) {
        let (due, waiting): (Vec<Expedition>, Vec<Expedition>) =
            std::mem::take(&mut self.expeditions)
                .into_iter()
                .partition(|e| OVERSEAS[e.race].week <= self.week || e.year != self.year);
        self.expeditions = waiting;
        for e in due {
            if let Some(i) = self.field.iter().position(|h| h.name == e.horse) {
                self.run_abroad(i, &OVERSEAS[e.race]);
            }
        }
    }

    pub(super) fn run_abroad(&mut self, i: usize, target: &OverseasRace) {
        let mut race = Self::free_race();
        race.week = target.week;
        race.name = target.name.to_string();
        race.grade = Grade::G1;
        race.surface = target.surface;
        race.distance = target.distance;
        race.prize = target.prize;
        let seed = rivals::mix_seed(&[
            self.world_seed,
            self.year as u64,
            target.week as u64,
            0xAB0A,
        ]);
        let mut mine = self.stable_entrant(i);
        mine.jockey = self.book_jockey(i, &race, &mine, &[]);
        let taken = self.field.iter().map(|h| h.name.clone()).collect();
        let mut entrants = rivals::generate(&race, entries::FULL_FIELD - 1, &taken, seed);
        for e in &mut entrants {
            e.speed += OVERSEAS_STRENGTH;
        }
        entrants.push(mine);
        let spec = RaceSpec {
            distance: race.distance,
            surface: race.surface,
            track: TrackCondition::Good,
            entrants,
            race: Some(race.clone()),
        };
        let results = Self::finish_order(&spec, seed ^ 0xAB0B);
        let name = self.field[i].name.clone();
        let place = results
            .iter()
            .position(|r| r.name == name)
            .unwrap_or(entries::FULL_FIELD);
        let prizes = Self::owner_prizes(&spec, &results, race.prize);
        self.earn(prizes.iter().map(|p| p.amount as i64).sum());
        self.add_to_records(&spec, &results);
        self.note_results(&spec, &results);
        self.pay_jockeys(&spec);
        self.tire_runners(&spec, race_fatigue(race.distance));
        self.news.push(format!(
            "{name}は{}に出走し、{}着でした。",
            race.name,
            place + 1
        ));
        if place == 0 {
            self.field[i]
                .career
                .titles
                .push(format!("{}制覇", race.name));
            self.awards.push(Award {
                year: self.year,
                title: format!("{OVERSEAS_AWARD}{}", race.name),
                horse: name,
                mine: true,
            });
        }
    }
}
