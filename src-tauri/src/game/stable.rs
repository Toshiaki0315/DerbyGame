//! The horses in the stable: looking after them, moving them on, what they cost.

use super::*;

impl Game {
    /// Takes a horse out of the racing stable, to stud or wherever it is going.
    pub(super) fn remove_slot(&mut self, i: usize) -> Stud {
        let horse = self.field.remove(i);
        self.forget(&horse.name);
        Stud {
            specials: horse.specials,
            career: horse.career,
            legs: horse.legs,
            record: horse.record,
            temper: horse.temper,
            sex: horse.sex,
            speed: horse.speed,
            stamina: horse.stamina,
            power: horse.power,
            dirt: horse.dirt,
            blood_id: horse.blood_id,
            retired_age: horse.age,
            retired_year: self.year,
            bought: false,
            starter: false,
            name: horse.name,
        }
    }

    /// Whatever is kept by name for a horse that has left the stable.
    pub(super) fn forget(&mut self, name: &str) {
        self.withdraw(name);
        self.orders.remove(name);
        self.priority.retain(|n| n != name);
        self.trial_rights.retain(|t| t.horse != name);
        self.expeditions.retain(|e| e.horse != name);
    }

    /// Takes a horse out of every race it is entered in this week.
    pub(super) fn withdraw(&mut self, name: &str) {
        for names in self.entries.values_mut() {
            names.retain(|n| n != name);
        }
    }

    /// Where a horse is in the stable.
    pub(super) fn slot(&self, name: &str) -> Result<usize, GameError> {
        self.field
            .iter()
            .position(|h| h.name == name)
            .ok_or(GameError::UnknownHorse)
    }

    /// Money coming in. The purse has a limit, even if no stable is likely to reach it.
    pub(super) fn earn(&mut self, amount: i64) {
        let total = self.money as i64 + amount;
        self.money = total.clamp(0, u32::MAX as i64) as u32;
    }

    /// A bill that never finishes a stable off.
    pub(super) fn charge(&mut self, bill: u32) {
        self.money -= bill.min(self.money.saturating_sub(1));
    }

    /// Adds to the history, which only goes back so far.
    pub(super) fn remember(&mut self, entry: HistoryEntry) {
        self.history.push(entry);
        if self.history.len() > HISTORY_LIMIT {
            self.history.remove(0);
        }
    }

    /// Rests a horse for the week (休養): it sheds fatigue at once, but may not
    /// race or be worked until the week is over.
    pub fn rest(&mut self, name: &str) -> Result<(), GameError> {
        let i = self.slot(name)?;
        if self.field[i].away.is_some() {
            return Err(GameError::Away);
        }
        if self.field[i].resting {
            return Err(GameError::Resting);
        }
        self.field[i].resting = true;
        self.field[i].fatigue = self.field[i].fatigue.saturating_sub(REST_RECOVERY);
        self.withdraw(name);
        Ok(())
    }

    /// Sets the equipment a horse races in (メンコ, シャドーロール); it stays until changed.
    pub fn set_gear(&mut self, name: &str, gear: Gear) -> Result<(), GameError> {
        let i = self.slot(name)?;
        self.field[i].gear = gear;
        Ok(())
    }

    /// Retires a racehorse to stud (繁殖入り), where it can be bred from.
    pub fn retire(&mut self, name: &str) -> Result<(), GameError> {
        let i = self.slot(name)?;
        if self.field.len() == 1 {
            return Err(GameError::LastHorse);
        }
        if self.field[i].age < DEBUT_AGE {
            return Err(GameError::TooYoung);
        }
        if self.field[i].gelded {
            let gone = self.remove_slot(i);
            self.consider_for_the_hall(&gone.name, gone.record.wins, &gone.career);
            return Ok(());
        }
        if self.studs.len() >= MAX_STUDS {
            return Err(GameError::StudFull);
        }
        let stud = self.remove_slot(i);
        self.consider_for_the_hall(&stud.name, stud.record.wins, &stud.career);
        self.studs.push(stud);
        Ok(())
    }

    /// At the turn of the year, horses old enough retire; a full stud book lets
    /// its longest-serving stud go to make room.
    pub(super) fn retire_the_old(&mut self) {
        while let Some(i) = self.field.iter().position(|h| h.age >= RETIRE_AGE) {
            if self.field[i].gelded {
                let gone = self.remove_slot(i);
                self.news.push(format!("{}が引退しました。", gone.name));
                self.consider_for_the_hall(&gone.name, gone.record.wins, &gone.career);
                continue;
            }
            if self.studs.len() >= MAX_STUDS {
                let gone = self.studs.remove(0);
                self.news
                    .push(format!("{}は繁殖を引退しました。", gone.name));
                self.blood_names.entry(gone.blood_id).or_insert(gone.name);
            }
            let stud = self.remove_slot(i);
            self.news
                .push(format!("{}が引退し、繁殖入りしました。", stud.name));
            self.consider_for_the_hall(&stud.name, stud.record.wins, &stud.career);
            self.studs.push(stud);
        }
    }

    /// Gelds a colt: it calms down, and has some weeks at pasture getting over it.
    pub fn geld(&mut self, name: &str) -> Result<(), GameError> {
        let i = self.slot(name)?;
        if self.field[i].sex != Sex::Male {
            return Err(GameError::WrongSex);
        }
        if self.field[i].gelded {
            return Err(GameError::AlreadyGelded);
        }
        if self.field[i].away.is_some() {
            return Err(GameError::Away);
        }
        // the whole of the money would end the game
        if GELDING_COST >= self.money {
            return Err(GameError::InsufficientFunds);
        }
        self.money -= GELDING_COST;
        let h = &mut self.field[i];
        h.gelded = true;
        h.temper = (h.temper + GELDING_CALM).min(100);
        h.resting = false;
        h.away = Some(Away {
            kind: AwayKind::Pasture,
            label: "去勢後の放牧".to_string(),
            weeks: GELDING_WEEKS,
        });
        self.withdraw(name);
        Ok(())
    }

    /// Sends a horse over to jumps (障害転向). It may still run on the flat.
    pub fn go_over_to_jumps(&mut self, name: &str) -> Result<(), GameError> {
        let i = self.slot(name)?;
        if self.field[i].jumper {
            return Err(GameError::AlreadyJumper);
        }
        if self.field[i].age < JUMP_MIN_AGE {
            return Err(GameError::TooYoung);
        }
        let start = self.jump_in_blood(i) * JUMP_START_SHARE;
        self.field[i].jumper = true;
        self.field[i].jump = (start.round() as u32).clamp(1, JUMP_EXAM_LEVEL - 1);
        Ok(())
    }

    /// The jump exam (障害試験): a horse that passes may run over fences.
    pub fn take_jump_exam(&mut self, name: &str) -> Result<(), GameError> {
        let i = self.slot(name)?;
        if !self.field[i].jumper {
            return Err(GameError::NotJumper);
        }
        if self.field[i].jump_license {
            return Err(GameError::AlreadyLicensed);
        }
        if self.field[i].jump < JUMP_EXAM_LEVEL {
            return Err(GameError::JumpTooLow);
        }
        // the whole of the money would end the game
        if JUMP_EXAM_COST >= self.money {
            return Err(GameError::InsufficientFunds);
        }
        self.money -= JUMP_EXAM_COST;
        self.field[i].jump_license = true;
        Ok(())
    }

    fn jump_in_blood(&self, i: usize) -> f64 {
        self.blood_record(self.field[i].blood_id)
            .map_or(bloodline::NEUTRAL_JUMP, |r| r.jump)
    }

    /// How good at fences schooling can make a horse: every one can pass the exam, and
    /// what is in its blood takes it further.
    pub(super) fn jump_ceiling(&self, i: usize) -> u32 {
        let reach = JUMP_EXAM_LEVEL as f64 + self.jump_in_blood(i) * JUMP_CEILING_SHARE;
        (reach.round() as u32).min(100)
    }

    /// The horses old enough to race, away for now or not.
    pub(super) fn racers(&self) -> usize {
        self.field.iter().filter(|h| h.age >= DEBUT_AGE).count()
    }

    /// The horses that will still be racing when a foal mated now would first run,
    /// foals on the way among them.
    pub(super) fn successors(&self) -> usize {
        let staying = self
            .field
            .iter()
            .filter(|h| h.age + FOAL_LEAD_YEARS < RETIRE_AGE)
            .count();
        staying + self.pregnancies.len()
    }

    /// As the breeding season opens, says so if the stable will run out of horses.
    pub(super) fn warn_of_succession(&mut self) {
        let successors = self.successors();
        if self.week == BREEDING_START_WEEK && successors < MIN_RACERS {
            self.news.push(format!(
                "後継の馬が{successors}頭しかいません。種付けかセリで用意しないと、{FOAL_LEAD_YEARS}年後には出走できる馬が足りなくなります。"
            ));
        }
    }

    /// Leaves a horse's weekly training to the trainer (お任せ), or takes it back.
    pub fn set_auto_training(&mut self, name: &str, on: bool) -> Result<(), GameError> {
        let i = self.slot(name)?;
        self.field[i].auto_training = on;
        Ok(())
    }

    /// The legs of a parent: one of the stable's, one at stud, or one on the market.
    pub(super) fn parent_legs(&self, name: &str) -> u32 {
        if let Some(i) = self.field.iter().position(|h| h.name == name) {
            return self.field[i].legs;
        }
        if let Some(s) = self.studs.iter().find(|s| s.name == name) {
            return s.legs;
        }
        self.stud_market()
            .into_iter()
            .find(|o| o.name == name)
            .map_or(LEGS_MIN, |o| o.legs)
    }

    /// Whether a horse breaks down after work or a race; the chance depends on how
    /// tired it is and on its legs. Decided apart from the game's other luck.
    pub(super) fn roll_injury(&mut self, i: usize, base: f64, salt: u64) {
        if self.field[i].away.is_some() {
            return;
        }
        let mut rng = rivals::Rng::new(rivals::mix_seed(&[
            self.world_seed,
            self.year as u64,
            self.week as u64,
            self.field[i].blood_id as u64,
            self.field[i].trainings as u64,
            self.field[i].record.starts as u64,
            salt,
        ]));
        let roll = rng.next_u64() as f64 / u64::MAX as f64;
        let base = if self.field[i].specials.contains(&Special::Iron) {
            base * 0.5
        } else {
            base
        };
        if roll < injury_chance(base, self.field[i].fatigue, self.field[i].legs) {
            self.injure(i, Injury::from_roll(rng.next_u64() % 100));
        }
    }

    /// The stable's runners in a race, now tired from it, may have broken down.
    pub(super) fn roll_race_injuries(&mut self, spec: &RaceSpec) {
        for e in spec.entrants.iter().filter(|e| e.mine) {
            if let Some(i) = self.field.iter().position(|h| h.name == e.name) {
                self.roll_injury(i, RACE_INJURY, 0x4ACE);
            }
        }
    }

    /// A horse breaks down: out for a while, a vet's bill, and maybe lasting harm.
    /// The bill never finishes a stable off.
    pub(super) fn injure(&mut self, i: usize, injury: Injury) {
        let name = self.field[i].name.clone();
        self.field[i].away = Some(Away {
            kind: AwayKind::Injured,
            label: injury.name().to_string(),
            weeks: injury.weeks(),
        });
        self.field[i].resting = false;
        self.field[i].speed = self.field[i].speed.saturating_sub(injury.speed_loss());
        self.charge(injury.vet_fee());
        self.withdraw(&name);
        self.news.push(format!(
            "{name}が故障しました({}、復帰まで{}週)。",
            injury.name(),
            injury.weeks()
        ));
    }

    /// Sends a horse to pasture (放牧): it can't race or work, but gets over its
    /// fatigue quickly, costs less, and comes back in good form.
    pub fn send_to_pasture(&mut self, name: &str) -> Result<(), GameError> {
        let i = self.slot(name)?;
        if self.field[i].away.is_some() {
            return Err(GameError::Away);
        }
        self.field[i].away = Some(Away {
            kind: AwayKind::Pasture,
            label: "放牧".to_string(),
            weeks: PASTURE_WEEKS,
        });
        self.field[i].resting = false;
        self.withdraw(name);
        Ok(())
    }

    /// Brings a horse back from pasture early (帰厩).
    pub fn recall(&mut self, name: &str) -> Result<(), GameError> {
        let i = self.slot(name)?;
        if !self.field[i]
            .away
            .as_ref()
            .is_some_and(|a| a.kind == AwayKind::Pasture)
        {
            return Err(GameError::NotOnPasture);
        }
        self.field[i].away = None;
        Ok(())
    }

    /// A week has passed for the horses away; those whose time is up come back in good form.
    pub(super) fn bring_back_horses(&mut self) {
        for h in &mut self.field {
            let Some(away) = &mut h.away else {
                continue;
            };
            away.weeks = away.weeks.saturating_sub(1);
            if away.weeks > 0 {
                continue;
            }
            let word = match away.kind {
                AwayKind::Pasture => "放牧から戻りました",
                AwayKind::Injured => "故障から復帰しました",
                AwayKind::Growing => "牧場から厩舎に入り、調教を始めました",
                AwayKind::Expedition => "遠征から帰国しました",
            };
            self.news.push(format!("{}が{word}。", h.name));
            // a good nursery brings a yearling on
            if away.kind == AwayKind::Growing {
                let bonus = self.farm.nursery as u32 * FARM_STAT_PER_LEVEL;
                h.speed = grown(h.speed as f64, bonus as f64) as u32;
                h.power = grown(h.power as f64, bonus as f64) as u32;
            }
            h.away = None;
            h.condition = Condition::Good;
        }
    }

    /// What the stable pays each week for its horses (預託料).
    pub fn weekly_upkeep(&self) -> u32 {
        let away = self.field.iter().filter(|h| h.away.is_some()).count() as u32;
        (self.field.len() as u32 - away) * UPKEEP_PER_HORSE
            + away * (UPKEEP_PER_HORSE / 2)
            + self.studs.len() as u32 * UPKEEP_PER_STUD
            + (self.farm.pasture + self.farm.nursery) as u32 * UPKEEP_PER_FACILITY_LEVEL
    }

    /// Pays the week's bill. A stable that cannot pay it and still have money
    /// left lets its cheapest horses go; with none to spare, it pays what it can.
    pub(super) fn pay_upkeep(&mut self) {
        if self.is_game_over() {
            return;
        }
        while self.weekly_upkeep() >= self.money && self.field.len() > 1 {
            let Some(cheapest) = (0..self.field.len())
                .filter(|&i| !self.abroad(i))
                .min_by_key(|&i| self.sale_price(i))
            else {
                break;
            };
            let name = self.field[cheapest].name.clone();
            let price = self.sale_price(cheapest);
            if self.sell_horse(&name).is_err() {
                break;
            }
            self.news
                .push(format!("預託料が払えず、{name}を{price}で手放しました。"));
        }
        let bill = self.weekly_upkeep();
        if bill >= self.money {
            self.news
                .push("預託料が払えず、資金が尽きました。".to_string());
        }
        self.money = self.money.saturating_sub(bill);
    }

    /// Whether a horse is away racing abroad.
    pub(super) fn abroad(&self, i: usize) -> bool {
        self.field[i]
            .away
            .as_ref()
            .is_some_and(|a| a.kind == AwayKind::Expedition)
    }

    /// What a horse would sell for: its ability and its wins, less as it ages.
    pub fn sale_price(&self, i: usize) -> u32 {
        let stats = self.field[i].speed + self.field[i].stamina + self.field[i].power;
        let value = stats.saturating_sub(SALE_FREE_STATS) * SALE_PER_STAT
            + self.field[i].record.wins * SALE_PER_WIN;
        let past_prime = self.field[i].age.saturating_sub(SALE_FULL_AGE) as f64;
        let kept = (1.0 - 0.2 * past_prime).max(0.0);
        let price = SALE_MIN as f64 + value as f64 * kept;
        let injured = self.field[i]
            .away
            .as_ref()
            .is_some_and(|a| a.kind == AwayKind::Injured);
        let price = if injured { price / 2.0 } else { price };
        (((price / 10.0).floor() * 10.0) as u32).max(SALE_MIN)
    }

    /// Sells a horse out of the stable.
    pub fn sell_horse(&mut self, name: &str) -> Result<(), GameError> {
        if self.is_game_over() {
            return Err(GameError::GameOver);
        }
        let i = self.slot(name)?;
        if self.field.len() == 1 {
            return Err(GameError::LastHorse);
        }
        // its expedition is paid for and under way
        if self.abroad(i) {
            return Err(GameError::Away);
        }
        self.earn(self.sale_price(i) as i64);
        self.remove_slot(i);
        Ok(())
    }

    /// Sends a horse to another trainer's stable, if it has room.
    pub fn set_trainer(&mut self, name: &str, id: u8) -> Result<(), GameError> {
        let i = self.slot(name)?;
        if self.trainers_now().iter().all(|t| t.id != id) {
            return Err(GameError::UnknownTrainer);
        }
        if self.field[i].trainer != id && self.stable_count(id) >= STABLE_CAPACITY {
            return Err(GameError::TrainerFull);
        }
        self.field[i].trainer = id;
        Ok(())
    }

    pub(super) fn stable_count(&self, id: u8) -> usize {
        self.field.iter().filter(|h| h.trainer == id).count()
    }

    /// The stable with the fewest horses, where a foal goes.
    pub(super) fn emptiest_stable(&self) -> u8 {
        self.trainers_now()
            .iter()
            .min_by_key(|t| self.stable_count(t.id))
            .map_or(0, |t| t.id)
    }

    pub(super) fn trainer_of_horse(&self, i: usize) -> &'static trainer::Trainer {
        trainer::trainer_by_id(self.field[i].trainer)
    }

    /// Leaves every horse's training and racing to its trainer, or takes them all back.
    pub fn set_all_auto(&mut self, on: bool) {
        for h in &mut self.field {
            h.auto_training = on;
            h.auto_entry = on;
        }
    }

    /// Enters a horse in a suitable race each week (自動出走), or takes it back.
    pub fn set_auto_entry(&mut self, name: &str, on: bool) -> Result<(), GameError> {
        let i = self.slot(name)?;
        self.field[i].auto_entry = on;
        Ok(())
    }

    /// Tells a horse's rider how to run it (作戦), or (none) leaves that to them.
    pub fn set_order(&mut self, name: &str, order: Option<Style>) -> Result<(), GameError> {
        if self.field.iter().all(|h| h.name != name) {
            return Err(GameError::UnknownHorse);
        }
        match order {
            Some(style) => self.orders.insert(name.to_string(), style),
            None => self.orders.remove(name),
        };
        Ok(())
    }

    /// Chooses a horse's jockey, or (none) leaves it to the trainer.
    pub fn set_jockey(&mut self, name: &str, id: Option<u8>) -> Result<(), GameError> {
        let i = self.slot(name)?;
        if id.is_some_and(|id| self.jockeys_now().iter().all(|j| j.id != id)) {
            return Err(GameError::UnknownJockey);
        }
        self.field[i].jockey = id;
        Ok(())
    }

    /// The stable's runners in a race come out of it tired.
    pub(super) fn tire_runners(&mut self, spec: &RaceSpec, tiring: u32) {
        for e in spec.entrants.iter().filter(|e| e.mine) {
            if let Some(i) = self.field.iter().position(|h| h.name == e.name) {
                let weight = self.field[i].body_weight;
                self.field[i].last_race_weight = weight;
                self.field[i].body_weight = weight.saturating_sub(RACE_WEIGHT_LOSS);
                self.field[i].fatigue = (self.field[i].fatigue + tiring).min(MAX_FATIGUE);
            }
        }
    }
}
