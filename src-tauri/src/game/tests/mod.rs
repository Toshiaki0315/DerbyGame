//! Tests of the game, by subject. What several subjects share is here.

use super::*;

mod abroad;
mod basics;
mod betting;
mod bloodlines;
mod body;
mod breeding;
mod calendar_races;
mod commands;
mod courses;
mod farm;
mod gelding;
mod growth;
mod honours;
mod injuries;
mod jockeys;
mod jumps;
mod leading;
mod money;
mod prizes;
mod race_entries;
mod regressions;
mod retirement;
mod saves;
mod specials;
mod staff;
mod succession;
mod trainers;
mod training;

/// A game as a version 1 save has it: what is known about the horses in lists side by side.
fn old_save(g: &Game) -> serde_json::Value {
    const LISTS: [(&str, &str); 22] = [
        ("trainings", "trainings"),
        ("conditions", "condition"),
        ("sexes", "sex"),
        ("blood_ids", "blood_id"),
        ("ages", "age"),
        ("dirts", "dirt"),
        ("powers", "power"),
        ("tempers", "temper"),
        ("dirt_progress", "dirt_progress"),
        ("records", "record"),
        ("gear", "gear"),
        ("fatigue", "fatigue"),
        ("resting", "resting"),
        ("auto_training", "auto_training"),
        ("auto_entry", "auto_entry"),
        ("trainer_of", "trainer"),
        ("since_race", "since_race"),
        ("legs", "legs"),
        ("away", "away"),
        ("jockey_of", "jockey"),
        ("partner", "partner"),
        ("careers", "career"),
    ];
    let mut save = serde_json::to_value(g).unwrap();
    let horses = save["field"].as_array().unwrap().clone();
    for (list, key) in LISTS {
        save[list] = horses.iter().map(|h| h[key].clone()).collect();
    }
    save["field"] = horses
        .iter()
        .map(|h| serde_json::json!({"name": h["name"], "speed": h["speed"], "stamina": h["stamina"]}))
        .collect();
    save.as_object_mut().unwrap().remove("save_version");
    save
}

impl Game {
    /// One thing about every horse in the stable.
    fn all<T>(&self, what: impl Fn(&StableHorse) -> T) -> Vec<T> {
        self.field.iter().map(what).collect()
    }

    /// Mates as a player does, trying again at no charge until the mare takes,
    /// for tests about what follows a conception.
    fn cover(
        &mut self,
        sire: &str,
        dam: &str,
        replace: &str,
        foal_name: &str,
    ) -> Result<(), GameError> {
        loop {
            let (money, carrying) = (self.money, self.pregnancies.len());
            self.breed(sire, dam, replace, foal_name)?;
            if self.pregnancies.len() > carrying {
                return Ok(());
            }
            self.money = money;
        }
    }

    /// Mates any stallion and mare and has the foal at once, ready to race, for
    /// tests about what a foal is rather than when and how it comes.
    fn breed_now(
        &mut self,
        sire: &str,
        dam: &str,
        replace: &str,
        foal_name: &str,
    ) -> Result<(), GameError> {
        self.mate(sire, dam, replace, foal_name, 100)?;
        self.pregnancies.iter_mut().for_each(|p| p.weeks_left = 1);
        self.deliver_foals();
        let i = self.field.iter().position(|h| h.name == foal_name).unwrap();
        self.field[i].age = DEBUT_AGE;
        self.field[i].away = None;
        Ok(())
    }
}

/// A game in the breeding season, with a stallion and a mare at stud and a horse to replace.
fn in_season() -> (Game, String, String, String) {
    let mut g = Game::with_seed(1);
    g.week = BREEDING_START_WEEK;
    let at_stud = |sex: Sex, g: &Game| g.studs.iter().find(|h| h.sex == sex).unwrap().name.clone();
    let (s, d) = (at_stud(Sex::Male, &g), at_stud(Sex::Female, &g));
    let r = g.field[5].name.clone();
    (g, s, d, r)
}

fn weeks_pass(g: &mut Game, weeks: u8) {
    g.money = 1_000_000;
    for _ in 0..weeks {
        g.advance_week();
    }
}

/// A game with a foal just born, and its place in the stable.
fn with_a_newborn() -> (Game, usize) {
    let (mut g, s, d, _) = in_season();
    g.cover(&s, &d, "", "ハナミズキ").unwrap();
    weeks_pass(&mut g, GESTATION_WEEKS);
    let i = g.field.iter().position(|h| h.name == "ハナミズキ").unwrap();
    (g, i)
}

/// The results of a race with one horse finishing in a given place.
fn with_horse_placed(spec: &RaceSpec, name: &str, place: usize) -> Vec<RaceResult> {
    let mut results = Game::finish_order(spec, 1);
    let at = results.iter().position(|r| r.name == name).unwrap();
    let horse = results.remove(at);
    results.insert(place, horse);
    results
}

fn shared_base() -> Game {
    Game::with_seed(1)
}

/// A copy of `base` that will roll its random numbers from `seed`. Making a whole
/// new game for each seed means building a whole new blood database each time.
fn with_rng(base: &Game, seed: u64) -> Game {
    let mut g = base.clone();
    g.rng = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    g.roll_conditions();
    g
}

fn breed_default(g: &mut Game) -> Result<(), GameError> {
    let (sire, dam, replace) = (
        g.field[0].name.clone(),
        g.field[4].name.clone(),
        g.field[5].name.clone(),
    );
    g.breed_now(&sire, &dam, &replace, "ニューホープ")
}

fn calendar_race(name: &str) -> crate::calendar::CalendarRace {
    crate::calendar::shared()
        .races
        .iter()
        .find(|&r| r.name == name)
        .cloned()
        .unwrap_or_else(|| panic!("no race {name}"))
}

fn key_of(r: &crate::calendar::CalendarRace) -> RaceKey {
    RaceKey::of(r)
}

fn runner(name: &str, power: u32, dirt: i8) -> crate::rivals::Entrant {
    crate::rivals::Entrant {
        jump: 0,
        body_weight: 480,
        body_change: 0,
        body_form: 1.0,
        specials: vec![],
        rested: false,
        name: name.into(),
        speed: 78,
        stamina: 78,
        condition: Condition::Normal,
        sex: Sex::Male,
        age: 5,
        dirt,
        power,
        peak_age: 4,
        apt_min: 1000,
        apt_max: 3600,
        style: None,
        temper: 50,
        coat_color: "鹿毛".into(),
        gear: Gear::default(),
        fatigue: 0,
        trainer_distance: None,
        course_fit: None,
        jockey: None,
        weight: None,
        star: false,
        mine: false,
    }
}

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

fn finish(names: &[&str]) -> Vec<RaceResult> {
    names
        .iter()
        .enumerate()
        .map(|(i, n)| RaceResult {
            rank: i as u32 + 1,
            name: n.to_string(),
            finish_time: 60.0 + i as f64,
            margin: String::new(),
        })
        .collect()
}

/// A seed for which `name` finishes in the given place in this calendar race
/// and no other horse of the stable earns a prize.
fn seed_where(g: &Game, key: &RaceKey, name: &str, place: usize) -> u64 {
    let spec = g.race_spec(Some(key)).unwrap();
    let mine: Vec<&str> = spec
        .entrants
        .iter()
        .filter(|e| e.mine && e.name != name)
        .map(|e| e.name.as_str())
        .collect();
    (0..5000u64)
        .find(|s| {
            let results = Game::run_race(&spec, *s).1;
            results.get(place - 1).is_some_and(|r| r.name == name)
                && results
                    .iter()
                    .take(PRIZE_SHARES.len())
                    .all(|r| !mine.contains(&r.name.as_str()))
        })
        .unwrap_or_else(|| panic!("{name} never finishes {place}"))
}

fn my_runner(g: &Game, key: &RaceKey) -> String {
    g.race_card(Some(key))
        .unwrap()
        .entrants
        .iter()
        .find(|e| e.mine)
        .unwrap()
        .name
        .clone()
}

fn a_rival_in(g: &Game, key: &RaceKey) -> String {
    g.race_card(Some(key))
        .unwrap()
        .entrants
        .iter()
        .find(|e| !e.mine)
        .unwrap()
        .name
        .clone()
}

fn blood(g: &Game, i: usize) -> crate::bloodline::BloodHorse {
    g.blood_record(g.field[i].blood_id)
        .expect("a linked horse has a record")
}

/// A database record with the given id and parents, for building family trees in tests.
fn rec(id: u32, sire: Option<u32>, dam: Option<u32>, sex: Sex) -> crate::bloodline::BloodHorse {
    let mut h = crate::bloodline::founders()[0].clone();
    h.id = id;
    h.sire_id = sire;
    h.dam_id = dam;
    h.gender = sex;
    h.gen = 10;
    h.lineage = "None".into();
    h
}

fn fit_runner(age: u8, peak_age: u8, apt: (u32, u32)) -> crate::rivals::Entrant {
    let mut e = runner("F", 60, 0);
    e.age = age;
    e.peak_age = peak_age;
    e.apt_min = apt.0;
    e.apt_max = apt.1;
    e
}

fn stallion_and_mare(g: &Game) -> (String, String) {
    let s = (0..g.field.len())
        .find(|&i| g.field[i].sex == Sex::Male)
        .unwrap();
    let d = (0..g.field.len())
        .find(|&i| g.field[i].sex == Sex::Female)
        .unwrap();
    (g.field[s].name.clone(), g.field[d].name.clone())
}

fn abilities(g: &Game, i: usize) -> (u32, u32, u32, u32) {
    (
        g.field[i].speed,
        g.field[i].stamina,
        g.field[i].power,
        g.field[i].temper,
    )
}

fn mid_horse() -> Game {
    let mut g = Game::new();
    g.money = 100_000;
    g.field[0].speed = 60;
    g.field[0].stamina = 60;
    g.field[0].power = 60;
    g.field[0].temper = 50;
    g
}

fn rich() -> Game {
    let mut g = Game::new();
    g.money = 100_000;
    g
}

/// A horse with middling stats, left to the trainer.
fn auto_horse() -> (Game, String) {
    let mut g = mid_horse();
    g.field[0].trainer = trainer::STANDARD;
    let name = g.field[0].name.clone();
    g.set_auto_training(&name, true).unwrap();
    (g, name)
}

/// A four-year-old with middling stats, entered automatically, in a week
/// that offers it races of more than one class.
fn auto_runner() -> (Game, String) {
    let mut g = mid_horse();
    g.field[0].age = 4;
    g.field[0].condition = Condition::Normal;
    g.field[0].record = RaceRecord::default();
    g.field[0].trainer = trainer::STANDARD;
    let name = g.field[0].name.clone();
    g.set_auto_entry(&name, true).unwrap();
    for week in 1..=calendar::WEEKS_PER_YEAR {
        g.week = week;
        let classes: std::collections::HashSet<i64> = g
            .auto_choices(0, None)
            .iter()
            .map(|r| rivals::speed_centre(r.grade))
            .collect();
        if classes.len() >= 2 {
            return (g, name);
        }
    }
    panic!("no week has a choice of races");
}

fn race_fees(g: &Game, key: &RaceKey) -> u32 {
    g.race_spec(Some(key))
        .unwrap()
        .entrants
        .iter()
        .filter_map(|e| e.jockey.as_ref().map(|j| j.fee))
        .sum()
}

/// A stable horse with the same abilities everywhere.
fn priced(stat: u32, age: u8, wins: u32) -> u32 {
    let mut g = Game::new();
    g.field[0].speed = stat;
    g.field[0].stamina = stat;
    g.field[0].power = stat;
    g.field[0].age = age;
    g.field[0].record = RaceRecord { starts: wins, wins };
    g.sale_price(0)
}

/// Breeds until every stable is full.
fn fill_stables(g: &mut Game, sire: &str, dam: &str) {
    const KANA: [char; 10] = ['ア', 'イ', 'ウ', 'エ', 'オ', 'カ', 'キ', 'ク', 'ケ', 'コ'];
    g.money = 10_000_000;
    let mut n = 0;
    while g.field.len() < STABLE_SIZE {
        let name = format!("ハナ{}{}", KANA[n / 10], KANA[n % 10]);
        g.breed_now(sire, dam, "", &name).unwrap();
        n += 1;
    }
}

/// The horse in the first place, in the hands of a trainer. It has no family
/// record, so any distance suits it.
fn with_trainer(id: u8) -> Game {
    let mut g = mid_horse();
    g.field[0].age = 4;
    g.field[0].condition = Condition::Normal;
    g.field[0].record = RaceRecord::default();
    g.field[0].blood_id = u32::MAX;
    g.field[0].trainer = id;
    g.field[0].auto_entry = true;
    g
}

fn away(kind: AwayKind, label: &str) -> Option<Away> {
    Some(Away {
        kind,
        label: label.to_string(),
        weeks: 3,
    })
}
