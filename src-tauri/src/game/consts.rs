//! The numbers the game is balanced by.

use super::*;

/// The main gain of a session of the speed, stamina and power workouts (tests).
#[cfg(test)]
pub(super) const TRAIN_GAIN: u32 = 3;

/// Coat colour of a horse with no blood record.
pub(super) const DEFAULT_COAT: &str = "鹿毛";

/// Best dirt aptitude (◎).
pub(super) const MAX_DIRT: i8 = 2;

/// ダートコース sessions per step of dirt aptitude.
pub(super) const DIRT_SESSIONS: u8 = 3;

/// Training never makes a horse's temperament lower than this.
pub(super) const MIN_TEMPER: u32 = 1;

pub(super) const MAX_STAT: u32 = 100;

pub(super) const TRAIN_BASE_COST: u32 = 100;

pub(super) const TRAIN_COST_STEP: u32 = 10;

pub(super) const TRAIN_MAX_COST: u32 = 300;

/// What a stable pays each week for each horse in training, and each at stud (預託料).
pub(super) const UPKEEP_PER_HORSE: u32 = 8;

pub(super) const UPKEEP_PER_STUD: u32 = 5;

/// A horse's price: this much for any horse, this much per point of ability above
/// `SALE_FREE_STATS` in speed, stamina and power together, and this much for each win.
pub(super) const SALE_MIN: u32 = 100;

pub(super) const SALE_FREE_STATS: u32 = 150;

pub(super) const SALE_PER_STAT: u32 = 6;

pub(super) const SALE_PER_WIN: u32 = 50;

/// From this age a horse is worth a fifth less each year, down to `SALE_MIN`.
pub(super) const SALE_FULL_AGE: u8 = 5;

/// A market stallion's fee for covering a mare, as a share of its price (種付け料).
pub(super) const STUD_FEE_RATE: f64 = 0.2;

pub(super) const HISTORY_LIMIT: usize = 50;

pub(super) const DEFAULT_SEED: u64 = 0x2545_F491_4F6C_DD1D;

/// Races simulated to find each horse's chances (the odds come from them).
/// More is steadier; tests use fewer so the suite stays quick.
#[cfg(not(test))]
pub(super) const ODDS_SIMULATIONS: u64 = 1000;

#[cfg(test)]
pub(super) const ODDS_SIMULATIONS: u64 = 200;

pub(super) const ODDS_SEED_SALT: u64 = 0xD1B5_4A32_D192_ED03;

pub(super) const MAX_ODDS: f64 = 999.9;

/// A place ticket has three chances to come in, so its price stops far sooner.
pub(super) const MAX_PLACE_ODDS: f64 = 99.9;

/// Most tickets that can be bought in one purchase.
pub const MAX_TICKETS: usize = 200;

/// Longest odds a combination ticket can pay.
pub(super) const MAX_COMBO_ODDS: f64 = 9999.9;

/// How many simulated races the model behind combination odds counts for.
pub(super) const MODEL_WEIGHT: f64 = 100.0;

pub(super) const MIN_PLACE_ODDS: f64 = 1.0;

pub(super) const FIRST_THREE: usize = 3;

pub(super) const BREED_COST: u32 = 500;

/// The breeding season (種付けシーズン): March to May on the calendar, about three months.
pub const BREEDING_START_WEEK: u8 = 10;

pub const BREEDING_END_WEEK: u8 = 22;

/// From the mating to the foal's coming: about eleven months, as with real mares.
pub const GESTATION_WEEKS: u8 = 48;

/// How likely a mating is to take: this for a mare up to the age it starts to fall,
/// then less each year, down to a floor (受胎率).
pub(super) const CONCEPTION_PERCENT: u32 = 80;

pub(super) const CONCEPTION_FALLS_FROM: u32 = 10;

pub(super) const CONCEPTION_FALL_PER_YEAR: u32 = 4;

pub(super) const CONCEPTION_FLOOR: u32 = 40;

/// A foal is born at nought and stays at the farm this long before it starts training.
pub(super) const FOAL_FARM_WEEKS: u32 = calendar::WEEKS_PER_YEAR as u32;

/// The great races abroad.
pub(super) const OVERSEAS: [OverseasRace; 6] = [
    OverseasRace {
        name: "ドバイワールドカップ",
        country: "ドバイ",
        week: 13,
        surface: Surface::Dirt,
        distance: 2000,
        prize: 60000,
    },
    OverseasRace {
        name: "キングジョージ",
        country: "イギリス",
        week: 30,
        surface: Surface::Turf,
        distance: 2400,
        prize: 40000,
    },
    OverseasRace {
        name: "凱旋門賞",
        country: "フランス",
        week: 40,
        surface: Surface::Turf,
        distance: 2400,
        prize: 50000,
    },
    OverseasRace {
        name: "BCクラシック",
        country: "アメリカ",
        week: 44,
        surface: Surface::Dirt,
        distance: 2000,
        prize: 60000,
    },
    OverseasRace {
        name: "メルボルンカップ",
        country: "オーストラリア",
        week: 45,
        surface: Surface::Turf,
        distance: 3200,
        prize: 40000,
    },
    OverseasRace {
        name: "香港カップ",
        country: "香港",
        week: 50,
        surface: Surface::Turf,
        distance: 2000,
        prize: 30000,
    },
];

/// What a trip abroad costs, how far ahead it can be set up, and how long the horse
/// takes to get back after the race.
pub(super) const EXPEDITION_COST: u32 = 1500;

pub(super) const EXPEDITION_EARLIEST: u8 = 8;

pub(super) const EXPEDITION_LATEST: u8 = 2;

pub(super) const EXPEDITION_RETURN_WEEKS: u32 = 2;

/// Earnings that let a horse without a G1 go abroad.
pub(super) const EXPEDITION_MIN_EARNINGS: u32 = 5000;

/// How far the rivals abroad are above the best at home.
pub(super) const OVERSEAS_STRENGTH: u32 = 4;

/// What a sire needs to found a line, and what its line is worth to the theories.
pub(super) const LINE_FOALS: u32 = 5;

pub(super) const LINE_G1_WINS: u32 = 3;

pub(super) const LINE_THEORY_POINTS: u32 = 2;

/// What each level of a farm facility costs, and what it adds to the weekly bill.
pub(super) const FARM_COSTS: [u32; 3] = [800, 1600, 3200];

pub(super) const UPKEEP_PER_FACILITY_LEVEL: u32 = 5;

/// What a level of pasture adds to a foal's legs and stamina, and of nursery to its
/// speed and power when it comes in from the farm.
pub(super) const FARM_LEGS_PER_LEVEL: u32 = 3;

pub(super) const FARM_STAT_PER_LEVEL: u32 = 2;

/// What a point of the mating theories adds to each of a foal's main stats.
pub(super) const THEORY_STAT_PER_POINT: f64 = 0.5;

/// Earnings needed to run in a G1 without a right from its trial.
pub(super) const G1_MIN_EARNINGS: u32 = 200;

pub(super) const G1_MIN_EARNINGS_TWO: u32 = 100;

/// How long before a G1 its trial can be, and how many places earn the right to run.
pub(super) const TRIAL_WINDOW_WEEKS: u8 = 8;

pub(super) const TRIAL_PLACES: usize = 3;

/// How many places earn priority in the next race (優先出走権).
pub(super) const PRIORITY_PLACES: usize = 5;

/// What it costs a horse's temperament to be ridden against its natural way.
pub(super) const ORDER_TEMPER_COST: u32 = 10;

/// The yearling sale (1歳馬のセリ): held in July, a few lots a year, bought at this age.
pub const AUCTION_START_WEEK: u8 = 27;

pub const AUCTION_END_WEEK: u8 = 30;

pub(super) const AUCTION_LOTS: u32 = 6;

/// The G1 wins that make a legend of an owner.
pub(super) const RANK_LEGEND_G1: u32 = 30;

pub(super) const AUCTION_AGE: u8 = 1;

/// A winning bid pays this much over the most the others will go to.
pub(super) const AUCTION_STEP: u32 = 10;

/// How far off the estimate of a lot can be, in stat points.
pub(super) const AUCTION_ESTIMATE_NOISE: i64 = 15;

/// The estimated stat total from which a lot is rated A, and B.
pub(super) const AUCTION_GRADE_A: f64 = 185.0;

pub(super) const AUCTION_GRADE_B: f64 = 170.0;

/// The age from which a horse can race, and so retire.
pub const DEBUT_AGE: u8 = 2;

pub(super) const FOAL_MIN_STAT: u32 = 40;

/// A foal's stats may reach the mutation cap (125); ordinary ones stay under 100.
pub(super) const FOAL_MAX_STAT: u32 = 125;

pub(super) const START_MONEY: u32 = 3000;

/// Distance of the free (practice) race.
pub(super) const FREE_DISTANCE: u32 = 2400;

/// Game money per 10,000 yen of a race's listed prize money.
pub(super) const PRIZE_RATE: f64 = 0.05;

/// Frame at which the commentary notices a horse pulling.
pub(super) const KAKARI_CALL_FRAME: usize = 3;

/// Share of the first prize paid to each of the first five horses home.
pub(super) const PRIZE_SHARES: [f64; 5] = [1.0, 0.4, 0.25, 0.15, 0.10];

/// The most horses from the player's stable that can run in one race; rivals fill the rest.
pub const MAX_OWN_ENTRIES: usize = 3;

pub(super) const MAX_AGE: u8 = 10;

/// Most horses racing for the stable at once.
pub const STABLE_SIZE: usize = TRAINERS.len() * STABLE_CAPACITY;

/// What gelding a colt costs (去勢).
pub const GELDING_COST: u32 = 300;
/// How many points calmer it makes a horse.
pub const GELDING_CALM: u32 = 20;
/// Weeks at pasture getting over it.
pub const GELDING_WEEKS: u32 = 8;

/// The youngest a horse may go over to jumps.
pub(super) const JUMP_MIN_AGE: u8 = 3;
/// The jumping ability the jump exam (障害試験) asks for, and what it costs.
pub const JUMP_EXAM_LEVEL: u32 = 30;
pub const JUMP_EXAM_COST: u32 = 100;
/// What a horse going over to jumps has of what is in it, and the most it can reach
/// besides the exam's level.
pub(super) const JUMP_START_SHARE: f64 = 0.2;
pub(super) const JUMP_CEILING_SHARE: f64 = 0.7;
/// How much more a horse carries over fences than on the flat.
pub(super) const JUMP_EXTRA_KG: u32 = 4;
/// What a jump race can start, and how many others are put forward for one.
pub(super) const JUMP_GATE: usize = 14;
pub(super) const JUMP_RIVALS: (u64, u64) = (8, 13);
/// Points of jumping a jockey's hand is worth for each of the jockey's own above the average.
pub(super) const JUMP_JOCKEY_DIVISOR: i64 = 4;
pub(super) const JUMP_JOCKEY_AVERAGE: i64 = 60;
/// Chance of a horse that pulled up being hurt.
pub(super) const STOP_INJURY: f64 = 0.3;

/// Most retired horses kept for breeding.
pub const MAX_STUDS: usize = 6;

/// Age at which a horse retires to stud at the turn of the year.
pub const RETIRE_AGE: u8 = 9;

/// A stable with fewer racing horses than this is running out of them, and with
/// fewer successors than this will be.
pub const MIN_RACERS: usize = 4;

/// Years from a mating to the foal's first race: a year in the mare, then two growing.
pub const FOAL_LEAD_YEARS: u8 = 3;

/// Dirt aptitude moves a horse's speed by this fraction per step on dirt.
pub(super) const DIRT_STEP: f64 = 0.05;

pub(super) const MIN_ODDS: f64 = 1.1;

/// Share of the pool returned to bettors (the rest is the house edge).
pub(super) const PAYOUT_RATE: f64 = 0.85;

/// The age up to which a horse's peak counts as early, and from which as late.
pub(super) const EARLY_PEAK_AGE: u8 = 3;

pub(super) const LATE_PEAK_AGE: u8 = 5;

/// Legs below this make a trainer careful.
pub(super) const WEAK_LEGS: u32 = 50;

/// The founding horses, in order; their slot picks the database record each is linked to.
pub(super) const FOUNDING: [&str; 8] = bloodline::STABLE_FOUNDERS;

/// The youngest generation of the database, from which the founding horses are drawn.
pub(super) const NEWEST_GENERATION: u8 = 10;

/// Fatigue (疲労度) runs from 0 (fresh) to this.
pub(super) const MAX_FATIGUE: u32 = 100;

/// Fatigue a horse sheds over a week.
pub(super) const WEEKLY_RECOVERY: u32 = 15;

/// Fatigue a week's rest (休養) takes away at once.
pub(super) const REST_RECOVERY: u32 = 30;

/// From this fatigue a horse may not be entered in a race.
pub const RACE_FATIGUE_LIMIT: u32 = 50;

/// From this fatigue a horse may not be worked.
pub(super) const TRAIN_FATIGUE_LIMIT: u32 = 90;

/// Up to this fatigue a horse runs at its best.
pub(super) const FATIGUE_FREE: u32 = 30;

/// Ability a horse loses at the most fatigue.
pub(super) const FATIGUE_LOSS: f64 = 0.2;

/// お任せ never spends the stable's money below this.
pub const AUTO_KEEP_MONEY: u32 = 500;

/// A horse hotter than this is walked in hand before anything else.
pub(super) const AUTO_CALM_TEMPER: u32 = 40;

/// お任せ also keeps back this many weeks of the stable's upkeep, if that is more.
pub(super) const AUTO_KEEP_WEEKS: u32 = 24;

/// How far behind a trainer counts the ability their distance calls for.
pub(super) const AUTO_DISTANCE_BIAS: i64 = 5;

/// Weeks ahead a trainer aiming at the G1 keeps a horse fresh for one.
pub(super) const AUTO_G1_LEAD: u8 = 4;

/// Weeks since the last race of a horse that has not run.
pub(super) const NEVER_RACED: u32 = 99;

/// Generations old enough to be on the market.
pub(super) const MARKET_MIN_GEN: u8 = 8;

/// The best horses of each sex the market chooses from.
pub(super) const MARKET_POOL: usize = 40;

/// Horses of each sex on offer at a time.
pub(super) const MARKET_EACH: usize = 4;

/// The Classics: a three-year-old that wins all three in a year takes the crown.
pub(super) const TRIPLE_CROWN: [&str; 3] = ["疾風賞", "王冠ステークス", "錦秋ロングステークス"];

pub(super) const FILLIES_CROWN: [&str; 3] = ["白梅賞", "月下美人賞", "秋桜ティアラ"];

pub(super) const TITLE_CROWN: &str = "三冠";

pub(super) const TITLE_FILLIES_CROWN: &str = "牝馬三冠";

pub(super) const TITLE_HORSE_OF_THE_YEAR: &str = "年度代表馬";

pub(super) const TITLE_BEST_JUMPER: &str = "最優秀障害馬";

pub(super) const TITLE_LINE: &str = "系統確立";

pub(super) const OVERSEAS_AWARD: &str = "海外G1 ";

/// What gets a retired horse into the hall of fame (殿堂).
pub(super) const HALL_EARNINGS: u32 = 15000;

pub(super) const HALL_G1_WINS: usize = 5;

/// Rivals born each year, the age they stop racing at, the most in one race, and the
/// weeks between their races; how far above itself a rival will try.
pub(super) const STARS_PER_YEAR: usize = 3;

pub(super) const STAR_FIRST_AGE: u8 = 2;

pub(super) const STAR_LAST_START_AGE: u8 = 6;

pub(super) const STAR_RETIRE_AGE: u8 = 7;

pub(super) const STARS_PER_RACE: usize = 3;

pub(super) const STAR_GAP: u32 = 5;

pub(super) const STAR_REACH: f64 = 4.0;

/// Speed above its class that counts as one win for a rival in an allowance race.
pub(super) const RIVAL_SPEED_PER_WIN: u32 = 2;

/// The strength of a horse's legs (脚元), and how much a foal can differ from its parents.
pub(super) const LEGS_MIN: u32 = 40;

pub(super) const LEGS_MAX: u32 = 90;

pub(super) const FOAL_LEGS_SPREAD: u32 = 8;

/// The chance of breaking down in a race and in a session, for a fresh horse of average legs.
pub(super) const RACE_INJURY: f64 = 0.002;

pub(super) const WORKOUT_INJURY: f64 = 0.001;

/// Weeks a horse goes to pasture for (放牧).
pub(super) const PASTURE_WEEKS: u32 = 4;

/// From this fatigue a trainer turns a horse out.
pub(super) const AUTO_PASTURE_FATIGUE: u32 = 70;
