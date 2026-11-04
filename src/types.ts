// What the game and the screen say to each other.

import { CalendarRace, Grade } from "./Calendar";

/** Racing equipment (馬具). */
export interface LeadingRow {
  name: string;
  wins: number;
  own_wins: number;
}

export interface Leading {
  jockeys: LeadingRow[];
  trainers: LeadingRow[];
  sires: LeadingRow[];
}

export interface LeadingYear {
  year: number;
  jockey: string;
  trainer: string;
  sire: string;
}

export interface Gear {
  menko: boolean;
  shadow_roll: boolean;
  blinkers?: boolean;
}

/** A racing record (戦績): calendar races run and won. */
export interface RaceRecord {
  starts: number;
  wins: number;
}

export type Condition = "great" | "good" | "normal" | "poor";

export type Sex = "male" | "female";

export type BetKind = "win" | "place" | "wide" | "exacta" | "quinella" | "trifecta" | "trio";

export type Track = "good" | "soft" | "yielding" | "heavy";

export interface Pedigree {
  sire: string;
  dam: string;
  sire_sire: string;
  sire_dam: string;
  dam_sire: string;
  dam_dam: string;
}

/** What a horse's record in the blood database says about it. */
export interface BloodTraits {
  lineage: string;
  family: string;
  gen: number;
  coat_color: string;
  growth_type: string;
  peak_age: number;
  aptitude_min: number;
  aptitude_max: number;
  running_style: string;
  personality: string;
  /** What it has in it over fences. */
  jump?: number;
  inbreed_vol: number;
  /** Stats that mutated, by name. */
  mutations: string[];
}

/** What a mating would bring, before it is made. */
export interface Outlook {
  /** What the mating theories say of the pair, and the rating they come to. */
  rating?: string;
  theories?: { name: string; points: number }[];
  inbreed_vol: number;
  /** Temperament points the inbreeding would cost. */
  penalty: number;
  nick: { stat: string; bonus: number } | null;
  /** How the foal is likely to grow up, in percent. */
  growth?: { early: number; normal: number; late: number };
}

export interface HorseEntry {
  name: string;
  speed: number;
  stamina: number;
  train_cost: number;
  condition: Condition;
  sex: Sex;
  age?: number;
  /** Dirt aptitude, -2..2. */
  dirt?: number;
  /** Power (0-100). */
  power?: number;
  /** Bad-track aptitude (0-100). */
  bad_track?: number;
  /** Temperament (気性, 0-100). */
  temper?: number;
  /** ダートコース sessions towards the next step of dirt aptitude. */
  dirt_progress?: number;
  /** Racing record (戦績). */
  record?: RaceRecord;
  /** Racing equipment (馬具). */
  gear?: Gear;
  /** Fatigue (疲労度, 0-100). */
  fatigue?: number;
  /** Resting (休養) this week. */
  resting?: boolean;
  /** Left to the trainer each week (お任せ調教). */
  auto_training?: boolean;
  /** Entered in a suitable race each week (自動出走). */
  auto_entry?: boolean;
  /** The id of the trainer whose stable it is in. */
  trainer?: number;
  /** What it would sell for. */
  sale_price?: number;
  gelded?: boolean;
  jumper?: boolean;
  jump?: number;
  jump_license?: boolean;
  jump_wins?: number;
  /** The strength of its legs (脚元). */
  legs?: number;
  /** At pasture or injured. */
  away?: Away | null;
  /** The jockey chosen for it; none means its trainer books one. */
  jockey?: number | null;
  /** What it has done on the track. */
  career?: Career;
  /** The order given to its jockey (作戦), if any. */
  order?: RunningStyle | null;
  pedigree: Pedigree | null;
  blood?: BloodTraits | null;
}

export interface HistoryEntry {
  kind: BetKind;
  race?: string | null;
  /** Prize money my horses earned in the race. */
  prize?: number;
  /** Tickets in the purchase; the other fields are totals. Absent in older data (one ticket). */
  ticket_count?: number;
  watched?: boolean;
  mixed_kinds?: boolean;
  round: number;
  horse: string;
  stake: number;
  won: boolean;
  payout: number;
  winner: string;
}

export type RunningStyle = "nige" | "senko" | "sashi" | "oikomi";

/** A lot in the yearling sale: its breeding and an estimate, not what it really is. */
export interface AuctionLot {
  lot: number;
  sire: string;
  dam: string;
  sex: Sex;
  estimate: string;
  start_price: number;
  closed: boolean;
}

/** A foal on the way. */
export interface Pregnancy {
  foal_name: string;
  sire: string;
  dam: string;
  replace: string;
  weeks_left: number;
}

export interface GameState {
  week: number;
  year: number;
  history: HistoryEntry[];
  breed_cost: number;
  /** Whether this week is in the breeding season, and the weeks it runs between. */
  breeding_open?: boolean;
  breeding_start_week?: number;
  breeding_end_week?: number;
  /** The foals on the way. */
  pregnancies?: Pregnancy[];
  /** The races abroad and who is away for them, and what a trip costs. */
  overseas?: { id: number; name: string; country: string; week: number; surface: "turf" | "dirt"; distance: number; first_prize: number; open: boolean; candidates: string[] }[];
  expeditions?: { horse: string; race: string; year: number }[];
  expedition_cost?: number;
  /** The sires used and how their lines stand. */
  sire_lines?: { sire: string; foals: number; g1_wins: number; established: boolean }[];
  line_foals?: number;
  line_g1_wins?: number;
  /** The farm's facilities and what the next level of each would cost. */
  farm?: { pasture: number; nursery: number; max_level: number; pasture_cost?: number | null; nursery_cost?: number | null; upkeep_per_level: number };
  /** The owner's title and the goals to aim for. */
  owner_rank?: string;
  goals?: { name: string; description: string; done: boolean }[];
  /** Whether this week is in the yearling sale, and the weeks it runs between. */
  auction_open?: boolean;
  auction_start_week?: number;
  auction_end_week?: number;
  money: number;
  round: number;
  game_over: boolean;
  field: HorseEntry[];
  /** Retired horses kept for breeding. */
  studs?: StudEntry[];
  /** What happened at the turn of this week (retirements). */
  news?: string[];
  max_studs?: number;
  stable_size?: number;
  /** The training menus (調教). */
  workouts?: WorkoutInfo[];
  /** ダートコース sessions per step of dirt aptitude. */
  dirt_sessions?: number;
  /** From this fatigue a horse may not be entered in a race. */
  race_fatigue_limit?: number;
  /** From this fatigue a horse may not be worked. */
  train_fatigue_limit?: number;
  /** What the stable pays each week for its horses (預託料). */
  weekly_upkeep?: number;
  /** What gelding a colt costs. */
  gelding_cost?: number;
  jump_exam_level?: number;
  jump_exam_cost?: number;
  leading?: Leading;
  leading_years?: LeadingYear[];
  /** The rivals racing now, honours won, the hall of fame and the trainers' records. */
  stars?: Star[];
  awards?: Award[];
  hall?: HallEntry[];
  trainer_records?: TrainerRecord[];
  /** The jockeys who can be booked. */
  jockeys?: JockeyInfo[];
  /** The trainers (調教師) and how many horses each stable holds. */
  trainers?: TrainerInfo[];
  stable_capacity?: number;
  /** How much a trainer's own distance adds to speed and stamina, in percent. */
  trainer_bonus_percent?: number;
  /** お任せ never spends the stable's money below this. */
  auto_keep_money?: number;
  /** The horses old enough to race. */
  racers?: number;
  /** The horses still racing when a foal mated now would first run. */
  successors?: number;
  min_racers?: number;
  short_of_racers?: boolean;
  short_of_successors?: boolean;
}

/** A graded race a horse has won. */
export interface StakesWin {
  year: number;
  race: string;
  grade: Grade;
}

/** What a horse has done on the track (実績). */
export interface Career {
  earnings: number;
  stakes_wins: StakesWin[];
  titles: string[];
}

/** A rival who races on year after year (ライバル). */
export interface Star {
  name: string;
  sex: Sex;
  age: number;
  starts: number;
  wins: number;
  g1_wins: number;
  earnings: number;
  met: number;
  beaten: number;
}

export interface Award {
  year: number;
  title: string;
  horse: string;
  mine: boolean;
}

export interface HallEntry {
  name: string;
  year: number;
  wins: number;
  g1_wins: number;
  earnings: number;
  titles: string[];
}

export interface TrainerRecord {
  starts: number;
  wins: number;
  g1_wins: number;
  earnings: number;
}

/** A jockey and their ways. */
export interface JockeyInfo {
  /** Their hand over fences. */
  jump?: number;
  id: number;
  name: string;
  /** Where they are based: 美浦 or 栗東. */
  region?: string;
  skill: number;
  style: RunStyle;
  turn: "left" | "right";
  tact: number;
  fee: number;
  description: string;
}

/** A horse away from the stable (放牧・故障), and for how long yet. */
export interface Away {
  kind: "pasture" | "injured" | "growing" | "expedition";
  label: string;
  weeks: number;
}

/** One track of a racecourse. */
export interface TrackInfo {
  lap: string;
  straight: string;
  elevation: string;
}

/** A racecourse (競馬場) and its layout. */
export interface CourseInfo {
  /** A map for each way round the turf and for the dirt. */
  maps?: { label: string; surface: "turf" | "dirt"; layout?: string | null; lap: number; straight: number; after_goal: number; straight_course?: boolean }[];
  venue: string;
  name: string;
  place: string;
  turn: "left" | "right";
  grass: string;
  turf: TrackInfo;
  dirt: TrackInfo;
  features: string;
  suits: string;
}

export type Factor = "plus" | "even" | "minus";

/** How a course suits a horse. */
export interface CourseFit {
  turn: Factor;
  straight: Factor;
  hill: Factor;
  turf: Factor;
  percent: number;
  rating: "great" | "good" | "fair" | "poor";
}

export interface VenueFit {
  venue: string;
  turf: CourseFit;
  dirt: CourseFit;
}

/** A trainer and their ways, in words. */
export interface TrainerInfo {
  id: number;
  name: string;
  /** Where their stable is: 美浦 or 栗東. */
  region?: string;
  training: string;
  distance: string;
  rotation: string;
  aim: string;
  description: string;
}

export type Ability = "speed" | "stamina" | "power" | "temper" | "dirt" | "jump";

export interface WorkoutInfo {
  workout: string;
  name: string;
  description: string;
  gains: { ability: Ability; amount: number }[];
  /** Fatigue a session adds. */
  fatigue?: number;
}

/** A stallion or mare on sale for the stud book. */
export interface StudOffer {
  /** The strength of its legs (脚元). */
  legs?: number;
  /** The fee to use a stallion without buying it (種付け料); none for a mare. */
  stud_fee?: number;
  blood_id: number;
  name: string;
  sex: Sex;
  speed: number;
  stamina: number;
  power: number;
  dirt: number;
  temper: number;
  price: number;
  blood: NonNullable<HorseEntry["blood"]>;
}

/** A horse retired from racing (or bought) and kept for breeding (繁殖馬). */
export interface StudEntry {
  /** How likely a mating is to take this year, in percent, for a mare. */
  conception?: number;
  /** The strength of its legs (脚元). */
  legs?: number;
  name: string;
  sex: Sex;
  speed: number;
  stamina: number;
  power: number;
  dirt: number;
  retired_age: number;
  retired_year: number;
  /** Bought rather than retired (`retired_year` is then the year it was bought). */
  bought?: boolean;
  /** One of the pair a new game starts with. */
  starter?: boolean;
  record?: RaceRecord;
  pedigree: Pedigree | null;
  blood: HorseEntry["blood"];
}

export type Weather = "sunny" | "cloudy" | "light_rain" | "rain" | "snow";

export interface RaceResult {
  rank: number;
  name: string;
  finish_time: number;
  /** How far behind the horse in front (着差); empty for the winner. */
  margin?: string;
}

export interface CommentLine {
  frame: number;
  text: string;
}

export interface TicketResult {
  kind: BetKind;
  horses: string[];
  label: string;
  amount: number;
  odds: number;
  won: boolean;
  payout: number;
}

/** A ticket waiting in the list to be bought with the others. */
export interface SlipTicket {
  kind: BetKind;
  horses: string[];
  amount: number;
}

export type BuyStyle = "normal" | "box" | "flow";

export interface RoundOutcome {
  /** Those who started and did not finish. */
  stopped?: string[];
  results: RaceResult[];
  frames: number[][];
  commentary: CommentLine[];
  /** Runner names in frame order; absent means the stable. */
  runners?: string[];
  /** Their coat colours (毛色), in the same order. */
  coats?: string[];
  /** Set for calendar races, which move the game on a week. */
  race_name?: string | null;
  /** How each ticket turned out. */
  tickets?: TicketResult[];
  /** Prize money my horses earned as their owner. Absent in older data. */
  prizes?: { name: string; rank: number; amount: number }[];
  prize_total?: number;
  distance: number;
  won: boolean;
  payout: number;
  money: number;
}

/** A runner as shown in a race's field. */
export interface Runner {
  name: string;
  speed: number;
  stamina: number;
  condition: Condition;
  sex: Sex;
  age?: number;
  dirt?: number;
  power?: number;
  bad_track?: number;
  growth?: "rising" | "peak" | "declining";
  age_form?: number;
  apt_min?: number;
  apt_max?: number;
  distance_fit?: "fit" | "too_short" | "too_long";
  /** Running over the distance its trainer favours. */
  trainer_fit?: boolean;
  /** How the course suits it (none in the free race). */
  course_fit?: CourseFit | null;
  /** Who rides it, for my horses in a calendar race. */
  jockey?: string | null;
  /** What it carries, in kilos (none in the free race). */
  weight?: number | null;
  /** One of the rivals who race on year after year. */
  star?: boolean;
  style?: RunStyle | null;
  temper?: number;
  coat_color?: string;
  gear?: Gear;
  mine: boolean;
  odds: number;
  place_odds: number;
  gelded?: boolean;
  jump?: number;
  /** What the season is doing to it this week. */
  season?: string | null;
  /** Its place in the betting (人気). */
  popularity?: number;
}

export interface RaceCard {
  race: CalendarRace | null;
  /** Where it is run. */
  course?: {
    name: string;
    turn: string;
    straight: number;
    lap?: number;
    layout?: string | null;
    after_goal?: number;
    corners?: number;
    turf_start?: boolean;
    straight_course?: boolean;
  };
  distance: number;
  surface: "turf" | "dirt" | "jump";
  /** Track condition (馬場状態) on the day. */
  track?: Track;
  weather?: Weather;
  /** How many fences there are (none on the flat). */
  fences?: number;
  entrants: Runner[];
  /** Every horse in the stable: whether it can run in this race and whether it is entered. */
  my_horses: { name: string; eligible: boolean; entered: boolean; gear?: Gear; fatigue?: number; resting?: boolean; away?: Away | null; barred?: string | null; registered?: boolean; drawn_out?: boolean }[];
  /** The G1s this race is a trial for. */
  trial_for?: string[];
  /** The most of my horses that may run. */
  max_entries: number;
  /** How many horses are put forward for the race, and how many of them can run. */
  nominated?: number;
  gate?: number;
  /** How many horses a place ticket pays on. */
  place_slots?: number;
  /** What the owner gets for a win, in game money (0 for the free race). */
  first_prize?: number;
}

export interface RaceKey {
  day: "sat" | "sun";
  venue: string;
  race_no: number;
}

export type TabId = "race" | "train" | "stable" | "pedigree" | "breeding" | "calendar" | "info" | "honours";

export type RunStyle = "nige" | "senko" | "sashi" | "oikomi";
