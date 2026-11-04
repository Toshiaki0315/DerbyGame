//! What a horse has that no other has (固有特性).

use super::*;
use crate::bloodline::Special;

/// A rival-free field to race the first horse in.
fn speed_in(g: &Game, spec: &RaceSpec) -> u32 {
    let at = spec
        .entrants
        .iter()
        .position(|e| e.name == g.field[0].name)
        .unwrap();
    Game::runners_for(spec)[at].speed
}

fn entered(g: &mut Game, r: &CalendarRace) -> RaceSpec {
    g.week = r.week;
    g.field[0].age = 4;
    g.field[0].sex = Sex::Male;
    g.field[0].away = None;
    let name = g.field[0].name.clone();
    g.field[0].record = RaceRecord { starts: 5, wins: 3 };
    g.set_entries(Some(&RaceKey::of(r)), std::slice::from_ref(&name))
        .unwrap();
    g.race_spec(Some(&RaceKey::of(r))).unwrap()
}

fn big_race() -> CalendarRace {
    calendar::shared()
        .races
        .iter()
        .find(|r| {
            r.grade == Grade::G1
                && r.surface == Surface::Turf
                && r.eligibility.age == AgeRule::ThreeUp
                && !r.eligibility.fillies_only
        })
        .unwrap()
        .clone()
}

#[test]
fn a_big_race_horse_runs_above_itself_only_in_a_g1() {
    let mut g = Game::new();
    let r = big_race();
    let plain = speed_in(&g.clone(), &entered(&mut g, &r));
    g.field[0].specials = vec![Special::BigStage];
    let spec = g.race_spec(Some(&RaceKey::of(&r))).unwrap();
    assert!(speed_in(&g, &spec) > plain);
    let free = g.race_spec(None).unwrap();
    let f = free.entrants.iter().position(|e| e.name == g.field[0].name);
    if let Some(at) = f {
        let without =
            Game::run_stats(&free.entrants[at], free.surface, free.distance, free.track).0;
        assert_eq!(Game::runners_for(&free)[at].speed, without);
    }
}

#[test]
fn a_steady_horse_is_never_out_of_form() {
    let mut g = Game::new();
    g.field[0].condition = Condition::Poor;
    assert_eq!(g.stable_entrant(0).condition, Condition::Poor);
    g.field[0].specials = vec![Special::Steady];
    assert_eq!(g.stable_entrant(0).condition, Condition::Normal);
    g.field[0].condition = Condition::Great;
    assert_eq!(g.stable_entrant(0).condition, Condition::Great);
}

#[test]
fn a_fresh_horse_runs_above_itself_after_a_rest_and_not_otherwise() {
    let mut g = Game::new();
    let r = big_race();
    let spec = entered(&mut g, &r);
    let plain = speed_in(&g, &spec);
    g.field[0].specials = vec![Special::Fresh];
    g.field[0].since_race = 2;
    let spec = g.race_spec(Some(&RaceKey::of(&r))).unwrap();
    assert_eq!(speed_in(&g, &spec), plain);
    g.field[0].since_race = FRESH_WEEKS;
    let spec = g.race_spec(Some(&RaceKey::of(&r))).unwrap();
    assert!(speed_in(&g, &spec) > plain);
}

#[test]
fn a_mudlark_runs_above_itself_only_on_a_bad_track() {
    let mut g = Game::new();
    let r = big_race();
    let mut spec = entered(&mut g, &r);
    g.field[0].specials = vec![Special::Mudlark];
    spec.track = TrackCondition::Good;
    let at = spec
        .entrants
        .iter()
        .position(|e| e.name == g.field[0].name)
        .unwrap();
    spec.entrants[at] = {
        let mut e = g.stable_entrant(0);
        e.jockey = spec.entrants[at].jockey.clone();
        e.weight = spec.entrants[at].weight;
        e.course_fit = spec.entrants[at].course_fit;
        e
    };
    let firm = Game::runners_for(&spec)[at].speed;
    let firm_plain = Game::run_stats(&spec.entrants[at], spec.surface, spec.distance, spec.track).0;
    assert!(firm <= (firm_plain as f64 * 1.031).round() as u32);
    spec.track = TrackCondition::Heavy;
    let heavy = Game::runners_for(&spec)[at].speed;
    let heavy_plain =
        Game::run_stats(&spec.entrants[at], spec.surface, spec.distance, spec.track).0;
    assert!(heavy > heavy_plain, "{heavy} {heavy_plain}");
}

#[test]
fn a_sound_horse_breaks_down_half_as_often() {
    let breaks = |special: bool| {
        (0..3000u64)
            .filter(|&n| {
                let mut g = Game::with_seed(1);
                if special {
                    g.field[0].specials = vec![Special::Iron];
                }
                g.field[0].trainings = n as u32;
                g.field[0].legs = 40;
                g.field[0].fatigue = 40;
                g.roll_injury(0, 0.05, 0x1111);
                g.field[0].away.is_some()
            })
            .count()
    };
    let (plain, iron) = (breaks(false), breaks(true));
    assert!((iron as f64) < plain as f64 * 0.75, "{iron} {plain}");
    assert!(iron > 0);
}

#[test]
fn winning_a_g1_may_bring_out_the_big_race_character_and_says_so() {
    let r = big_race();
    let mut got = 0;
    for year in 1..=60 {
        let mut g = Game::new();
        g.year = year;
        let spec = entered(&mut g, &r);
        let name = g.field[0].name.clone();
        g.add_to_records(
            &spec,
            &[RaceResult {
                rank: 1,
                name: name.clone(),
                finish_time: 100.0,
                margin: String::new(),
            }],
        );
        if g.field[0].specials.contains(&Special::BigStage) {
            got += 1;
            assert!(
                g.news
                    .iter()
                    .any(|n| n.contains(&name) && n.contains("大舞台")),
                "{:?}",
                g.news
            );
        }
    }
    assert!((5..=30).contains(&got), "{got}");
}

#[test]
fn a_foal_may_take_after_its_parents_character() {
    let g = Game::new();
    let mut parents: Vec<BloodHorse> = g.base_db().iter().take(2).cloned().collect();
    parents[0].gender = Sex::Male;
    parents[1].gender = Sex::Female;
    let foals = |given: Vec<Special>| {
        let (mut s, mut d) = (parents[0].clone(), parents[1].clone());
        s.specials = given.clone();
        d.specials = given;
        let mut rng = rivals::Rng::new(3);
        (0..1000u32)
            .map(|n| {
                bloodline::breed(
                    &s,
                    &d,
                    11,
                    90_000 + n,
                    "ハナミズキ".to_string(),
                    &std::collections::HashMap::new(),
                    &mut rng,
                )
            })
            .filter(|f| f.specials.contains(&Special::BigStage))
            .count()
    };
    let with = foals(vec![Special::BigStage]);
    let without = foals(vec![]);
    assert!((250..=750).contains(&with), "{with}");
    assert!(without < 60, "{without}");
}

#[test]
fn a_horse_keeps_its_character_at_stud_and_passes_it_on() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    g.field[0].age = 5;
    g.field[0].specials = vec![Special::Steady];
    g.retire(&name).unwrap();
    let stud = g.studs.iter().find(|s| s.name == name).unwrap();
    assert_eq!(stud.specials, vec![Special::Steady]);
    let (rec, _) = g.parent(&name).unwrap();
    assert_eq!(rec.specials, vec![Special::Steady]);
}

#[test]
fn a_new_foal_brings_its_character_to_the_stable() {
    let (mut g, s, d, _) = in_season();
    g.money = 100_000;
    g.breed(&s, &d, "", "ハナミズキ").unwrap();
    let foal = g
        .pregnancies
        .iter()
        .find(|p| p.foal.name == "ハナミズキ")
        .map(|p| p.foal.specials.clone());
    weeks_pass(&mut g, GESTATION_WEEKS);
    if let (Some(specials), Some(h)) = (foal, g.field.iter().find(|h| h.name == "ハナミズキ"))
    {
        assert_eq!(h.specials, specials);
    }
}

#[test]
fn the_screen_is_told_each_horses_character_with_what_it_does() {
    let mut g = Game::new();
    g.field[0].specials = vec![Special::Mudlark];
    let v = serde_json::to_value(g.state()).unwrap();
    assert_eq!(v["field"][0]["specials"][0]["name"], "道悪巧者");
    assert!(v["field"][0]["specials"][0]["description"].is_string());
    let name = g.field[0].name.clone();
    g.set_entries(None, std::slice::from_ref(&name)).unwrap();
    let card = g.race_card(None).unwrap();
    let me = card.entrants.iter().find(|e| e.name == name).unwrap();
    assert_eq!(me.specials[0].name, "道悪巧者");
}

#[test]
fn old_saves_take_their_character_from_the_blood_and_a_save_keeps_it() {
    let mut g = Game::new();
    g.field[0].specials = vec![Special::Iron];
    let back = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(back.field[0].specials, vec![Special::Iron]);
    let old = Game::from_json(&old_save(&Game::new()).to_string()).unwrap();
    for h in &old.field {
        assert_eq!(h.specials, old.blood_record(h.blood_id).unwrap().specials);
    }
}
