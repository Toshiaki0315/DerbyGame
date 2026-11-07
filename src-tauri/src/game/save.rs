//! Saving and loading a game, including saves from older versions.

use super::*;

/// The format saves are written in. Version 1 (a save with no version) kept what was
/// known about the horses in lists side by side; each horse now holds its own. Version 3
/// adds their weight and character (固有特性), which older saves take from the blood.
pub(super) const SAVE_VERSION: u32 = 3;

/// What a version 1 save keeps about its horses. The lists it may not have yet, which
/// are older than the save, are made up from the horses.
#[derive(Deserialize)]
struct ListsOfVersion1 {
    field: Vec<Horse>,
    trainings: Vec<u32>,
    #[serde(default)]
    conditions: Vec<Condition>,
    #[serde(default)]
    sexes: Vec<Sex>,
    #[serde(default)]
    blood_ids: Vec<u32>,
    #[serde(default)]
    ages: Vec<u8>,
    #[serde(default)]
    dirts: Vec<i8>,
    #[serde(default)]
    powers: Vec<u32>,
    #[serde(default)]
    tempers: Vec<u32>,
    #[serde(default)]
    dirt_progress: Vec<u8>,
    #[serde(default)]
    records: Vec<RaceRecord>,
    #[serde(default)]
    gear: Vec<Gear>,
    #[serde(default)]
    fatigue: Vec<u32>,
    #[serde(default)]
    resting: Vec<bool>,
    #[serde(default)]
    auto_training: Vec<bool>,
    #[serde(default)]
    auto_entry: Vec<bool>,
    #[serde(default)]
    trainer_of: Vec<u8>,
    #[serde(default)]
    since_race: Vec<u32>,
    #[serde(default)]
    legs: Vec<u32>,
    #[serde(default)]
    away: Vec<Option<Away>>,
    #[serde(default)]
    jockey_of: Vec<Option<u8>>,
    #[serde(default)]
    partner: Vec<Option<(u8, u32)>>,
    #[serde(default)]
    careers: Vec<Career>,
}

/// One list of a version 1 save: as it is if it is there, and otherwise made up.
/// A list with the wrong number of entries spoils the save.
fn list_for<T>(
    horses: usize,
    name: &str,
    list: Vec<T>,
    make_up: impl FnOnce() -> Vec<T>,
) -> Result<Vec<T>, String> {
    let list = if list.is_empty() { make_up() } else { list };
    if list.len() == horses {
        Ok(list)
    } else {
        Err(format!(
            "セーブデータが不正です: {name} の数({})が馬の数({horses})と一致しません",
            list.len()
        ))
    }
}

impl ListsOfVersion1 {
    /// Puts each horse together from the lists. A list only the game can make up (blood
    /// records and what comes from them) is left to `Game::take_version_1_horses`.
    fn into_horses(self, world_seed: u64) -> Result<Vec<StableHorse>, String> {
        let n = self.field.len();
        let names: Vec<String> = self.field.iter().map(|h| h.name.clone()).collect();
        let powers = list_for(n, "powers", self.powers, || {
            names.iter().map(|h| default_power(h)).collect()
        })?;
        let ages = list_for(n, "ages", self.ages, || {
            names.iter().map(|h| default_age(h)).collect()
        })?;
        let dirts = list_for(n, "dirts", self.dirts, || {
            names.iter().map(|h| default_dirt(h)).collect()
        })?;
        let sexes = list_for(n, "sexes", self.sexes, || {
            names.iter().map(|h| default_sex(h)).collect()
        })?;
        let conditions = list_for(n, "conditions", self.conditions, || {
            vec![Condition::Normal; n]
        })?;
        let trainings = list_for(n, "trainings", self.trainings, Vec::new)?;
        let blood_ids = list_for(n, "blood_ids", self.blood_ids, || vec![0; n])?;
        let tempers = list_for(n, "tempers", self.tempers, || vec![0; n])?;
        let dirt_progress = list_for(n, "dirt_progress", self.dirt_progress, || vec![0; n])?;
        let records = list_for(n, "records", self.records, || {
            names.iter().map(|h| default_record(h)).collect()
        })?;
        let gear = list_for(n, "gear", self.gear, || vec![Gear::default(); n])?;
        let fatigue = list_for(n, "fatigue", self.fatigue, || vec![0; n])?;
        let resting = list_for(n, "resting", self.resting, || vec![false; n])?;
        let auto_training = list_for(n, "auto_training", self.auto_training, || vec![false; n])?;
        let auto_entry = list_for(n, "auto_entry", self.auto_entry, || vec![false; n])?;
        let trainers = list_for(n, "trainer_of", self.trainer_of, || {
            starting_trainers(world_seed, n)
        })?;
        let since_race = list_for(n, "since_race", self.since_race, || vec![NEVER_RACED; n])?;
        let legs = list_for(n, "legs", self.legs, || vec![0; n])?;
        let away = list_for(n, "away", self.away, || vec![None; n])?;
        let jockeys = list_for(n, "jockey_of", self.jockey_of, || vec![None; n])?;
        let partners = list_for(n, "partner", self.partner, || vec![None; n])?;
        let careers = list_for(n, "careers", self.careers, || vec![Career::default(); n])?;

        Ok((0..n)
            .map(|i| StableHorse {
                name: self.field[i].name.clone(),
                speed: self.field[i].speed,
                stamina: self.field[i].stamina,
                power: powers[i],
                dirt: dirts[i],
                temper: tempers[i],
                sex: sexes[i],
                gelded: false,
                age: ages[i],
                blood_id: blood_ids[i],
                condition: conditions[i],
                trainings: trainings[i],
                dirt_progress: dirt_progress[i],
                jumper: false,
                jump: 0,
                jump_license: false,
                jump_wins: 0,
                year_jump_wins: 0,
                body_weight: 0,
                last_race_weight: 0,
                specials: vec![],
                record: records[i],
                career: careers[i].clone(),
                gear: gear[i],
                fatigue: fatigue[i],
                resting: resting[i],
                auto_training: auto_training[i],
                auto_entry: auto_entry[i],
                trainer: trainers[i],
                since_race: since_race[i],
                legs: legs[i],
                away: away[i].clone(),
                jockey: jockeys[i],
                partner: partners[i],
            })
            .collect())
    }
}

impl Game {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("game state is serializable")
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        let bad = |e: serde_json::Error| format!("セーブデータが不正です: {e}");
        let mut save: serde_json::Value = serde_json::from_str(json).map_err(bad)?;
        let version = save
            .get("save_version")
            .and_then(|v| v.as_u64())
            .unwrap_or(1);
        if version > SAVE_VERSION as u64 {
            return Err("新しいバージョンのゲームで作られたセーブデータです".into());
        }
        // the horses of an old save are dealt with once the rest of the game is there
        let old_lists = if version == 1 {
            let lists: ListsOfVersion1 = serde_json::from_value(save.clone()).map_err(bad)?;
            save["field"] = serde_json::json!([]);
            Some(lists)
        } else {
            None
        };
        let mut game: Game = serde_json::from_value(save).map_err(bad)?;
        game.save_version = SAVE_VERSION;
        if game.rng == 0 {
            game.rng = DEFAULT_SEED;
        }
        if game.world_seed == 0 {
            game.world_seed = DEFAULT_SEED;
        }
        if game.week == 0 || game.week > calendar::WEEKS_PER_YEAR {
            game.week = 1;
        }
        if let Some(lists) = old_lists {
            game.take_version_1_horses(lists)?;
        }
        if version < 3 {
            game.set_bodies_and_specials();
        }
        let unknown_blood = |id: u32| game.blood_record(id).is_none();
        if game.field.iter().any(|h| unknown_blood(h.blood_id)) {
            return Err("セーブデータが不正です: 馬と血統データベースの対応が一致しません".into());
        }
        if game.studs.iter().any(|s| unknown_blood(s.blood_id)) {
            return Err(
                "セーブデータが不正です: 繁殖馬と血統データベースの対応が一致しません".into(),
            );
        }
        // a save may be from before there were trainers to come
        game.trainer_records
            .resize(trainer::ALL_TRAINERS, TrainerRecord::default());
        if game.jockey_staff.is_empty() || game.trainer_staff.is_empty() {
            game.first_staff();
        }
        if game.stars.is_empty() {
            game.stars = game.first_stars();
        }
        for i in 0..game.studs.len() {
            let id = game.studs[i].blood_id;
            if game.studs[i].legs == 0 {
                game.studs[i].legs = legs_of_blood(id);
            }
            if game.studs[i].temper == 0 {
                game.studs[i].temper = game
                    .blood_record(id)
                    .map_or(50, |r| temper_of(r.stats.temper));
            }
        }
        if game
            .field
            .iter()
            .any(|h| h.trainer as usize >= trainer::ALL_TRAINERS)
        {
            return Err("セーブデータが不正です: いない調教師の厩舎に預けられた馬がいます".into());
        }
        if game.expeditions.iter().any(|e| e.race >= OVERSEAS.len()) {
            return Err("セーブデータが不正です: ない海外レースへの遠征があります".into());
        }
        Ok(game)
    }

    /// Brings in the horses of a version 1 save, and works out what it has no list for.
    fn take_version_1_horses(&mut self, lists: ListsOfVersion1) -> Result<(), String> {
        let no_blood = lists.blood_ids.is_empty();
        let no_tempers = lists.tempers.is_empty();
        let no_legs = lists.legs.is_empty();
        self.field = lists.into_horses(self.world_seed)?;
        if no_blood {
            self.link_blood();
        }
        if no_tempers {
            self.set_tempers_from_records();
        }
        if no_legs {
            self.set_legs_from_blood();
        }
        Ok(())
    }

    pub fn save_to(&self, path: &std::path::Path) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("セーブ先フォルダを作成できません: {e}"))?;
        }
        // written to a scratch file first and then put in place, so a save that cannot be
        // finished leaves the last one as it was
        let mut scratch = path.as_os_str().to_owned();
        scratch.push(".tmp");
        let scratch = std::path::PathBuf::from(scratch);
        let written = std::fs::write(&scratch, self.to_json())
            .and_then(|()| std::fs::rename(&scratch, path));
        written.map_err(|e| {
            if scratch.is_file() {
                std::fs::remove_file(&scratch).ok();
            }
            format!("セーブファイルを書き込めません: {e}")
        })
    }

    pub fn load_from(path: &std::path::Path) -> Result<Self, String> {
        let json =
            std::fs::read_to_string(path).map_err(|_| "セーブデータがありません".to_string())?;
        Self::from_json(&json)
    }
}
