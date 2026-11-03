//! The weight a horse carries in a race (斤量), by the race's weight rule.
//!
//! Age-limited races follow the JRA's age weights: at two 55kg (56kg for colts
//! from October), at three 57kg (fillies 55kg). In races open to older horses the
//! base is 58kg (mares 56kg), and a three-year-old has 2kg off, a simplification
//! of the JRA's month-by-month weight-for-age scale. An allowance race adds for
//! wins, and a handicap sets the weight by how the horse stands against its class.

use crate::calendar::{AgeRule, Eligibility, WeightRule};

/// The week from which a two-year-old colt carries a kilo more (October).
const AUTUMN_WEEK: u8 = 40;
/// Weight allowance for fillies and mares, and for three-year-olds racing older horses.
const FEMALE_ALLOWANCE: u32 = 2;
const THREE_YEAR_OLD_ALLOWANCE: u32 = 2;
/// An allowance race adds a kilo for this many wins, up to the limit.
const WINS_PER_KILO: u32 = 2;
const MAX_ADDED_KILOS: u32 = 3;
/// A handicap: this weight for a horse level with its class, a kilo for this much
/// speed above or below it, and the limits a handicapper keeps to.
const HANDICAP_BASE: i64 = 55;
const HANDICAP_SPEED_PER_KILO: i64 = 2;
const HANDICAP_MIN: i64 = 48;
const HANDICAP_MAX: i64 = 60;

/// The weight at which a horse neither gains nor loses by what it carries.
const NEUTRAL_KG: f64 = 56.0;
/// How much of its ability a horse loses per kilo above that (or gains below it), in percent.
const PERCENT_PER_KG: f64 = 0.3;

/// What a horse carries under the race's weight rule.
pub fn carried(
    eligibility: &Eligibility,
    week: u8,
    age: u8,
    female: bool,
    wins: u32,
    speed: u32,
    class_speed: i64,
) -> u32 {
    if eligibility.weight == WeightRule::Handicap {
        let kg = HANDICAP_BASE + (speed as i64 - class_speed) / HANDICAP_SPEED_PER_KILO;
        return kg.clamp(HANDICAP_MIN, HANDICAP_MAX) as u32;
    }
    let base = base_weight(eligibility.age, week, age, female);
    if eligibility.weight == WeightRule::Allowance {
        base + (wins / WINS_PER_KILO).min(MAX_ADDED_KILOS)
    } else {
        base
    }
}

fn base_weight(rule: AgeRule, week: u8, age: u8, female: bool) -> u32 {
    let sex_off = if female { FEMALE_ALLOWANCE } else { 0 };
    match rule {
        AgeRule::Two => {
            if !female && week >= AUTUMN_WEEK {
                56
            } else {
                55
            }
        }
        AgeRule::Three => 57 - sex_off,
        AgeRule::ThreeUp | AgeRule::FourUp => {
            let young_off = if age <= 3 {
                THREE_YEAR_OLD_ALLOWANCE
            } else {
                0
            };
            58 - sex_off - young_off
        }
    }
}

/// What carrying this weight does to a horse's speed and stamina.
pub fn multiplier(kg: u32) -> f64 {
    1.0 + (NEUTRAL_KG - kg as f64) * PERCENT_PER_KG / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(age: AgeRule, weight: WeightRule) -> Eligibility {
        Eligibility {
            age,
            fillies_only: false,
            weight,
        }
    }

    fn age_race(age: AgeRule, week: u8, horse_age: u8, female: bool) -> u32 {
        carried(
            &rule(age, WeightRule::Age),
            week,
            horse_age,
            female,
            0,
            70,
            70,
        )
    }

    #[test]
    fn two_year_olds_carry_the_jra_age_weights() {
        assert_eq!(age_race(AgeRule::Two, 30, 2, false), 55);
        assert_eq!(age_race(AgeRule::Two, 39, 2, false), 55);
        assert_eq!(age_race(AgeRule::Two, 40, 2, false), 56);
        assert_eq!(age_race(AgeRule::Two, 40, 2, true), 55);
    }

    #[test]
    fn three_year_olds_carry_the_jra_age_weights() {
        assert_eq!(age_race(AgeRule::Three, 10, 3, false), 57);
        assert_eq!(age_race(AgeRule::Three, 10, 3, true), 55);
    }

    #[test]
    fn older_horses_carry_58kg_with_allowances_for_mares_and_three_year_olds() {
        let fixed = |rule_age, age, female| {
            carried(
                &rule(rule_age, WeightRule::Fixed),
                20,
                age,
                female,
                0,
                70,
                70,
            )
        };
        assert_eq!(fixed(AgeRule::FourUp, 4, false), 58);
        assert_eq!(fixed(AgeRule::FourUp, 6, true), 56);
        assert_eq!(fixed(AgeRule::ThreeUp, 3, false), 56);
        assert_eq!(fixed(AgeRule::ThreeUp, 3, true), 54);
        assert_eq!(fixed(AgeRule::ThreeUp, 5, false), 58);
    }

    #[test]
    fn a_fixed_weight_takes_no_account_of_wins() {
        let r = rule(AgeRule::FourUp, WeightRule::Fixed);
        assert_eq!(carried(&r, 20, 5, false, 10, 90, 70), 58);
    }

    #[test]
    fn an_allowance_race_adds_for_wins_up_to_a_limit() {
        let r = rule(AgeRule::FourUp, WeightRule::Allowance);
        let with = |wins| carried(&r, 20, 5, false, wins, 70, 70);
        assert_eq!(with(0), 58);
        assert_eq!(with(1), 58);
        assert_eq!(with(2), 59);
        assert_eq!(with(4), 60);
        assert_eq!(with(6), 61);
        assert_eq!(with(50), 61);
    }

    #[test]
    fn a_handicap_weighs_a_horse_by_how_it_stands_against_its_class() {
        let r = rule(AgeRule::FourUp, WeightRule::Handicap);
        let with = |speed| carried(&r, 20, 5, false, 0, speed, 70);
        assert_eq!(with(70), 55);
        assert_eq!(with(74), 57);
        assert_eq!(with(66), 53);
    }

    #[test]
    fn a_handicap_has_limits_and_ignores_sex_and_wins() {
        let r = rule(AgeRule::FourUp, WeightRule::Handicap);
        assert_eq!(carried(&r, 20, 5, false, 0, 100, 60), 60);
        assert_eq!(carried(&r, 20, 5, false, 0, 10, 80), 48);
        assert_eq!(
            carried(&r, 20, 5, true, 9, 74, 70),
            carried(&r, 20, 5, false, 0, 74, 70)
        );
    }

    #[test]
    fn a_heavier_load_slows_a_horse_and_a_lighter_one_helps() {
        assert_eq!(multiplier(56), 1.0);
        assert!(multiplier(58) < 1.0 && multiplier(54) > 1.0);
        assert!((multiplier(58) - 0.994).abs() < 1e-9);
        assert!((multiplier(48) - 1.024).abs() < 1e-9);
    }
}
