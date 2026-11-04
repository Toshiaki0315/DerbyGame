//! Rivals, goals, awards and the hall of fame.

use super::*;

impl Game {
    /// The owner's title, by what the stable has earned and won.
    pub(super) fn owner_rank(&self) -> &'static str {
        let (_, g1, earnings) = self.stable_totals();
        match (g1, earnings) {
            (g1, _) if g1 >= RANK_LEGEND_G1 => "伝説の馬主",
            (_, e) if e >= 100_000 => "名門馬主",
            (_, e) if e >= 30_000 => "有力馬主",
            (_, e) if e >= 10_000 => "中堅馬主",
            (_, e) if e >= 2_000 => "駆け出し馬主",
            _ => "新米馬主",
        }
    }

    /// Wins, G1 wins and earnings of every horse the stable has run.
    pub(super) fn stable_totals(&self) -> (u32, u32, u32) {
        self.trainer_records.iter().fold((0, 0, 0), |t, r| {
            (t.0 + r.wins, t.1 + r.g1_wins, t.2 + r.earnings)
        })
    }

    /// The goals an owner can aim for, each with whether the records now meet it.
    pub(super) fn goals_met(&self) -> Vec<(&'static str, &'static str, bool)> {
        let (wins, g1, earnings) = self.stable_totals();
        let careers = || {
            self.field
                .iter()
                .map(|h| &h.career)
                .chain(self.studs.iter().map(|s| &s.career))
        };
        let won_title = |title: &str| self.awards.iter().any(|a| a.mine && a.title == title);
        let home_bred = |id: u32| self.bred.iter().any(|h| h.id == id);
        let bred_g1 = self
            .field
            .iter()
            .map(|h| (h.blood_id, &h.career))
            .chain(self.studs.iter().map(|s| (s.blood_id, &s.career)))
            .any(|(id, c)| home_bred(id) && c.g1_wins() > 0);
        vec![
            ("初勝利", "どれかのレースで1勝する", wins >= 1),
            ("通算10勝", "厩舎全体で10勝する", wins >= 10),
            ("通算100勝", "厩舎全体で100勝する", wins >= 100),
            (
                "重賞初制覇",
                "重賞を勝つ",
                careers().any(|c| !c.stakes_wins.is_empty()),
            ),
            ("G1初制覇", "G1を勝つ", g1 >= 1),
            ("G1を10勝", "厩舎全体でG1を10勝する", g1 >= 10),
            ("賞金1万", "獲得賞金の合計を1万にする", earnings >= 10_000),
            (
                "賞金10万",
                "獲得賞金の合計を10万にする",
                earnings >= 100_000,
            ),
            ("自家生産馬でG1制覇", "自分で配合した馬でG1を勝つ", bred_g1),
            (
                "年度代表馬",
                "自分の馬が年度代表馬に選ばれる",
                won_title(TITLE_HORSE_OF_THE_YEAR),
            ),
            (
                "三冠馬",
                "三冠か牝馬三冠を達成する",
                won_title(TITLE_CROWN) || won_title(TITLE_FILLIES_CROWN),
            ),
            ("殿堂入り", "自分の馬が殿堂入りする", !self.hall.is_empty()),
            (
                "海外G1制覇",
                "海外の大レースを勝つ",
                self.awards
                    .iter()
                    .any(|a| a.mine && a.title.starts_with(OVERSEAS_AWARD)),
            ),
            (
                "系統の確立",
                "自分の種牡馬の系統を確立する",
                won_title(TITLE_LINE),
            ),
        ]
    }

    /// Announces the goals reached since last time; once reached a goal stays reached.
    pub(super) fn note_goals(&mut self) {
        for (name, _, met) in self.goals_met() {
            if met && !self.goals_done.iter().any(|g| g == name) {
                self.goals_done.push(name.to_string());
                self.news.push(format!("目標達成: {name}"));
            }
        }
    }

    /// The rivals of the first year, a few of each age.
    pub(super) fn first_stars(&self) -> Vec<Star> {
        let mut rng = rivals::Rng::new(rivals::mix_seed(&[self.world_seed, 0x57A1]));
        let mut taken: std::collections::HashSet<String> =
            self.field.iter().map(|h| h.name.clone()).collect();
        let mut stars = Vec::new();
        for age in STAR_FIRST_AGE..=STAR_LAST_START_AGE {
            for _ in 0..STARS_PER_YEAR {
                let star = rivals::new_star(&mut rng, age, &taken);
                taken.insert(star.name.clone());
                stars.push(star);
            }
        }
        stars
    }

    /// A week number that keeps counting from year to year.
    pub(super) fn absolute_week(&self) -> u32 {
        self.year * calendar::WEEKS_PER_YEAR as u32 + self.week as u32
    }

    /// The race a rival is aimed at this week: the richest graded race it could run
    /// in and is up to, given time since its last.
    pub(super) fn star_target(&self, star: &Star) -> Option<CalendarRace> {
        if self.absolute_week().saturating_sub(star.last_run) < STAR_GAP {
            return None;
        }
        let reach = star.speed as f64 * form::age_form(star.age, star.peak_age) + STAR_REACH;
        calendar::shared()
            .races
            .iter()
            .rev()
            .filter(|r| {
                r.week == self.week
                    && matches!(r.grade, Grade::G1 | Grade::G2 | Grade::G3)
                    && r.eligibility.allows(star.age, star.sex == Sex::Female)
                    && (star.apt_min..=star.apt_max).contains(&r.distance)
                    && (r.surface == Surface::Turf || star.dirt >= 0)
                    && rivals::speed_centre(r.grade) as f64 <= reach
            })
            .max_by_key(|r| r.prize)
            .cloned()
    }

    /// The rivals aimed at this race, the strongest if there are too many.
    pub(super) fn stars_in(&self, race: &CalendarRace) -> Vec<&Star> {
        let mut stars: Vec<&Star> = self
            .stars
            .iter()
            .filter(|s| {
                self.star_target(s)
                    .is_some_and(|r| race_place(&r) == race_place(race))
            })
            .collect();
        stars.sort_by(|a, b| b.speed.cmp(&a.speed).then(a.name.cmp(&b.name)));
        stars.truncate(STARS_PER_RACE);
        stars
    }

    /// The week's graded races the stable had no part in still have their rivals.
    pub(super) fn run_star_races(&mut self, done: &[(Day, Venue, u8)]) {
        let races: Vec<CalendarRace> = calendar::shared()
            .races
            .iter()
            .filter(|r| {
                r.week == self.week
                    && !done.contains(&race_place(r))
                    && !self.stars_in(r).is_empty()
            })
            .cloned()
            .collect();
        for race in races {
            let key = RaceKey::of(&race);
            let Ok((_, slot)) = self.race_and_slot(Some(&key)) else {
                continue;
            };
            self.entries.insert(slot, Vec::new());
            let Ok(spec) = self.race_spec(Some(&key)) else {
                continue;
            };
            let seed = self.race_seed(&race, 0x57A3);
            let results = Self::finish_order(&spec, seed);
            self.note_results(&spec, &results);
        }
    }

    /// Writes a calendar race into the careers of everyone it concerns.
    pub(super) fn note_results(&mut self, spec: &RaceSpec, results: &[RaceResult]) {
        let Some(race) = &spec.race else {
            return;
        };
        let now = self.absolute_week();
        let best_of_mine = results
            .iter()
            .position(|r| spec.entrants.iter().any(|e| e.mine && e.name == r.name));
        for (place, r) in results.iter().enumerate() {
            let won = place == 0;
            let g1 = won && race.grade == Grade::G1;
            let graded = won && matches!(race.grade, Grade::G1 | Grade::G2 | Grade::G3);
            let money = PRIZE_SHARES.get(place).map_or(0, |share| {
                (race.prize as f64 * PRIZE_RATE * share).round() as u32
            });
            if let Some(i) = self.field.iter().position(|h| h.name == r.name) {
                self.priority.retain(|n| n != &r.name);
                if place < PRIORITY_PLACES {
                    self.priority.push(r.name.clone());
                }
                if place < TRIAL_PLACES {
                    for g1 in trials_for(race) {
                        self.news
                            .push(format!("{}が{g1}の優先出走権を獲得しました。", r.name));
                        self.trial_rights.push(TrialRight {
                            horse: r.name.clone(),
                            year: self.year,
                            race: g1,
                        });
                    }
                }
                let t = self.field[i].trainer as usize;
                if let Some(rec) = self.trainer_records.get_mut(t) {
                    rec.starts += 1;
                    rec.wins += won as u32;
                    rec.g1_wins += g1 as u32;
                    rec.earnings += money;
                }
                let career = &mut self.field[i].career;
                career.earnings += money;
                career.year_earnings += money;
                career.year_g1 += g1 as u32;
                if graded {
                    career.stakes_wins.push(StakesWin {
                        year: self.year,
                        race: race.name.clone(),
                        grade: race.grade,
                    });
                }
                if g1 {
                    self.check_for_a_crown(i);
                    self.credit_the_sire(i);
                }
            } else if let Some(star) = self.stars.iter_mut().find(|s| s.name == r.name) {
                star.starts += 1;
                star.wins += won as u32;
                star.g1_wins += g1 as u32;
                star.year_g1 += g1 as u32;
                star.earnings += money;
                star.year_earnings += money;
                star.last_run = now;
                if let Some(best) = best_of_mine {
                    star.met += 1;
                    star.beaten += (best < place) as u32;
                }
                if g1 {
                    self.news
                        .push(format!("ライバルの{}が{}を制しました。", r.name, race.name));
                }
            }
        }
    }

    /// Has this G1 winner now taken all of a set of Classics this year?
    pub(super) fn check_for_a_crown(&mut self, i: usize) {
        for (races, title) in [
            (TRIPLE_CROWN, TITLE_CROWN),
            (FILLIES_CROWN, TITLE_FILLIES_CROWN),
        ] {
            let year = self.year;
            let won_all = races.iter().all(|name| {
                self.field[i]
                    .career
                    .stakes_wins
                    .iter()
                    .any(|w| w.year == year && w.race == *name)
            });
            if won_all && !self.field[i].career.titles.iter().any(|t| t == title) {
                let name = self.field[i].name.clone();
                self.field[i].career.titles.push(title.to_string());
                self.awards.push(Award {
                    year,
                    title: title.to_string(),
                    horse: name.clone(),
                    mine: true,
                });
                self.news.push(format!("{name}が{title}を達成しました!"));
            }
        }
    }

    /// The year's best horse, of the stable or the rivals: the most G1s, then the
    /// most money. Nobody wins it for doing nothing.
    pub(super) fn choose_horse_of_the_year(&mut self) {
        // the stable's best over fences
        let best_jumper = self
            .field
            .iter()
            .filter(|h| h.year_jump_wins > 0)
            .max_by_key(|h| h.year_jump_wins)
            .map(|h| h.name.clone());
        if let Some(horse) = best_jumper {
            self.awards.push(Award {
                year: self.year,
                title: TITLE_BEST_JUMPER.to_string(),
                horse,
                mine: true,
            });
        }
        for h in &mut self.field {
            h.year_jump_wins = 0;
        }
        let mine = self.field.iter().map(|h| {
            (
                (h.career.year_g1, h.career.year_earnings),
                h.name.clone(),
                true,
            )
        });
        let stars = self
            .stars
            .iter()
            .map(|s| ((s.year_g1, s.year_earnings), s.name.clone(), false));
        let best = mine
            .chain(stars)
            .filter(|(score, _, _)| score.1 > 0)
            .max_by_key(|(score, _, _)| *score);
        if let Some((_, horse, is_mine)) = best {
            self.awards.push(Award {
                year: self.year,
                title: TITLE_HORSE_OF_THE_YEAR.to_string(),
                horse: horse.clone(),
                mine: is_mine,
            });
            self.news.push(if is_mine {
                format!("{horse}が{}年目の年度代表馬に選ばれました!", self.year)
            } else {
                format!(
                    "ライバルの{horse}が{}年目の年度代表馬に選ばれました。",
                    self.year
                )
            });
            if is_mine {
                if let Some(i) = self.field.iter().position(|h| h.name == horse) {
                    self.field[i]
                        .career
                        .titles
                        .push(TITLE_HORSE_OF_THE_YEAR.to_string());
                }
            }
        }
        for h in &mut self.field {
            h.career.year_g1 = 0;
            h.career.year_earnings = 0;
        }
        for s in &mut self.stars {
            s.year_g1 = 0;
            s.year_earnings = 0;
        }
    }

    /// A new year: the rivals are a year older, the oldest stop, and new ones come along.
    pub(super) fn age_the_stars(&mut self) {
        for s in &mut self.stars {
            s.age += 1;
        }
        let mut gone = Vec::new();
        self.stars.retain(|s| {
            let stays = s.age < STAR_RETIRE_AGE;
            if !stays && s.g1_wins > 0 {
                gone.push(s.name.clone());
            }
            stays
        });
        for name in gone {
            self.news.push(format!("ライバルの{name}が引退しました。"));
        }
        let mut taken: std::collections::HashSet<String> =
            self.field.iter().map(|h| h.name.clone()).collect();
        taken.extend(self.stars.iter().map(|s| s.name.clone()));
        let mut rng = rivals::Rng::new(rivals::mix_seed(&[
            self.world_seed,
            self.year as u64,
            0x57A4,
        ]));
        for _ in 0..STARS_PER_YEAR {
            let star = rivals::new_star(&mut rng, STAR_FIRST_AGE, &taken);
            taken.insert(star.name.clone());
            self.stars.push(star);
        }
    }

    /// A great horse goes into the stable's hall of fame when it retires.
    pub(super) fn consider_for_the_hall(&mut self, name: &str, wins: u32, career: &Career) {
        if !career.for_the_hall() {
            return;
        }
        self.hall.push(HallEntry {
            name: name.to_string(),
            year: self.year,
            wins,
            g1_wins: career.g1_wins() as u32,
            earnings: career.earnings,
            titles: career.titles.clone(),
        });
        self.news.push(format!("{name}が殿堂入りしました!"));
    }
}
