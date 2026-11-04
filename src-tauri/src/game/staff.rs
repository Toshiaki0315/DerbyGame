//! Jockeys and trainers retiring and others coming in (騎手・調教師の入れ替わり).

use super::*;
use crate::jockey::{Jockey, ROOKIES};
use crate::trainer::{Trainer, SUCCESSORS};

/// Years in the job of those there at the start, fewest and most.
const FIRST_TENURE: (u64, u64) = (3, 20);
/// Years in the job of those who come in later.
const NEW_TENURE: (u64, u64) = (10, 25);

fn tenure(seed: u64, who: u64, (fewest, most): (u64, u64)) -> u32 {
    (fewest + rivals::mix_seed(&[seed, who, 0x57AF]) % (most - fewest + 1)) as u32
}

impl Game {
    /// Those there as a game starts, each with their own years to go.
    pub(super) fn first_staff(&mut self) {
        let year = self.year;
        let seed = self.world_seed;
        self.jockey_staff = JOCKEYS
            .iter()
            .map(|j| Staff {
                id: j.id,
                since: 0,
                until: year + tenure(seed, j.id as u64, FIRST_TENURE),
            })
            .collect();
        self.trainer_staff = TRAINERS
            .iter()
            .map(|t| Staff {
                id: t.id,
                since: 0,
                until: year + tenure(seed, 1000 + t.id as u64, FIRST_TENURE),
            })
            .collect();
    }

    /// The jockeys riding now, as good as they are by now.
    pub(super) fn jockeys_now(&self) -> Vec<Jockey> {
        self.jockey_staff
            .iter()
            .filter(|s| s.until > self.year)
            .filter_map(|s| {
                let mut j = *jockey::jockey_by_id(s.id)?;
                j.skill = jockey::skill_after(j.skill, s.since > 0, self.year - s.since);
                Some(j)
            })
            .collect()
    }

    /// The trainers with a stable now.
    pub(super) fn trainers_now(&self) -> Vec<&'static Trainer> {
        self.trainer_staff
            .iter()
            .filter(|s| s.until > self.year)
            .map(|s| trainer::trainer_by_id(s.id))
            .collect()
    }

    /// At the turn of the year, those whose time is up retire. Someone from the same
    /// side takes each place; with nobody left to come, they stay on.
    pub(super) fn change_staff(&mut self) {
        for i in 0..self.jockey_staff.len() {
            let gone = self.jockey_staff[i];
            if gone.until != self.year {
                continue;
            }
            let old = jockey::jockey_by_id(gone.id).expect("a jockey on the staff");
            let used = |id: u8| self.jockey_staff.iter().any(|s| s.id == id);
            let Some(new) = ROOKIES
                .iter()
                .find(|j| j.region == old.region && !used(j.id))
            else {
                self.jockey_staff[i].until = u32::MAX;
                continue;
            };
            let until = self.year + tenure(self.world_seed, new.id as u64, NEW_TENURE);
            self.jockey_staff.push(Staff {
                id: new.id,
                since: self.year,
                until,
            });
            for h in &mut self.field {
                if h.jockey == Some(gone.id) {
                    h.jockey = None;
                }
            }
            self.news.push(format!(
                "{}騎手が引退しました。新人の{}騎手がデビューします。",
                old.name, new.name
            ));
        }
        for i in 0..self.trainer_staff.len() {
            let gone = self.trainer_staff[i];
            if gone.until != self.year {
                continue;
            }
            let old = trainer::trainer_by_id(gone.id);
            let used = |id: u8| self.trainer_staff.iter().any(|s| s.id == id);
            let Some(new) = SUCCESSORS
                .iter()
                .find(|t| t.region == old.region && !used(t.id))
            else {
                self.trainer_staff[i].until = u32::MAX;
                continue;
            };
            let until = self.year + tenure(self.world_seed, 1000 + new.id as u64, NEW_TENURE);
            self.trainer_staff.push(Staff {
                id: new.id,
                since: self.year,
                until,
            });
            for h in &mut self.field {
                if h.trainer == gone.id {
                    h.trainer = new.id;
                }
            }
            self.news.push(format!(
                "{}調教師が引退しました。{}調教師が厩舎を引き継ぎます。",
                old.name, new.name
            ));
        }
    }
}
