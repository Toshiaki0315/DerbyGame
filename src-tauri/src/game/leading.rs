//! The year's leading jockey, trainer and sire (リーディング). Only the stable's own
//! races are ever run, so the rest of a year's wins are made up from the seed, more
//! for the better; the stable's wins come on top.

use super::*;

/// A year's wins elsewhere for each point of strength, and how much luck adds.
const WINS_PER_POINT: f64 = 1.4;
const LUCK_WINS: u64 = 20;
/// How many of the generation's best stallions make up the sires' table.
const SIRES_IN_TABLE: usize = 10;

impl Game {
    /// Wins elsewhere so far this year for someone of this strength.
    fn wins_elsewhere(&self, who: u64, strength: f64) -> u32 {
        let seed = rivals::mix_seed(&[self.world_seed, self.year as u64, who, 0x1EAD]);
        let year = strength.max(0.0) * WINS_PER_POINT + (seed % LUCK_WINS) as f64;
        (year * self.week as f64 / calendar::WEEKS_PER_YEAR as f64).round() as u32
    }

    /// This year's tables so far.
    pub fn leading(&self) -> Leading {
        let ranked = |mut rows: Vec<LeadingRow>| {
            rows.sort_by(|a, b| b.wins.cmp(&a.wins).then(a.name.cmp(&b.name)));
            rows
        };
        let row = |name: &str, elsewhere: u32, own: u32| LeadingRow {
            name: name.to_string(),
            wins: elsewhere + own,
            own_wins: own,
        };
        let own_u8 =
            |m: &std::collections::HashMap<u8, u32>, id: u8| m.get(&id).copied().unwrap_or(0);
        let jockeys = self
            .jockeys_now()
            .iter()
            .map(|j| {
                let strength = j.skill as f64 - 40.0;
                row(
                    j.name,
                    self.wins_elsewhere(j.id as u64, strength),
                    own_u8(&self.year_wins.jockeys, j.id),
                )
            })
            .collect();
        let trainers = self
            .trainers_now()
            .iter()
            .map(|t| {
                let who = 1000 + t.id as u64;
                let strength = (rivals::mix_seed(&[self.world_seed, who]) % 40) as f64 + 10.0;
                row(
                    t.name,
                    self.wins_elsewhere(who, strength),
                    own_u8(&self.year_wins.trainers, t.id),
                )
            })
            .collect();
        Leading {
            jockeys: ranked(jockeys),
            trainers: ranked(trainers),
            sires: ranked(self.sire_rows()),
        }
    }

    /// The generation's best stallions, with the stable's own and any that have sired
    /// one of its winners.
    fn sire_rows(&self) -> Vec<LeadingRow> {
        let base = self.base_db();
        let mut best: Vec<&BloodHorse> = base
            .iter()
            .filter(|h| h.gen == NEWEST_GENERATION && h.gender == Sex::Male)
            .collect();
        best.sort_by(|a, b| stat_sum(b).total_cmp(&stat_sum(a)).then(a.id.cmp(&b.id)));
        let mut ids: Vec<u32> = best.iter().take(SIRES_IN_TABLE).map(|h| h.id).collect();
        let others = self
            .studs
            .iter()
            .filter(|s| s.sex == Sex::Male)
            .map(|s| s.blood_id)
            .chain(self.year_wins.sires.keys().copied());
        for id in others {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
        ids.sort_unstable();
        ids.iter()
            .filter_map(|&id| {
                let r = self.blood_record(id)?;
                let strength = stat_sum(&r) / 4.0 - 30.0;
                let own = self.year_wins.sires.get(&id).copied().unwrap_or(0);
                Some(LeadingRow {
                    name: r.name,
                    wins: self.wins_elsewhere(2000 + id as u64, strength) + own,
                    own_wins: own,
                })
            })
            .collect()
    }

    /// At the end of a year, names its leaders and starts the count again.
    pub(super) fn close_leading(&mut self) {
        let t = self.leading();
        let first = |rows: &[LeadingRow]| rows.first().map_or(String::new(), |r| r.name.clone());
        let year = LeadingYear {
            year: self.year,
            jockey: first(&t.jockeys),
            trainer: first(&t.trainers),
            sire: first(&t.sires),
        };
        self.news.push(format!(
            "{}年目のリーディングは、騎手が{}、調教師が{}、種牡馬が{}でした。",
            year.year, year.jockey, year.trainer, year.sire
        ));
        self.leading_years.push(year);
        self.year_wins = YearWins::default();
    }
}
