//! The data of the game: what is saved, what the screen is sent, and small helpers.

use super::*;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum GameError {
    UnknownHorse,
    InvalidAmount,
    InsufficientFunds,
    GameOver,
    MaxedOut,
    WrongSex,
    InvalidName,
    DuplicateName,
    NoSuchRace,
    /// The wrong number of horses for the kind of ticket.
    InvalidSelection,
    /// The same horse picked twice.
    DuplicateHorse,
    /// More horses entered than the limit.
    TooManyEntries,
    /// A horse that does not meet the race's conditions.
    NotEligible,
    /// A purchase with no tickets in it.
    NoTickets,
    /// More tickets than can be bought at once.
    TooManyTickets,
    /// Retiring the only horse left racing.
    LastHorse,
    /// No room left in the stud book.
    StudFull,
    /// A mating outside the breeding season.
    OutOfSeason,
    /// A mare already carrying a foal.
    MareInFoal,
    /// A mare still racing: only one retired to stud can be bred from.
    DamNotRetired,
    /// A stallion still racing: only one retired to stud can cover a mare.
    SireNotRetired,
    /// A bid outside the weeks of the sale.
    SaleClosed,
    /// A lot that is not in this year's sale.
    UnknownLot,
    /// A lot already sold or bid on.
    LotClosed,
    /// A farm facility at its top level.
    FacilityFull,
    /// A race abroad that does not exist.
    UnknownRace,
    /// A bid under the opening price.
    BidTooLow,
    /// A horse too young to retire.
    TooYoung,
    /// No room in the stable for a foal unless a horse makes way.
    StableFull,
    /// A horse that is not on offer.
    NotForSale,
    /// Too tired to be worked.
    TooTired,
    /// Resting (休養) this week.
    Resting,
    /// A jockey who does not exist.
    UnknownJockey,
    /// At pasture or injured.
    Away,
    /// Calling back a horse that is not at pasture.
    NotOnPasture,
    /// A trainer who does not exist.
    UnknownTrainer,
    /// A trainer's stable with no room for another horse.
    TrainerFull,
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = match self {
            GameError::UnknownHorse => "その馬は出走していません",
            GameError::InvalidAmount => "賭け金は1以上にしてください",
            GameError::InsufficientFunds => "所持金が足りません",
            GameError::GameOver => "ゲームオーバーです",
            GameError::MaxedOut => "能力はすでに上限です",
            GameError::WrongSex => "父は牡馬、母は牝馬を選んでください",
            GameError::InvalidName => "馬名は全角カタカナ2〜9文字で入力してください",
            GameError::DuplicateName => "その馬名はすでに使われています",
            GameError::NoSuchRace => "そのレースは今週は開催されません",
            GameError::InvalidSelection => "馬券の種類に合う頭数の馬を選んでください",
            GameError::DuplicateHorse => "同じ馬は選べません",
            GameError::TooManyEntries => "出走させられる自分の馬は最大3頭までです",
            GameError::NotEligible => "その馬はこのレースの出走条件を満たしていません",
            GameError::NoTickets => "買い目がありません",
            GameError::TooManyTickets => "一度に買える馬券は200点までです",
            GameError::LastHorse => "最後の1頭は引退させられません",
            GameError::StudFull => "繁殖馬は6頭までです。先に繁殖から外す馬を選んでください",
            GameError::StableFull => "厩舎がいっぱいです。入れ替える馬を選んでください",
            GameError::OutOfSeason => "種付けは繁殖シーズン(3月〜5月)の間だけです",
            GameError::MareInFoal => "その牝馬はすでに受胎しています",
            GameError::DamNotRetired => "母馬には引退した繁殖牝馬を選んでください",
            GameError::SireNotRetired => "父馬には引退した種牡馬か外部の種牡馬を選んでください",
            GameError::TooYoung => "2歳になるまで引退できません",
            GameError::SaleClosed => "今は受け付けていません",
            GameError::UnknownLot => "その上場馬はいません",
            GameError::LotClosed => "その上場馬のセリは終わっています",
            GameError::FacilityFull => "その施設はこれ以上拡張できません",
            GameError::UnknownRace => "そのレースはありません",
            GameError::BidTooLow => "開始価格より高い上限額を入れてください",
            GameError::UnknownJockey => "その騎手はいません",
            GameError::Away => "その馬は放牧、故障、育成、遠征のいずれかで不在です",
            GameError::NotOnPasture => "その馬は放牧に出ていません",
            GameError::UnknownTrainer => "その調教師はいません",
            GameError::TrainerFull => "その調教師の厩舎は満員です",
            GameError::NotForSale => "その馬は販売されていません",
            GameError::TooTired => "疲労がたまっています。休ませてください",
            GameError::Resting => "その馬は今週休養中です",
        };
        write!(f, "{msg}")
    }
}

/// One ticket in a purchase: a kind, the horses picked (in order) and the stake.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ticket {
    pub kind: BetKind,
    pub horses: Vec<String>,
    pub amount: u32,
}

/// How one ticket of a purchase turned out.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TicketResult {
    pub kind: BetKind,
    pub horses: Vec<String>,
    /// The selection as written: names joined by → (ordered) or - (unordered).
    pub label: String,
    pub amount: u32,
    pub odds: f64,
    pub won: bool,
    pub payout: i64,
}

/// How a set of horses is turned into tickets: ボックス or 流し.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BetStyle {
    Box,
    Flow,
}

/// Prize money one of the player's horses earned in a race.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrizeWon {
    pub name: String,
    /// Finishing place, 1-5.
    pub rank: u32,
    pub amount: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoundOutcome {
    pub results: Vec<RaceResult>,
    /// Runner positions (field order) after each second of the race.
    pub frames: Vec<Vec<f64>>,
    /// Live commentary, each line tied to the replay frame it appears at.
    pub commentary: Vec<CommentLine>,
    /// Runner names in the same order as each frame's positions.
    pub runners: Vec<String>,
    /// Their coat colours (毛色), in the same order.
    pub coats: Vec<String>,
    /// Set for calendar races (which advance the week).
    pub race_name: Option<String>,
    pub distance: f64,
    /// True if any ticket hit.
    pub won: bool,
    /// Paid out over all the tickets.
    pub payout: i64,
    /// How each ticket turned out.
    pub tickets: Vec<TicketResult>,
    /// Prize money my horses earned as their owner (calendar races only).
    pub prizes: Vec<PrizeWon>,
    pub prize_total: u32,
    pub money: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoryEntry {
    #[serde(default)]
    pub kind: BetKind,
    /// Name of the calendar race, if it was one.
    #[serde(default)]
    pub race: Option<String>,
    pub round: u32,
    /// The selection as written: a name, or names joined by → (ordered) or - (unordered).
    pub horse: String,
    /// The horses picked, in the order picked.
    #[serde(default)]
    pub horses: Vec<String>,
    pub stake: u32,
    pub won: bool,
    pub payout: i64,
    /// Prize money my horses earned in the race, as their owner.
    #[serde(default)]
    pub prize: u32,
    /// Tickets bought in the purchase (the other fields total them up).
    #[serde(default = "one")]
    pub ticket_count: u32,
    /// The player started the race without buying anything.
    #[serde(default)]
    pub watched: bool,
    /// The tickets were not all of one kind.
    #[serde(default)]
    pub mixed_kinds: bool,
    pub winner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameState {
    pub week: u8,
    pub year: u32,
    pub history: Vec<HistoryEntry>,
    pub money: u32,
    pub round: u32,
    pub game_over: bool,
    pub field: Vec<HorseEntry>,
    pub breed_cost: u32,
    /// Whether this week is in the breeding season, and the weeks it runs between.
    pub breeding_open: bool,
    pub breeding_start_week: u8,
    pub breeding_end_week: u8,
    /// The farm's facilities and what the next level of each would cost.
    pub farm: FarmEntry,
    /// The races abroad and who is away for them, and what a trip costs.
    pub overseas: Vec<OverseasEntry>,
    pub expeditions: Vec<ExpeditionEntry>,
    pub expedition_cost: u32,
    /// The sires used and how their lines stand, and what founds a line.
    pub sire_lines: Vec<SireLine>,
    pub line_foals: u32,
    pub line_g1_wins: u32,
    /// The owner's title and the goals to aim for.
    pub owner_rank: String,
    pub goals: Vec<Goal>,
    /// The foals on the way.
    pub pregnancies: Vec<PregnancyInfo>,
    /// Whether this week is in the yearling sale, and the weeks it runs between.
    pub auction_open: bool,
    pub auction_start_week: u8,
    pub auction_end_week: u8,
    /// Retired horses kept for breeding.
    pub studs: Vec<StudEntry>,
    /// What happened at the turn of this week (retirements).
    pub news: Vec<String>,
    pub max_studs: usize,
    pub stable_size: usize,
    /// The rivals racing now, honours won, the hall of fame and the trainers' records.
    pub stars: Vec<Star>,
    pub awards: Vec<Award>,
    pub hall: Vec<HallEntry>,
    pub trainer_records: Vec<TrainerRecord>,
    /// The jockeys who can be booked.
    pub jockeys: Vec<JockeyInfo>,
    /// The trainers, and how many horses each can take.
    pub trainers: Vec<TrainerInfo>,
    pub stable_capacity: usize,
    /// What the stable pays each week for its horses (預託料).
    pub weekly_upkeep: u32,
    /// How much a trainer's own distance adds to speed and stamina, in percent.
    pub trainer_bonus_percent: u32,
    /// The training menus (調教).
    pub workouts: Vec<WorkoutInfo>,
    /// ダートコース sessions per step of dirt aptitude.
    pub dirt_sessions: u8,
    /// From this fatigue a horse may not be entered in a race.
    pub race_fatigue_limit: u32,
    /// From this fatigue a horse may not be worked.
    pub train_fatigue_limit: u32,
    /// お任せ never spends the stable's money below this.
    pub auto_keep_money: u32,
}

/// What a horse's record in the blood database says about it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BloodTraits {
    pub lineage: String,
    pub family: String,
    pub gen: u8,
    pub coat_color: String,
    pub growth_type: String,
    pub peak_age: u8,
    pub aptitude_min: u32,
    pub aptitude_max: u32,
    pub running_style: String,
    pub personality: String,
    /// The database's temperament stat (higher is calmer).
    pub temper: f64,
    pub turf: f64,
    pub inbreed_vol: f64,
    pub mutations: Vec<String>,
}

/// A horse retired from racing and kept for breeding (繁殖馬).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Stud {
    pub name: String,
    pub sex: Sex,
    pub speed: u32,
    pub stamina: u32,
    pub power: u32,
    pub dirt: i8,
    /// Temperament at retirement (0 in saves from before it was kept).
    #[serde(default)]
    pub temper: u32,
    /// Its racing record (戦績).
    #[serde(default)]
    pub record: RaceRecord,
    pub blood_id: u32,
    pub retired_age: u8,
    pub retired_year: u32,
    /// Bought from the market rather than retired from the stable
    /// (`retired_year` is then the year it was bought).
    #[serde(default)]
    pub bought: bool,
    /// One of the pair a new game starts with.
    #[serde(default)]
    pub starter: bool,
    /// The strength of its legs (脚元).
    #[serde(default)]
    pub legs: u32,
    /// What it did on the track.
    #[serde(default)]
    pub career: Career,
}

/// Why a horse is away from the stable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AwayKind {
    Pasture,
    Injured,
    /// A foal at the farm, not yet in training.
    Growing,
    /// Abroad for a race.
    Expedition,
}

/// A horse away from the stable (放牧・故障), and for how long yet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Away {
    pub kind: AwayKind,
    pub label: String,
    pub weeks: u32,
}

/// The course a race is run on, as the race card shows it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RaceCourse {
    pub name: String,
    pub turn: String,
    /// The length of its final straight, in metres.
    pub straight: f64,
    /// The length of a lap, in metres.
    pub lap: f64,
    /// 内回り, 外回り or 直線, where a course has more than one way round.
    pub layout: Option<String>,
    /// From the goal on to the first corner, in metres.
    pub after_goal: f64,
    /// The corners the race goes round, whether it starts on the turf (a dirt race), and
    /// whether it is run on a straight course.
    pub corners: u8,
    pub turf_start: bool,
    pub straight_course: bool,
}

/// A great race abroad.
pub(super) struct OverseasRace {
    pub(super) name: &'static str,
    pub(super) country: &'static str,
    pub(super) week: u8,
    pub(super) surface: Surface,
    pub(super) distance: u32,
    pub(super) prize: u32,
}

/// A race abroad as the UI shows it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OverseasEntry {
    pub id: usize,
    pub name: String,
    pub country: String,
    pub week: u8,
    pub surface: Surface,
    pub distance: u32,
    pub first_prize: u32,
    /// Whether a horse can be sent for it now, and which could go.
    pub open: bool,
    pub candidates: Vec<String>,
}

/// A horse sent abroad, waiting for its race.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Expedition {
    pub(super) horse: String,
    pub(super) race: usize,
    pub(super) year: u32,
}

/// A trip abroad as the UI shows it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExpeditionEntry {
    pub horse: String,
    pub race: String,
    pub year: u32,
}

/// What a sire has got.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SireRecord {
    pub(super) foals: u32,
    pub(super) g1_wins: u32,
    pub(super) established: bool,
}

/// A sire and its line as the UI shows them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SireLine {
    pub sire: String,
    pub foals: u32,
    pub g1_wins: u32,
    pub established: bool,
}

/// Whether a horse can be sent for a race abroad in this week.
pub(super) fn expedition_open(race: &OverseasRace, week: u8) -> bool {
    week + EXPEDITION_LATEST <= race.week && race.week <= week + EXPEDITION_EARLIEST
}

/// A farm's facilities, by level.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct Farm {
    pub pasture: u8,
    pub nursery: u8,
}

/// A facility of the farm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Facility {
    /// 放牧地: stronger legs and more stamina in the foals born on it.
    Pasture,
    /// 育成施設: more speed and power in the yearlings brought on in it.
    Nursery,
}

/// The farm as the UI shows it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FarmEntry {
    pub pasture: u8,
    pub nursery: u8,
    pub max_level: u8,
    pub pasture_cost: Option<u32>,
    pub nursery_cost: Option<u32>,
    pub upkeep_per_level: u32,
}

/// What a horse's trainer has to say, and what the trainer would do this week.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Advice {
    pub horse: String,
    pub comment: String,
    pub proposal: Option<String>,
}

/// A step the trainer proposes.
pub(super) enum Proposal {
    Pasture,
    Rest,
    Work(Workout),
    Enter(CalendarRace),
}

impl Proposal {
    pub(super) fn label(&self) -> String {
        match self {
            Proposal::Pasture => "放牧に出す".to_string(),
            Proposal::Rest => "休養させる".to_string(),
            Proposal::Work(w) => format!("{}で調教", w.name()),
            Proposal::Enter(race) => format!("{}に出走", race.name),
        }
    }
}

/// A right to run in a G1, earned in its trial.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrialRight {
    pub(super) horse: String,
    pub(super) year: u32,
    pub(super) race: String,
}

pub(super) fn entry_rules_on() -> bool {
    true
}

/// The trial for a G1: the last graded race for the same horses on the same surface in
/// the weeks before it, if there is one in the same year.
pub(super) fn trial_of(g1: &CalendarRace) -> Option<CalendarRace> {
    calendar::shared()
        .races
        .iter()
        .filter(|r| {
            matches!(r.grade, Grade::G2 | Grade::G3)
                && r.surface == g1.surface
                && r.eligibility.age == g1.eligibility.age
                && r.eligibility.fillies_only == g1.eligibility.fillies_only
                && r.week < g1.week
                && g1.week - r.week <= TRIAL_WINDOW_WEEKS
        })
        .max_by_key(|r| (r.week, std::cmp::Reverse(r.distance.abs_diff(g1.distance))))
        .cloned()
}

/// The G1s a race is the trial for.
pub(super) fn trials_for(race: &CalendarRace) -> Vec<String> {
    if !matches!(race.grade, Grade::G2 | Grade::G3) {
        return vec![];
    }
    calendar::shared()
        .races
        .iter()
        .filter(|r| r.grade == Grade::G1)
        .filter(|g1| {
            trial_of(g1).is_some_and(|t| {
                (t.day, t.venue, t.race_no, t.week)
                    == (race.day, race.venue, race.race_no, race.week)
            })
        })
        .map(|g1| g1.name.clone())
        .collect()
}

/// Something an owner can aim for.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Goal {
    pub name: String,
    pub description: String,
    pub done: bool,
}

/// A foal on the way: made at the mating, it joins the stable when its time comes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pregnancy {
    pub(super) foal: BloodHorse,
    pub(super) legs: u32,
    pub(super) sire: String,
    pub(super) dam: String,
    /// The horse that makes way for it, if any.
    pub(super) replace: String,
    pub(super) weeks_left: u8,
}

/// A foal on the way as the UI shows it; what it will be stays unknown until it comes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PregnancyInfo {
    pub foal_name: String,
    pub sire: String,
    pub dam: String,
    pub replace: String,
    pub weeks_left: u8,
}

/// A lot in the yearling sale, as the buyer sees it: its breeding and an estimate, not
/// what it really is.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuctionLot {
    pub lot: u32,
    pub sire: String,
    pub dam: String,
    pub sex: Sex,
    pub estimate: String,
    pub start_price: u32,
    /// Sold or bid on already.
    pub closed: bool,
}

/// How a bid went: bought at the price, or lost to the others at it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BidOutcome {
    pub won: bool,
    pub price: u32,
}

/// A lot with what the seller knows: the horse itself and the others' limit.
pub(super) struct AuctionEntry {
    pub(super) lot: AuctionLot,
    pub(super) foal: BloodHorse,
    pub(super) legs: u32,
    pub(super) rival_top: u32,
}

/// A stallion or mare for sale to the stud book (繁殖馬の販売).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StudOffer {
    /// The strength of its legs (脚元).
    pub legs: u32,
    /// The fee to use a stallion without buying it (種付け料); none for a mare.
    pub stud_fee: u32,
    pub blood_id: u32,
    pub name: String,
    pub sex: Sex,
    pub speed: u32,
    pub stamina: u32,
    pub power: u32,
    pub dirt: i8,
    pub temper: u32,
    pub price: u32,
    pub blood: BloodTraits,
}

/// A stud as the UI shows it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StudEntry {
    /// What it did on the track.
    pub career: Career,
    /// How likely a mating is to take this year, in percent, for a mare.
    pub conception: u32,
    /// The strength of its legs (脚元).
    pub legs: u32,
    pub name: String,
    pub sex: Sex,
    pub speed: u32,
    pub stamina: u32,
    pub power: u32,
    pub dirt: i8,
    pub temper: u32,
    pub record: RaceRecord,
    pub retired_age: u8,
    pub retired_year: u32,
    pub bought: bool,
    pub starter: bool,
    pub pedigree: Option<Pedigree>,
    pub blood: Option<BloodTraits>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HorseEntry {
    /// The order given to its jockey (作戦), if any.
    pub order: Option<Style>,
    pub name: String,
    pub speed: u32,
    pub stamina: u32,
    pub train_cost: u32,
    pub condition: Condition,
    pub sex: Sex,
    pub age: u8,
    /// Dirt aptitude, -2..=2.
    pub dirt: i8,
    /// Power (0-100).
    pub power: u32,
    /// Bad-track aptitude (0-100).
    pub bad_track: f64,
    /// Temperament (気性, 0-100): trained by 曳き運動 and プール.
    pub temper: u32,
    /// ダートコース sessions towards the next step of dirt aptitude.
    pub dirt_progress: u8,
    /// Racing record (戦績).
    pub record: RaceRecord,
    /// Racing equipment (馬具).
    pub gear: Gear,
    /// Fatigue (疲労度, 0-100).
    pub fatigue: u32,
    /// Resting (休養) this week.
    pub resting: bool,
    /// Left to the trainer each week (お任せ調教).
    pub auto_training: bool,
    /// Entered in a suitable race each week (自動出走).
    pub auto_entry: bool,
    /// The trainer whose stable the horse is in (調教師).
    pub trainer: u8,
    /// What it would sell for.
    pub sale_price: u32,
    /// What it has done on the track.
    pub career: Career,
    /// The jockey chosen for it; none means its trainer books one.
    pub jockey: Option<u8>,
    /// The strength of its legs (脚元).
    pub legs: u32,
    /// At pasture or injured, and for how long.
    pub away: Option<Away>,
    pub pedigree: Option<Pedigree>,
    pub blood: Option<BloodTraits>,
}

/// The kind of ticket: 単勝, 複勝, 馬単, 馬連, 三連単 or 三連複 (see `bets`).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BetKind {
    #[default]
    Win,
    Place,
    Exacta,
    Quinella,
    Trifecta,
    Trio,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

/// Sex of the stable's founding horses (mares listed; everything else is a stallion).
pub(super) fn default_sex(name: &str) -> Sex {
    match name {
        "ヘキクウブレイブ" | "サクラフブキ" | "ミナモムーン" => Sex::Female,
        _ => Sex::Male,
    }
}

/// A horse's form for the coming race; visible to the player and priced into the odds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Condition {
    Great,
    Good,
    Normal,
    Poor,
}

impl Condition {
    /// Maps a uniform roll in [0, 1) to a condition: 10% great, 25% good, 40% normal, 25% poor.
    pub fn from_roll(r: f64) -> Condition {
        if r < 0.10 {
            Condition::Great
        } else if r < 0.35 {
            Condition::Good
        } else if r < 0.75 {
            Condition::Normal
        } else {
            Condition::Poor
        }
    }

    pub fn multiplier(self) -> f64 {
        match self {
            Condition::Great => 1.05,
            Condition::Good => 1.02,
            Condition::Normal => 1.0,
            Condition::Poor => 0.94,
        }
    }
}

/// A training menu (調教). Each raises one ability most and others a little.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Workout {
    /// 坂路: uphill gallops build power.
    Hill,
    /// ウッドコース追い: fast work on the wood-chip course builds speed.
    Wood,
    /// 併せ馬: working alongside another horse. Speed and power, but it gets keyed up.
    Pair,
    /// プール: swimming builds stamina without strain on the legs, and relaxes.
    Pool,
    /// 曳き運動: walking in hand settles the temperament.
    Walk,
    /// ダートコース: work on dirt teaches it the surface.
    Dirt,
}

/// An ability a workout can change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ability {
    Speed,
    Stamina,
    Power,
    Temper,
    /// Progress towards the next step of dirt aptitude.
    Dirt,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gain {
    pub ability: Ability,
    pub amount: i32,
}

/// A training menu as the UI shows it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkoutInfo {
    pub workout: Workout,
    pub name: String,
    pub description: String,
    /// What a session changes; the first is the main ability, and the menu
    /// cannot be done once that is at its limit.
    pub gains: Vec<Gain>,
    /// Fatigue a session adds.
    pub fatigue: u32,
}

impl Workout {
    pub const ALL: [Workout; 6] = [
        Workout::Wood,
        Workout::Hill,
        Workout::Pool,
        Workout::Pair,
        Workout::Walk,
        Workout::Dirt,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Workout::Hill => "坂路",
            Workout::Wood => "ウッドコース追い",
            Workout::Pair => "併せ馬",
            Workout::Pool => "プール",
            Workout::Walk => "曳き運動",
            Workout::Dirt => "ダートコース",
        }
    }

    pub(super) fn description(self) -> &'static str {
        match self {
            Workout::Hill => "坂を駆け上がって力をつけます。",
            Workout::Wood => "ウッドチップのコースで追い切り、速さを磨きます。",
            Workout::Pair => "他の馬と併せて走らせます。闘争心がつきますが、気が立ちます。",
            Workout::Pool => "脚に負担をかけずにスタミナをつけ、リラックスさせます。",
            Workout::Walk => "人が曳いて歩かせ、気性を落ち着かせます。",
            Workout::Dirt => "ダートを走らせて砂に慣れさせます。3回で適性が1段階上がります。",
        }
    }

    pub fn gains(self) -> Vec<Gain> {
        use Ability::*;
        let g = |ability, amount| Gain { ability, amount };
        match self {
            Workout::Hill => vec![g(Power, 3), g(Stamina, 1)],
            Workout::Wood => vec![g(Speed, 3), g(Stamina, 1)],
            Workout::Pair => vec![g(Speed, 2), g(Power, 2), g(Temper, -2)],
            Workout::Pool => vec![g(Stamina, 3), g(Temper, 1)],
            Workout::Walk => vec![g(Temper, 3)],
            Workout::Dirt => vec![g(Dirt, 1), g(Power, 1)],
        }
    }

    /// Fatigue a session adds (疲労度).
    pub fn fatigue(self) -> u32 {
        match self {
            Workout::Hill => 10,
            Workout::Wood => 8,
            Workout::Pair => 12,
            Workout::Pool => 3,
            Workout::Walk => 0,
            Workout::Dirt => 8,
        }
    }

    pub fn info(self) -> WorkoutInfo {
        WorkoutInfo {
            fatigue: self.fatigue(),
            workout: self,
            name: self.name().to_string(),
            description: self.description().to_string(),
            gains: self.gains(),
        }
    }
}

/// Which race of the current week, by its place on the weekend card.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RaceKey {
    pub day: Day,
    pub venue: Venue,
    pub race_no: u8,
}

impl RaceKey {
    pub fn of(race: &CalendarRace) -> Self {
        Self {
            day: race.day,
            venue: race.venue,
            race_no: race.race_no,
        }
    }
}

/// Everything needed to run (or price) one race.
#[derive(Debug, Clone, PartialEq)]
pub struct RaceSpec {
    pub race: Option<CalendarRace>,
    pub distance: u32,
    pub surface: Surface,
    pub entrants: Vec<Entrant>,
    pub track: TrackCondition,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardEntry {
    pub name: String,
    pub speed: u32,
    pub stamina: u32,
    pub condition: Condition,
    pub sex: Sex,
    pub age: u8,
    pub dirt: i8,
    /// Power (0-100).
    pub power: u32,
    /// Bad-track aptitude (0-100): mostly power, partly dirt aptitude.
    pub bad_track: f64,
    /// Growth stage (成長期) at this age: rising, peak or declining.
    pub growth: form::Growth,
    /// Share of its ability it shows at this age (1.0 in its prime).
    pub age_form: f64,
    /// The distances (m) it is best over.
    pub apt_min: u32,
    pub apt_max: u32,
    /// How this race's distance sits against that range.
    pub distance_fit: form::DistanceFit,
    /// Running style (脚質), if known.
    pub style: Option<Style>,
    /// Temperament (気性), 0-100.
    pub temper: u32,
    /// Coat colour (毛色).
    pub coat_color: String,
    /// Racing equipment (馬具).
    pub gear: Gear,
    pub mine: bool,
    /// Running over the distance its trainer favours.
    pub trainer_fit: bool,
    /// How the course suits it (none in the free race).
    pub course_fit: Option<CourseFit>,
    /// Who rides it, for the stable's own in a calendar race.
    pub jockey: Option<String>,
    /// What it carries, in kilos (none in the free race).
    pub weight: Option<u32>,
    /// One of the rivals who race on year after year.
    pub star: bool,
    pub odds: f64,
    pub place_odds: f64,
}

/// The field for a race with its odds, as shown before betting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RaceCard {
    /// `None` for the free race.
    pub race: Option<CalendarRace>,
    /// Where it is run.
    pub course: RaceCourse,
    /// The G1s this race is a trial for.
    pub trial_for: Vec<String>,
    pub distance: u32,
    pub surface: Surface,
    /// Track condition (馬場状態) on the day.
    pub track: TrackCondition,
    pub entrants: Vec<CardEntry>,
    /// Every horse in the stable: whether it may run in this race and whether it is entered.
    pub my_horses: Vec<MyHorse>,
    /// The most of the stable's horses that may be entered.
    pub max_entries: usize,
    /// How many horses are put forward for the race (登録頭数).
    pub nominated: usize,
    /// How many of them can run (フルゲート).
    pub gate: usize,
    /// How many horses a 複勝 pays on.
    pub place_slots: usize,
    /// What the owner gets for a win, in game money (0 for the free race).
    pub first_prize: u32,
}

/// One of the player's horses, as shown on the entry list of a race.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MyHorse {
    /// At pasture or injured.
    pub away: Option<Away>,
    /// Why the entry rules keep it out of this race, if they do.
    pub barred: Option<String>,
    pub name: String,
    /// Meets the race's age and sex conditions.
    pub eligible: bool,
    /// Put forward for the race (出走登録).
    pub registered: bool,
    /// Registered, but the draw went against it (除外).
    pub drawn_out: bool,
    /// Really runs in the race.
    pub entered: bool,
    /// The equipment it races in.
    pub gear: Gear,
    pub fatigue: u32,
    pub resting: bool,
}

pub(super) fn one() -> u32 {
    1
}

pub(super) fn default_week() -> u8 {
    1
}

pub(super) fn default_year() -> u32 {
    1
}

/// Ages of the founding horses.
pub(super) fn default_age(name: &str) -> u8 {
    match name {
        "ギンセイストーム" | "ホクトセイバー" => 5,
        "グレンキング" | "サクラフブキ" | "ミナモムーン" => 3,
        _ => 4,
    }
}

/// Dirt aptitude of the founding horses (-2 poor .. +2 specialist).
pub(super) fn default_dirt(name: &str) -> i8 {
    match name {
        "グレンキング" | "オウゴンオーシャン" => -1,
        "サクラフブキ" | "ホクトセイバー" => 1,
        "ミナモムーン" => 2,
        _ => 0,
    }
}

/// A horse's racing record (戦績): calendar races run and won.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RaceRecord {
    pub starts: u32,
    pub wins: u32,
}

/// Records the founding horses bring with them: older and stronger horses have won more.
pub(super) fn default_record(name: &str) -> RaceRecord {
    let (starts, wins) = match name {
        "ギンセイストーム" => (12, 4),
        "アサギリライズ" => (8, 3),
        "ホクトセイバー" => (10, 3),
        "オウゴンオーシャン" => (7, 2),
        "ヘキクウブレイブ" => (6, 1),
        "グレンキング" => (4, 1),
        "ミナモムーン" => (3, 0),
        "サクラフブキ" => (0, 0),
        _ => (0, 0),
    };
    RaceRecord { starts, wins }
}

/// Power of the founding horses: the dirt specialists are the powerful ones.
pub(super) fn default_power(name: &str) -> u32 {
    match name {
        "アサギリライズ" => 70,
        "ギンセイストーム" => 55,
        "グレンキング" => 80,
        "オウゴンオーシャン" => 75,
        "ヘキクウブレイブ" => 65,
        "サクラフブキ" => 70,
        "ホクトセイバー" => 85,
        "ミナモムーン" => 78,
        _ => 60,
    }
}

/// Bad-track aptitude from power and dirt aptitude, to one decimal.
pub(super) fn bad_track_of(power: u32, dirt: i8) -> f64 {
    let v = track::bad_track_aptitude(power as f64, track::dirt_score(dirt));
    (v * 10.0).round() / 10.0
}

/// The records the founding horses are linked to: for each (slot, sex), a distinct
/// record of that sex from the newest generation, chosen from the seed. The slots
/// take the lines still alive in that generation in turn, so the stable is not all
/// one line (which would leave no good crosses, since nicks cross two lines).
pub(super) fn founder_links(
    db: &[BloodHorse],
    world_seed: u64,
    horses: &[(usize, Sex)],
) -> Vec<u32> {
    let newest: Vec<&BloodHorse> = db.iter().filter(|h| h.gen == NEWEST_GENERATION).collect();
    let mut lines: Vec<&str> = newest.iter().map(|h| h.lineage.as_str()).collect();
    lines.sort();
    lines.dedup();
    let mut used = std::collections::HashSet::new();
    horses
        .iter()
        .map(|&(slot, sex)| {
            let of_sex: Vec<u32> = newest
                .iter()
                .filter(|h| h.gender == sex)
                .map(|h| h.id)
                .collect();
            let line = lines[slot % lines.len()];
            let mut pool: Vec<u32> = newest
                .iter()
                .filter(|h| h.gender == sex && h.lineage == line)
                .map(|h| h.id)
                .collect();
            if pool.is_empty() {
                pool = of_sex; // that line has no horse of this sex left
            }
            let mut n = rivals::mix_seed(&[world_seed, 0xB10D, slot as u64]) as usize % pool.len();
            loop {
                if used.insert(pool[n]) {
                    return pool[n];
                }
                n = (n + 1) % pool.len();
            }
        })
        .collect()
}

pub(super) fn blood_traits(r: BloodHorse) -> BloodTraits {
    BloodTraits {
        lineage: r.lineage,
        family: r.family,
        gen: r.gen,
        coat_color: r.coat_color,
        growth_type: r.growth_type,
        peak_age: r.peak_age,
        aptitude_min: r.aptitude_min,
        aptitude_max: r.aptitude_max,
        running_style: r.running_style,
        personality: r.personality,
        temper: r.stats.temper,
        turf: r.stats.turf,
        inbreed_vol: r.inbreed_vol,
        mutations: r.mutations,
    }
}

/// Fatigue from running a calendar race: longer races take more out of a horse.
pub(super) fn race_fatigue(distance: u32) -> u32 {
    20 + distance / 80
}

/// Share of its ability a horse this tired shows.
pub(super) fn fatigue_form(fatigue: u32) -> f64 {
    if fatigue <= FATIGUE_FREE {
        1.0
    } else {
        let over =
            (fatigue.min(MAX_FATIGUE) - FATIGUE_FREE) as f64 / (MAX_FATIGUE - FATIGUE_FREE) as f64;
        1.0 - FATIGUE_LOSS * over
    }
}

/// A horse in the racing stable, and everything kept about it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StableHorse {
    pub name: String,
    pub speed: u32,
    pub stamina: u32,
    /// Power (0-100).
    pub(super) power: u32,
    /// Dirt aptitude.
    pub(super) dirt: i8,
    /// Temperament (気性, 0-100): from the blood record, then trained.
    pub(super) temper: u32,
    pub(super) sex: Sex,
    pub(super) age: u8,
    /// The horse's record in the blood database.
    pub(super) blood_id: u32,
    pub condition: Condition,
    /// Training sessions done.
    pub(super) trainings: u32,
    /// ダートコース sessions towards the next step of dirt aptitude.
    pub(super) dirt_progress: u8,
    /// Racing record (戦績).
    pub(super) record: RaceRecord,
    /// What the horse has done on the track.
    pub(super) career: Career,
    /// Equipment the horse races in (馬具).
    pub(super) gear: Gear,
    /// Fatigue (疲労度, 0-100).
    pub(super) fatigue: u32,
    /// Resting (休養) this week.
    pub(super) resting: bool,
    /// Left to the trainer each week (お任せ調教).
    pub(super) auto_training: bool,
    /// Entered in a suitable race each week (自動出走).
    pub(super) auto_entry: bool,
    /// Whose stable the horse is in (調教師の id).
    pub(super) trainer: u8,
    /// Weeks since the horse last ran on the calendar.
    pub(super) since_race: u32,
    /// The strength of the horse's legs (脚元).
    pub(super) legs: u32,
    /// Time away (放牧・故障).
    pub(super) away: Option<Away>,
    /// The jockey chosen (none: the trainer books one).
    pub(super) jockey: Option<u8>,
    /// The last jockey and their races in a row together.
    pub(super) partner: Option<(u8, u32)>,
}

impl StableHorse {
    /// A horse the game starts with. Its blood record, temperament, legs and condition
    /// come later, once the game has its database.
    pub(super) fn starter(name: &str, speed: u32, stamina: u32, trainer: u8) -> Self {
        Self {
            name: name.to_string(),
            speed,
            stamina,
            power: default_power(name),
            dirt: default_dirt(name),
            temper: 0,
            sex: default_sex(name),
            age: default_age(name),
            blood_id: 0,
            condition: Condition::Normal,
            trainings: 0,
            dirt_progress: 0,
            record: default_record(name),
            career: Career::default(),
            gear: Gear::default(),
            fatigue: 0,
            resting: false,
            auto_training: false,
            auto_entry: false,
            trainer,
            since_race: NEVER_RACED,
            legs: 0,
            away: None,
            jockey: None,
            partner: None,
        }
    }
}

/// Shares the first horses out among the trainers, a stable each.
pub(super) fn starting_trainers(seed: u64, horses: usize) -> Vec<u8> {
    let mut ids: Vec<u8> = TRAINERS.iter().map(|t| t.id).collect();
    let mut rng = rivals::Rng::new(rivals::mix_seed(&[seed, 0x7EA1]));
    for i in (1..ids.len()).rev() {
        let j = rng.range(0, i as i64) as usize;
        ids.swap(i, j);
    }
    (0..horses).map(|i| ids[i % ids.len()]).collect()
}

/// What the trainer does with a horse left to them (お任せ).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AutoPlan {
    Work(Workout),
    Rest,
    Pasture,
}

pub(super) fn stat_sum(h: &BloodHorse) -> f64 {
    h.stats.speed + h.stats.stamina + h.stats.power
}

/// A stat from the blood database as a whole number (mutations may take it past 100).
pub(super) fn stat_of(v: f64) -> u32 {
    v.round().max(0.0) as u32
}

/// Dirt aptitude (-2..=2) from the blood database's dirt score.
pub(super) fn dirt_level(score: f64) -> i8 {
    ((score - 50.0) / 25.0).round().clamp(-2.0, 2.0) as i8
}

/// How likely a mating with a mare of this age is to take, in percent.
pub(super) fn conception_percent(age: u32) -> u32 {
    let fall = age.saturating_sub(CONCEPTION_FALLS_FROM) * CONCEPTION_FALL_PER_YEAR;
    CONCEPTION_PERCENT
        .saturating_sub(fall)
        .max(CONCEPTION_FLOOR)
}

/// A foal's stat with something added: only a mutation takes a stat past 100.
pub(super) fn grown(stat: f64, bonus: f64) -> f64 {
    if stat > 100.0 {
        stat
    } else {
        (stat + bonus).min(100.0)
    }
}

/// Whether the yearling sale is on in this week of the year.
pub(super) fn auction_open(week: u8) -> bool {
    (AUCTION_START_WEEK..=AUCTION_END_WEEK).contains(&week)
}

/// Whether a mating can be made in this week of the year.
pub(super) fn breeding_open(week: u8) -> bool {
    (BREEDING_START_WEEK..=BREEDING_END_WEEK).contains(&week)
}

/// What a horse costs for the stud book: dearer the stronger it is.
pub(super) fn stud_price(h: &BloodHorse) -> u32 {
    ((price_for_total(stat_sum(h)) / 10.0).round() * 10.0) as u32
}

/// What a horse with this stat total is worth.
pub(super) fn price_for_total(total: f64) -> f64 {
    200.0 + (total - 150.0).max(0.0) * 15.0
}

/// A temperament from the blood database as a whole number, 0-100.
pub(super) fn temper_of(v: f64) -> u32 {
    v.round().clamp(0.0, 100.0) as u32
}

/// A graded race a horse has won.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StakesWin {
    pub year: u32,
    pub race: String,
    pub grade: Grade,
}

/// What a horse has done on the track (実績).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Career {
    pub earnings: u32,
    pub stakes_wins: Vec<StakesWin>,
    pub titles: Vec<String>,
    /// So far this year, for 年度代表馬.
    pub year_earnings: u32,
    pub year_g1: u32,
}

impl Career {
    pub fn g1_wins(&self) -> usize {
        self.stakes_wins
            .iter()
            .filter(|w| w.grade == Grade::G1)
            .count()
    }

    pub(super) fn for_the_hall(&self) -> bool {
        self.g1_wins() >= HALL_G1_WINS
            || self.earnings >= HALL_EARNINGS
            || self
                .titles
                .iter()
                .any(|t| t == TITLE_CROWN || t == TITLE_FILLIES_CROWN)
    }
}

/// An honour won by a horse of the stable or a rival.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Award {
    pub year: u32,
    pub title: String,
    pub horse: String,
    pub mine: bool,
}

/// A horse in the stable's hall of fame (殿堂).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HallEntry {
    pub name: String,
    pub year: u32,
    pub wins: u32,
    pub g1_wins: u32,
    pub earnings: u32,
    pub titles: Vec<String>,
}

/// How a trainer has done with the stable's horses.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct TrainerRecord {
    pub starts: u32,
    pub wins: u32,
    pub g1_wins: u32,
    pub earnings: u32,
}

/// The legs of a horse from the database, fixed for the horse.
pub(super) fn legs_of_blood(id: u32) -> u32 {
    let roll = rivals::mix_seed(&[id as u64, 0x1E65]) % (LEGS_MAX - LEGS_MIN + 1) as u64;
    LEGS_MIN + roll as u32
}

/// The chance of breaking down: more when tired, and more on weak legs.
pub(super) fn injury_chance(base: f64, fatigue: u32, legs: u32) -> f64 {
    let tired = fatigue as f64 / 25.0;
    base * (1.0 + tired * tired) * (1.6 - legs as f64 / 100.0)
}

/// What can go wrong (故障), from the commonest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Injury {
    Soe,
    Fracture,
    Tendon,
}

impl Injury {
    pub(super) fn name(self) -> &'static str {
        match self {
            Injury::Soe => "ソエ",
            Injury::Fracture => "骨折",
            Injury::Tendon => "屈腱炎",
        }
    }

    pub(super) fn weeks(self) -> u32 {
        match self {
            Injury::Soe => 4,
            Injury::Fracture => 16,
            Injury::Tendon => 40,
        }
    }

    pub(super) fn vet_fee(self) -> u32 {
        match self {
            Injury::Soe => 50,
            Injury::Fracture => 200,
            Injury::Tendon => 400,
        }
    }

    /// Speed it never gets back.
    pub(super) fn speed_loss(self) -> u32 {
        match self {
            Injury::Tendon => 5,
            _ => 0,
        }
    }

    /// Which injury a roll in 0..100 means.
    pub(super) fn from_roll(roll: u64) -> Injury {
        match roll {
            0..=64 => Injury::Soe,
            65..=92 => Injury::Fracture,
            _ => Injury::Tendon,
        }
    }
}

/// Where a race is on the week's programme, which is what tells it from another.
pub(super) fn race_place(r: &CalendarRace) -> (Day, Venue, u8) {
    (r.day, r.venue, r.race_no)
}

/// What the odds are made from: each runner's (win, place) chances and the first
/// three home, as runner indexes, in each simulated race.
pub(super) struct Priced {
    pub(super) chances: Vec<(f64, f64)>,
    pub(super) orders: Vec<Vec<usize>>,
}

/// What the next session costs a horse that has had this many.
pub(super) fn session_cost(sessions: u32) -> u32 {
    (TRAIN_BASE_COST + TRAIN_COST_STEP * sessions).min(TRAIN_MAX_COST)
}

/// How a course suits a horse, on each surface.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VenueFit {
    pub venue: Venue,
    pub turf: CourseFit,
    pub dirt: CourseFit,
}

/// How a bid went, and the game after it.
#[derive(Serialize)]
pub struct BidResult {
    pub(super) won: bool,
    pub(super) price: u32,
    pub(super) state: GameState,
}
