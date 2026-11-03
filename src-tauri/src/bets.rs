//! The six kinds of betting ticket and the maths behind them: how likely a
//! combination is, whether a ticket hit, and how a selection is written.
//!
//! * 単勝 pick the winner · 複勝 pick a horse that finishes in the top three
//!   (the top two in a field of seven or fewer)
//! * 馬単 pick 1st and 2nd, in order · 馬連 pick the top two, in any order
//! * 三連単 pick 1st, 2nd and 3rd, in order · 三連複 pick the top three, in any order

use crate::game::BetKind;

/// Horses a ticket of this kind picks.
pub fn picks(kind: BetKind) -> usize {
    match kind {
        BetKind::Win | BetKind::Place => 1,
        BetKind::Exacta | BetKind::Quinella => 2,
        BetKind::Trifecta | BetKind::Trio => 3,
    }
}

/// The smallest field in which a 複勝 pays on the first three.
const FULL_PLACE_FIELD: usize = 8;

/// How many horses a 複勝 pays on: the first three, or the first two in a field of seven or fewer.
pub fn place_slots(runners: usize) -> usize {
    if runners >= FULL_PLACE_FIELD {
        3
    } else {
        2
    }
}

/// Whether the order of the picks matters.
pub fn ordered(kind: BetKind) -> bool {
    matches!(kind, BetKind::Exacta | BetKind::Trifecta)
}

/// How a selection is written: ordered picks joined with →, unordered ones with -.
pub fn selection_label(kind: BetKind, names: &[String]) -> String {
    names.join(if ordered(kind) { "→" } else { "-" })
}

/// How the field is shared out for 2nd place: each remaining horse's chance of
/// winning is raised to this power (below 1 flattens it).
///
/// Plain Harville's model (exponent 1) makes the favourites too likely to fill
/// the places and the outsiders too unlikely. These two values come from
/// Henery's refinement, fitted by maximum likelihood to the simulated finishing
/// orders of six different races.
const SECOND_PLACE_EXPONENT: f64 = 0.7;
/// The same for 3rd place.
const THIRD_PLACE_EXPONENT: f64 = 0.5;

/// Chance that these horses finish first, second, third... in exactly this order,
/// given each horse's chance `p` of winning. The winner is drawn by `p`; each
/// later place is drawn from the horses not yet placed by `p` raised to the
/// place's exponent, so every step shares out the whole of the probability.
fn ordered_probability(p: &[f64], seq: &[usize]) -> f64 {
    const EXPONENTS: [f64; 3] = [1.0, SECOND_PLACE_EXPONENT, THIRD_PLACE_EXPONENT];
    let mut prob = 1.0;
    for (k, &i) in seq.iter().enumerate() {
        let g = EXPONENTS[k];
        let share: f64 = (0..p.len())
            .filter(|j| !seq[..k].contains(j))
            .map(|j| p[j].powf(g))
            .sum();
        if share <= f64::EPSILON {
            return 0.0;
        }
        prob *= p[i].powf(g) / share;
    }
    prob
}

/// All the orders `idx` can be written in.
fn permutations(idx: &[usize]) -> Vec<Vec<usize>> {
    if idx.len() <= 1 {
        return vec![idx.to_vec()];
    }
    let mut out = Vec::new();
    for i in 0..idx.len() {
        let mut rest = idx.to_vec();
        let first = rest.remove(i);
        for mut tail in permutations(&rest) {
            tail.insert(0, first);
            out.push(tail);
        }
    }
    out
}

/// Chance that a ticket of `kind` on the horses at `idx` (indexes into `p`) hits,
/// from each horse's chance of winning (see [`ordered_probability`]). `None` for 複勝, which is priced from
/// top-three counts rather than from win chances.
pub fn combo_probability(p: &[f64], kind: BetKind, idx: &[usize]) -> Option<f64> {
    if idx.len() != picks(kind) {
        return None;
    }
    match kind {
        BetKind::Place => None,
        BetKind::Win => Some(p[idx[0]]),
        BetKind::Exacta | BetKind::Trifecta => Some(ordered_probability(p, idx)),
        BetKind::Quinella | BetKind::Trio => Some(
            permutations(idx)
                .iter()
                .map(|seq| ordered_probability(p, seq))
                .sum(),
        ),
    }
}

/// Whether a ticket on `picked` hit, given the full finishing `order` (winner first).
pub fn is_hit(kind: BetKind, picked: &[String], order: &[String]) -> bool {
    let n = picks(kind);
    if picked.len() != n || order.len() < n.max(1) {
        return false;
    }
    let top = |k: usize| &order[..k.min(order.len())];
    let same_set = |a: &[String], b: &[String]| {
        let mut x = a.to_vec();
        let mut y = b.to_vec();
        x.sort();
        y.sort();
        x == y
    };
    match kind {
        BetKind::Win => order[0] == picked[0],
        BetKind::Place => top(place_slots(order.len())).contains(&picked[0]),
        BetKind::Exacta => top(2) == picked,
        BetKind::Quinella => same_set(top(2), picked),
        BetKind::Trifecta => top(3) == picked,
        BetKind::Trio => same_set(top(3), picked),
    }
}

/// Every way of choosing `n` of `items`, in order, without repeats (unordered picks).
fn combinations(items: &[String], n: usize) -> Vec<Vec<String>> {
    if n == 0 {
        return vec![vec![]];
    }
    let mut out = Vec::new();
    for (i, first) in items.iter().enumerate() {
        for mut rest in combinations(&items[i + 1..], n - 1) {
            rest.insert(0, first.clone());
            out.push(rest);
        }
    }
    out
}

/// Every ordering of every choice of `n` of `items` (ordered picks).
fn arrangements(items: &[String], n: usize) -> Vec<Vec<String>> {
    if n == 0 {
        return vec![vec![]];
    }
    let mut out = Vec::new();
    for (i, first) in items.iter().enumerate() {
        let mut others = items.to_vec();
        others.remove(i);
        for mut rest in arrangements(&others, n - 1) {
            rest.insert(0, first.clone());
            out.push(rest);
        }
    }
    out
}

/// ボックス: a ticket for every way of picking the right number of the given
/// horses (in every order for 馬単 and 三連単). `None` for 単勝 and 複勝, or when
/// fewer horses are given than the ticket needs.
pub fn box_selections(kind: BetKind, horses: &[String]) -> Option<Vec<Vec<String>>> {
    let n = picks(kind);
    if n < 2 || horses.len() < n {
        return None;
    }
    Some(if ordered(kind) {
        arrangements(horses, n)
    } else {
        combinations(horses, n)
    })
}

/// 流し: the `axis` horse with every way of completing the ticket from the
/// `partners`. For 馬単 and 三連単 the axis must finish first, and the partners
/// fill the other places in every order. `None` for 単勝 and 複勝, or with too
/// few partners.
pub fn nagashi_selections(
    kind: BetKind,
    axis: &str,
    partners: &[String],
) -> Option<Vec<Vec<String>>> {
    let n = picks(kind);
    if n < 2 || partners.len() < n - 1 {
        return None;
    }
    let rest = if ordered(kind) {
        arrangements(partners, n - 1)
    } else {
        combinations(partners, n - 1)
    };
    Some(
        rest.into_iter()
            .map(|mut r| {
                r.insert(0, axis.to_string());
                r
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_small_field_pays_a_place_ticket_on_the_first_two_only() {
        assert_eq!(place_slots(6), 2);
        assert_eq!(place_slots(7), 2);
        assert_eq!(place_slots(8), 3);
        assert_eq!(place_slots(18), 3);
        let order = |n: usize| (0..n).map(|i| format!("H{i}")).collect::<Vec<_>>();
        let third = vec!["H2".to_string()];
        assert!(!is_hit(BetKind::Place, &third, &order(7)));
        assert!(is_hit(BetKind::Place, &third, &order(8)));
        let second = vec!["H1".to_string()];
        assert!(is_hit(BetKind::Place, &second, &order(7)));
    }
    use BetKind::*;

    const ALL: [BetKind; 6] = [Win, Place, Exacta, Quinella, Trifecta, Trio];

    fn names(n: &[&str]) -> Vec<String> {
        n.iter().map(|s| s.to_string()).collect()
    }

    /// Win chances of five horses, summing to 1.
    fn field() -> Vec<f64> {
        vec![0.40, 0.25, 0.15, 0.12, 0.08]
    }

    #[test]
    fn each_kind_picks_the_right_number_of_horses() {
        assert_eq!(ALL.map(picks), [1, 1, 2, 2, 3, 3]);
    }

    #[test]
    fn only_exacta_and_trifecta_care_about_order() {
        assert_eq!(ALL.map(ordered), [false, false, true, false, true, false]);
    }

    #[test]
    fn selections_are_written_with_arrows_when_ordered_and_hyphens_when_not() {
        let abc = names(&["A", "B", "C"]);
        assert_eq!(selection_label(Win, &abc[..1]), "A");
        assert_eq!(selection_label(Place, &abc[..1]), "A");
        assert_eq!(selection_label(Exacta, &abc[..2]), "A→B");
        assert_eq!(selection_label(Quinella, &abc[..2]), "A-B");
        assert_eq!(selection_label(Trifecta, &abc), "A→B→C");
        assert_eq!(selection_label(Trio, &abc), "A-B-C");
    }

    #[test]
    fn the_names_of_the_kinds_in_json_are_lowercase_english() {
        let json = |k: BetKind| serde_json::to_string(&k).unwrap();
        assert_eq!(
            ALL.map(json),
            [
                "\"win\"",
                "\"place\"",
                "\"exacta\"",
                "\"quinella\"",
                "\"trifecta\"",
                "\"trio\""
            ]
        );
        let back: BetKind = serde_json::from_str("\"trifecta\"").unwrap();
        assert_eq!(back, Trifecta);
    }

    // ---- probabilities

    #[test]
    fn a_win_ticket_is_just_the_horses_chance_of_winning() {
        let p = field();
        assert_eq!(combo_probability(&p, Win, &[1]), Some(0.25));
    }

    #[test]
    fn a_place_ticket_has_no_win_chance_based_price() {
        assert_eq!(combo_probability(&field(), Place, &[0]), None);
    }

    #[test]
    fn a_wrong_number_of_picks_has_no_probability() {
        let p = field();
        assert_eq!(combo_probability(&p, Exacta, &[0]), None);
        assert_eq!(combo_probability(&p, Trifecta, &[0, 1]), None);
        assert_eq!(combo_probability(&p, Win, &[0, 1]), None);
    }

    #[test]
    fn exacta_chance_is_the_winners_chance_times_the_runner_ups_share_of_the_rest() {
        let p = field();
        // 0.40 wins; the runner-up is drawn from the other four by their chances to the power 0.7
        let rest: f64 = [0.25_f64, 0.15, 0.12, 0.08]
            .iter()
            .map(|x| x.powf(SECOND_PLACE_EXPONENT))
            .sum();
        let want = 0.40 * 0.25_f64.powf(SECOND_PLACE_EXPONENT) / rest;
        assert!((combo_probability(&p, Exacta, &[0, 1]).unwrap() - want).abs() < 1e-12);
    }

    #[test]
    fn trifecta_chance_chains_the_same_rule_once_more_with_the_third_place_exponent() {
        let p = field();
        let second: f64 = [0.25_f64, 0.15, 0.12, 0.08]
            .iter()
            .map(|x| x.powf(SECOND_PLACE_EXPONENT))
            .sum();
        let third: f64 = [0.15_f64, 0.12, 0.08]
            .iter()
            .map(|x| x.powf(THIRD_PLACE_EXPONENT))
            .sum();
        let want = 0.40
            * (0.25_f64.powf(SECOND_PLACE_EXPONENT) / second)
            * (0.15_f64.powf(THIRD_PLACE_EXPONENT) / third);
        assert!((combo_probability(&p, Trifecta, &[0, 1, 2]).unwrap() - want).abs() < 1e-12);
    }

    #[test]
    fn the_exponents_are_below_one_so_later_places_are_flatter_than_plain_harville() {
        const { assert!(SECOND_PLACE_EXPONENT < 1.0 && THIRD_PLACE_EXPONENT < SECOND_PLACE_EXPONENT) };
        let p = field();
        // Harville: the favourite wins, then the 8% outsider takes 2nd with 0.08 / 0.60
        let harville_outsider = 0.40 * (0.08 / 0.60);
        let ours = combo_probability(&p, Exacta, &[0, 4]).unwrap();
        assert!(ours > harville_outsider, "{ours} vs {harville_outsider}");
        // ... and so the strong runner-up is a little less likely than Harville says
        let harville_second = 0.40 * (0.25 / 0.60);
        assert!(combo_probability(&p, Exacta, &[0, 1]).unwrap() < harville_second);
    }

    fn all_sequences(n: usize, len: usize) -> Vec<Vec<usize>> {
        if len == 0 {
            return vec![vec![]];
        }
        let mut out = vec![];
        for tail in all_sequences(n, len - 1) {
            for i in 0..n {
                if !tail.contains(&i) {
                    let mut s = tail.clone();
                    s.push(i);
                    out.push(s);
                }
            }
        }
        out
    }

    #[test]
    fn the_chances_of_every_exacta_add_up_to_one() {
        let p = field();
        let total: f64 = all_sequences(5, 2)
            .iter()
            .map(|s| combo_probability(&p, Exacta, s).unwrap())
            .sum();
        assert!((total - 1.0).abs() < 1e-9, "{total}");
    }

    #[test]
    fn the_chances_of_every_trifecta_add_up_to_one() {
        let p = field();
        let total: f64 = all_sequences(5, 3)
            .iter()
            .map(|s| combo_probability(&p, Trifecta, s).unwrap())
            .sum();
        assert!((total - 1.0).abs() < 1e-9, "{total}");
    }

    #[test]
    fn the_chances_of_every_quinella_and_trio_add_up_to_one() {
        let p = field();
        let sets = |len: usize| -> Vec<Vec<usize>> {
            all_sequences(5, len)
                .into_iter()
                .filter(|s| s.windows(2).all(|w| w[0] < w[1]))
                .collect()
        };
        let quinellas: f64 = sets(2)
            .iter()
            .map(|s| combo_probability(&p, Quinella, s).unwrap())
            .sum();
        let trios: f64 = sets(3)
            .iter()
            .map(|s| combo_probability(&p, Trio, s).unwrap())
            .sum();
        assert!((quinellas - 1.0).abs() < 1e-9, "{quinellas}");
        assert!((trios - 1.0).abs() < 1e-9, "{trios}");
    }

    #[test]
    fn a_quinella_is_both_exactas_and_a_trio_is_all_six_trifectas() {
        let p = field();
        let q = combo_probability(&p, Quinella, &[1, 3]).unwrap();
        let both = combo_probability(&p, Exacta, &[1, 3]).unwrap()
            + combo_probability(&p, Exacta, &[3, 1]).unwrap();
        assert!((q - both).abs() < 1e-12);
        let t = combo_probability(&p, Trio, &[0, 2, 4]).unwrap();
        let six: f64 = permutations(&[0, 2, 4])
            .iter()
            .map(|s| combo_probability(&p, Trifecta, s).unwrap())
            .sum();
        assert!((t - six).abs() < 1e-12);
    }

    #[test]
    fn unordered_tickets_ignore_the_order_picked() {
        let p = field();
        assert_eq!(
            combo_probability(&p, Quinella, &[3, 1]),
            combo_probability(&p, Quinella, &[1, 3])
        );
        let a = combo_probability(&p, Trio, &[4, 0, 2]).unwrap();
        let b = combo_probability(&p, Trio, &[0, 2, 4]).unwrap();
        assert!((a - b).abs() < 1e-12);
    }

    #[test]
    fn ordered_tickets_depend_on_the_order_picked() {
        let p = field();
        // the favourite winning then the outsider placing is likelier than the reverse
        assert!(
            combo_probability(&p, Exacta, &[0, 4]).unwrap()
                > combo_probability(&p, Exacta, &[4, 0]).unwrap()
        );
    }

    #[test]
    fn harder_tickets_are_less_likely_than_easier_ones_on_the_same_horses() {
        let p = field();
        let win = combo_probability(&p, Win, &[0]).unwrap();
        let exacta = combo_probability(&p, Exacta, &[0, 1]).unwrap();
        let quinella = combo_probability(&p, Quinella, &[0, 1]).unwrap();
        let trifecta = combo_probability(&p, Trifecta, &[0, 1, 2]).unwrap();
        let trio = combo_probability(&p, Trio, &[0, 1, 2]).unwrap();
        assert!(exacta < win && exacta < quinella);
        assert!(trifecta < exacta && trifecta < trio);
        assert!(trio < 1.0 && quinella < 1.0);
    }

    #[test]
    fn a_horse_with_no_chance_makes_a_ticket_impossible() {
        let p = vec![0.7, 0.3, 0.0];
        assert_eq!(combo_probability(&p, Exacta, &[0, 2]), Some(0.0));
        assert_eq!(combo_probability(&p, Trio, &[0, 1, 2]), Some(0.0));
    }

    #[test]
    fn a_certain_winner_leaves_nothing_for_the_rest() {
        // after a horse with probability 1 is placed, nothing is left: no NaN
        let p = vec![1.0, 0.0, 0.0];
        let v = combo_probability(&p, Exacta, &[0, 1]).unwrap();
        assert!(v.is_finite() && v == 0.0, "{v}");
        assert_eq!(combo_probability(&p, Win, &[0]), Some(1.0));
    }

    // ---- hits

    fn order() -> Vec<String> {
        names(&["A", "B", "C", "D", "E"])
    }

    #[test]
    fn a_win_ticket_hits_only_the_winner() {
        assert!(is_hit(Win, &names(&["A"]), &order()));
        assert!(!is_hit(Win, &names(&["B"]), &order()));
    }

    #[test]
    fn a_place_ticket_hits_any_of_the_first_three() {
        let full = names(&["A", "B", "C", "D", "E", "F", "G", "H"]);
        for ok in ["A", "B", "C"] {
            assert!(is_hit(Place, &names(&[ok]), &full), "{ok}");
        }
        for no in ["D", "E", "Z"] {
            assert!(!is_hit(Place, &names(&[no]), &full), "{no}");
        }
    }

    #[test]
    fn an_exacta_needs_first_and_second_in_order() {
        assert!(is_hit(Exacta, &names(&["A", "B"]), &order()));
        assert!(!is_hit(Exacta, &names(&["B", "A"]), &order()));
        assert!(!is_hit(Exacta, &names(&["A", "C"]), &order()));
        assert!(!is_hit(Exacta, &names(&["B", "C"]), &order()));
    }

    #[test]
    fn a_quinella_needs_the_first_two_in_either_order() {
        assert!(is_hit(Quinella, &names(&["A", "B"]), &order()));
        assert!(is_hit(Quinella, &names(&["B", "A"]), &order()));
        assert!(!is_hit(Quinella, &names(&["A", "C"]), &order()));
        assert!(!is_hit(Quinella, &names(&["C", "D"]), &order()));
    }

    #[test]
    fn a_trifecta_needs_the_first_three_in_order() {
        assert!(is_hit(Trifecta, &names(&["A", "B", "C"]), &order()));
        for wrong in [
            ["A", "C", "B"],
            ["B", "A", "C"],
            ["C", "B", "A"],
            ["A", "B", "D"],
        ] {
            assert!(!is_hit(Trifecta, &names(&wrong), &order()), "{wrong:?}");
        }
    }

    #[test]
    fn a_trio_needs_the_first_three_in_any_order() {
        for ok in [
            ["A", "B", "C"],
            ["C", "A", "B"],
            ["B", "C", "A"],
            ["C", "B", "A"],
        ] {
            assert!(is_hit(Trio, &names(&ok), &order()), "{ok:?}");
        }
        for wrong in [["A", "B", "D"], ["B", "C", "D"], ["A", "D", "E"]] {
            assert!(!is_hit(Trio, &names(&wrong), &order()), "{wrong:?}");
        }
    }

    #[test]
    fn a_ticket_with_the_wrong_number_of_horses_never_hits() {
        assert!(!is_hit(Win, &names(&["A", "B"]), &order()));
        assert!(!is_hit(Exacta, &names(&["A"]), &order()));
        assert!(!is_hit(Trio, &names(&["A", "B"]), &order()));
        assert!(!is_hit(Trifecta, &[], &order()));
    }

    #[test]
    fn a_short_finishing_order_cannot_hit_a_longer_ticket() {
        let two = names(&["A", "B"]);
        assert!(is_hit(Exacta, &names(&["A", "B"]), &two));
        assert!(!is_hit(Trifecta, &names(&["A", "B", "C"]), &two));
        assert!(!is_hit(Win, &names(&["A"]), &[]));
    }

    #[test]
    fn a_place_ticket_in_a_two_horse_race_hits_both() {
        let two = names(&["A", "B"]);
        assert!(is_hit(Place, &names(&["A"]), &two));
        assert!(is_hit(Place, &names(&["B"]), &two));
    }

    // ---- ボックス and 流し

    fn horses(n: usize) -> Vec<String> {
        ["A", "B", "C", "D", "E", "F", "G", "H"]
            .iter()
            .take(n)
            .map(|s| s.to_string())
            .collect()
    }

    fn flat(v: &[Vec<String>]) -> Vec<String> {
        v.iter().map(|s| s.join("")).collect()
    }

    #[test]
    fn a_quinella_box_pairs_every_two_horses_once() {
        let b = box_selections(Quinella, &horses(3)).unwrap();
        assert_eq!(flat(&b), ["AB", "AC", "BC"]);
        assert_eq!(box_selections(Quinella, &horses(4)).unwrap().len(), 6);
    }

    #[test]
    fn an_exacta_box_covers_both_orders() {
        let b = box_selections(Exacta, &horses(3)).unwrap();
        assert_eq!(flat(&b), ["AB", "AC", "BA", "BC", "CA", "CB"]);
        assert_eq!(box_selections(Exacta, &horses(4)).unwrap().len(), 12);
    }

    #[test]
    fn a_trio_box_picks_every_three_once() {
        let b = box_selections(Trio, &horses(4)).unwrap();
        assert_eq!(flat(&b), ["ABC", "ABD", "ACD", "BCD"]);
        assert_eq!(box_selections(Trio, &horses(5)).unwrap().len(), 10);
    }

    #[test]
    fn a_trifecta_box_covers_every_order() {
        assert_eq!(box_selections(Trifecta, &horses(3)).unwrap().len(), 6);
        assert_eq!(box_selections(Trifecta, &horses(4)).unwrap().len(), 24);
        assert_eq!(box_selections(Trifecta, &horses(5)).unwrap().len(), 60);
        let b = box_selections(Trifecta, &horses(3)).unwrap();
        assert_eq!(flat(&b), ["ABC", "ACB", "BAC", "BCA", "CAB", "CBA"]);
    }

    #[test]
    fn box_tickets_are_all_different_and_use_only_the_given_horses() {
        for kind in [Exacta, Quinella, Trifecta, Trio] {
            let b = box_selections(kind, &horses(5)).unwrap();
            let mut keys: Vec<String> = b.iter().map(|s| s.join(",")).collect();
            keys.sort();
            keys.dedup();
            assert_eq!(keys.len(), b.len(), "{kind:?}");
            for t in &b {
                assert_eq!(t.len(), picks(kind));
                assert!(t.iter().all(|h| horses(5).contains(h)));
                let mut d = t.clone();
                d.sort();
                d.dedup();
                assert_eq!(d.len(), t.len(), "a horse repeated in {t:?}");
            }
        }
    }

    #[test]
    fn a_box_needs_at_least_as_many_horses_as_the_ticket_picks() {
        assert!(box_selections(Quinella, &horses(1)).is_none());
        assert!(box_selections(Trio, &horses(2)).is_none());
        assert!(box_selections(Trifecta, &horses(2)).is_none());
        assert!(box_selections(Quinella, &horses(2)).is_some());
        assert!(box_selections(Trio, &horses(3)).is_some());
    }

    #[test]
    fn singles_cannot_be_boxed_or_flowed() {
        assert!(box_selections(Win, &horses(4)).is_none());
        assert!(box_selections(Place, &horses(4)).is_none());
        assert!(nagashi_selections(Win, "A", &horses(4)[1..]).is_none());
        assert!(nagashi_selections(Place, "A", &horses(4)[1..]).is_none());
    }

    #[test]
    fn a_quinella_flow_pairs_the_axis_with_each_partner() {
        let n = nagashi_selections(Quinella, "A", &horses(4)[1..]).unwrap();
        assert_eq!(flat(&n), ["AB", "AC", "AD"]);
    }

    #[test]
    fn an_exacta_flow_puts_the_axis_first() {
        let n = nagashi_selections(Exacta, "A", &horses(4)[1..]).unwrap();
        assert_eq!(flat(&n), ["AB", "AC", "AD"]);
        assert!(n.iter().all(|t| t[0] == "A"));
    }

    #[test]
    fn a_trio_flow_pairs_the_partners_up() {
        let n = nagashi_selections(Trio, "A", &horses(4)[1..]).unwrap();
        assert_eq!(flat(&n), ["ABC", "ABD", "ACD"]);
        assert_eq!(
            nagashi_selections(Trio, "A", &horses(5)[1..])
                .unwrap()
                .len(),
            6
        );
    }

    #[test]
    fn a_trifecta_flow_puts_the_axis_first_and_orders_the_partners() {
        let n = nagashi_selections(Trifecta, "A", &horses(4)[1..]).unwrap();
        assert_eq!(flat(&n), ["ABC", "ABD", "ACB", "ACD", "ADB", "ADC"]);
        assert!(n.iter().all(|t| t[0] == "A"));
    }

    #[test]
    fn a_flow_needs_enough_partners() {
        assert!(nagashi_selections(Quinella, "A", &[]).is_none());
        assert!(nagashi_selections(Trio, "A", &horses(2)[1..]).is_none()); // one partner, two needed
        assert!(nagashi_selections(Trio, "A", &horses(3)[1..]).is_some());
        assert!(nagashi_selections(Trifecta, "A", &horses(2)[1..]).is_none());
    }

    #[test]
    fn a_flow_never_repeats_a_horse_within_a_ticket() {
        for kind in [Exacta, Quinella, Trifecta, Trio] {
            for t in nagashi_selections(kind, "A", &horses(6)[1..]).unwrap() {
                let mut d = t.clone();
                d.sort();
                d.dedup();
                assert_eq!(d.len(), t.len(), "{kind:?} {t:?}");
            }
        }
    }

    #[test]
    fn every_generated_ticket_is_a_valid_selection_for_its_kind() {
        let h = horses(6);
        for kind in [Exacta, Quinella, Trifecta, Trio] {
            for t in box_selections(kind, &h).unwrap() {
                assert_eq!(t.len(), picks(kind));
            }
            for t in nagashi_selections(kind, "A", &h[1..]).unwrap() {
                assert_eq!(t.len(), picks(kind));
            }
        }
    }
}
