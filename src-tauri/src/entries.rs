//! How many horses a race takes: who else has been put forward for it (登録頭数),
//! how many its course can start (フルゲート), and the draw (抽選) when there are
//! more horses than places.

use crate::calendar::{Grade, Venue};

/// The most runners any race has.
pub const FULL_FIELD: usize = 18;
/// The fewest runners a race on the calendar has.
pub const MIN_FIELD: usize = 6;
/// What the small courses can start.
const SMALL_COURSE_GATE: usize = 16;
/// What the local tracks can start.
const OI_GATE: usize = 16;
const KANTO_LOCAL_GATE: usize = 14;
const SONODA_GATE: usize = 12;

/// The most horses a race at this course can start.
pub fn full_gate(venue: Venue) -> usize {
    match venue {
        Venue::Sapporo | Venue::Hakodate | Venue::Fukushima => SMALL_COURSE_GATE,
        Venue::Oi => OI_GATE,
        Venue::Kawasaki | Venue::Funabashi => KANTO_LOCAL_GATE,
        Venue::Sonoda => SONODA_GATE,
        _ => FULL_FIELD,
    }
}

/// How many horses other stables may put forward for a race of this class, fewest
/// and most. The bigger the race, the more want to run in it.
pub fn nominated_range(grade: Grade) -> (usize, usize) {
    match grade {
        Grade::G1 => (16, 24),
        Grade::G2 | Grade::G3 => (12, 22),
        Grade::Listed | Grade::Open => (9, 20),
        Grade::Win3 | Grade::Win2 => (7, 20),
        Grade::Win1 | Grade::Maiden => (8, 24),
        Grade::Debut => (MIN_FIELD, 20),
    }
}

/// How many horses other stables have put forward for a race, from its seed.
pub fn nominated_rivals(grade: Grade, seed: u64) -> usize {
    let (fewest, most) = nominated_range(grade);
    fewest + (seed % (most - fewest + 1) as u64) as usize
}

/// Whether a horse is drawn out when `rivals` others are after `gate` places: the
/// more of them there are, the likelier. `horse_seed` settles it for this horse.
pub fn drawn_out(rivals: usize, gate: usize, horse_seed: u64) -> bool {
    let all = rivals + 1;
    let too_many = all.saturating_sub(gate);
    horse_seed % 100 < (too_many * 100 / all) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    const GRADES: [Grade; 10] = [
        Grade::G1,
        Grade::G2,
        Grade::G3,
        Grade::Listed,
        Grade::Open,
        Grade::Win3,
        Grade::Win2,
        Grade::Win1,
        Grade::Maiden,
        Grade::Debut,
    ];

    #[test]
    fn the_small_courses_start_fewer_horses_than_the_big_ones() {
        for v in [Venue::Sapporo, Venue::Hakodate, Venue::Fukushima] {
            assert_eq!(full_gate(v), 16, "{v:?}");
        }
        for v in [
            Venue::Niigata,
            Venue::Tokyo,
            Venue::Nakayama,
            Venue::Chukyo,
            Venue::Kyoto,
            Venue::Hanshin,
            Venue::Kokura,
        ] {
            assert_eq!(full_gate(v), FULL_FIELD, "{v:?}");
        }
    }

    #[test]
    fn every_class_has_enough_horses_for_a_race_and_can_fill_the_gate() {
        for g in GRADES {
            let (fewest, most) = nominated_range(g);
            assert!(fewest >= MIN_FIELD, "{g:?}");
            assert!(most > FULL_FIELD, "{g:?}");
        }
    }

    #[test]
    fn the_graded_races_draw_more_horses_than_the_others() {
        let fewest = |g| nominated_range(g).0;
        assert!(fewest(Grade::G1) >= 16);
        assert!(fewest(Grade::G1) > fewest(Grade::G2));
        assert_eq!(fewest(Grade::G2), fewest(Grade::G3));
        for g in [
            Grade::Listed,
            Grade::Open,
            Grade::Win3,
            Grade::Win2,
            Grade::Win1,
            Grade::Maiden,
            Grade::Debut,
        ] {
            assert!(fewest(Grade::G3) > fewest(g), "{g:?}");
        }
        assert_eq!(fewest(Grade::Debut), MIN_FIELD);
    }

    #[test]
    fn the_number_put_forward_stays_within_its_range_and_uses_all_of_it() {
        for g in GRADES {
            let (fewest, most) = nominated_range(g);
            let seen: Vec<usize> = (0..2000u64)
                .map(|seed| nominated_rivals(g, crate::rivals::mix_seed(&[seed])))
                .collect();
            assert_eq!(seen.iter().copied().min(), Some(fewest), "{g:?}");
            assert_eq!(seen.iter().copied().max(), Some(most), "{g:?}");
        }
    }

    #[test]
    fn nobody_is_drawn_out_while_there_are_places_for_all() {
        for seed in 0..1000u64 {
            let seed = crate::rivals::mix_seed(&[seed]);
            assert!(!drawn_out(5, 18, seed));
            // the last place is this horse's
            assert!(!drawn_out(17, 18, seed));
        }
    }

    #[test]
    fn the_more_horses_there_are_after_the_places_the_likelier_the_draw_goes_against_a_horse() {
        let out = |rivals| {
            (0..5000u64)
                .filter(|&s| drawn_out(rivals, 18, crate::rivals::mix_seed(&[s])))
                .count()
        };
        let (few, many) = (out(19), out(24));
        // 2 too many of 20, and 7 too many of 25
        assert!((300..=700).contains(&few), "{few}");
        assert!((1150..=1650).contains(&many), "{many}");
    }
}
