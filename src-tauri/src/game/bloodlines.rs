//! How the stable's horses are tied to the blood database.

use super::*;

impl Game {
    /// The generated blood database of this game (shared, built once per seed).
    pub(super) fn base_db(&self) -> std::sync::Arc<Vec<BloodHorse>> {
        bloodline::base_database(self.bloodline_seed())
    }

    /// A record of the blood database by id: a generated horse, or one the player added.
    /// The founding horses carry the names of the stable's horses.
    pub(super) fn blood_record(&self, id: u32) -> Option<BloodHorse> {
        let base = self.base_db();
        let mut rec = match base.get((id as usize).wrapping_sub(1)) {
            Some(r) if r.id == id => r.clone(),
            _ => self.bred.iter().find(|h| h.id == id)?.clone(),
        };
        if let Some(name) = self.blood_names.get(&id) {
            rec.name = name.clone();
        }
        rec.owned = self.owns(id);
        Some(rec)
    }

    /// The whole database as a map (for working out ancestry).
    pub(super) fn blood_map(&self) -> std::collections::HashMap<u32, BloodHorse> {
        self.bloodline_all()
            .into_iter()
            .map(|h| (h.id, h))
            .collect()
    }

    /// Links every horse in the stable to a record: the founding horses to generated
    /// ones, any other (a foal from before the database) to a new record of its own.
    pub(super) fn link_blood(&mut self) {
        let base = self.base_db();
        let founding: Vec<(usize, usize)> = self
            .field
            .iter()
            .enumerate()
            .filter_map(|(i, h)| {
                FOUNDING
                    .iter()
                    .position(|n| *n == h.name)
                    .map(|slot| (i, slot))
            })
            .collect();
        let wanted: Vec<(usize, Sex)> = founding
            .iter()
            .map(|&(i, slot)| (slot, self.field[i].sex))
            .collect();
        let links = founder_links(&base, self.world_seed, &wanted);
        let mut ids = vec![0u32; self.field.len()];
        self.blood_names.clear();
        for (&(i, _), id) in founding.iter().zip(links) {
            ids[i] = id;
            self.blood_names.insert(id, self.field[i].name.clone());
        }
        for (i, slot) in ids.iter_mut().enumerate() {
            if *slot == 0 {
                let id = self.next_blood_id();
                let stats = bloodline::Stats {
                    speed: self.field[i].speed as f64,
                    stamina: self.field[i].stamina as f64,
                    temper: 50.0,
                    power: self.field[i].power as f64,
                    turf: 50.0,
                    dirt: track::dirt_score(self.field[i].dirt),
                };
                self.bred.push(bloodline::unrelated(
                    id,
                    &self.field[i].name,
                    self.field[i].sex,
                    stats,
                ));
                *slot = id;
            }
        }
        for (horse, id) in self.field.iter_mut().zip(ids) {
            horse.blood_id = id;
        }
    }

    /// Whether a horse in the database already has this name.
    pub(super) fn name_in_database(&self, name: &str) -> bool {
        self.base_db().iter().any(|h| h.name == name)
            || self.bred.iter().any(|h| h.name == name)
            || self.pregnancies.iter().any(|p| p.foal.name == name)
            || self.blood_names.values().any(|n| n == name)
            || self.stars.iter().any(|s| s.name == name)
    }

    /// Gives each racehorse the temperament its blood record has.
    pub(super) fn set_tempers_from_records(&mut self) {
        for i in 0..self.field.len() {
            self.field[i].temper = self
                .blood_record(self.field[i].blood_id)
                .map_or(race::NEUTRAL_TEMPER as u32, |r| temper_of(r.stats.temper));
        }
    }

    /// Gives each racehorse the legs its blood record says it was born with.
    pub(super) fn set_legs_from_blood(&mut self) {
        for horse in &mut self.field {
            horse.legs = legs_of_blood(horse.blood_id);
        }
    }

    /// A record of a horse racing for the stable or kept at stud.
    pub(super) fn owns(&self, id: u32) -> bool {
        self.field.iter().any(|h| h.blood_id == id) || self.studs.iter().any(|s| s.blood_id == id)
    }

    pub(super) fn next_blood_id(&self) -> u32 {
        let base = self.base_db().len() as u32;
        let on_the_way = self.pregnancies.iter().map(|p| &p.foal);
        self.bred
            .iter()
            .chain(on_the_way)
            .map(|h| h.id)
            .max()
            .unwrap_or(0)
            .max(base)
            + 1
    }

    /// The database as the player sees it: generated horses plus their own foals,
    /// the stable's horses under their own names and marked as owned.
    pub fn bloodline_all(&self) -> Vec<BloodHorse> {
        let base = self.base_db();
        let mut all: Vec<BloodHorse> = base
            .iter()
            .cloned()
            .chain(self.bred.iter().cloned())
            .collect();
        for h in &mut all {
            if let Some(name) = self.blood_names.get(&h.id) {
                h.name = name.clone();
            }
            h.owned = self.owns(h.id);
        }
        all.sort_by_key(|h| h.id);
        all
    }

    /// A stable horse's record, with the stats it has now (training included) in place of its birth stats.
    pub(super) fn stable_blood(&self, i: usize) -> Option<BloodHorse> {
        let mut r = self.blood_record(self.field[i].blood_id)?;
        r.stats.speed = self.field[i].speed as f64;
        r.stats.stamina = self.field[i].stamina as f64;
        r.stats.power = self.field[i].power as f64;
        r.stats.dirt = track::dirt_score(self.field[i].dirt);
        r.stats.temper = self.field[i].temper as f64;
        r.gender = self.field[i].sex;
        Some(r)
    }

    /// The names of a record's parents and grandparents, if it has a family tree.
    pub(super) fn pedigree_of_record(&self, rec: &BloodHorse) -> Option<Pedigree> {
        let name = |id: Option<u32>| -> String {
            id.and_then(|i| self.blood_record(i))
                .map(|h| h.name)
                .unwrap_or_else(|| "不明".to_string())
        };
        let (sire, dam) = (
            rec.sire_id.and_then(|i| self.blood_record(i))?,
            rec.dam_id.and_then(|i| self.blood_record(i))?,
        );
        Some(Pedigree {
            sire_sire: name(sire.sire_id),
            sire_dam: name(sire.dam_id),
            dam_sire: name(dam.sire_id),
            dam_dam: name(dam.dam_id),
            sire: sire.name,
            dam: dam.name,
        })
    }

    /// Seeds the master pedigree, so it is rebuilt identically instead of being saved.
    pub fn bloodline_seed(&self) -> u64 {
        rivals::mix_seed(&[self.world_seed, 0xB100D])
    }
}
