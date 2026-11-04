//! Odds and tickets, and running a race the player has bet on.

use super::*;

impl Game {
    /// Win and place chances for each runner (spec order), estimated by running
    /// many fresh races with fixed seeds, so the odds always reflect how the
    /// race really plays out and stay identical between display and payout.
    pub(super) fn chances(spec: &RaceSpec) -> Vec<(f64, f64)> {
        Self::chances_n(spec, ODDS_SIMULATIONS)
    }

    /// [`Self::chances`] with the number of simulated races given.
    pub(super) fn chances_n(spec: &RaceSpec, sims: u64) -> Vec<(f64, f64)> {
        Self::priced_n(spec, sims).chances
    }

    /// What the odds of every kind of ticket are made from.
    pub(super) fn priced(spec: &RaceSpec) -> Priced {
        Self::priced_n(spec, ODDS_SIMULATIONS)
    }

    pub(super) fn priced_n(spec: &RaceSpec, sims: u64) -> Priced {
        let n = spec.entrants.len();
        let slots = bets::place_slots(n);
        let (mut wins, mut places) = (vec![0u32; n], vec![0u32; n]);
        let mut orders = Vec::with_capacity(sims as usize);
        for seed in 0..sims {
            let results = Self::finish_order(spec, seed ^ ODDS_SEED_SALT);
            let mut first_three = Vec::with_capacity(FIRST_THREE);
            for (rank, r) in results.iter().enumerate() {
                if let Some(i) = spec.entrants.iter().position(|e| e.name == r.name) {
                    if rank == 0 {
                        wins[i] += 1;
                    }
                    if rank < slots {
                        places[i] += 1;
                    }
                    if rank < FIRST_THREE {
                        first_three.push(i);
                    }
                }
            }
            orders.push(first_three);
        }
        // add-one smoothing keeps a never-seen outcome from giving infinite odds
        let total = sims as f64 + 2.0;
        let chances = (0..n)
            .map(|i| {
                (
                    (wins[i] as f64 + 1.0) / total,
                    (places[i] as f64 + 1.0) / total,
                )
            })
            .collect();
        Priced { chances, orders }
    }

    pub(super) fn to_odds(p: f64, min: f64) -> f64 {
        let raw = (PAYOUT_RATE / p).clamp(min, MAX_ODDS);
        // round to one decimal so displayed odds equal the paid odds
        (raw * 10.0).round() / 10.0
    }

    /// (win odds, place odds) for every runner in spec order.
    pub(super) fn odds_table_for(spec: &RaceSpec) -> Vec<(f64, f64)> {
        Self::chances(spec)
            .into_iter()
            .map(|(w, p)| {
                (
                    Self::to_odds(w, MIN_ODDS),
                    Self::to_odds(p.max(w), MIN_PLACE_ODDS).min(MAX_PLACE_ODDS),
                )
            })
            .collect()
    }

    /// (win odds, place odds) for the whole stable alone over 2400m (tests of the odds).
    #[cfg(test)]
    pub(super) fn odds_table(&self) -> Vec<(f64, f64)> {
        Self::odds_table_for(&self.practice_spec())
    }

    #[cfg(test)]
    pub fn odds(&self, name: &str) -> Option<f64> {
        let idx = self.field.iter().position(|h| h.name == name)?;
        Some(self.odds_table()[idx].0)
    }

    /// 複勝 odds (top-3 finish).
    #[cfg(test)]
    pub fn place_odds(&self, name: &str) -> Option<f64> {
        let idx = self.field.iter().position(|h| h.name == name)?;
        Some(self.odds_table()[idx].1)
    }

    /// A bet on the stable horses racing alone (see `practice_spec`).
    #[cfg(test)]
    pub fn play_round(
        &mut self,
        horse: &str,
        amount: u32,
        seed: u64,
    ) -> Result<RoundOutcome, GameError> {
        self.play_bet(horse, amount, seed, BetKind::Win)
    }

    /// A bet on the stable horses racing alone (see `practice_spec`).
    #[cfg(test)]
    pub fn play_bet(
        &mut self,
        horse: &str,
        amount: u32,
        seed: u64,
        kind: BetKind,
    ) -> Result<RoundOutcome, GameError> {
        let spec = self.practice_spec();
        self.play_spec(spec, &[horse.to_string()], amount, seed, kind)
    }

    /// Indexes (in the race's field) of the horses a ticket names, after checking the selection.
    pub(super) fn pick_indexes(
        spec: &RaceSpec,
        kind: BetKind,
        picks: &[String],
    ) -> Result<Vec<usize>, GameError> {
        if picks.len() != bets::picks(kind) {
            return Err(GameError::InvalidSelection);
        }
        let mut seen = std::collections::HashSet::new();
        if !picks.iter().all(|p| seen.insert(p)) {
            return Err(GameError::DuplicateHorse);
        }
        picks
            .iter()
            .map(|p| {
                spec.entrants
                    .iter()
                    .position(|e| &e.name == p)
                    .ok_or(GameError::UnknownHorse)
            })
            .collect()
    }

    /// Odds of a ticket. Singles come straight from the simulated win and
    /// top-three counts; combinations are priced from each horse's chance of
    /// winning, by Henery's refinement of Harville's model (see `bets`), and kept
    /// between 1.1 and 9999.9.
    pub(super) fn ticket_odds(priced: &Priced, kind: BetKind, idx: &[usize]) -> f64 {
        let chances = &priced.chances;
        match kind {
            BetKind::Win => Self::to_odds(chances[idx[0]].0, MIN_ODDS),
            BetKind::Place => {
                let (win, place) = chances[idx[0]];
                Self::to_odds(place.max(win), MIN_PLACE_ODDS).min(MAX_PLACE_ODDS)
            }
            _ => {
                let total: f64 = chances.iter().map(|c| c.0).sum();
                let win: Vec<f64> = chances.iter().map(|c| c.0 / total).collect();
                let model = bets::combo_probability(&win, kind, idx).unwrap_or(0.0);
                // the model misjudges fields with a few far stronger horses, so what the
                // simulated races actually showed counts too, more the commoner the result
                let hits = priced
                    .orders
                    .iter()
                    .filter(|top| Self::combo_hit(kind, idx, top))
                    .count() as f64;
                let prob =
                    (hits + MODEL_WEIGHT * model) / (priced.orders.len() as f64 + MODEL_WEIGHT);
                let raw = (PAYOUT_RATE / prob).clamp(MIN_ODDS, MAX_COMBO_ODDS); // no chance: the cap
                (raw * 10.0).round() / 10.0
            }
        }
    }

    /// Whether a combination ticket came in, given the first three home.
    pub(super) fn combo_hit(kind: BetKind, idx: &[usize], top: &[usize]) -> bool {
        let n = idx.len();
        if top.len() < n {
            return false;
        }
        match kind {
            BetKind::Exacta | BetKind::Trifecta => top[..n] == *idx,
            BetKind::Wide => idx.iter().all(|i| top.contains(i)),
            _ => idx.iter().all(|i| top[..n].contains(i)),
        }
    }

    /// The odds a ticket of this kind on these horses would pay in the race.
    pub fn quote_bet(
        &self,
        key: Option<&RaceKey>,
        kind: BetKind,
        picks: &[String],
    ) -> Result<f64, GameError> {
        let spec = self.race_spec(key)?;
        let idx = Self::pick_indexes(&spec, kind, picks)?;
        Ok(Self::ticket_odds(&Self::priced(&spec), kind, &idx))
    }

    /// A single-horse bet (単勝 or 複勝) on the chosen race.
    #[cfg(test)]
    pub fn play_bet_in(
        &mut self,
        key: Option<&RaceKey>,
        horse: &str,
        amount: u32,
        seed: u64,
        kind: BetKind,
    ) -> Result<RoundOutcome, GameError> {
        self.play_combo(key, &[horse.to_string()], amount, seed, kind)
    }

    /// Buys a ticket of any kind and runs the race. A calendar race uses its
    /// own distance, surface and field, and moves the game on one week.
    #[cfg(test)]
    pub fn play_combo(
        &mut self,
        key: Option<&RaceKey>,
        picks: &[String],
        amount: u32,
        seed: u64,
        kind: BetKind,
    ) -> Result<RoundOutcome, GameError> {
        if self.is_game_over() {
            return Err(GameError::GameOver);
        }
        let spec = self.race_spec(key)?;
        self.play_spec(spec, picks, amount, seed, kind)
    }

    /// Prize money for the player's horses among the first five home: 100%, 40%,
    /// 25%, 15% and 10% of the first prize, in game money.
    pub(super) fn owner_prizes(
        spec: &RaceSpec,
        results: &[RaceResult],
        first_prize: u32,
    ) -> Vec<PrizeWon> {
        results
            .iter()
            .zip(PRIZE_SHARES)
            .enumerate()
            .filter(|(_, (r, _))| spec.entrants.iter().any(|e| e.mine && e.name == r.name))
            .map(|(i, (r, share))| PrizeWon {
                name: r.name.clone(),
                rank: i as u32 + 1,
                amount: (first_prize as f64 * PRIZE_RATE * share).round() as u32,
            })
            .filter(|p| p.amount > 0)
            .collect()
    }

    /// Buys a single ticket in an already built race and runs it.
    #[cfg(test)]
    pub(super) fn play_spec(
        &mut self,
        spec: RaceSpec,
        picks: &[String],
        amount: u32,
        seed: u64,
        kind: BetKind,
    ) -> Result<RoundOutcome, GameError> {
        let ticket = Ticket {
            kind,
            horses: picks.to_vec(),
            amount,
        };
        self.play_tickets_in(spec, &[ticket], seed)
    }

    /// Buys several tickets in one race of this week (`None`: the free race): the
    /// race is run once and every ticket is judged on its own. Nothing is bought
    /// unless every ticket is valid and the total can be paid.
    pub fn play_tickets(
        &mut self,
        key: Option<&RaceKey>,
        tickets: &[Ticket],
        seed: u64,
    ) -> Result<RoundOutcome, GameError> {
        if self.is_game_over() {
            return Err(GameError::GameOver);
        }
        let spec = self.race_spec(key)?;
        if tickets.is_empty() {
            return Err(GameError::NoTickets);
        }
        self.play_tickets_in(spec, tickets, seed)
    }

    /// Runs a race of this week (`None`: the free race) without buying anything.
    pub fn watch_race(
        &mut self,
        key: Option<&RaceKey>,
        seed: u64,
    ) -> Result<RoundOutcome, GameError> {
        let spec = self.race_spec(key)?;
        self.play_tickets_in(spec, &[], seed)
    }

    /// The odds each ticket would pay (the stakes do not matter), priced together.
    pub fn quote_tickets(
        &self,
        key: Option<&RaceKey>,
        tickets: &[Ticket],
    ) -> Result<Vec<f64>, GameError> {
        let spec = self.race_spec(key)?;
        if tickets.is_empty() {
            return Err(GameError::NoTickets);
        }
        if tickets.len() > MAX_TICKETS {
            return Err(GameError::TooManyTickets);
        }
        let idx: Vec<Vec<usize>> = tickets
            .iter()
            .map(|t| Self::pick_indexes(&spec, t.kind, &t.horses))
            .collect::<Result<_, _>>()?;
        let priced = Self::priced(&spec);
        Ok(tickets
            .iter()
            .zip(&idx)
            .map(|(t, i)| Self::ticket_odds(&priced, t.kind, i))
            .collect())
    }

    /// The tickets that make up a ボックス or 流し on these horses in this race.
    /// For a 流し, `axis` is the horse every ticket is built around and `horses` are the partners.
    pub fn expand_selection(
        &self,
        key: Option<&RaceKey>,
        style: BetStyle,
        kind: BetKind,
        axis: Option<&str>,
        horses: &[String],
    ) -> Result<Vec<Vec<String>>, GameError> {
        let spec = self.race_spec(key)?;
        // every horse named must be a runner, and none may be named twice
        let mut named: Vec<&String> = horses.iter().collect();
        let axis_owned = axis.map(str::to_string);
        if let Some(a) = &axis_owned {
            named.push(a);
        }
        let mut seen = std::collections::HashSet::new();
        for n in &named {
            if !spec.entrants.iter().any(|e| &e.name == *n) {
                return Err(GameError::UnknownHorse);
            }
            if !seen.insert(*n) {
                return Err(GameError::DuplicateHorse);
            }
        }
        let tickets = match style {
            BetStyle::Box => bets::box_selections(kind, horses),
            BetStyle::Flow => match axis {
                Some(a) => bets::nagashi_selections(kind, a, horses),
                None => None,
            },
        }
        .ok_or(GameError::InvalidSelection)?;
        if tickets.len() > MAX_TICKETS {
            return Err(GameError::TooManyTickets);
        }
        Ok(tickets)
    }

    /// Buys tickets (maybe none) in an already built race and runs it.
    pub(super) fn play_tickets_in(
        &mut self,
        spec: RaceSpec,
        tickets: &[Ticket],
        seed: u64,
    ) -> Result<RoundOutcome, GameError> {
        if self.is_game_over() {
            return Err(GameError::GameOver);
        }
        if tickets.len() > MAX_TICKETS {
            return Err(GameError::TooManyTickets);
        }
        let idx: Vec<Vec<usize>> = tickets
            .iter()
            .map(|t| Self::pick_indexes(&spec, t.kind, &t.horses))
            .collect::<Result<_, _>>()?;
        if tickets.iter().any(|t| t.amount == 0) {
            return Err(GameError::InvalidAmount);
        }
        let stake: u64 = tickets.iter().map(|t| t.amount as u64).sum();
        if stake > self.money as u64 {
            return Err(GameError::InsufficientFunds);
        }
        let priced = Self::priced(&spec);
        let odds: Vec<f64> = tickets
            .iter()
            .zip(&idx)
            .map(|(t, i)| Self::ticket_odds(&priced, t.kind, i))
            .collect();

        let (frames, results) = Self::run_race(&spec, seed);

        self.round += 1;
        self.money -= stake as u32;
        let order: Vec<String> = results.iter().map(|r| r.name.clone()).collect();
        let judged: Vec<TicketResult> = tickets
            .iter()
            .zip(&odds)
            .map(|(t, &o)| {
                let won = bets::is_hit_among(t.kind, &t.horses, &order, spec.entrants.len());
                TicketResult {
                    kind: t.kind,
                    horses: t.horses.clone(),
                    label: bets::selection_label(t.kind, &t.horses),
                    amount: t.amount,
                    odds: o,
                    won,
                    payout: if won {
                        (t.amount as f64 * o).round() as i64
                    } else {
                        0
                    },
                }
            })
            .collect();
        let payout: i64 = judged.iter().map(|t| t.payout).sum();
        let won = judged.iter().any(|t| t.won);
        self.earn(payout);
        let first_prize = spec.race.as_ref().map_or(0, |r| r.prize);
        let prizes = Self::owner_prizes(&spec, &results, first_prize);
        let prize_total: u32 = prizes.iter().map(|p| p.amount).sum();
        self.earn(prize_total as i64);

        if spec.race.is_some() {
            self.add_to_records(&spec, &results);
            self.pay_jockeys(&spec);
        }
        let race_name = spec.race.as_ref().map(|r| r.name.clone());
        // without a ticket, the entry tells how the player's own horses did
        let mine: Vec<(&str, usize)> = results
            .iter()
            .enumerate()
            .filter(|(_, r)| spec.entrants.iter().any(|e| e.mine && e.name == r.name))
            .map(|(i, r)| (r.name.as_str(), i + 1))
            .collect();
        let entry_won = if judged.is_empty() {
            mine.iter().any(|(_, rank)| *rank == 1)
        } else {
            won
        };
        let (kind, horse, horses) = match judged.first() {
            Some(first) if judged.len() > 1 => (
                first.kind,
                format!("{} ほか{}点", first.label, judged.len() - 1),
                first.horses.clone(),
            ),
            Some(first) => (first.kind, first.label.clone(), first.horses.clone()),
            None => (
                BetKind::Win,
                mine.iter()
                    .map(|(name, rank)| format!("{name} {rank}着"))
                    .chain(
                        Self::did_not_finish(&spec, &results)
                            .into_iter()
                            .filter(|n| spec.entrants.iter().any(|e| e.mine && &e.name == n))
                            .map(|n| format!("{n} 競走中止")),
                    )
                    .collect::<Vec<_>>()
                    .join("、"),
                mine.iter().map(|(name, _)| name.to_string()).collect(),
            ),
        };
        let mixed_kinds = judged.iter().any(|t| t.kind != kind);
        self.remember(HistoryEntry {
            kind,
            race: race_name.clone(),
            round: self.round,
            horse,
            horses,
            stake: stake as u32,
            won: entry_won,
            payout,
            prize: prize_total,
            ticket_count: judged.len() as u32,
            watched: judged.is_empty(),
            mixed_kinds,
            winner: results.first().map(|r| r.name.clone()).unwrap_or_default(),
        });
        // running takes it out of a horse; the free race, only a gallop, half as much
        let tiring = race_fatigue(spec.distance) / if race_name.is_some() { 1 } else { 2 };
        self.tire_runners(&spec, tiring);
        if race_name.is_some() {
            self.finish_week(Some((&spec, &results)));
        } else {
            self.roll_conditions();
        }

        let runners: Vec<String> = spec.entrants.iter().map(|e| e.name.clone()).collect();
        let mut commentary =
            commentary::generate(&runners, &frames, spec.distance as f64, &results);
        Self::call_pulling(&mut commentary, &Self::pulled(&spec, seed), frames.len());
        let stopped = Self::did_not_finish(&spec, &results);
        Self::call_mishaps(&mut commentary, &spec, seed, &frames, &stopped);

        Ok(RoundOutcome {
            pace: Self::start_race(&spec, seed).pace.label().to_string(),
            stopped,
            results,
            frames,
            commentary,
            runners,
            coats: spec.entrants.iter().map(|e| e.coat_color.clone()).collect(),
            race_name,
            distance: spec.distance as f64,
            won,
            payout,
            tickets: judged,
            prizes,
            prize_total,
            money: self.money,
        })
    }
}
