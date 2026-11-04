//! The stud book, matings, foals, the farm and the yearling sale.

use super::*;

impl Game {
    /// The stallions and mares on sale this year: picked from the best of the
    /// database's recent generations that nobody owns, half of each sex.
    pub fn stud_market(&self) -> Vec<StudOffer> {
        let mut offers = vec![];
        for sex in [Sex::Male, Sex::Female] {
            let mut best = self.best_unowned(sex);
            // a different few each year
            best.sort_by_key(|h| {
                rivals::Rng::new(rivals::mix_seed(&[
                    self.world_seed,
                    self.year as u64,
                    h.id as u64,
                ]))
                .next_u64()
            });
            for h in best.into_iter().take(MARKET_EACH) {
                let rec = self.blood_record(h.id).expect("a database record");
                offers.push(StudOffer {
                    blood_id: rec.id,
                    name: rec.name.clone(),
                    sex: rec.gender,
                    speed: stat_of(rec.stats.speed),
                    stamina: stat_of(rec.stats.stamina),
                    power: stat_of(rec.stats.power),
                    dirt: dirt_level(rec.stats.dirt),
                    temper: temper_of(rec.stats.temper),
                    legs: legs_of_blood(rec.id),
                    price: stud_price(&rec),
                    stud_fee: if rec.gender == Sex::Male {
                        ((stud_price(&rec) as f64 * STUD_FEE_RATE / 10.0).round() * 10.0) as u32
                    } else {
                        0
                    },
                    blood: blood_traits(rec),
                });
            }
        }
        offers
    }

    /// The strongest horses of a sex from the database's recent generations
    /// that nobody owns: the ones the market and a new game choose from.
    pub(super) fn best_unowned(&self, sex: Sex) -> Vec<BloodHorse> {
        let base = self.base_db();
        let mut pool: Vec<&BloodHorse> = base
            .iter()
            .filter(|h| h.gen >= MARKET_MIN_GEN && h.gender == sex && !self.owns(h.id))
            .collect();
        pool.sort_by(|a, b| stat_sum(b).total_cmp(&stat_sum(a)).then(a.id.cmp(&b.id)));
        pool.into_iter().take(MARKET_POOL).cloned().collect()
    }

    /// A stallion and a mare, chosen at random, for the stud book of a new game.
    pub(super) fn starting_studs(&self) -> Vec<Stud> {
        let mut rng = rivals::Rng::new(rivals::mix_seed(&[self.world_seed, 0x57D_B00C]));
        [Sex::Male, Sex::Female]
            .into_iter()
            .map(|sex| {
                let pool = self.best_unowned(sex);
                let pick = &pool[rng.range(0, pool.len() as i64 - 1) as usize];
                let rec = self.blood_record(pick.id).expect("a database record");
                Stud {
                    specials: rec.specials.clone(),
                    career: Career::default(),
                    legs: legs_of_blood(rec.id),
                    name: rec.name.clone(),
                    sex,
                    speed: stat_of(rec.stats.speed),
                    stamina: stat_of(rec.stats.stamina),
                    power: stat_of(rec.stats.power),
                    dirt: dirt_level(rec.stats.dirt),
                    temper: temper_of(rec.stats.temper),
                    record: RaceRecord::default(),
                    blood_id: rec.id,
                    retired_age: 0,
                    retired_year: self.year,
                    bought: false,
                    starter: true,
                }
            })
            .collect()
    }

    /// Buys a stallion or mare on offer for the stud book.
    pub fn buy_stud(&mut self, blood_id: u32) -> Result<(), GameError> {
        if self.is_game_over() {
            return Err(GameError::GameOver);
        }
        let offer = self
            .stud_market()
            .into_iter()
            .find(|o| o.blood_id == blood_id)
            .ok_or(GameError::NotForSale)?;
        if self.studs.len() >= MAX_STUDS {
            return Err(GameError::StudFull);
        }
        if offer.price > self.money {
            return Err(GameError::InsufficientFunds);
        }
        self.money -= offer.price;
        self.studs.push(Stud {
            specials: self
                .blood_record(offer.blood_id)
                .map_or(vec![], |r| r.specials),
            career: Career::default(),
            legs: offer.legs,
            name: offer.name,
            sex: offer.sex,
            speed: offer.speed,
            stamina: offer.stamina,
            power: offer.power,
            dirt: offer.dirt,
            temper: offer.temper,
            record: RaceRecord::default(),
            blood_id,
            retired_age: 0,
            retired_year: self.year,
            bought: true,
            starter: false,
        });
        Ok(())
    }

    /// Lets a stud go (its record stays in the database, no longer the stable's).
    pub fn release_stud(&mut self, name: &str) -> Result<(), GameError> {
        let i = self
            .studs
            .iter()
            .position(|s| s.name == name)
            .ok_or(GameError::UnknownHorse)?;
        let stud = self.studs.remove(i);
        // the record keeps the name the stable gave it
        self.blood_names.entry(stud.blood_id).or_insert(stud.name);
        Ok(())
    }

    /// A parent for breeding, from the racing stable or the stud book, with its current stats.
    pub(super) fn parent(&self, name: &str) -> Result<(BloodHorse, Sex), GameError> {
        if let Some(i) = self.field.iter().position(|h| h.name == name) {
            let rec = self.stable_blood(i).ok_or(GameError::UnknownHorse)?;
            return Ok((rec, self.field[i].sex));
        }
        let Some(s) = self.studs.iter().find(|s| s.name == name) else {
            // a stallion on the market can be used for a fee
            return self
                .stud_market()
                .into_iter()
                .find(|o| o.sex == Sex::Male && o.name == name)
                .and_then(|o| self.blood_record(o.blood_id))
                .map(|r| (r, Sex::Male))
                .ok_or(GameError::UnknownHorse);
        };
        let mut r = self
            .blood_record(s.blood_id)
            .ok_or(GameError::UnknownHorse)?;
        r.name = s.name.clone();
        r.stats.speed = s.speed as f64;
        r.stats.stamina = s.stamina as f64;
        r.stats.power = s.power as f64;
        r.stats.dirt = track::dirt_score(s.dirt);
        r.stats.temper = s.temper as f64;
        r.gender = s.sex;
        r.specials = s.specials.clone();
        Ok((r, s.sex))
    }

    /// The fee to use this stallion: nothing for the stable's own, a share of its
    /// price for one on the market.
    pub(super) fn stud_fee(&self, sire: &str) -> u32 {
        let owned =
            self.field.iter().any(|h| h.name == sire) || self.studs.iter().any(|s| s.name == sire);
        if owned {
            return 0;
        }
        self.stud_market()
            .into_iter()
            .find(|o| o.sex == Sex::Male && o.name == sire)
            .map_or(0, |o| o.stud_fee)
    }

    /// The sires the player has used, with what they have got.
    pub(super) fn sire_lines(&self) -> Vec<SireLine> {
        let mut lines: Vec<SireLine> = self
            .sire_records
            .iter()
            .filter_map(|(id, r)| {
                Some(SireLine {
                    sire: self.blood_record(*id)?.name,
                    foals: r.foals,
                    g1_wins: r.g1_wins,
                    established: r.established,
                })
            })
            .collect();
        lines.sort_by(|a, b| b.g1_wins.cmp(&a.g1_wins).then(a.sire.cmp(&b.sire)));
        lines
    }

    /// Whether a sire is the founder of a line, or a son of one.
    pub(super) fn line_founded(&self, sire: &BloodHorse) -> bool {
        [Some(sire.id), sire.sire_id]
            .into_iter()
            .flatten()
            .any(|id| self.sire_records.get(&id).is_some_and(|r| r.established))
    }

    /// Counts a G1 win for the sire of a horse, and founds its line if that is enough.
    pub(super) fn credit_the_sire(&mut self, i: usize) {
        let Some(sire) = self
            .blood_record(self.field[i].blood_id)
            .and_then(|r| r.sire_id)
        else {
            return;
        };
        let Some(record) = self.sire_records.get_mut(&sire) else {
            return;
        };
        record.g1_wins += 1;
        if record.established || record.foals < LINE_FOALS || record.g1_wins < LINE_G1_WINS {
            return;
        }
        record.established = true;
        let name = self.blood_record(sire).map_or(String::new(), |r| r.name);
        self.news.push(format!("{name}の系統が確立しました!"));
        self.awards.push(Award {
            year: self.year,
            title: TITLE_LINE.to_string(),
            horse: name,
            mine: true,
        });
    }

    /// Improves a facility of the farm.
    pub fn upgrade_farm(&mut self, facility: Facility) -> Result<(), GameError> {
        let level = match facility {
            Facility::Pasture => self.farm.pasture,
            Facility::Nursery => self.farm.nursery,
        };
        let cost = *FARM_COSTS
            .get(level as usize)
            .ok_or(GameError::FacilityFull)?;
        // the whole of the money would end the game
        if cost >= self.money {
            return Err(GameError::InsufficientFunds);
        }
        self.money -= cost;
        match facility {
            Facility::Pasture => self.farm.pasture += 1,
            Facility::Nursery => self.farm.nursery += 1,
        }
        Ok(())
    }

    /// Mates a stallion and a mare, both at stud (or a stallion on the market), in the
    /// breeding season. It may not take; the mating is paid for either way.
    pub fn breed(
        &mut self,
        sire: &str,
        dam: &str,
        replace: &str,
        foal_name: &str,
    ) -> Result<(), GameError> {
        if self.is_game_over() {
            return Err(GameError::GameOver);
        }
        if !breeding_open(self.week) {
            return Err(GameError::OutOfSeason);
        }
        let racing = |name: &str| self.field.iter().position(|h| h.name == name);
        if racing(dam).is_some_and(|i| self.field[i].sex == Sex::Female) {
            return Err(GameError::DamNotRetired);
        }
        if racing(sire).is_some_and(|i| self.field[i].sex == Sex::Male) {
            return Err(GameError::SireNotRetired);
        }
        let conception = self
            .studs
            .iter()
            .find(|h| h.name == dam)
            .map_or(CONCEPTION_PERCENT, |h| self.conception_of(h));
        self.mate(sire, dam, replace, foal_name, conception)
    }

    /// How likely a mare at stud is to take this year.
    pub(super) fn conception_of(&self, mare: &Stud) -> u32 {
        conception_percent(mare.retired_age as u32 + (self.year - mare.retired_year))
    }

    /// Pays for a mating and, if it takes, makes the foal. It comes `GESTATION_WEEKS`
    /// later, taking the place of `replace` if that horse is still in the stable, or an
    /// empty place.
    pub(super) fn mate(
        &mut self,
        sire: &str,
        dam: &str,
        replace: &str,
        foal_name: &str,
        conception: u32,
    ) -> Result<(), GameError> {
        if self.is_game_over() {
            return Err(GameError::GameOver);
        }
        let (sire_rec, sire_sex) = self.parent(sire)?;
        let (dam_rec, dam_sex) = self.parent(dam)?;
        if !replace.is_empty() && self.field.iter().all(|h| h.name != replace) {
            return Err(GameError::UnknownHorse);
        }
        if sire_sex != Sex::Male || dam_sex != Sex::Female {
            return Err(GameError::WrongSex);
        }
        if self.pregnancies.iter().any(|p| p.dam == dam) {
            return Err(GameError::MareInFoal);
        }
        if !self.room_for_a_foal(replace) {
            return Err(GameError::StableFull);
        }
        if !crate::horse::is_valid_name(foal_name) {
            return Err(GameError::InvalidName);
        }
        if self.field.iter().any(|h| h.name == foal_name)
            || self.studs.iter().any(|s| s.name == foal_name)
            || self.name_in_database(foal_name)
        {
            return Err(GameError::DuplicateName);
        }
        let legs = {
            let (sl, dl) = (self.parent_legs(sire), self.parent_legs(dam));
            let spread = FOAL_LEGS_SPREAD as i64;
            let roll = rivals::mix_seed(&[self.world_seed, self.next_blood_id() as u64, 0x1E66]);
            let drift = (roll % (2 * spread as u64 + 1)) as i64 - spread;
            ((sl + dl) as i64 / 2 + drift).clamp(LEGS_MIN as i64, LEGS_MAX as i64) as u32
        };
        let cost = BREED_COST + self.stud_fee(sire);
        if cost > self.money {
            return Err(GameError::InsufficientFunds);
        }

        self.money -= cost;
        if conception < 100 && self.next_u64() % 100 >= conception as u64 {
            return Ok(());
        }

        // the foal is made by the blood database: the parents' current stats, their
        // ancestry (for inbreeding), their lines (for a nick) and a chance of mutation
        let db = self.blood_map();
        let id = self.next_blood_id();
        let gen = sire_rec.gen.max(dam_rec.gen) + 1;
        let mut rng = rivals::Rng::new(self.next_u64());
        let founded = if self.line_founded(&sire_rec) {
            LINE_THEORY_POINTS
        } else {
            0
        };
        let points: u32 = founded
            + bloodline::theories(&sire_rec, &dam_rec, &db)
                .iter()
                .map(|t| t.points)
                .sum::<u32>();
        let mut foal = bloodline::breed(
            &sire_rec,
            &dam_rec,
            gen,
            id,
            foal_name.to_string(),
            &db,
            &mut rng,
        );
        // a mating the theories favour puts something on the foal
        let bonus = points as f64 * THEORY_STAT_PER_POINT;
        foal.stats.speed = grown(foal.stats.speed, bonus);
        foal.stats.stamina = grown(foal.stats.stamina, bonus);
        foal.stats.power = grown(foal.stats.power, bonus);
        self.pregnancies.push(Pregnancy {
            foal,
            legs,
            sire: sire.to_string(),
            dam: dam.to_string(),
            replace: replace.to_string(),
            weeks_left: GESTATION_WEEKS,
        });
        Ok(())
    }

    /// Whether the stable can take one more foal, with those on the way already
    /// counted. A horse can make way for only one foal.
    pub(super) fn room_for_a_foal(&self, replace: &str) -> bool {
        let mut making_way = std::collections::HashSet::new();
        let in_stable = |name: &str| self.field.iter().any(|h| h.name == name);
        let needing_a_place = self
            .pregnancies
            .iter()
            .filter(|p| !(in_stable(&p.replace) && making_way.insert(p.replace.as_str())))
            .count();
        (in_stable(replace) && !making_way.contains(replace))
            || self.field.len() + needing_a_place < STABLE_SIZE
    }

    /// A week nearer for every foal on the way; those whose time has come join the stable.
    pub(super) fn deliver_foals(&mut self) {
        for p in &mut self.pregnancies {
            p.weeks_left = p.weeks_left.saturating_sub(1);
        }
        let (due, waiting) = std::mem::take(&mut self.pregnancies)
            .into_iter()
            .partition(|p| p.weeks_left == 0);
        self.pregnancies = waiting;
        let due: Vec<Pregnancy> = due;
        for p in due {
            self.deliver(p);
        }
    }

    /// A foal joins the stable, in the place of the horse making way for it or in a new one.
    pub(super) fn deliver(&mut self, p: Pregnancy) {
        let Pregnancy {
            foal,
            legs,
            sire,
            dam,
            replace,
            ..
        } = p;
        let foal_name = foal.name.clone();
        let mutated = if foal.mutations.is_empty() {
            ""
        } else {
            " ✦突然変異"
        };
        self.news.push(format!(
            "「{foal_name}」が誕生しました(父 {sire}・母 {dam})。{mutated}"
        ));
        if self.field.iter().any(|h| h.name == replace) {
            self.news
                .push(format!("{replace}は「{foal_name}」と入れ替わりました。"));
        }
        let farm = Away {
            kind: AwayKind::Growing,
            label: "牧場".to_string(),
            weeks: FOAL_FARM_WEEKS,
        };
        if let Some(sire) = foal.sire_id {
            self.sire_records.entry(sire).or_default().foals += 1;
        }
        // good pasture shows in a foal's legs and stamina
        let level = self.farm.pasture as u32;
        let legs = (legs + level * FARM_LEGS_PER_LEVEL).min(LEGS_MAX);
        let mut foal = foal;
        foal.stats.stamina = grown(foal.stats.stamina, (level * FARM_STAT_PER_LEVEL) as f64);
        self.place_foal(foal, legs, &replace, 0, Some(farm));
    }

    /// Puts a young horse in the stable, in the place of `replace` or in a new one.
    pub(super) fn place_foal(
        &mut self,
        foal: BloodHorse,
        legs: u32,
        replace: &str,
        age: u8,
        away: Option<Away>,
    ) {
        let foal_name = foal.name.clone();
        let stat = |v: f64| (v.round() as u32).clamp(FOAL_MIN_STAT, FOAL_MAX_STAT);
        let dirt = dirt_level(foal.stats.dirt);
        // a stable left entirely to itself goes on so with its foals too
        let some_horses = !self.field.is_empty();
        let left_alone = some_horses && self.field.iter().all(|h| h.auto_training);
        let run_alone = some_horses && self.field.iter().all(|h| h.auto_entry);
        let slot = self.field.iter().position(|h| h.name == replace);
        let trainer = match slot {
            Some(ri) => {
                let (wins, career) = (self.field[ri].record.wins, self.field[ri].career.clone());
                self.consider_for_the_hall(replace, wins, &career);
                self.forget(replace);
                self.field[ri].trainer
            }
            None => self.emptiest_stable(),
        };
        let horse = StableHorse {
            name: foal_name,
            speed: stat(foal.stats.speed),
            stamina: stat(foal.stats.stamina),
            power: stat(foal.stats.power),
            dirt,
            temper: temper_of(foal.stats.temper),
            sex: foal.gender,
            gelded: false,
            age,
            blood_id: foal.id,
            condition: self.roll_condition(),
            trainings: 0,
            dirt_progress: 0,
            jumper: false,
            jump: 0,
            jump_license: false,
            jump_wins: 0,
            year_jump_wins: 0,
            body_weight: ideal_weight(foal.id),
            last_race_weight: ideal_weight(foal.id),
            specials: foal.specials.clone(),
            record: RaceRecord::default(),
            career: Career::default(),
            gear: Gear::default(),
            fatigue: 0,
            resting: false,
            auto_training: left_alone,
            auto_entry: run_alone,
            trainer,
            since_race: NEVER_RACED,
            legs,
            away,
            jockey: None,
            partner: None,
        };
        self.bred.push(foal);
        match slot {
            Some(ri) => self.field[ri] = horse,
            None => self.field.push(horse),
        }
    }

    /// This year's lots at the yearling sale, while it is on.
    pub fn auction(&self) -> Vec<AuctionLot> {
        if !auction_open(self.week) {
            return vec![];
        }
        (1..=AUCTION_LOTS)
            .filter_map(|lot| self.auction_entry(lot))
            .map(|e| e.lot)
            .collect()
    }

    /// A lot of this year's sale, the same however often it is asked for.
    pub(super) fn auction_entry(&self, lot: u32) -> Option<AuctionEntry> {
        if !(1..=AUCTION_LOTS).contains(&lot) {
            return None;
        }
        let mut rng = rivals::Rng::new(rivals::mix_seed(&[
            self.world_seed,
            self.year as u64,
            lot as u64,
            0x5A1E,
        ]));
        let base = self.base_db();
        let mut pick = |sex: Sex| {
            let mut pool: Vec<&BloodHorse> = base
                .iter()
                .filter(|h| h.gen >= MARKET_MIN_GEN && h.gender == sex)
                .collect();
            pool.sort_by(|a, b| stat_sum(b).total_cmp(&stat_sum(a)).then(a.id.cmp(&b.id)));
            pool[rng.range(0, MARKET_POOL.min(pool.len()) as i64 - 1) as usize].clone()
        };
        let (sire, dam) = (pick(Sex::Male), pick(Sex::Female));
        let db = self.blood_map();
        let foal = bloodline::breed(
            &sire,
            &dam,
            sire.gen.max(dam.gen) + 1,
            self.next_blood_id(),
            String::new(),
            &db,
            &mut rng,
        );
        let estimate =
            stat_sum(&foal) + rng.range(-AUCTION_ESTIMATE_NOISE, AUCTION_ESTIMATE_NOISE) as f64;
        let grade = if estimate >= AUCTION_GRADE_A {
            "A"
        } else if estimate >= AUCTION_GRADE_B {
            "B"
        } else {
            "C"
        };
        let tens = |v: f64| ((v / 10.0).round() * 10.0) as u32;
        let start_price = tens(price_for_total(estimate) / 2.0);
        let spread = rng.range(70, 150) as f64 / 100.0;
        let rival_top = tens(price_for_total(stat_sum(&foal)) * spread);
        let drift = rng.range(-(FOAL_LEGS_SPREAD as i64), FOAL_LEGS_SPREAD as i64);
        let legs = ((legs_of_blood(sire.id) + legs_of_blood(dam.id)) as i64 / 2 + drift)
            .clamp(LEGS_MIN as i64, LEGS_MAX as i64) as u32;
        Some(AuctionEntry {
            lot: AuctionLot {
                lot,
                sire: self.blood_record(sire.id)?.name,
                dam: self.blood_record(dam.id)?.name,
                sex: foal.gender,
                estimate: grade.to_string(),
                start_price,
                closed: self.auction_closed.contains(&(self.year, lot)),
            },
            foal,
            legs,
            rival_top,
        })
    }

    /// Bids up to `max_bid` for a lot. The others go to their limit: the horse goes to
    /// whoever will pay more, for just over what the other would. Only one bid per lot.
    pub fn bid_at_auction(
        &mut self,
        lot: u32,
        max_bid: u32,
        name: &str,
    ) -> Result<BidOutcome, GameError> {
        if self.is_game_over() {
            return Err(GameError::GameOver);
        }
        if !auction_open(self.week) {
            return Err(GameError::SaleClosed);
        }
        let entry = self.auction_entry(lot).ok_or(GameError::UnknownLot)?;
        if entry.lot.closed {
            return Err(GameError::LotClosed);
        }
        if max_bid < entry.lot.start_price {
            return Err(GameError::BidTooLow);
        }
        if !self.room_for_a_foal("") {
            return Err(GameError::StableFull);
        }
        if !crate::horse::is_valid_name(name) {
            return Err(GameError::InvalidName);
        }
        if self.field.iter().any(|h| h.name == name)
            || self.studs.iter().any(|s| s.name == name)
            || self.name_in_database(name)
        {
            return Err(GameError::DuplicateName);
        }
        // the whole of the money would end the game
        if max_bid >= self.money {
            return Err(GameError::InsufficientFunds);
        }
        self.auction_closed.push((self.year, lot));
        let price = entry.lot.start_price.max(entry.rival_top + AUCTION_STEP);
        if max_bid < price {
            return Ok(BidOutcome {
                won: false,
                price: entry.rival_top,
            });
        }
        self.money -= price;
        let mut foal = entry.foal;
        foal.id = self.next_blood_id();
        foal.name = name.to_string();
        self.place_foal(foal, entry.legs, "", AUCTION_AGE, None);
        Ok(BidOutcome { won: true, price })
    }

    /// What mating this stallion and mare would bring: the foal's inbreeding, the
    /// temperament it would cost, and any good cross (ニック) of their lines.
    pub fn breeding_outlook(&self, sire: &str, dam: &str) -> Result<bloodline::Outlook, GameError> {
        let (sire_rec, sire_sex) = self.parent(sire)?;
        let (dam_rec, dam_sex) = self.parent(dam)?;
        if sire_sex != Sex::Male || dam_sex != Sex::Female {
            return Err(GameError::WrongSex);
        }
        let mut outlook = bloodline::outlook(&sire_rec, &dam_rec, &self.blood_map());
        if self.line_founded(&sire_rec) {
            outlook.theories.push(bloodline::Theory {
                name: "確立した系統".to_string(),
                points: LINE_THEORY_POINTS,
            });
            outlook.rating = bloodline::rating_for(outlook.theories.iter().map(|t| t.points).sum());
        }
        Ok(outlook)
    }
}
