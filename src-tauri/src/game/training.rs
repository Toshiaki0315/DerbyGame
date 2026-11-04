//! Work (調教) and what a horse's trainer proposes for it.

use super::*;

impl Game {
    pub fn train_cost(&self, name: &str) -> Option<u32> {
        let i = self.field.iter().position(|h| h.name == name)?;
        Some(session_cost(self.field[i].trainings))
    }

    /// One session of a training menu (調教) for a horse.
    pub fn workout(&mut self, name: &str, workout: Workout) -> Result<(), GameError> {
        if self.is_game_over() {
            return Err(GameError::GameOver);
        }
        let i = self.slot(name)?;
        if workout == Workout::Jump && !self.field[i].jumper {
            return Err(GameError::NotJumper);
        }
        let cost = session_cost(self.field[i].trainings);
        let gains = workout.gains();
        if self.field[i].away.is_some() {
            return Err(GameError::Away);
        }
        if self.field[i].resting {
            return Err(GameError::Resting);
        }
        if self.field[i].fatigue >= TRAIN_FATIGUE_LIMIT {
            return Err(GameError::TooTired);
        }
        if self.at_limit(i, gains[0].ability) {
            return Err(GameError::MaxedOut);
        }
        if cost > self.money {
            return Err(GameError::InsufficientFunds);
        }
        for gain in &gains {
            self.apply_gain(i, gain);
        }
        self.money -= cost;
        self.field[i].trainings += 1;
        self.field[i].body_weight = self.field[i]
            .body_weight
            .saturating_sub(workout.weight_loss());
        self.field[i].fatigue = (self.field[i].fatigue + workout.fatigue()).min(MAX_FATIGUE);
        self.roll_injury(i, WORKOUT_INJURY, 0xAC71);
        Ok(())
    }

    pub(super) fn at_limit(&self, i: usize, ability: Ability) -> bool {
        match ability {
            Ability::Speed => self.field[i].speed >= MAX_STAT,
            Ability::Stamina => self.field[i].stamina >= MAX_STAT,
            Ability::Power => self.field[i].power >= MAX_STAT,
            Ability::Temper => self.field[i].temper >= MAX_STAT,
            Ability::Dirt => self.field[i].dirt >= MAX_DIRT,
            Ability::Jump => self.field[i].jump >= self.jump_ceiling(i),
        }
    }

    pub(super) fn apply_gain(&mut self, i: usize, gain: &Gain) {
        // training never takes an ability past its limit, nor lowers one that is already over it
        let add = |v: u32| -> u32 {
            if gain.amount >= 0 {
                if v >= MAX_STAT {
                    v
                } else {
                    (v + gain.amount as u32).min(MAX_STAT)
                }
            } else {
                v.saturating_sub(gain.amount.unsigned_abs()).max(MIN_TEMPER)
            }
        };
        match gain.ability {
            Ability::Speed => self.field[i].speed = add(self.field[i].speed),
            Ability::Stamina => self.field[i].stamina = add(self.field[i].stamina),
            Ability::Power => self.field[i].power = add(self.field[i].power),
            Ability::Temper => self.field[i].temper = add(self.field[i].temper),
            Ability::Jump => {
                let ceiling = self.jump_ceiling(i);
                self.field[i].jump = (self.field[i].jump + gain.amount.max(0) as u32).min(ceiling);
            }
            Ability::Dirt => {
                if self.field[i].dirt < MAX_DIRT {
                    self.field[i].dirt_progress += gain.amount as u8;
                    if self.field[i].dirt_progress >= DIRT_SESSIONS {
                        self.field[i].dirt_progress = 0;
                        self.field[i].dirt += 1;
                    }
                }
            }
        }
    }

    /// What the horse's trainer would do with it this week: rest it if it is more
    /// tired than they allow, settle it if it is hot, otherwise bring on its
    /// weakest ability, leaning to what their distance calls for, as long as that
    /// leaves it as fresh as they want it.
    pub(super) fn auto_plan(&self, i: usize) -> Option<AutoPlan> {
        let t = self.trainer_of_horse(i);
        if self.field[i].away.is_some() {
            return None;
        }
        if self.field[i].fatigue >= AUTO_PASTURE_FATIGUE {
            return Some(AutoPlan::Pasture);
        }
        if self.field[i].fatigue >= t.training.rest_from() {
            return Some(AutoPlan::Rest);
        }
        if self.field[i].temper < AUTO_CALM_TEMPER {
            return Some(AutoPlan::Work(Workout::Walk));
        }
        // a jumper is schooled until it can run over fences
        if self.field[i].jumper
            && self.field[i].jump < JUMP_EXAM_LEVEL
            && !self.at_limit(i, Ability::Jump)
            && self.field[i].fatigue + Workout::Jump.fatigue() <= t.training.work_ceiling()
        {
            return Some(AutoPlan::Work(Workout::Jump));
        }
        if self.field[i].jumper
            && !self.field[i].jump_license
            && self.field[i].jump >= JUMP_EXAM_LEVEL
        {
            return None;
        }
        let (mut speed, mut stamina, power) = (
            self.field[i].speed as i64,
            self.field[i].stamina as i64,
            self.field[i].power as i64,
        );
        match t.distance {
            trainer::Distance::Sprint => speed -= AUTO_DISTANCE_BIAS,
            trainer::Distance::Stayer => stamina -= AUTO_DISTANCE_BIAS,
            _ => {}
        }
        let weakest = if speed <= stamina && speed <= power {
            Workout::Wood
        } else if power <= stamina {
            Workout::Hill
        } else {
            Workout::Pool
        };
        let limited = self.at_limit(i, weakest.gains()[0].ability);
        let fresh = self.field[i].fatigue + weakest.fatigue() <= t.training.work_ceiling();
        (!limited && fresh).then_some(AutoPlan::Work(weakest))
    }

    /// Carries out the week's お任せ training, keeping back `AUTO_KEEP_MONEY`.
    pub(super) fn run_auto_training(&mut self) {
        for i in 0..self.field.len() {
            if !self.field[i].auto_training {
                continue;
            }
            let name = self.field[i].name.clone();
            if self.field[i].jumper
                && !self.field[i].jump_license
                && self.field[i].jump >= JUMP_EXAM_LEVEL
                && JUMP_EXAM_COST + self.auto_keep() <= self.money
                && self.take_jump_exam(&name).is_ok()
            {
                self.news.push(format!("{name}が障害試験に合格しました。"));
            }
            for _ in 0..self.trainer_of_horse(i).training.sessions() {
                match self.auto_plan(i) {
                    Some(AutoPlan::Pasture) => {
                        if self.send_to_pasture(&name).is_ok() {
                            self.news.push(format!("{name}はお任せで放牧に出ました。"));
                        }
                        break;
                    }
                    Some(AutoPlan::Rest) => {
                        if self.rest(&name).is_ok() {
                            self.news.push(format!("{name}はお任せで休養しました。"));
                        }
                        break;
                    }
                    Some(AutoPlan::Work(workout)) => {
                        let cost = session_cost(self.field[i].trainings);
                        if cost + self.auto_keep() > self.money
                            || self.workout(&name, workout).is_err()
                        {
                            break;
                        }
                        self.news
                            .push(format!("{name}はお任せで{}をしました。", workout.name()));
                    }
                    None => break,
                }
            }
        }
    }

    /// The race this week the player has entered a horse for, if any.
    pub(super) fn entered_for(&self, i: usize) -> Option<String> {
        let name = &self.field[i].name;
        calendar::shared()
            .races
            .iter()
            .filter(|r| r.week == self.week)
            .find(|r| {
                let key = RaceKey::of(r);
                self.race_and_slot(Some(&key)).is_ok_and(|(_, slot)| {
                    self.entries
                        .get(&slot)
                        .is_some_and(|names| names.contains(name))
                })
            })
            .map(|r| r.name.clone())
    }

    /// What the trainer would do with a horse this week, if anything needs deciding.
    pub(super) fn proposal(&self, i: usize) -> Option<Proposal> {
        let plan = self.auto_plan(i);
        match plan {
            None => return None,
            Some(AutoPlan::Pasture) => return Some(Proposal::Pasture),
            Some(AutoPlan::Rest) => return Some(Proposal::Rest),
            Some(AutoPlan::Work(_)) => {}
        }
        let t = self.trainer_of_horse(i);
        let ready = self.field[i].since_race >= t.rotation.gap(self.field[i].condition);
        if ready && !self.saving_for_g1(i) {
            if let Some(race) = self.auto_race(i, &[], None) {
                if self.entered_for(i).is_none() {
                    return Some(Proposal::Enter(race));
                }
            }
        }
        match plan {
            Some(AutoPlan::Work(w)) => Some(Proposal::Work(w)),
            _ => None,
        }
    }

    pub(super) fn advice_for(&self, i: usize) -> Advice {
        let horse = self.field[i].name.clone();
        if let Some(away) = &self.field[i].away {
            let comment = match away.kind {
                AwayKind::Expedition => {
                    format!("海外に遠征中です。あと{}週で戻ります。", away.weeks)
                }
                AwayKind::Growing => format!(
                    "牧場で育っています。あと{}週で厩舎に入り、調教を始めます。",
                    away.weeks
                ),
                AwayKind::Pasture => {
                    format!("放牧でリフレッシュ中です。あと{}週で戻ります。", away.weeks)
                }
                AwayKind::Injured => format!(
                    "{}の治療中です。あと{}週かかります。",
                    away.label, away.weeks
                ),
            };
            return Advice {
                horse,
                comment,
                proposal: None,
            };
        }
        let t = self.trainer_of_horse(i);
        let (age, peak) = (self.field[i].age, self.stable_entrant(i).peak_age);
        let mut words: Vec<String> = vec![];
        if age < DEBUT_AGE {
            words.push("デビューは2歳になってからです。今は基礎を作ります。".to_string());
        }
        if age > peak + 1 {
            words.push("盛りを過ぎてきました。".to_string());
        } else if peak >= LATE_PEAK_AGE {
            words.push("晩成タイプで、まだまだ良くなります。".to_string());
        } else if peak <= EARLY_PEAK_AGE {
            words.push("早くから走れそうなタイプです。".to_string());
        }
        if self.field[i].fatigue >= AUTO_PASTURE_FATIGUE {
            words.push("疲れがたまっています。放牧で立て直しましょう。".to_string());
        } else if self.field[i].fatigue >= t.training.rest_from() {
            words.push("少し疲れが見えます。無理はさせません。".to_string());
        }
        if self.field[i].legs < WEAK_LEGS {
            words.push("脚元が弱いので気をつけています。".to_string());
        }
        let proposal = self.proposal(i);
        if let Some(race) = self.entered_for(i) {
            words.push(format!("{race}に登録しました。"));
        }
        match &proposal {
            Some(Proposal::Enter(race)) => {
                words.push(format!("今週は{}を狙いましょう。", race.name))
            }
            _ if age >= DEBUT_AGE && self.saving_for_g1(i) => {
                words.push("次のG1に向けて温存します。".to_string())
            }
            _ if age >= DEBUT_AGE
                && self.field[i].since_race < t.rotation.gap(self.field[i].condition) =>
            {
                words.push("前走の疲れを抜いて、次に備えます。".to_string())
            }
            _ => {}
        }
        if words.is_empty() {
            words.push("順調です。".to_string());
        }
        Advice {
            horse,
            comment: words.join(""),
            proposal: proposal.map(|p| p.label()),
        }
    }

    /// What each horse's trainer has to say.
    pub fn advice(&self) -> Vec<Advice> {
        (0..self.field.len()).map(|i| self.advice_for(i)).collect()
    }

    /// Does what the trainer proposes for a horse.
    pub fn accept_advice(&mut self, name: &str) -> Result<(), GameError> {
        let i = self.slot(name)?;
        match self.proposal(i) {
            None => Ok(()),
            Some(Proposal::Pasture) => self.send_to_pasture(name),
            Some(Proposal::Rest) => self.rest(name),
            Some(Proposal::Work(w)) => self.workout(name, w).map(|_| ()),
            Some(Proposal::Enter(race)) => {
                let key = RaceKey::of(&race);
                let (_, slot) = self.race_and_slot(Some(&key))?;
                let mut names = self.entries.get(&slot).cloned().unwrap_or_default();
                if !names.iter().any(|n| n == name) {
                    names.push(name.to_string());
                }
                self.set_entries(Some(&key), &names)
            }
        }
    }

    /// What お任せ training never spends the stable's money below.
    pub fn auto_keep(&self) -> u32 {
        AUTO_KEEP_MONEY.max(self.weekly_upkeep() * AUTO_KEEP_WEEKS)
    }
}
