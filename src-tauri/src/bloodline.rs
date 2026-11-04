//! The master pedigree: a family tree of horses generated generation by
//! generation from a handful of founders, with inherited coat colour, stats,
//! growth type, distance aptitude, running style and temperament, plus
//! inbreeding and "nick" (good cross) effects.
//!
//! A Rust port of the pedigree-generation script. The whole database is
//! rebuilt from a seed, so it never has to be saved.

use crate::game::{GameStore, Sex};
use crate::rivals::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Population average each stat regresses toward.
const GLOBAL_AVG: f64 = 50.0;
/// Share of the parents' average replaced by the population average.
const REGRESSION_RATE: f64 = 0.2;
/// How far back (counted from the sire or dam) ancestors are looked for.
const MAX_ANCESTOR_DEPTH: u8 = 4;
/// Upper limit of the inbreeding coefficient, as a safety net.
const INBREED_CAP: f64 = 0.5;
/// Gentle penalty below this coefficient, steeper above it (a 3x4 cross under the old additive sums).
const INBREED_REFERENCE: f64 = 0.1875;
/// Most temperament points inbreeding can cost.
const INBREED_MAX_PENALTY: f64 = 30.0;
/// Temperament never falls below this, so no horse is left at exactly zero.
const MIN_TEMPER: f64 = 1.0;
/// The usual ceiling for a stat.
const NORMAL_CAP: f64 = 100.0;
/// Chance that any one stat of a foal mutates.
const MUTATION_CHANCE: f64 = 0.01;
/// A mutation adds this many points (low, high)...
const MUTATION_BONUS: (f64, f64) = (15.0, 25.0);
/// ...and may carry a stat past 100, but never past this.
const MUTATION_CAP: f64 = 125.0;
/// Foals born in each generation, from generation 2 on.
const TARGET_COUNTS: [usize; 9] = [10, 20, 35, 60, 95, 140, 200, 260, 320];

#[cfg(test)]
pub const RUNNING_STYLES: [&str; 4] = ["逃げ", "先行", "差し", "追込"];

/// A good cross: this sire lineage on this dam lineage adds to one stat.
const NICKS: [(&str, &str, StatKey, f64); 2] = [
    ("Ignis", "Aqua", StatKey::Speed, 7.0),
    ("Terra", "Ignis", StatKey::Power, 5.0),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatKey {
    Speed,
    Stamina,
    Temper,
    Power,
    Turf,
    Dirt,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    pub speed: f64,
    pub stamina: f64,
    /// Higher is calmer.
    pub temper: f64,
    pub power: f64,
    pub turf: f64,
    pub dirt: f64,
}

impl StatKey {
    /// The name used in JSON and in a horse's list of mutations.
    fn name(self) -> &'static str {
        match self {
            StatKey::Speed => "speed",
            StatKey::Stamina => "stamina",
            StatKey::Temper => "temper",
            StatKey::Power => "power",
            StatKey::Turf => "turf",
            StatKey::Dirt => "dirt",
        }
    }
}

impl Stats {
    fn get(&self, k: StatKey) -> f64 {
        match k {
            StatKey::Speed => self.speed,
            StatKey::Stamina => self.stamina,
            StatKey::Temper => self.temper,
            StatKey::Power => self.power,
            StatKey::Turf => self.turf,
            StatKey::Dirt => self.dirt,
        }
    }

    fn set(&mut self, k: StatKey, v: f64) {
        match k {
            StatKey::Speed => self.speed = v,
            StatKey::Stamina => self.stamina = v,
            StatKey::Temper => self.temper = v,
            StatKey::Power => self.power = v,
            StatKey::Turf => self.turf = v,
            StatKey::Dirt => self.dirt = v,
        }
    }

    fn rounded(mut self) -> Self {
        for k in [
            StatKey::Speed,
            StatKey::Stamina,
            StatKey::Temper,
            StatKey::Power,
            StatKey::Turf,
            StatKey::Dirt,
        ] {
            self.set(k, (self.get(k) * 10.0).round() / 10.0);
        }
        self
    }
}

/// Coat-colour genes: grey (G), extension (E) and agouti (A), two alleles each, sorted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Genes {
    #[serde(rename = "G")]
    pub g: String,
    #[serde(rename = "E")]
    pub e: String,
    #[serde(rename = "A")]
    pub a: String,
}

impl Genes {
    fn new(g: &str, e: &str, a: &str) -> Self {
        Genes {
            g: g.into(),
            e: e.into(),
            a: a.into(),
        }
    }
}

/// What a record from before jumping was kept has in it over fences.
pub const NEUTRAL_JUMP: f64 = 50.0;

fn neutral_jump() -> f64 {
    NEUTRAL_JUMP
}

/// A foal's jumping ability, taken from its parents. It has its own luck, so that the
/// rest of the database comes out the same as it did before jumping was kept.
fn inherited_jump(sire: &BloodHorse, dam: &BloodHorse, id: u32, stats: &Stats) -> f64 {
    let mut rng = Rng::new(mix(&[
        sire.id as u64,
        dam.id as u64,
        id as u64,
        stats.speed.to_bits(),
    ]));
    let (jump, _) = base_stat_with(sire.jump, dam.jump, 8.0, 0.0, &mut rng);
    (jump * 10.0).round() / 10.0
}

/// What a founder has in it over fences.
fn founder_jump(id: u32) -> f64 {
    (20 + mix(&[id as u64, 0x1A4B]) % 61) as f64
}

fn mix(parts: &[u64]) -> u64 {
    parts.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, p| {
        (h ^ p).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BloodHorse {
    pub id: u32,
    pub name: String,
    pub sire_id: Option<u32>,
    pub dam_id: Option<u32>,
    pub gender: Sex,
    /// Sire line (系統), passed down the male line.
    pub lineage: String,
    /// Dam family (牝系).
    pub family: String,
    pub gen: u8,
    pub coat_color: String,
    pub genes: Genes,
    pub growth_factor: f64,
    pub growth_type: String,
    pub peak_age: u8,
    /// What it has in it over fences (飛越), 0-100.
    #[serde(default = "neutral_jump")]
    pub jump: f64,
    /// Best distance range in metres.
    pub aptitude_min: u32,
    pub aptitude_max: u32,
    pub running_style: String,
    pub personality: String,
    /// Strength of inbreeding (0 = none).
    pub inbreed_vol: f64,
    pub stats: Stats,
    /// Stats that mutated (突然変異), by name: they may exceed 100, up to 125.
    #[serde(default)]
    pub mutations: Vec<String>,
    /// One of the player's own horses (or a retired one), as opposed to the wider database.
    #[serde(default)]
    pub owned: bool,
}

// ---------------------------------------------------------------- genes & coat

/// Each parent passes on one random allele per locus.
fn inherit_genes(sire: &Genes, dam: &Genes, rng: &mut Rng) -> Genes {
    let mut pick = |s: &str, d: &str| -> String {
        let s: Vec<char> = s.chars().collect();
        let d: Vec<char> = d.chars().collect();
        let mut pair = [*rng.choice(&s), *rng.choice(&d)];
        pair.sort();
        pair.iter().collect()
    };
    Genes {
        g: pick(&sire.g, &dam.g),
        e: pick(&sire.e, &dam.e),
        a: pick(&sire.a, &dam.a),
    }
}

/// Grey is dominant; without it, "ee" is chestnut; otherwise agouti decides bay or black.
fn coat_color(genes: &Genes) -> &'static str {
    if genes.g.contains('G') {
        "芦毛"
    } else if genes.e == "ee" {
        "栗毛"
    } else if genes.a.contains('A') {
        "鹿毛"
    } else {
        "青毛"
    }
}

// ----------------------------------------------------- stats, growth, aptitude

/// Child stat: the parents' average pulled toward the population average, plus
/// noise, and a 1% chance of a mutation. Returns the stat and whether it mutated.
#[cfg(test)]
fn base_stat(sire: f64, dam: f64, variance: f64, rng: &mut Rng) -> (f64, bool) {
    base_stat_with(sire, dam, variance, MUTATION_CHANCE, rng)
}

/// [`base_stat`] with the mutation chance given, so a test can force or forbid it.
///
/// The ordinary result is kept within 0-100. A mutation then adds 15-25 points,
/// which can carry the stat past 100, up to a final cap of 125.
fn base_stat_with(
    sire: f64,
    dam: f64,
    variance: f64,
    mutation_chance: f64,
    rng: &mut Rng,
) -> (f64, bool) {
    let base = (sire + dam) / 2.0;
    let expected = base * (1.0 - REGRESSION_RATE) + GLOBAL_AVG * REGRESSION_RATE;
    let stat = rng.gauss(expected, variance).clamp(0.0, NORMAL_CAP);
    if rng.chance(mutation_chance) {
        let bonus = rng.range_f64(MUTATION_BONUS.0, MUTATION_BONUS.1);
        return ((stat + bonus).min(MUTATION_CAP), true);
    }
    (stat, false)
}

/// (type, lowest and highest peak age) for a growth factor.
fn growth_class(factor: f64) -> (&'static str, u8, u8) {
    if factor < 0.35 {
        ("早熟", 2, 3)
    } else if factor < 0.70 {
        ("普通", 4, 5)
    } else {
        ("晩成", 6, 7)
    }
}

fn growth(sire_factor: f64, dam_factor: f64, rng: &mut Rng) -> (f64, &'static str, u8) {
    let factor = rng
        .gauss((sire_factor + dam_factor) / 2.0, 0.15)
        .clamp(0.0, 1.0);
    let (kind, lo, hi) = growth_class(factor);
    (factor, kind, rng.range(lo as i64, hi as i64) as u8)
}

/// Rounds to the nearest 200m, ties to even (as Python's `round` does).
fn round_to_200(metres: f64) -> i64 {
    (metres / 200.0).round_ties_even() as i64 * 200
}

/// Best distance range: a stamina-heavy horse stays longer, a quick-tempered one gives out sooner.
pub fn aptitude_distance(speed: f64, stamina: f64, temper: f64) -> (u32, u32) {
    let total = speed + stamina;
    if total == 0.0 {
        return (1200, 1200);
    }
    let ratio = (stamina / total).clamp(0.2, 0.8);
    let center = 1000.0 + ((ratio - 0.2) / 0.6) * 2400.0;
    let width = 400.0 + stamina * 6.0;

    let mut min_dist = round_to_200(center - width / 2.0);
    let mut max_dist = round_to_200(center + width / 2.0);
    if temper < 50.0 {
        let penalty_units = (((50.0 - temper) / 50.0) * 2.0) as i64; // truncates
        max_dist -= penalty_units * 200;
    }
    min_dist = min_dist.max(1000);
    max_dist = max_dist.min(3600).max(min_dist + 200);
    (min_dist as u32, max_dist as u32)
}

fn personality(temper: f64, rng: &mut Rng) -> &'static str {
    let pool: &[&'static str] = if temper < 35.0 {
        &["狂気", "気分屋", "臆病", "頑固"]
    } else if temper < 65.0 {
        &["負けず嫌い", "やんちゃ", "普通", "臆病"]
    } else {
        &["冷静", "優等生", "勇敢", "素直"]
    };
    rng.choice(pool)
}

/// 60% of the time a parent's style is inherited; otherwise it shows from the horse's own build.
fn running_style(
    sire_style: &str,
    dam_style: &str,
    speed: f64,
    stamina: f64,
    temper: f64,
    rng: &mut Rng,
) -> String {
    if rng.chance(0.60) {
        return (*rng.choice(&[sire_style, dam_style])).to_string();
    }
    let style = if temper < 40.0 && speed > 60.0 {
        "逃げ" // hot-blooded and quick: goes to the front
    } else if temper >= 60.0 && speed < stamina {
        *rng.choice(&["差し", "追込"]) // calm, with stamina to spare: saves its run
    } else {
        *rng.choice(&["先行", "差し"])
    };
    style.to_string()
}

// ------------------------------------------------------------------ inbreeding

/// Every ancestor within four generations, with how many generations back it is
/// (parents are 1); the shortest route wins.
pub fn ancestors(horse_id: u32, db: &HashMap<u32, BloodHorse>) -> HashMap<u32, u8> {
    fn trace(id: u32, depth: u8, db: &HashMap<u32, BloodHorse>, out: &mut HashMap<u32, u8>) {
        if depth > MAX_ANCESTOR_DEPTH {
            return;
        }
        let Some(h) = db.get(&id) else { return };
        for parent in [h.sire_id, h.dam_id].into_iter().flatten() {
            let d = out.entry(parent).or_insert(depth);
            *d = (*d).min(depth);
            trace(parent, depth + 1, db, out);
        }
    }
    let mut out = HashMap::new();
    trace(horse_id, 1, db, &mut out);
    out
}

/// Inbreeding coefficient of a foal of `sire_id` and `dam_id` (Wright's coefficient).
///
/// Each ancestor the two share contributes `(1/2)^(n1 + n2 + 1)`, where `n1`
/// and `n2` are how many generations the ancestor is behind the sire and
/// behind the dam. Counted from the foal instead, that is `(1/2)^(g1 + g2 - 1)`
/// (a great-grandparent on both sides, 3 and 3, gives 0.5^5 = 3.125%).
/// Full siblings come out at 0.25, half siblings 0.125, first cousins 0.0625.
///
/// Only the shortest route to each ancestor is counted, and the sire and dam
/// themselves are not treated as their own ancestors.
pub fn inbreed_volume(sire_id: u32, dam_id: u32, db: &HashMap<u32, BloodHorse>) -> f64 {
    let (s, d) = (ancestors(sire_id, db), ancestors(dam_id, db));
    let total: f64 = s
        .iter()
        .filter_map(|(id, &sire_depth)| {
            let dam_depth = *d.get(id)?;
            Some(0.5_f64.powi(sire_depth as i32 + dam_depth as i32 + 1))
        })
        // not sum(): an empty sum is -0.0 in Rust, and adding 0.0 to it is optimized away
        .fold(0.0, |sum, part| sum + part);
    total.min(INBREED_CAP)
}

/// Temperament points lost to inbreeding: a gentle straight line up to the
/// reference coefficient (10 points there), then squared growth, never more than 30.
pub fn inbreed_penalty(blood_vol: f64) -> f64 {
    if blood_vol > INBREED_REFERENCE {
        (10.0 * (blood_vol / INBREED_REFERENCE).powi(2)).min(INBREED_MAX_PENALTY)
    } else if blood_vol > 0.0 {
        (10.0 * (blood_vol / INBREED_REFERENCE)).min(10.0)
    } else {
        0.0
    }
}

// ---------------------------------------------------------------------- mating

/// One foal of `sire` and `dam`.
pub fn breed(
    sire: &BloodHorse,
    dam: &BloodHorse,
    gen: u8,
    id: u32,
    name: String,
    db: &HashMap<u32, BloodHorse>,
    rng: &mut Rng,
) -> BloodHorse {
    breed_with(sire, dam, gen, id, name, db, MUTATION_CHANCE, rng)
}

/// [`breed`] with the chance of a mutation per stat given.
#[allow(clippy::too_many_arguments)]
pub fn breed_with(
    sire: &BloodHorse,
    dam: &BloodHorse,
    gen: u8,
    id: u32,
    name: String,
    db: &HashMap<u32, BloodHorse>,
    mutation_chance: f64,
    rng: &mut Rng,
) -> BloodHorse {
    let blood_vol = inbreed_volume(sire.id, dam.id, db);
    // inbreeding spreads the stats out a little, and costs temperament
    let variance = 5.0 + blood_vol * 15.0;
    let penalty = inbreed_penalty(blood_vol);

    let mut mutated: Vec<StatKey> = Vec::new();
    let mut stats = sire.stats; // overwritten key by key below
    for k in [
        StatKey::Speed,
        StatKey::Stamina,
        StatKey::Power,
        StatKey::Turf,
        StatKey::Dirt,
    ] {
        let (v, m) = base_stat_with(
            sire.stats.get(k),
            dam.stats.get(k),
            variance,
            mutation_chance,
            rng,
        );
        stats.set(k, v);
        if m {
            mutated.push(k);
        }
    }
    let (raw_temper, temper_mutated) = base_stat_with(
        sire.stats.temper,
        dam.stats.temper,
        variance,
        mutation_chance,
        rng,
    );
    if temper_mutated {
        mutated.push(StatKey::Temper);
    }
    // a mutated stat may go past 100 whatever else adds to it
    let cap = |k: StatKey| {
        if mutated.contains(&k) {
            MUTATION_CAP
        } else {
            NORMAL_CAP
        }
    };
    // power carries over to dirt
    if stats.power > 60.0 {
        stats.dirt = (stats.dirt + (stats.power - 60.0) * 0.5).min(cap(StatKey::Dirt));
    }
    stats.temper = (raw_temper - penalty).max(MIN_TEMPER);

    if let Some((key, bonus)) = nick_for(&sire.lineage, &dam.lineage) {
        stats.set(key, (stats.get(key) + bonus).min(cap(key)));
    }
    // listed in a fixed order, whatever order they were rolled in
    let mutations: Vec<String> = [
        StatKey::Speed,
        StatKey::Stamina,
        StatKey::Temper,
        StatKey::Power,
        StatKey::Turf,
        StatKey::Dirt,
    ]
    .into_iter()
    .filter(|k| mutated.contains(k))
    .map(|k| k.name().to_string())
    .collect();

    let (dist_min, dist_max) = aptitude_distance(stats.speed, stats.stamina, stats.temper);
    let (growth_factor, growth_type, peak_age) = growth(sire.growth_factor, dam.growth_factor, rng);
    let genes = inherit_genes(&sire.genes, &dam.genes, rng);
    let personality = personality(stats.temper, rng).to_string();
    let style = running_style(
        &sire.running_style,
        &dam.running_style,
        stats.speed,
        stats.stamina,
        stats.temper,
        rng,
    );
    let gender = if rng.chance(0.5) {
        Sex::Male
    } else {
        Sex::Female
    };

    BloodHorse {
        id,
        name,
        sire_id: Some(sire.id),
        dam_id: Some(dam.id),
        gender,
        lineage: sire.lineage.clone(),
        family: dam.family.clone(),
        gen,
        coat_color: coat_color(&genes).to_string(),
        genes,
        growth_factor,
        growth_type: growth_type.to_string(),
        peak_age,
        jump: inherited_jump(sire, dam, id, &stats),
        aptitude_min: dist_min,
        aptitude_max: dist_max,
        running_style: style,
        personality,
        inbreed_vol: (blood_vol * 10000.0).round() / 10000.0,
        stats: stats.rounded(),
        mutations,
        owned: false,
    }
}

// ------------------------------------------------------------------ simulation

/// The good cross for these two lines, if there is one: the stat it lifts and by how much.
fn nick_for(sire_lineage: &str, dam_lineage: &str) -> Option<(StatKey, f64)> {
    NICKS
        .iter()
        .find(|(s, d, _, _)| *s == sire_lineage && *d == dam_lineage)
        .map(|&(_, _, key, bonus)| (key, bonus))
}

/// A good cross (ニック) between a sire line and a dam line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Nick {
    /// The stat it lifts, by name.
    pub stat: String,
    pub bonus: f64,
}

/// What a mating would bring, before it is made.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Outlook {
    /// Inbreeding coefficient of the foal (0 = none).
    pub inbreed_vol: f64,
    /// Temperament points the inbreeding would cost.
    pub penalty: f64,
    pub nick: Option<Nick>,
    /// What the mating theories say of the pair (配合理論), and the rating they come to.
    pub theories: Vec<Theory>,
    pub rating: String,
    /// How the foal is likely to grow up.
    pub growth: GrowthOutlook,
}

/// How likely a foal is to turn out each growth type, in percent.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct GrowthOutlook {
    pub early: u32,
    pub normal: u32,
    pub late: u32,
}

/// What a pair's foals turn out like, from the same draw as the foal's own growth.
fn growth_outlook(sire_factor: f64, dam_factor: f64) -> GrowthOutlook {
    const DRAWS: u32 = 1000;
    let mut rng = Rng::new(0x6807);
    let (mut early, mut late) = (0, 0);
    for _ in 0..DRAWS {
        match growth(sire_factor, dam_factor, &mut rng).1 {
            "早熟" => early += 1,
            "晩成" => late += 1,
            _ => {}
        }
    }
    let percent = |n: u32| (n as f64 * 100.0 / DRAWS as f64).round() as u32;
    let (early, late) = (percent(early), percent(late));
    GrowthOutlook {
        early,
        normal: 100 - early - late,
        late,
    }
}

/// A mating theory the pair meets, and the points it is worth.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theory {
    pub name: String,
    pub points: u32,
}

/// The range of inbreeding held to bring out the best in a foal (about 4x4 to 3x3).
const MODERATE_INBREEDING: (f64, f64) = (0.03, 0.13);

fn moderate_inbreeding(vol: f64) -> bool {
    (MODERATE_INBREEDING.0..=MODERATE_INBREEDING.1).contains(&vol)
}

/// The stat a horse is strongest in.
fn best_stat(h: &BloodHorse) -> StatKey {
    let mut best = StatKey::Speed;
    for k in [StatKey::Stamina, StatKey::Power] {
        if h.stats.get(k) > h.stats.get(best) {
            best = k;
        }
    }
    best
}

/// The mating theories a stallion and a mare meet.
pub fn theories(sire: &BloodHorse, dam: &BloodHorse, db: &HashMap<u32, BloodHorse>) -> Vec<Theory> {
    let vol = inbreed_volume(sire.id, dam.id, db);
    let overlap = sire.aptitude_min < dam.aptitude_max && dam.aptitude_min < sire.aptitude_max;
    [
        (vol <= 0.0, "血の活性化", 1),
        (moderate_inbreeding(vol), "適度なインブリード", 2),
        (nick_for(&sire.lineage, &dam.lineage).is_some(), "ニック", 2),
        (best_stat(sire) != best_stat(dam), "能力の補完", 1),
        (overlap, "距離の一致", 1),
        (sire.running_style == dam.running_style, "脚質の一致", 1),
    ]
    .into_iter()
    .filter(|(met, _, _)| *met)
    .map(|(_, name, points)| Theory {
        name: name.to_string(),
        points,
    })
    .collect()
}

/// What a total of theory points comes to.
pub fn rating_for(points: u32) -> String {
    match points {
        0 => "C",
        1..=2 => "B",
        3..=4 => "A",
        _ => "S",
    }
    .to_string()
}

/// The inbreeding and the nick of mating `sire` with `dam`, exactly as [`breed`] will apply them.
pub fn outlook(sire: &BloodHorse, dam: &BloodHorse, db: &HashMap<u32, BloodHorse>) -> Outlook {
    let vol = inbreed_volume(sire.id, dam.id, db);
    let met = theories(sire, dam, db);
    Outlook {
        growth: growth_outlook(sire.growth_factor, dam.growth_factor),
        rating: rating_for(met.iter().map(|t| t.points).sum()),
        theories: met,
        inbreed_vol: (vol * 10000.0).round() / 10000.0,
        penalty: inbreed_penalty(vol),
        nick: nick_for(&sire.lineage, &dam.lineage).map(|(k, bonus)| Nick {
            stat: k.name().to_string(),
            bonus,
        }),
    }
}

/// The generated database for a seed, built once and shared (it takes a moment to make).
pub fn base_database(seed: u64) -> std::sync::Arc<Vec<BloodHorse>> {
    use std::sync::{Arc, Mutex, OnceLock};
    type Cache = Mutex<HashMap<u64, Arc<Vec<BloodHorse>>>>;
    static CACHE: OnceLock<Cache> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(db) = cache.lock().unwrap().get(&seed) {
        return db.clone();
    }
    let db = Arc::new(generate(seed));
    let mut map = cache.lock().unwrap();
    if map.len() >= 16 {
        map.clear(); // seeds change only with a new game; keep the cache small
    }
    map.insert(seed, db.clone());
    db
}

/// The names of the stable's founding horses. The generated database never uses
/// them, so a name stays one horse's.
pub const STABLE_FOUNDERS: [&str; 8] = [
    "アサギリライズ",
    "ギンセイストーム",
    "グレンキング",
    "オウゴンオーシャン",
    "ヘキクウブレイブ",
    "サクラフブキ",
    "ホクトセイバー",
    "ミナモムーン",
];

const NAME_PREFIXES: [&str; 40] = [
    "アカツキ",
    "アサヒ",
    "アオゾラ",
    "ウミ",
    "オウゴン",
    "カゼ",
    "キリ",
    "クロ",
    "コガネ",
    "サクラ",
    "シロ",
    "スイ",
    "ソラ",
    "タイヨウ",
    "ツキ",
    "ハヤテ",
    "ヒカリ",
    "ホシ",
    "マツリ",
    "ミドリ",
    "ユキ",
    "ライト",
    "ルナ",
    "レッド",
    "ロイヤル",
    "アラシ",
    "イナズマ",
    "カグラ",
    "ギンガ",
    "グレン",
    "コウヨウ",
    "ザンゲツ",
    "シンゲツ",
    "セイラン",
    "ダイチ",
    "テンマ",
    "ナギサ",
    "ヒバナ",
    "フジ",
    "ミカヅキ",
];
const NAME_SUFFIXES: [&str; 40] = [
    "アロー",
    "ウイング",
    "エース",
    "オーラ",
    "カイザー",
    "キング",
    "クイーン",
    "ゴールド",
    "サンダー",
    "スター",
    "ストーム",
    "ダンサー",
    "チャンプ",
    "ドリーム",
    "ナイト",
    "ハート",
    "ファイター",
    "フェザー",
    "ブレイブ",
    "ブレード",
    "ホープ",
    "マスター",
    "ランナー",
    "レジェンド",
    "ウィンド",
    "エンペラー",
    "クラウン",
    "グランド",
    "シャドウ",
    "スピリット",
    "スラッシュ",
    "セイバー",
    "タイガー",
    "ティアラ",
    "ドラゴン",
    "ノヴァ",
    "バロン",
    "フレイム",
    "プリンス",
    "ミラージュ",
];

fn unique_name(rng: &mut Rng, used: &HashSet<String>) -> String {
    loop {
        let name = format!(
            "{}{}",
            rng.choice(&NAME_PREFIXES),
            rng.choice(&NAME_SUFFIXES)
        );
        if !used.contains(&name) {
            return name;
        }
    }
}

/// Fills in everything derived from a founder's basic traits.
#[allow(clippy::too_many_arguments)]
fn founder(
    id: u32,
    name: &str,
    lineage: &str,
    family: &str,
    gender: Sex,
    genes: Genes,
    growth_factor: f64,
    running_style: &str,
    personality: &str,
    stats: Stats,
) -> BloodHorse {
    let (growth_type, lo, _) = growth_class(growth_factor);
    let (aptitude_min, aptitude_max) = aptitude_distance(stats.speed, stats.stamina, stats.temper);
    BloodHorse {
        id,
        name: name.into(),
        sire_id: None,
        dam_id: None,
        gender,
        lineage: lineage.into(),
        family: family.into(),
        gen: 1,
        coat_color: coat_color(&genes).into(),
        genes,
        growth_factor,
        growth_type: growth_type.into(),
        jump: founder_jump(id),
        peak_age: lo, // founders peak at the early end of their type's range
        aptitude_min,
        aptitude_max,
        running_style: running_style.into(),
        personality: personality.into(),
        inbreed_vol: 0.0,
        stats,
        mutations: Vec::new(),
        owned: false,
    }
}

/// A horse with no known family, for a horse that has no record of its own yet.
pub fn unrelated(id: u32, name: &str, gender: Sex, stats: Stats) -> BloodHorse {
    founder(
        id,
        name,
        "None",
        "F0",
        gender,
        Genes::new("gg", "Ee", "Aa"),
        0.5,
        "先行",
        "普通",
        stats,
    )
}

/// The three founding sires and five founding dams.
pub fn founders() -> Vec<BloodHorse> {
    let base = Stats {
        speed: 60.0,
        stamina: 60.0,
        temper: 60.0,
        power: 60.0,
        turf: 60.0,
        dirt: 60.0,
    };
    let mut v = vec![
        founder(
            1,
            "イグニス",
            "Ignis",
            "Sire",
            Sex::Male,
            Genes::new("gg", "EE", "AA"),
            0.5,
            "逃げ",
            "狂気",
            Stats {
                speed: 85.0,
                temper: 30.0,
                ..base
            },
        ),
        founder(
            2,
            "アクア",
            "Aqua",
            "Sire",
            Sex::Male,
            Genes::new("Gg", "Ee", "Aa"),
            0.5,
            "差し",
            "冷静",
            Stats {
                stamina: 85.0,
                turf: 80.0,
                ..base
            },
        ),
        founder(
            3,
            "テラ",
            "Terra",
            "Sire",
            Sex::Male,
            Genes::new("gg", "ee", "aa"),
            0.8,
            "先行",
            "頑固",
            Stats {
                power: 85.0,
                dirt: 80.0,
                ..base
            },
        ),
    ];
    for (i, name) in ["ハルカ", "アカネ", "ミドリ", "ユキノ", "サクヤ"]
        .iter()
        .enumerate()
    {
        v.push(founder(
            4 + i as u32,
            name,
            "None",
            &format!("F{}", i + 1),
            Sex::Female,
            Genes::new("gg", "Ee", "Aa"),
            0.5,
            "先行",
            "普通",
            base,
        ));
    }
    v
}

/// The whole master pedigree for `seed`, ordered by id: 8 founders plus the
/// foals of generations 2 to 10 (1148 horses).
pub fn generate(seed: u64) -> Vec<BloodHorse> {
    let mut rng = Rng::new(seed);
    let mut db: HashMap<u32, BloodHorse> = founders().into_iter().map(|h| (h.id, h)).collect();
    let mut used: HashSet<String> = db.values().map(|h| h.name.clone()).collect();
    used.extend(STABLE_FOUNDERS.iter().map(|n| n.to_string()));
    let by_sex = |db: &HashMap<u32, BloodHorse>, sex: Sex, min_gen: u8| -> Vec<u32> {
        let mut ids: Vec<u32> = db
            .values()
            .filter(|h| h.gender == sex && h.gen >= min_gen)
            .map(|h| h.id)
            .collect();
        ids.sort(); // HashMap order is arbitrary; keep the result reproducible
        ids
    };
    let mut active_sires = by_sex(&db, Sex::Male, 1);
    let mut active_dams = by_sex(&db, Sex::Female, 1);
    let mut next_id = db.len() as u32 + 1;

    for (i, &target) in TARGET_COUNTS.iter().enumerate() {
        let gen = i as u8 + 2;
        let mut born: Vec<u32> = Vec::with_capacity(target);
        for _ in 0..target {
            let sire_id = *rng.choice(&active_sires);
            let dam_id = *rng.choice(&active_dams);
            let name = unique_name(&mut rng, &used);
            used.insert(name.clone());
            let child = breed(
                &db[&sire_id],
                &db[&dam_id],
                gen,
                next_id,
                name,
                &db,
                &mut rng,
            );
            born.push(child.id);
            db.insert(child.id, child);
            next_id += 1;
        }
        active_sires = born
            .iter()
            .copied()
            .filter(|id| db[id].gender == Sex::Male)
            .collect();
        active_dams = born
            .iter()
            .copied()
            .filter(|id| db[id].gender == Sex::Female)
            .collect();
        if active_sires.is_empty() {
            active_sires = by_sex(&db, Sex::Male, gen - 1);
        }
        if active_dams.is_empty() {
            active_dams = by_sex(&db, Sex::Female, gen - 1);
        }
    }

    let mut all: Vec<BloodHorse> = db.into_values().collect();
    all.sort_by_key(|h| h.id);
    all
}

/// The master pedigree of the current game: the generated database plus the
/// player's own foals, the stable's horses carrying their own names.
#[tauri::command]
pub fn get_bloodline(store: tauri::State<GameStore>) -> Vec<BloodHorse> {
    store.0.lock().unwrap().bloodline_all()
}

#[cfg(test)]
mod theory_tests {
    use super::*;

    fn horse(id: u32, gender: Sex, line: &str) -> BloodHorse {
        let base = base_database(1);
        let mut h = base.iter().find(|h| h.gender == gender).unwrap().clone();
        h.id = id;
        h.sire_id = None;
        h.dam_id = None;
        h.lineage = line.to_string();
        h
    }

    fn named(sire: &BloodHorse, dam: &BloodHorse, db: &HashMap<u32, BloodHorse>) -> Vec<String> {
        theories(sire, dam, db)
            .into_iter()
            .map(|t| t.name)
            .collect()
    }

    #[test]
    fn unrelated_parents_freshen_the_blood() {
        let (s, d) = (
            horse(90_001, Sex::Male, "X"),
            horse(90_002, Sex::Female, "Y"),
        );
        assert!(named(&s, &d, &HashMap::new())
            .iter()
            .any(|n| n == "血の活性化"));
    }

    #[test]
    fn a_moderate_inbreeding_is_a_theory_and_a_close_one_is_not() {
        assert!(moderate_inbreeding(0.03));
        assert!(moderate_inbreeding(0.13));
        assert!(!moderate_inbreeding(0.01));
        assert!(!moderate_inbreeding(0.25));
    }

    #[test]
    fn parents_that_make_up_for_each_other_are_a_theory() {
        let (mut s, mut d) = (
            horse(90_001, Sex::Male, "X"),
            horse(90_002, Sex::Female, "Y"),
        );
        s.stats.speed = 90.0;
        s.stats.stamina = 50.0;
        s.stats.power = 50.0;
        d.stats.speed = 50.0;
        d.stats.stamina = 90.0;
        d.stats.power = 50.0;
        assert!(named(&s, &d, &HashMap::new())
            .iter()
            .any(|n| n == "能力の補完"));
        d.stats.speed = 90.0;
        d.stats.stamina = 50.0;
        assert!(!named(&s, &d, &HashMap::new())
            .iter()
            .any(|n| n == "能力の補完"));
    }

    #[test]
    fn a_shared_distance_and_running_style_are_theories() {
        let (mut s, mut d) = (
            horse(90_001, Sex::Male, "X"),
            horse(90_002, Sex::Female, "Y"),
        );
        s.aptitude_min = 1600;
        s.aptitude_max = 2400;
        d.aptitude_min = 2000;
        d.aptitude_max = 3000;
        s.running_style = "差し".to_string();
        d.running_style = "差し".to_string();
        let t = named(&s, &d, &HashMap::new());
        assert!(t.iter().any(|n| n == "距離の一致"));
        assert!(t.iter().any(|n| n == "脚質の一致"));
        d.aptitude_min = 2600;
        d.running_style = "逃げ".to_string();
        let t = named(&s, &d, &HashMap::new());
        assert!(!t.iter().any(|n| n == "距離の一致"));
        assert!(!t.iter().any(|n| n == "脚質の一致"));
    }

    #[test]
    fn the_rating_rises_with_the_points() {
        assert_eq!(rating_for(0), "C");
        assert_eq!(rating_for(1), "B");
        assert_eq!(rating_for(3), "A");
        assert_eq!(rating_for(5), "S");
    }

    #[test]
    fn the_outlook_lists_the_theories_and_the_rating() {
        let (s, d) = (
            horse(90_001, Sex::Male, "X"),
            horse(90_002, Sex::Female, "Y"),
        );
        let o = outlook(&s, &d, &HashMap::new());
        let points: u32 = o.theories.iter().map(|t| t.points).sum();
        assert_eq!(o.rating, rating_for(points));
        assert!(!o.theories.is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::horse::is_valid_name;

    fn rng(seed: u64) -> Rng {
        Rng::new(seed)
    }

    fn genes(g: &str, e: &str, a: &str) -> Genes {
        Genes::new(g, e, a)
    }

    // ---- coat colour & genes

    #[test]
    fn grey_is_dominant_over_every_other_colour() {
        for g in ["GG", "Gg"] {
            for e in ["EE", "Ee", "ee"] {
                for a in ["AA", "Aa", "aa"] {
                    assert_eq!(coat_color(&genes(g, e, a)), "芦毛", "{g}{e}{a}");
                }
            }
        }
    }

    #[test]
    fn without_grey_ee_is_chestnut_and_otherwise_agouti_decides() {
        assert_eq!(coat_color(&genes("gg", "ee", "AA")), "栗毛");
        assert_eq!(coat_color(&genes("gg", "ee", "aa")), "栗毛");
        for e in ["EE", "Ee"] {
            assert_eq!(coat_color(&genes("gg", e, "AA")), "鹿毛");
            assert_eq!(coat_color(&genes("gg", e, "Aa")), "鹿毛");
            assert_eq!(coat_color(&genes("gg", e, "aa")), "青毛");
        }
    }

    #[test]
    fn children_get_one_allele_from_each_parent_in_sorted_order() {
        let mut r = rng(1);
        for _ in 0..200 {
            let c = inherit_genes(&genes("GG", "EE", "AA"), &genes("gg", "ee", "aa"), &mut r);
            assert_eq!(
                (c.g.as_str(), c.e.as_str(), c.a.as_str()),
                ("Gg", "Ee", "Aa")
            );
        }
        let mut seen = HashSet::new();
        for _ in 0..400 {
            let c = inherit_genes(&genes("Gg", "Ee", "Aa"), &genes("Gg", "Ee", "Aa"), &mut r);
            assert!(["GG", "Gg", "gg"].contains(&c.g.as_str()));
            seen.insert(c.g);
        }
        assert_eq!(seen.len(), 3, "all three genotypes should appear");
    }

    // ---- stats, growth, aptitude

    #[test]
    fn stats_follow_the_parents_pulled_toward_the_average() {
        let mean = |sire: f64, dam: f64| {
            let mut r = rng(3);
            (0..3000)
                .map(|_| base_stat_with(sire, dam, 5.0, 0.0, &mut r).0)
                .sum::<f64>()
                / 3000.0
        };
        assert!((mean(80.0, 80.0) - 74.0).abs() < 0.5); // 80*0.8 + 50*0.2
        assert!((mean(20.0, 20.0) - 26.0).abs() < 0.5);
        assert!((mean(50.0, 50.0) - 50.0).abs() < 0.5);
        let mut r = rng(4);
        assert!((0..2000)
            .all(|_| (0.0..=100.0).contains(&base_stat_with(99.0, 99.0, 30.0, 0.0, &mut r).0)));
    }

    // ---- mutation (突然変異)

    #[test]
    fn without_mutation_a_stat_never_passes_100_and_is_never_flagged() {
        let mut r = rng(31);
        for _ in 0..5000 {
            let (v, mutated) = base_stat_with(100.0, 100.0, 30.0, 0.0, &mut r);
            assert!((0.0..=100.0).contains(&v) && !mutated, "{v}");
        }
    }

    #[test]
    fn a_certain_mutation_adds_fifteen_to_twenty_five_points() {
        // same seed, so the ordinary part of the roll is identical with and without it
        for seed in 0..500u64 {
            let (plain, _) = base_stat_with(70.0, 60.0, 5.0, 0.0, &mut rng(seed));
            let (lifted, mutated) = base_stat_with(70.0, 60.0, 5.0, 1.0, &mut rng(seed));
            assert!(mutated);
            let bonus = lifted - plain;
            assert!((15.0..=25.0).contains(&bonus), "seed {seed}: bonus {bonus}");
        }
    }

    #[test]
    fn a_mutation_can_break_through_the_ceiling_of_100() {
        let mut r = rng(32);
        let over = (0..2000)
            .map(|_| base_stat_with(100.0, 100.0, 5.0, 1.0, &mut r).0)
            .filter(|v| *v > 100.0)
            .count();
        assert!(over > 1900, "{over} of 2000");
    }

    #[test]
    fn a_mutated_stat_tops_out_at_125() {
        // a normal stat is at most 100 and the bonus at most 25, so 125 is the ceiling
        // (the final `min` is a safety net if either number is ever raised)
        let mut r = rng(33);
        let vals: Vec<f64> = (0..3000)
            .map(|_| base_stat_with(100.0, 100.0, 30.0, 1.0, &mut r).0)
            .collect();
        let top = vals.iter().cloned().fold(0.0, f64::max);
        assert!(top <= 125.0, "{top}");
        assert!(
            top > 124.0,
            "the ceiling should be nearly reached, got {top}"
        );
    }

    #[test]
    fn mutation_happens_about_once_in_a_hundred_stats() {
        let mut r = rng(34);
        let n = 60000;
        let hits = (0..n)
            .filter(|_| base_stat(60.0, 60.0, 5.0, &mut r).1)
            .count();
        let rate = hits as f64 / n as f64;
        assert!((0.007..0.013).contains(&rate), "rate {rate}");
    }

    #[test]
    fn the_mutation_flag_is_exactly_whether_the_bonus_was_added() {
        let mut r = rng(35);
        let mut mutated_seen = 0;
        for _ in 0..4000 {
            let seed = r.next_u64();
            let (plain, _) = base_stat_with(60.0, 60.0, 5.0, 0.0, &mut rng(seed));
            let (v, mutated) = base_stat_with(60.0, 60.0, 5.0, 0.05, &mut rng(seed));
            if mutated {
                mutated_seen += 1;
                assert!(v > plain + 14.9, "{v} vs {plain}");
            } else {
                assert_eq!(v, plain);
            }
        }
        assert!(mutated_seen > 100, "{mutated_seen}");
    }

    #[test]
    fn a_mutation_is_decided_by_the_injected_generator_alone() {
        let roll = |seed| base_stat_with(70.0, 70.0, 5.0, 0.5, &mut rng(seed));
        for seed in 0..100 {
            assert_eq!(roll(seed), roll(seed));
        }
        assert!((0..100).map(roll).any(|(_, m)| m) && (0..100).map(roll).any(|(_, m)| !m));
    }

    #[test]
    fn growth_types_match_their_peak_ages() {
        let mut r = rng(5);
        for _ in 0..2000 {
            let (factor, kind, age) = growth(0.5, 0.5, &mut r);
            assert!((0.0..=1.0).contains(&factor));
            let range = match kind {
                "早熟" => 2..=3,
                "普通" => 4..=5,
                "晩成" => 6..=7,
                other => panic!("unknown growth type {other}"),
            };
            assert!(range.contains(&age), "{kind} peaks at {age}");
            assert_eq!(kind, growth_class(factor).0);
        }
    }

    #[test]
    fn growth_factor_is_inherited() {
        let share = |s: f64, d: f64, kind: &str| {
            let mut r = rng(6);
            (0..2000).filter(|_| growth(s, d, &mut r).1 == kind).count() as f64 / 2000.0
        };
        assert!(share(0.05, 0.05, "早熟") > 0.9);
        assert!(share(0.95, 0.95, "晩成") > 0.9);
        assert!(share(0.5, 0.5, "普通") > 0.7);
    }

    #[test]
    fn distance_aptitude_follows_stamina_and_is_capped_by_temperament() {
        assert_eq!(aptitude_distance(85.0, 30.0, 60.0), (1000, 1600)); // sprinter
        assert_eq!(aptitude_distance(50.0, 50.0, 50.0), (1800, 2600)); // all-rounder
        assert_eq!(aptitude_distance(10.0, 90.0, 50.0), (3000, 3600)); // stayer, capped at 3600
                                                                       // each full 25 points under 50 temper takes 200m off the top end
        assert_eq!(aptitude_distance(50.0, 50.0, 25.0), (1800, 2400));
        assert_eq!(aptitude_distance(50.0, 50.0, 0.0), (1800, 2200));
        assert_eq!(aptitude_distance(50.0, 50.0, 40.0), (1800, 2600)); // 0.4 of a unit: truncated away
        assert_eq!(aptitude_distance(0.0, 0.0, 50.0), (1200, 1200));
    }

    #[test]
    fn distance_aptitude_is_always_a_valid_range() {
        let mut r = rng(7);
        for _ in 0..5000 {
            let (s, st, t) = (
                r.next_f64() * 100.0,
                r.next_f64() * 100.0,
                r.next_f64() * 100.0,
            );
            let (lo, hi) = aptitude_distance(s, st, t);
            assert!(lo % 200 == 0 && hi % 200 == 0, "{lo}-{hi}");
            assert!(
                lo >= 1000 && hi <= 3600 && hi >= lo + 200,
                "{lo}-{hi} for {s} {st} {t}"
            );
        }
    }

    #[test]
    fn rounding_to_200m_ties_to_even_like_python() {
        assert_eq!(round_to_200(900.0), 800); // 4.5 -> 4
        assert_eq!(round_to_200(1100.0), 1200); // 5.5 -> 6
        assert_eq!(round_to_200(1300.0), 1200); // 6.5 -> 6
        assert_eq!(round_to_200(1010.0), 1000);
        assert_eq!(round_to_200(1190.0), 1200);
    }

    // ---- temperament & running style

    #[test]
    fn personality_depends_on_temper() {
        let mut r = rng(8);
        for _ in 0..500 {
            assert!(["狂気", "気分屋", "臆病", "頑固"].contains(&personality(10.0, &mut r)));
            assert!(["負けず嫌い", "やんちゃ", "普通", "臆病"].contains(&personality(50.0, &mut r)));
            assert!(["冷静", "優等生", "勇敢", "素直"].contains(&personality(90.0, &mut r)));
        }
        // boundaries: 35 and 65 move up a band
        let low: HashSet<_> = (0..300).map(|_| personality(34.9, &mut r)).collect();
        assert!(low.contains("狂気"));
        let mid: HashSet<_> = (0..300).map(|_| personality(35.0, &mut r)).collect();
        assert!(!mid.contains("狂気"));
        let high: HashSet<_> = (0..300).map(|_| personality(65.0, &mut r)).collect();
        assert!(high.contains("冷静") && !high.contains("普通"));
    }

    #[test]
    fn running_style_is_mostly_inherited() {
        let mut r = rng(9);
        let n = 4000;
        let inherited = (0..n)
            .filter(|_| running_style("逃げ", "逃げ", 50.0, 50.0, 50.0, &mut r) == "逃げ")
            .count() as f64
            / n as f64;
        assert!((0.55..0.65).contains(&inherited), "{inherited}");
        // both parents show up
        let styles: HashSet<String> = (0..300)
            .map(|_| running_style("逃げ", "追込", 50.0, 50.0, 50.0, &mut r))
            .collect();
        assert!(styles.contains("逃げ") && styles.contains("追込"));
    }

    #[test]
    fn a_hot_fast_horse_goes_to_the_front_when_not_inheriting() {
        let mut r = rng(10);
        // parents are 追込, so any 逃げ must come from the horse's own build
        let styles: Vec<String> = (0..2000)
            .map(|_| running_style("追込", "追込", 80.0, 60.0, 30.0, &mut r))
            .collect();
        assert!(styles.iter().all(|s| s == "追込" || s == "逃げ"));
        assert!(styles.iter().filter(|s| *s == "逃げ").count() > 700);
    }

    #[test]
    fn a_calm_horse_with_stamina_to_spare_saves_its_run() {
        let mut r = rng(11);
        // speed < stamina and temper >= 60: only 差し / 追込 besides the inherited style
        let styles: Vec<String> = (0..2000)
            .map(|_| running_style("逃げ", "逃げ", 40.0, 80.0, 80.0, &mut r))
            .collect();
        assert!(!styles.iter().any(|s| s == "先行"));
        assert!(styles.iter().any(|s| s == "差し") && styles.iter().any(|s| s == "追込"));
        // a calm horse that is faster than it is stout falls back to 先行 / 差し
        let styles: Vec<String> = (0..2000)
            .map(|_| running_style("追込", "追込", 80.0, 40.0, 80.0, &mut r))
            .collect();
        assert!(styles.iter().any(|s| s == "先行"));
    }

    // ---- ancestors & inbreeding

    fn mini(id: u32, sire: Option<u32>, dam: Option<u32>, sex: Sex) -> BloodHorse {
        let mut h = founders()[0].clone();
        h.id = id;
        h.sire_id = sire;
        h.dam_id = dam;
        h.gender = sex;
        h
    }

    fn db_of(horses: Vec<BloodHorse>) -> HashMap<u32, BloodHorse> {
        horses.into_iter().map(|h| (h.id, h)).collect()
    }

    #[test]
    fn ancestors_are_counted_in_generations_with_the_shortest_route() {
        // 4 -> (2, 3); 2 -> (1, -); 3 -> (1, -): horse 1 is both a grandsire twice
        let db = db_of(vec![
            mini(1, None, None, Sex::Male),
            mini(2, Some(1), None, Sex::Male),
            mini(3, Some(1), None, Sex::Female),
            mini(4, Some(2), Some(3), Sex::Male),
        ]);
        let a = ancestors(4, &db);
        assert_eq!(a.get(&2), Some(&1));
        assert_eq!(a.get(&3), Some(&1));
        assert_eq!(a.get(&1), Some(&2));
        assert_eq!(a.len(), 3);
        // a horse is not its own ancestor, and a founder has none
        assert!(ancestors(1, &db).is_empty());
        assert!(!a.contains_key(&4));
    }

    #[test]
    fn ancestors_stop_after_four_generations() {
        let mut v = vec![mini(1, None, None, Sex::Male)];
        for id in 2..=7u32 {
            v.push(mini(id, Some(id - 1), None, Sex::Male));
        }
        let db = db_of(v);
        let a = ancestors(7, &db);
        assert_eq!(a.len(), 4); // 6, 5, 4, 3
        assert_eq!(a.get(&3), Some(&4));
        assert!(!a.contains_key(&2));
    }

    #[test]
    fn no_inbreeding_is_a_plain_zero_never_negative_zero() {
        let db = db_of(founders());
        assert!(!inbreed_volume(1, 4, &db).is_sign_negative());
        assert!(generate(1)
            .iter()
            .all(|h| !h.inbreed_vol.is_sign_negative()));
        assert_eq!(
            serde_json::to_string(&inbreed_volume(1, 4, &db)).unwrap(),
            "0.0"
        );
    }

    #[test]
    fn unrelated_parents_have_no_inbreeding() {
        let db = db_of(founders().into_iter().collect());
        assert_eq!(inbreed_volume(1, 4, &db), 0.0);
    }

    #[test]
    fn half_siblings_mated_together_are_inbred_one_eighth() {
        // sire 2 and dam 3 share only their sire, 1
        let db = db_of(vec![
            mini(1, None, None, Sex::Male),
            mini(2, Some(1), None, Sex::Male),
            mini(3, Some(1), None, Sex::Female),
        ]);
        assert_eq!(inbreed_volume(2, 3, &db), 0.125);
    }

    #[test]
    fn full_siblings_mated_together_are_inbred_one_quarter() {
        // sire 2 and dam 3 share both parents, 1 and 4
        let db = db_of(vec![
            mini(1, None, None, Sex::Male),
            mini(4, None, None, Sex::Female),
            mini(2, Some(1), Some(4), Sex::Male),
            mini(3, Some(1), Some(4), Sex::Female),
        ]);
        assert_eq!(inbreed_volume(2, 3, &db), 0.25);
    }

    #[test]
    fn first_cousins_mated_together_are_inbred_one_sixteenth() {
        // full siblings 3 and 4 (children of 1 and 2) each have a child, 5 and 6
        let db = db_of(vec![
            mini(1, None, None, Sex::Male),
            mini(2, None, None, Sex::Female),
            mini(3, Some(1), Some(2), Sex::Male),
            mini(4, Some(1), Some(2), Sex::Female),
            mini(5, Some(3), None, Sex::Male),
            mini(6, None, Some(4), Sex::Female),
        ]);
        // two shared grandparents, each 0.5^5
        assert_eq!(inbreed_volume(5, 6, &db), 2.0 * 0.03125);
    }

    #[test]
    fn a_common_ancestor_three_generations_back_on_both_sides_is_3_125_percent() {
        // the foal of 5 and 6 has 1 as a great-grandparent through each parent:
        // 3 generations back (counted from the foal) on both sides = 0.5^(3+3-1)
        let db = db_of(vec![
            mini(1, None, None, Sex::Male),
            mini(3, Some(1), None, Sex::Male),
            mini(4, Some(1), None, Sex::Female),
            mini(5, Some(3), None, Sex::Male),
            mini(6, Some(4), None, Sex::Female),
        ]);
        assert_eq!(inbreed_volume(5, 6, &db), 0.5_f64.powi(5));
        assert_eq!(inbreed_volume(5, 6, &db), 0.03125);
    }

    #[test]
    fn the_sire_and_dam_may_be_nearer_the_ancestor_on_one_side() {
        // sire 5 -> 4 -> 1 (2 back), dam 3 -> 1 (1 back): 0.5^(2+1+1)
        let db = db_of(vec![
            mini(1, None, None, Sex::Male),
            mini(3, Some(1), None, Sex::Female),
            mini(4, Some(1), None, Sex::Male),
            mini(5, Some(4), None, Sex::Male),
        ]);
        assert_eq!(inbreed_volume(5, 3, &db), 0.0625);
    }

    #[test]
    fn inbreeding_is_the_same_whichever_parent_is_called_the_sire() {
        let db = db_of(vec![
            mini(1, None, None, Sex::Male),
            mini(3, Some(1), None, Sex::Female),
            mini(4, Some(1), None, Sex::Male),
            mini(5, Some(4), None, Sex::Male),
        ]);
        assert_eq!(inbreed_volume(5, 3, &db), inbreed_volume(3, 5, &db));
    }

    #[test]
    fn each_further_generation_back_halves_the_inbreeding() {
        // a single shared ancestor, one generation further back on the sire's side each time
        let chain = |extra: u32| {
            let mut v = vec![
                mini(1, None, None, Sex::Male),
                mini(100, Some(1), None, Sex::Female),
            ];
            let mut parent = 1;
            for i in 0..extra {
                v.push(mini(10 + i, Some(parent), None, Sex::Male));
                parent = 10 + i;
            }
            let sire = mini(50, Some(parent), None, Sex::Male);
            v.push(sire);
            let db = db_of(v);
            inbreed_volume(50, 100, &db)
        };
        assert_eq!(chain(0), 0.125); // sire 50 -> 1, dam 100 -> 1: half siblings
        assert_eq!(chain(1), 0.0625);
        assert_eq!(chain(2), 0.03125);
    }

    #[test]
    fn a_sire_that_is_the_dams_own_sire_is_not_counted_as_shared() {
        // the script only looks at ancestors above the sire and dam themselves
        let db = db_of(vec![
            mini(1, None, None, Sex::Male),
            mini(2, Some(1), None, Sex::Female),
        ]);
        assert_eq!(inbreed_volume(1, 2, &db), 0.0);
    }

    // ---- mating

    fn fresh() -> (BloodHorse, BloodHorse, HashMap<u32, BloodHorse>) {
        let f = founders();
        let db = db_of(f.clone());
        (f[0].clone(), f[3].clone(), db)
    }

    fn foals(
        sire: &BloodHorse,
        dam: &BloodHorse,
        db: &HashMap<u32, BloodHorse>,
        n: usize,
    ) -> Vec<BloodHorse> {
        let mut r = rng(21);
        (0..n)
            .map(|i| {
                breed(
                    sire,
                    dam,
                    2,
                    100 + i as u32,
                    format!("テスト{i}"),
                    db,
                    &mut r,
                )
            })
            .collect()
    }

    #[test]
    fn a_foal_records_its_parents_and_takes_their_lines() {
        let (sire, dam, db) = fresh();
        let foal = &foals(&sire, &dam, &db, 1)[0];
        assert_eq!((foal.sire_id, foal.dam_id), (Some(sire.id), Some(dam.id)));
        assert_eq!(foal.lineage, sire.lineage);
        assert_eq!(foal.family, dam.family);
        assert_eq!(foal.gen, 2);
        assert_eq!(foal.coat_color, coat_color(&foal.genes));
        assert_eq!(foal.inbreed_vol, 0.0);
    }

    #[test]
    fn foal_stats_are_in_range_and_rounded_to_one_decimal() {
        let (sire, dam, db) = fresh();
        for f in foals(&sire, &dam, &db, 300) {
            for k in [
                StatKey::Speed,
                StatKey::Stamina,
                StatKey::Temper,
                StatKey::Power,
                StatKey::Turf,
                StatKey::Dirt,
            ] {
                let v = f.stats.get(k);
                assert!((0.0..=MUTATION_CAP).contains(&v));
                assert!(
                    v <= 100.0 || f.mutations.contains(&k.name().to_string()),
                    "{v} is over 100 but not a mutation"
                );
                assert!(
                    ((v * 10.0).round() - v * 10.0).abs() < 1e-6,
                    "{v} is not 1dp"
                );
            }
            assert!(f.aptitude_min >= 1000 && f.aptitude_max <= 3600);
            let (kind, lo, hi) = growth_class(f.growth_factor);
            assert_eq!(f.growth_type, kind);
            assert!((lo..=hi).contains(&f.peak_age));
        }
    }

    #[test]
    fn both_sexes_are_born_about_equally() {
        let (sire, dam, db) = fresh();
        let males = foals(&sire, &dam, &db, 1000)
            .iter()
            .filter(|f| f.gender == Sex::Male)
            .count();
        assert!((430..570).contains(&males), "{males}");
    }

    #[test]
    fn a_nick_adds_its_bonus_only_for_the_listed_cross() {
        let (mut sire, mut dam, db) = fresh();
        let mean = |s: &BloodHorse, d: &BloodHorse, k: StatKey| {
            foals(s, d, &db, 800)
                .iter()
                .map(|f| f.stats.get(k))
                .sum::<f64>()
                / 800.0
        };
        for h in [&mut sire, &mut dam] {
            h.stats = Stats {
                speed: 60.0,
                stamina: 60.0,
                temper: 60.0,
                power: 60.0,
                turf: 60.0,
                dirt: 60.0,
            };
        }
        // Ignis sire x Aqua dam: +7 speed
        sire.lineage = "Ignis".into();
        dam.lineage = "Aqua".into();
        let with = mean(&sire, &dam, StatKey::Speed);
        // the reverse cross is not in the table
        sire.lineage = "Aqua".into();
        dam.lineage = "Ignis".into();
        let without = mean(&sire, &dam, StatKey::Speed);
        assert!((with - without - 7.0).abs() < 1.0, "{with} vs {without}");
        // Terra x Ignis: +5 power
        sire.lineage = "Terra".into();
        let nick_power = mean(&sire, &dam, StatKey::Power);
        sire.lineage = "Aqua".into();
        let plain_power = mean(&sire, &dam, StatKey::Power);
        assert!(
            (nick_power - plain_power - 5.0).abs() < 1.0,
            "{nick_power} vs {plain_power}"
        );
    }

    #[test]
    fn strong_power_lifts_dirt_aptitude() {
        let (mut sire, mut dam, db) = fresh();
        for h in [&mut sire, &mut dam] {
            h.stats = Stats {
                speed: 60.0,
                stamina: 60.0,
                temper: 60.0,
                power: 100.0,
                turf: 60.0,
                dirt: 60.0,
            };
        }
        let fs = foals(&sire, &dam, &db, 600);
        let dirt = fs.iter().map(|f| f.stats.dirt).sum::<f64>() / 600.0;
        let turf = fs.iter().map(|f| f.stats.turf).sum::<f64>() / 600.0;
        // dirt gets the same pull as turf (about 58) plus (power - 60) / 2 of roughly 90 power
        assert!(dirt > turf + 10.0, "dirt {dirt} turf {turf}");
        assert!(fs.iter().all(|f| f.stats.dirt <= 100.0));
    }

    fn flat_stats(v: f64) -> Stats {
        Stats {
            speed: v,
            stamina: v,
            temper: v,
            power: v,
            turf: v,
            dirt: v,
        }
    }

    fn mean(v: &[BloodHorse], k: StatKey) -> f64 {
        v.iter().map(|f| f.stats.get(k)).sum::<f64>() / v.len() as f64
    }

    fn sd(v: &[BloodHorse], k: StatKey) -> f64 {
        let m = mean(v, k);
        (v.iter().map(|f| (f.stats.get(k) - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt()
    }

    /// Foals of two same-stat parents, related as the given database says.
    fn related_foals(extra: Vec<BloodHorse>, sire: u32, dam: u32, stat: f64) -> Vec<BloodHorse> {
        let mut all = founders();
        for mut h in extra {
            h.stats = flat_stats(stat);
            h.lineage = "None".into();
            all.push(h);
        }
        let db = db_of(all);
        foals(&db[&sire], &db[&dam], &db, 800)
    }

    fn unrelated_foals(stat: f64) -> Vec<BloodHorse> {
        let (mut sire, mut dam, db) = fresh();
        for h in [&mut sire, &mut dam] {
            h.stats = flat_stats(stat);
            h.lineage = "None".into();
        }
        foals(&sire, &dam, &db, 800)
    }

    fn half_siblings() -> Vec<BloodHorse> {
        vec![
            mini(20, Some(1), None, Sex::Male),
            mini(21, Some(1), None, Sex::Female),
        ]
    }

    fn full_siblings() -> Vec<BloodHorse> {
        vec![
            mini(20, Some(1), Some(4), Sex::Male),
            mini(21, Some(1), Some(4), Sex::Female),
        ]
    }

    #[test]
    fn mating_half_siblings_costs_a_little_temperament_and_widens_the_spread() {
        let inbred = related_foals(half_siblings(), 20, 21, 60.0);
        let clean = unrelated_foals(60.0);
        assert!(inbred.iter().all(|x| x.inbreed_vol == 0.125));
        // penalty 10 * 0.125 / 0.1875 = 6.7 points
        let drop = mean(&clean, StatKey::Temper) - mean(&inbred, StatKey::Temper);
        assert!((5.0..8.5).contains(&drop), "temper drop {drop}");
        // spread: 5 + 0.125 * 15 = 6.9 against 5
        let ratio = sd(&inbred, StatKey::Speed) / sd(&clean, StatKey::Speed);
        assert!((1.2..1.55).contains(&ratio), "spread ratio {ratio}");
    }

    #[test]
    fn mating_full_siblings_costs_much_more_but_not_everything() {
        let inbred = related_foals(full_siblings(), 20, 21, 60.0);
        let clean = unrelated_foals(60.0);
        assert!(inbred.iter().all(|x| x.inbreed_vol == 0.25));
        // penalty 10 * (0.25 / 0.1875)^2 = 17.8 points
        let drop = mean(&clean, StatKey::Temper) - mean(&inbred, StatKey::Temper);
        assert!((14.0..21.0).contains(&drop), "temper drop {drop}");
        assert!(
            mean(&inbred, StatKey::Temper) > 30.0,
            "{}",
            mean(&inbred, StatKey::Temper)
        );
    }

    #[test]
    fn the_temperament_penalty_grows_with_inbreeding_but_stops_at_thirty() {
        assert_eq!(inbreed_penalty(0.0), 0.0);
        // below the reference, a gentle straight line: 10 points at the reference
        assert!((inbreed_penalty(INBREED_REFERENCE / 2.0) - 5.0).abs() < 1e-9);
        assert!((inbreed_penalty(INBREED_REFERENCE) - 10.0).abs() < 1e-9);
        // above it, squared growth
        assert!(
            (inbreed_penalty(0.25) - 10.0 * (0.25_f64 / INBREED_REFERENCE).powi(2)).abs() < 1e-9
        );
        // capped
        assert_eq!(inbreed_penalty(0.5), 30.0);
        assert_eq!(inbreed_penalty(5.0), 30.0);
    }

    #[test]
    fn the_penalty_never_decreases_never_jumps_and_never_exceeds_thirty() {
        let mut last = 0.0;
        for i in 0..=500 {
            let vol = i as f64 / 500.0;
            let p = inbreed_penalty(vol);
            assert!(p >= last - 1e-12, "penalty fell at {vol}");
            assert!(p - last < 0.5, "penalty jumped at {vol}: {last} -> {p}");
            assert!(p <= 30.0);
            last = p;
        }
    }

    #[test]
    fn temperament_never_drops_below_one_however_inbred() {
        // dull parents of a full-sibling mating, then the whole database
        let foals = related_foals(full_siblings(), 20, 21, 3.0);
        assert!(
            foals.iter().all(|f| f.stats.temper >= 1.0),
            "{:?}",
            foals
                .iter()
                .map(|f| f.stats.temper)
                .fold(f64::MAX, f64::min)
        );
        assert!(generate(8).iter().all(|h| h.stats.temper >= 1.0));
    }

    // ---- mutation in a foal

    const ALL_STATS: [StatKey; 6] = [
        StatKey::Speed,
        StatKey::Stamina,
        StatKey::Temper,
        StatKey::Power,
        StatKey::Turf,
        StatKey::Dirt,
    ];
    const STAT_NAMES: [&str; 6] = ["speed", "stamina", "temper", "power", "turf", "dirt"];

    fn foals_with_chance(
        sire: &BloodHorse,
        dam: &BloodHorse,
        db: &HashMap<u32, BloodHorse>,
        chance: f64,
        n: usize,
    ) -> Vec<BloodHorse> {
        let mut r = rng(77);
        (0..n)
            .map(|i| {
                breed_with(
                    sire,
                    dam,
                    2,
                    100 + i as u32,
                    format!("テスト{i}"),
                    db,
                    chance,
                    &mut r,
                )
            })
            .collect()
    }

    fn flat_parents(v: f64) -> (BloodHorse, BloodHorse, HashMap<u32, BloodHorse>) {
        let (mut sire, mut dam, db) = fresh();
        for h in [&mut sire, &mut dam] {
            h.stats = Stats {
                speed: v,
                stamina: v,
                temper: v,
                power: v,
                turf: v,
                dirt: v,
            };
        }
        (sire, dam, db)
    }

    fn stat_named(h: &BloodHorse, name: &str) -> f64 {
        let key = ALL_STATS[STAT_NAMES
            .iter()
            .position(|n| *n == name)
            .expect("unknown stat")];
        h.stats.get(key)
    }

    #[test]
    fn without_mutation_a_foal_lists_none_and_stays_within_100() {
        let (sire, dam, db) = flat_parents(100.0);
        for f in foals_with_chance(&sire, &dam, &db, 0.0, 300) {
            assert!(f.mutations.is_empty());
            for k in ALL_STATS {
                assert!(f.stats.get(k) <= 100.0);
            }
        }
    }

    #[test]
    fn a_certain_mutation_marks_every_stat_and_lifts_it_past_100() {
        let (sire, dam, db) = flat_parents(100.0);
        let fs = foals_with_chance(&sire, &dam, &db, 1.0, 400);
        for f in &fs {
            assert_eq!(f.mutations, STAT_NAMES);
            for k in ALL_STATS {
                assert!(f.stats.get(k) <= MUTATION_CAP);
            }
        }
        let mean = |k: StatKey| fs.iter().map(|f| f.stats.get(k)).sum::<f64>() / fs.len() as f64;
        for k in ALL_STATS {
            assert!(mean(k) > 100.0, "{k:?} averages {}", mean(k));
        }
    }

    #[test]
    fn a_nick_bonus_does_not_cut_a_mutated_stat_back_to_100() {
        let (mut sire, mut dam, db) = flat_parents(100.0);
        sire.lineage = "Ignis".into();
        dam.lineage = "Aqua".into(); // +7 speed
        let fs = foals_with_chance(&sire, &dam, &db, 1.0, 400);
        let mean = fs.iter().map(|f| f.stats.speed).sum::<f64>() / fs.len() as f64;
        assert!(mean > 105.0, "{mean}");
        assert!(fs.iter().all(|f| f.stats.speed <= MUTATION_CAP));
    }

    #[test]
    fn power_still_lifts_a_mutated_dirt_aptitude_past_100() {
        let (sire, dam, db) = flat_parents(100.0);
        let fs = foals_with_chance(&sire, &dam, &db, 1.0, 400);
        let mean = fs.iter().map(|f| f.stats.dirt).sum::<f64>() / fs.len() as f64;
        // about 110 from the roll itself, plus (110 - 60) / 2 from power, up to the cap
        assert!(mean > 115.0, "{mean}");
        assert!(fs.iter().all(|f| f.stats.dirt <= MUTATION_CAP));
    }

    #[test]
    fn without_a_mutation_a_nick_still_stops_at_100() {
        let (mut sire, mut dam, db) = flat_parents(100.0);
        sire.lineage = "Ignis".into();
        dam.lineage = "Aqua".into();
        for f in foals_with_chance(&sire, &dam, &db, 0.0, 300) {
            assert!(f.stats.speed <= 100.0);
            assert!(f.stats.dirt <= 100.0);
        }
    }

    #[test]
    fn a_mutated_temperament_still_pays_the_inbreeding_penalty_and_keeps_the_floor() {
        let mut all = founders();
        for mut h in full_siblings() {
            h.stats = flat_stats(3.0);
            h.lineage = "None".into();
            all.push(h);
        }
        let db = db_of(all);
        let fs = foals_with_chance(&db[&20], &db[&21], &db, 1.0, 300);
        assert!(fs.iter().all(|f| f.stats.temper >= MIN_TEMPER));
        assert!(fs.iter().all(|f| f.mutations.iter().any(|m| m == "temper")));
    }

    #[test]
    fn founders_never_carry_a_mutation() {
        assert!(founders().iter().all(|h| h.mutations.is_empty()));
    }

    #[test]
    fn about_six_percent_of_foals_carry_a_mutation() {
        // six stats at 1% each: 1 - 0.99^6 = 5.9%
        for seed in [1u64, 2, 3] {
            let foals: Vec<BloodHorse> = generate(seed).into_iter().filter(|h| h.gen > 1).collect();
            let mutants = foals.iter().filter(|h| !h.mutations.is_empty()).count();
            let share = mutants as f64 / foals.len() as f64;
            assert!(
                (0.03..0.09).contains(&share),
                "seed {seed}: {mutants}/{} = {share}",
                foals.len()
            );
        }
    }

    #[test]
    fn every_stat_over_100_is_a_listed_mutation_and_none_pass_125() {
        for seed in [1u64, 2, 3] {
            for h in generate(seed) {
                for name in STAT_NAMES {
                    let v = stat_named(&h, name);
                    assert!(v <= MUTATION_CAP, "{} {name} {v}", h.name);
                    if v > 100.0 {
                        assert!(
                            h.mutations.iter().any(|m| m == name),
                            "{} {name} {v} is not flagged",
                            h.name
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn mutation_lists_use_valid_stat_names_once_each_in_a_fixed_order() {
        for h in generate(4) {
            let positions: Vec<usize> = h
                .mutations
                .iter()
                .map(|m| {
                    STAT_NAMES
                        .iter()
                        .position(|n| n == m)
                        .unwrap_or_else(|| panic!("bad name {m}"))
                })
                .collect();
            assert!(
                positions.windows(2).all(|w| w[0] < w[1]),
                "{:?}",
                h.mutations
            );
        }
    }

    #[test]
    fn mutations_appear_in_json_as_stat_names_and_old_data_has_none() {
        let mut h = founders()[0].clone();
        h.mutations = vec!["speed".into(), "power".into()];
        let v = serde_json::to_value(&h).unwrap();
        assert_eq!(v["mutations"], serde_json::json!(["speed", "power"]));
        let mut old = serde_json::to_value(&founders()[1]).unwrap();
        old.as_object_mut().unwrap().remove("mutations");
        let back: BloodHorse = serde_json::from_value(old).unwrap();
        assert!(back.mutations.is_empty());
    }

    // ---- founders & the whole database

    #[test]
    fn the_founders_are_three_sires_and_five_dams() {
        let f = founders();
        assert_eq!(f.len(), 8);
        assert_eq!(f.iter().filter(|h| h.gender == Sex::Male).count(), 3);
        assert_eq!(f.iter().filter(|h| h.gender == Sex::Female).count(), 5);
        assert_eq!(
            f.iter().map(|h| h.id).collect::<Vec<_>>(),
            (1..=8).collect::<Vec<_>>()
        );
        assert!(f
            .iter()
            .all(|h| h.gen == 1 && h.sire_id.is_none() && h.dam_id.is_none()));
        let by = |lineage: &str| f.iter().find(|h| h.lineage == lineage).unwrap();
        assert_eq!(by("Ignis").stats.speed, 85.0);
        assert_eq!(by("Ignis").stats.temper, 30.0);
        assert_eq!(by("Aqua").stats.stamina, 85.0);
        assert_eq!(by("Terra").stats.dirt, 80.0);
        assert_eq!(by("Terra").growth_type, "晩成");
        assert_eq!(by("Ignis").growth_type, "普通");
        assert_eq!(by("Ignis").coat_color, "鹿毛");
        assert_eq!(by("Aqua").coat_color, "芦毛");
        assert_eq!(by("Terra").coat_color, "栗毛");
        assert!(f.iter().all(|h| is_valid_name(&h.name)));
    }

    #[test]
    fn the_database_has_the_scripted_number_of_horses_per_generation() {
        let all = generate(1);
        assert_eq!(all.len(), 8 + TARGET_COUNTS.iter().sum::<usize>());
        assert_eq!(all.len(), 1148);
        assert_eq!(all.iter().filter(|h| h.gen == 1).count(), 8);
        for (i, &n) in TARGET_COUNTS.iter().enumerate() {
            assert_eq!(
                all.iter().filter(|h| h.gen == i as u8 + 2).count(),
                n,
                "gen {}",
                i + 2
            );
        }
        assert_eq!(
            all.iter().map(|h| h.id).collect::<Vec<_>>(),
            (1..=1148).collect::<Vec<_>>()
        );
    }

    #[test]
    fn every_foal_has_real_parents_of_the_right_sex_from_an_earlier_generation() {
        let all = generate(2);
        let by_id: HashMap<u32, &BloodHorse> = all.iter().map(|h| (h.id, h)).collect();
        for h in all.iter().filter(|h| h.gen > 1) {
            let sire = by_id[&h.sire_id.expect("foal without a sire")];
            let dam = by_id[&h.dam_id.expect("foal without a dam")];
            assert_eq!(sire.gender, Sex::Male, "{} sire", h.name);
            assert_eq!(dam.gender, Sex::Female, "{} dam", h.name);
            assert!(sire.gen < h.gen && dam.gen < h.gen);
            assert!(sire.id < h.id && dam.id < h.id);
            assert_eq!(h.lineage, sire.lineage);
            assert_eq!(h.family, dam.family);
        }
    }

    #[test]
    fn generated_horses_are_internally_consistent() {
        for h in generate(3) {
            assert_eq!(h.coat_color, coat_color(&h.genes), "{}", h.name);
            let (kind, lo, hi) = growth_class(h.growth_factor);
            assert_eq!(h.growth_type, kind);
            assert!((lo..=hi).contains(&h.peak_age) || h.gen == 1);
            assert!(h.aptitude_min % 200 == 0 && h.aptitude_max % 200 == 0);
            // (a horse with exactly no speed and stamina gets the script's fixed 1200m;
            // the stats shown are rounded, so a 0.0 here may still be a tiny positive value)
            assert!(
                h.aptitude_min >= 1000 && h.aptitude_max <= 3600,
                "{}",
                h.name
            );
            assert!(
                h.aptitude_max > h.aptitude_min || (h.aptitude_min, h.aptitude_max) == (1200, 1200),
                "{}",
                h.name
            );
            assert!(
                RUNNING_STYLES.contains(&h.running_style.as_str()),
                "{}",
                h.running_style
            );
            assert!(h.inbreed_vol >= 0.0);
            assert!(["Ignis", "Aqua", "Terra", "None"].contains(&h.lineage.as_str()));
            for v in [
                h.stats.speed,
                h.stats.stamina,
                h.stats.temper,
                h.stats.power,
                h.stats.turf,
                h.stats.dirt,
            ] {
                assert!((0.0..=MUTATION_CAP).contains(&v));
            }
        }
    }

    #[test]
    fn horse_names_are_unique_and_follow_the_naming_rule() {
        let all = generate(4);
        let mut seen = HashSet::new();
        for h in &all {
            assert!(is_valid_name(&h.name), "{}", h.name);
            assert!(seen.insert(h.name.clone()), "duplicate {}", h.name);
        }
    }

    #[test]
    fn the_name_pools_never_overflow_nine_characters() {
        assert_eq!(NAME_PREFIXES.iter().collect::<HashSet<_>>().len(), 40);
        assert_eq!(NAME_SUFFIXES.iter().collect::<HashSet<_>>().len(), 40);
        for p in NAME_PREFIXES {
            for s in NAME_SUFFIXES {
                assert!(is_valid_name(&format!("{p}{s}")), "{p}{s}");
            }
        }
    }

    #[test]
    fn the_same_seed_gives_the_same_pedigree_and_another_seed_differs() {
        assert_eq!(generate(5), generate(5));
        assert_ne!(generate(5), generate(6));
    }

    fn gen_horses(all: &[BloodHorse], gen: u8) -> Vec<&BloodHorse> {
        all.iter().filter(|h| h.gen == gen).collect()
    }

    #[test]
    fn temperament_settles_at_a_realistic_level_in_every_later_generation() {
        for seed in [1u64, 2, 3, 4] {
            let all = generate(seed);
            for gen in 4..=10u8 {
                let v = gen_horses(&all, gen);
                let temper: Vec<f64> = v.iter().map(|h| h.stats.temper).collect();
                let mean = temper.iter().sum::<f64>() / temper.len() as f64;
                assert!(
                    (25.0..=55.0).contains(&mean),
                    "seed {seed} gen {gen}: mean temper {mean:.1}"
                );
                let at_floor = temper.iter().filter(|t| **t <= MIN_TEMPER).count();
                assert!(
                    (at_floor as f64) < temper.len() as f64 * 0.05,
                    "seed {seed} gen {gen}: {at_floor}/{} horses at the floor",
                    temper.len()
                );
            }
        }
    }

    #[test]
    fn stats_no_longer_pile_up_at_zero_or_a_hundred() {
        for seed in [1u64, 2, 3] {
            let all = generate(seed);
            let extreme = all
                .iter()
                .filter(|h| {
                    h.stats.speed == 0.0 || h.stats.stamina == 0.0 || h.stats.speed == 100.0
                })
                .count();
            assert!(
                extreme * 100 < all.len(),
                "seed {seed}: {extreme} extreme horses of {}",
                all.len()
            );
        }
    }

    #[test]
    fn inbreeding_stays_under_the_cap_and_eases_as_the_gene_pool_grows() {
        for seed in [1u64, 2, 3] {
            let all = generate(seed);
            assert!(all.iter().all(|h| h.inbreed_vol <= INBREED_CAP));
            let mean_vol = |gen: u8| {
                let v = gen_horses(&all, gen);
                v.iter().map(|h| h.inbreed_vol).sum::<f64>() / v.len() as f64
            };
            assert!(
                mean_vol(10) < mean_vol(4),
                "seed {seed}: {} vs {}",
                mean_vol(10),
                mean_vol(4)
            );
            assert!(mean_vol(10) < 0.06, "seed {seed}: {}", mean_vol(10));
        }
    }

    #[test]
    fn inbreeding_appears_once_the_founders_start_to_repeat() {
        let all = generate(7);
        let mean_vol = |gen: u8| {
            let v: Vec<f64> = all
                .iter()
                .filter(|h| h.gen == gen)
                .map(|h| h.inbreed_vol)
                .collect();
            v.iter().sum::<f64>() / v.len() as f64
        };
        // generation 2's parents are unrelated founders; later on, with only
        // eight founders, shared ancestors are everywhere
        assert_eq!(mean_vol(2), 0.0);
        assert!(mean_vol(4) > 0.1, "{}", mean_vol(4));
        assert!(all.iter().any(|h| h.inbreed_vol > 0.0));
    }

    #[test]
    fn json_keeps_the_scripts_gene_keys_and_uses_the_games_sex_names() {
        let h = &founders()[0];
        let v = serde_json::to_value(h).unwrap();
        assert_eq!(v["genes"]["G"], "gg");
        assert_eq!(v["genes"]["E"], "EE");
        assert_eq!(v["genes"]["A"], "AA");
        assert_eq!(v["gender"], "male");
        assert!(v["sire_id"].is_null());
        for field in [
            "id",
            "name",
            "lineage",
            "family",
            "gen",
            "coat_color",
            "growth_factor",
            "growth_type",
            "peak_age",
            "aptitude_min",
            "aptitude_max",
            "running_style",
            "personality",
            "inbreed_vol",
            "stats",
            "mutations",
            "owned",
        ] {
            assert!(v.get(field).is_some(), "missing {field}");
        }
        for stat in ["speed", "stamina", "temper", "power", "turf", "dirt"] {
            assert!(v["stats"].get(stat).is_some(), "missing stat {stat}");
        }
        let back: BloodHorse = serde_json::from_value(v).unwrap();
        assert_eq!(&back, h);
    }

    // ---- the database inside a game: sharing, nicks and previews

    #[test]
    fn the_base_database_is_built_once_per_seed_and_matches_generate() {
        let a = base_database(901);
        let b = base_database(901);
        assert!(std::sync::Arc::ptr_eq(&a, &b));
        assert_eq!(*a, generate(901));
        assert_ne!(*a, *base_database(902));
    }

    #[test]
    fn only_the_listed_crosses_are_nicks() {
        assert_eq!(nick_for("Ignis", "Aqua"), Some((StatKey::Speed, 7.0)));
        assert_eq!(nick_for("Terra", "Ignis"), Some((StatKey::Power, 5.0)));
        assert_eq!(nick_for("Aqua", "Ignis"), None);
        assert_eq!(nick_for("Ignis", "Ignis"), None);
        assert_eq!(nick_for("Ignis", "None"), None);
    }

    #[test]
    fn the_outlook_of_unrelated_parents_is_clean() {
        let (mut sire, mut dam, db) = fresh();
        sire.lineage = "Terra".into();
        dam.lineage = "None".into();
        let o = outlook(&sire, &dam, &db);
        assert_eq!((o.inbreed_vol, o.penalty, o.nick), (0.0, 0.0, None));
    }

    #[test]
    fn the_outlook_reports_inbreeding_and_the_penalty_it_costs() {
        // half siblings: coefficient 0.125, a gentle penalty
        let db = db_of(vec![
            mini(1, None, None, Sex::Male),
            mini(2, Some(1), None, Sex::Male),
            mini(3, Some(1), None, Sex::Female),
        ]);
        let o = outlook(&db[&2], &db[&3], &db);
        assert_eq!(o.inbreed_vol, 0.125);
        assert!((o.penalty - inbreed_penalty(0.125)).abs() < 1e-12);
        assert!(o.penalty > 5.0 && o.penalty < 8.0);
    }

    #[test]
    fn the_outlook_names_the_nick_with_its_stat_and_bonus() {
        let (mut sire, mut dam, db) = fresh();
        sire.lineage = "Ignis".into();
        dam.lineage = "Aqua".into();
        let o = outlook(&sire, &dam, &db);
        assert_eq!(
            o.nick,
            Some(Nick {
                stat: "speed".into(),
                bonus: 7.0
            })
        );
        sire.lineage = "Terra".into();
        dam.lineage = "Ignis".into();
        assert_eq!(
            outlook(&sire, &dam, &db).nick,
            Some(Nick {
                stat: "power".into(),
                bonus: 5.0
            })
        );
    }

    #[test]
    fn what_the_outlook_says_is_what_the_foal_gets() {
        let db = db_of(vec![
            mini(1, None, None, Sex::Male),
            mini(2, Some(1), None, Sex::Male),
            mini(3, Some(1), None, Sex::Female),
        ]);
        let o = outlook(&db[&2], &db[&3], &db);
        let foals = foals(&db[&2], &db[&3], &db, 5);
        assert!(foals.iter().all(|f| f.inbreed_vol == o.inbreed_vol));
    }

    #[test]
    fn the_owned_flag_defaults_to_false_and_is_in_the_json() {
        assert!(founders().iter().all(|h| !h.owned));
        assert!(generate(3).iter().all(|h| !h.owned));
        let mut old = serde_json::to_value(&founders()[0]).unwrap();
        old.as_object_mut().unwrap().remove("owned");
        let back: BloodHorse = serde_json::from_value(old).unwrap();
        assert!(!back.owned);
        let v = serde_json::to_value(&founders()[0]).unwrap();
        assert_eq!(v["owned"], false);
    }

    #[test]
    fn generated_names_never_use_the_stable_founders_names() {
        for seed in 0..30u64 {
            let all = generate(seed);
            for h in &all {
                assert!(
                    !STABLE_FOUNDERS.contains(&h.name.as_str()),
                    "seed {seed}: {}",
                    h.name
                );
            }
        }
        assert!(STABLE_FOUNDERS.iter().all(|n| is_valid_name(n)));
    }

    #[test]
    fn the_outlook_says_what_growth_the_foal_is_likely_to_have() {
        let parent = |factor: f64, gender: Sex| {
            let mut h = founders()[0].clone();
            h.growth_factor = factor;
            h.gender = gender;
            h
        };
        let db = HashMap::new();
        let early = outlook(&parent(0.1, Sex::Male), &parent(0.1, Sex::Female), &db).growth;
        let late = outlook(&parent(0.9, Sex::Male), &parent(0.9, Sex::Female), &db).growth;
        let mixed = outlook(&parent(0.1, Sex::Male), &parent(0.9, Sex::Female), &db).growth;
        assert!(early.early > 80, "{early:?}");
        assert!(late.late > 80, "{late:?}");
        assert!(
            mixed.normal > mixed.early && mixed.normal > mixed.late,
            "{mixed:?}"
        );
        for g in [early, late, mixed] {
            assert_eq!(g.early + g.normal + g.late, 100);
        }
    }

    #[test]
    fn foals_tend_to_take_after_their_parents_growth() {
        let mut rng = Rng::new(9);
        let share = |s: f64, d: f64, kind: &str, rng: &mut Rng| {
            (0..1000).filter(|_| growth(s, d, rng).1 == kind).count()
        };
        assert!(share(0.1, 0.1, "早熟", &mut rng) > 800);
        assert!(share(0.9, 0.9, "晩成", &mut rng) > 800);
        // not always: a late pair can still have an ordinary foal
        assert!(share(0.8, 0.8, "普通", &mut rng) > 50);
    }
}
