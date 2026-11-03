//! How a horse's age and the race distance change how well it runs.
//!
//! * Growth (成長): a horse runs at its best around the age its growth type peaks
//!   at (早熟 2-3, 普通 4-5, 晩成 6-7). It is a year or so either side of that
//!   with no loss; younger it is still developing, older it is slowing down.
//! * Distance aptitude (距離適性): each horse has a range of distances it is
//!   best at. Beyond it, stamina gives out (and speed a little); short of it, the
//!   horse cannot use its speed.

use serde::{Deserialize, Serialize};

/// Years either side of the peak age with no loss.
const PLATEAU: i32 = 1;
/// Loss per year of youth beyond the plateau, and its limit.
const YOUTH_LOSS: f64 = 0.04;
const YOUTH_FLOOR: f64 = 0.80;
/// Loss per year of age beyond the plateau, and its limit.
const AGE_LOSS: f64 = 0.03;
const AGE_FLOOR: f64 = 0.75;

/// Metres per step of distance aptitude.
const STEP: u32 = 200;
/// Too long: stamina lost per step beyond the range, and its limit.
const LONG_STAMINA_LOSS: f64 = 0.06;
const LONG_STAMINA_FLOOR: f64 = 0.60;
/// Too long: speed lost per step beyond the range, and its limit.
const LONG_SPEED_LOSS: f64 = 0.01;
const LONG_SPEED_FLOOR: f64 = 0.90;
/// Too short: speed lost per step short of the range, and its limit.
const SHORT_SPEED_LOSS: f64 = 0.03;
const SHORT_SPEED_FLOOR: f64 = 0.80;

/// Where a horse is on its growth curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Growth {
    /// Still developing.
    Rising,
    /// In its prime.
    Peak,
    /// Past its best.
    Declining,
}

pub fn growth_stage(age: u8, peak_age: u8) -> Growth {
    let diff = age as i32 - peak_age as i32;
    if diff < -PLATEAU {
        Growth::Rising
    } else if diff > PLATEAU {
        Growth::Declining
    } else {
        Growth::Peak
    }
}

/// Share of its ability a horse of this age shows (1.0 in its prime).
pub fn age_form(age: u8, peak_age: u8) -> f64 {
    let diff = age as i32 - peak_age as i32;
    if diff < -PLATEAU {
        (1.0 - YOUTH_LOSS * (-diff - PLATEAU) as f64).max(YOUTH_FLOOR)
    } else if diff > PLATEAU {
        (1.0 - AGE_LOSS * (diff - PLATEAU) as f64).max(AGE_FLOOR)
    } else {
        1.0
    }
}

/// How a race distance sits against a horse's range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistanceFit {
    Fit,
    TooShort,
    TooLong,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistanceEffect {
    pub fit: DistanceFit,
    /// Steps of 200m outside the range (0 when it fits).
    pub steps: u32,
    pub speed: f64,
    pub stamina: f64,
}

/// What running `distance` metres does to a horse best over `min..=max`.
pub fn distance_effect(distance: u32, min: u32, max: u32) -> DistanceEffect {
    if distance > max {
        let steps = (distance - max).div_ceil(STEP);
        DistanceEffect {
            fit: DistanceFit::TooLong,
            steps,
            speed: (1.0 - LONG_SPEED_LOSS * steps as f64).max(LONG_SPEED_FLOOR),
            stamina: (1.0 - LONG_STAMINA_LOSS * steps as f64).max(LONG_STAMINA_FLOOR),
        }
    } else if distance < min {
        let steps = (min - distance).div_ceil(STEP);
        DistanceEffect {
            fit: DistanceFit::TooShort,
            steps,
            speed: (1.0 - SHORT_SPEED_LOSS * steps as f64).max(SHORT_SPEED_FLOOR),
            stamina: 1.0,
        }
    } else {
        DistanceEffect {
            fit: DistanceFit::Fit,
            steps: 0,
            speed: 1.0,
            stamina: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn a_horse_within_a_year_of_its_peak_runs_at_full_ability() {
        for (age, peak) in [(4u8, 4u8), (3, 4), (5, 4), (2, 3), (4, 3), (7, 6), (5, 6)] {
            assert_eq!(age_form(age, peak), 1.0, "age {age} peak {peak}");
            assert_eq!(growth_stage(age, peak), Growth::Peak);
        }
    }

    #[test]
    fn a_young_horse_that_peaks_late_is_held_back_four_percent_a_year() {
        // a peak at 6: 4 is one year beyond the plateau
        assert!(close(age_form(4, 6), 0.96));
        assert!(close(age_form(3, 6), 0.92));
        assert!(close(age_form(2, 6), 0.88));
        assert_eq!(growth_stage(3, 6), Growth::Rising);
    }

    #[test]
    fn youth_costs_at_most_twenty_percent() {
        assert!(close(age_form(2, 7), 0.84));
        assert!(close(age_form(1, 10), 0.80));
        assert!(close(age_form(0, 20), 0.80));
    }

    #[test]
    fn an_old_horse_slows_three_percent_a_year_after_its_prime() {
        // a peak at 3: 5 is the last year of the plateau
        assert_eq!(age_form(5, 4), 1.0);
        assert!(close(age_form(6, 4), 0.97));
        assert!(close(age_form(7, 3), 0.91));
        assert!(close(age_form(8, 3), 0.88));
        assert_eq!(growth_stage(7, 3), Growth::Declining);
    }

    #[test]
    fn age_costs_at_most_twenty_five_percent() {
        assert!(close(age_form(20, 2), 0.75));
        assert!(close(age_form(10, 3), 0.82));
        assert!(close(age_form(13, 3), 0.75));
    }

    #[test]
    fn the_growth_curve_rises_then_stays_then_falls() {
        let peak = 5u8;
        let curve: Vec<f64> = (0..=12u8).map(|age| age_form(age, peak)).collect();
        let top = curve.iter().cloned().fold(0.0, f64::max);
        assert_eq!(top, 1.0);
        for age in 1..=12usize {
            if age < peak as usize {
                assert!(curve[age] >= curve[age - 1], "rising at {age}");
            } else if age >= peak as usize + 2 {
                assert!(curve[age] <= curve[age - 1], "falling at {age}");
            }
        }
        for v in curve {
            assert!((0.75..=1.0).contains(&v));
        }
    }

    #[test]
    fn early_and_late_bloomers_are_best_at_different_ages() {
        // an early horse (peak 2) beats a late one (peak 7) at two, loses to it at seven
        assert!(age_form(2, 2) > age_form(2, 7));
        assert!(age_form(7, 7) > age_form(7, 2));
    }

    #[test]
    fn a_distance_within_the_range_costs_nothing() {
        for d in [1600u32, 1800, 2000, 2400] {
            let e = distance_effect(d, 1600, 2400);
            assert_eq!(
                e,
                DistanceEffect {
                    fit: DistanceFit::Fit,
                    steps: 0,
                    speed: 1.0,
                    stamina: 1.0
                }
            );
        }
    }

    #[test]
    fn running_beyond_the_range_drains_stamina_six_percent_a_step() {
        let e = distance_effect(2600, 1600, 2400);
        assert_eq!((e.fit, e.steps), (DistanceFit::TooLong, 1));
        assert!(close(e.stamina, 0.94) && close(e.speed, 0.99));
        let e = distance_effect(2800, 1600, 2400);
        assert_eq!(e.steps, 2);
        assert!(close(e.stamina, 0.88) && close(e.speed, 0.98));
    }

    #[test]
    fn too_long_a_distance_has_limits_on_the_loss() {
        let e = distance_effect(3600, 1000, 1400); // 11 steps
        assert_eq!(e.steps, 11);
        assert!(close(e.stamina, 0.60) && close(e.speed, 0.90));
    }

    #[test]
    fn running_short_of_the_range_costs_speed_three_percent_a_step() {
        let e = distance_effect(1400, 1800, 2600);
        assert_eq!((e.fit, e.steps), (DistanceFit::TooShort, 2));
        assert!(close(e.speed, 0.94) && close(e.stamina, 1.0));
    }

    #[test]
    fn too_short_a_distance_has_a_limit_on_the_loss() {
        let e = distance_effect(1000, 3000, 3600); // 10 steps
        assert!(close(e.speed, 0.80) && close(e.stamina, 1.0));
    }

    #[test]
    fn a_part_step_counts_as_a_whole_one() {
        assert_eq!(distance_effect(2450, 1600, 2400).steps, 1);
        assert_eq!(distance_effect(1550, 1600, 2400).steps, 1);
        assert_eq!(distance_effect(2401, 1600, 2400).steps, 1);
    }

    #[test]
    fn the_boundaries_of_the_range_are_inside_it() {
        assert_eq!(distance_effect(1600, 1600, 2400).fit, DistanceFit::Fit);
        assert_eq!(distance_effect(2400, 1600, 2400).fit, DistanceFit::Fit);
        assert_eq!(distance_effect(2600, 1600, 2400).fit, DistanceFit::TooLong);
        assert_eq!(distance_effect(1400, 1600, 2400).fit, DistanceFit::TooShort);
    }

    #[test]
    fn being_too_far_out_is_worse_than_being_a_little_out() {
        let near = distance_effect(2600, 1600, 2400);
        let far = distance_effect(3200, 1600, 2400);
        assert!(far.stamina < near.stamina && far.speed < near.speed);
    }

    #[test]
    fn the_json_names_are_lowercase_and_snake_case() {
        assert_eq!(
            serde_json::to_string(&Growth::Rising).unwrap(),
            "\"rising\""
        );
        assert_eq!(
            serde_json::to_string(&Growth::Declining).unwrap(),
            "\"declining\""
        );
        assert_eq!(
            serde_json::to_string(&DistanceFit::TooLong).unwrap(),
            "\"too_long\""
        );
        assert_eq!(serde_json::to_string(&DistanceFit::Fit).unwrap(), "\"fit\"");
    }
}
