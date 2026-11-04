//! The game as the screen sees it.

use super::*;

impl Game {
    pub fn state(&self) -> GameState {
        GameState {
            week: self.week,
            year: self.year,
            history: self.history.clone(),
            money: self.money,
            round: self.round,
            game_over: self.is_game_over(),
            breed_cost: BREED_COST,
            overseas: OVERSEAS
                .iter()
                .enumerate()
                .map(|(id, r)| OverseasEntry {
                    id,
                    name: r.name.to_string(),
                    country: r.country.to_string(),
                    week: r.week,
                    surface: r.surface,
                    distance: r.distance,
                    first_prize: (r.prize as f64 * PRIZE_RATE).round() as u32,
                    open: expedition_open(r, self.week),
                    candidates: (0..self.field.len())
                        .filter(|&i| self.may_go_abroad(i))
                        .map(|i| self.field[i].name.clone())
                        .collect(),
                })
                .collect(),
            expeditions: self
                .expeditions
                .iter()
                .map(|e| ExpeditionEntry {
                    horse: e.horse.clone(),
                    race: OVERSEAS[e.race].name.to_string(),
                    year: e.year,
                })
                .collect(),
            expedition_cost: EXPEDITION_COST,
            sire_lines: self.sire_lines(),
            line_foals: LINE_FOALS,
            line_g1_wins: LINE_G1_WINS,
            farm: FarmEntry {
                pasture: self.farm.pasture,
                nursery: self.farm.nursery,
                max_level: FARM_COSTS.len() as u8,
                pasture_cost: FARM_COSTS.get(self.farm.pasture as usize).copied(),
                nursery_cost: FARM_COSTS.get(self.farm.nursery as usize).copied(),
                upkeep_per_level: UPKEEP_PER_FACILITY_LEVEL,
            },
            owner_rank: self.owner_rank().to_string(),
            goals: self
                .goals_met()
                .into_iter()
                .map(|(name, description, met)| Goal {
                    name: name.to_string(),
                    description: description.to_string(),
                    done: met || self.goals_done.iter().any(|g| g == name),
                })
                .collect(),
            breeding_open: breeding_open(self.week),
            breeding_start_week: BREEDING_START_WEEK,
            breeding_end_week: BREEDING_END_WEEK,
            auction_open: auction_open(self.week),
            auction_start_week: AUCTION_START_WEEK,
            auction_end_week: AUCTION_END_WEEK,
            pregnancies: self
                .pregnancies
                .iter()
                .map(|p| PregnancyInfo {
                    foal_name: p.foal.name.clone(),
                    sire: p.sire.clone(),
                    dam: p.dam.clone(),
                    replace: p.replace.clone(),
                    weeks_left: p.weeks_left,
                })
                .collect(),
            field: self
                .field
                .iter()
                .enumerate()
                .map(|(i, h)| HorseEntry {
                    name: h.name.clone(),
                    speed: h.speed,
                    stamina: h.stamina,
                    train_cost: self.train_cost(&h.name).unwrap(),
                    condition: self.field[i].condition,
                    sex: self.field[i].sex,
                    age: self.field[i].age,
                    dirt: self.field[i].dirt,
                    power: self.field[i].power,
                    bad_track: bad_track_of(self.field[i].power, self.field[i].dirt),
                    temper: self.field[i].temper,
                    dirt_progress: self.field[i].dirt_progress,
                    record: self.field[i].record,
                    gear: self.field[i].gear,
                    fatigue: self.field[i].fatigue,
                    resting: self.field[i].resting,
                    auto_training: self.field[i].auto_training,
                    auto_entry: self.field[i].auto_entry,
                    trainer: self.field[i].trainer,
                    sale_price: self.sale_price(i),
                    career: self.field[i].career.clone(),
                    order: self.orders.get(&h.name).copied(),
                    jockey: self.field[i].jockey,
                    legs: self.field[i].legs,
                    away: self.field[i].away.clone(),
                    pedigree: self
                        .blood_record(self.field[i].blood_id)
                        .and_then(|r| self.pedigree_of_record(&r)),
                    blood: self.blood_record(self.field[i].blood_id).map(blood_traits),
                })
                .collect(),
            studs: self
                .studs
                .iter()
                .map(|s| StudEntry {
                    conception: self.conception_of(s),
                    career: s.career.clone(),
                    legs: s.legs,
                    name: s.name.clone(),
                    sex: s.sex,
                    speed: s.speed,
                    stamina: s.stamina,
                    power: s.power,
                    dirt: s.dirt,
                    temper: s.temper,
                    record: s.record,
                    retired_age: s.retired_age,
                    retired_year: s.retired_year,
                    bought: s.bought,
                    starter: s.starter,
                    pedigree: self
                        .blood_record(s.blood_id)
                        .and_then(|r| self.pedigree_of_record(&r)),
                    blood: self.blood_record(s.blood_id).map(blood_traits),
                })
                .collect(),
            news: self.news.clone(),
            max_studs: MAX_STUDS,
            stable_size: STABLE_SIZE,
            stars: self.stars.clone(),
            awards: self.awards.clone(),
            hall: self.hall.clone(),
            trainer_records: self.trainer_records.clone(),
            jockeys: JOCKEYS.iter().map(|j| j.info()).collect(),
            trainers: TRAINERS.iter().map(|t| t.info()).collect(),
            stable_capacity: STABLE_CAPACITY,
            weekly_upkeep: self.weekly_upkeep(),
            trainer_bonus_percent: ((trainer::DISTANCE_BONUS - 1.0) * 100.0).round() as u32,
            workouts: Workout::ALL.iter().map(|w| w.info()).collect(),
            dirt_sessions: DIRT_SESSIONS,
            race_fatigue_limit: RACE_FATIGUE_LIMIT,
            train_fatigue_limit: TRAIN_FATIGUE_LIMIT,
            auto_keep_money: self.auto_keep(),
            racers: self.racers(),
            successors: self.successors(),
            min_racers: MIN_RACERS,
            short_of_racers: self.racers() < MIN_RACERS,
            short_of_successors: self.successors() < MIN_RACERS,
        }
    }
}
