//! Goals, rivals and what the stable has achieved (目標・ライバル・実績).

use super::*;

// ---- goals and the owner's standing (目標と称号) ----

fn goal(g: &Game, name: &str) -> bool {
    g.state()
        .goals
        .iter()
        .find(|x| x.name == name)
        .unwrap()
        .done
}

#[test]
fn a_new_owner_starts_at_the_bottom_with_every_goal_ahead() {
    let st = Game::with_seed(1).state();
    assert_eq!(st.owner_rank, "新米馬主");
    assert!(st.goals.len() >= 8);
    assert!(st.goals.iter().all(|x| !x.done));
}

#[test]
fn the_owners_standing_rises_with_earnings_and_g1_wins() {
    let mut g = Game::with_seed(1);
    g.trainer_records[0].earnings = 2_000;
    assert_eq!(g.state().owner_rank, "駆け出し馬主");
    g.trainer_records[0].earnings = 200_000;
    assert_eq!(g.state().owner_rank, "名門馬主");
    g.trainer_records[1].g1_wins = 30;
    assert_eq!(g.state().owner_rank, "伝説の馬主");
}

#[test]
fn goals_come_true_from_the_records() {
    let mut g = Game::with_seed(1);
    assert!(!goal(&g, "初勝利"));
    g.trainer_records[0].wins = 1;
    assert!(goal(&g, "初勝利"));
    g.trainer_records[0].g1_wins = 1;
    assert!(goal(&g, "G1初制覇"));
    assert!(!goal(&g, "G1を10勝"));
}

#[test]
fn a_goal_is_announced_once_and_stays_done_when_its_horse_has_gone() {
    let mut g = Game::with_seed(1);
    g.money = 1_000_000;
    let i = 0;
    g.field[i].career.stakes_wins.push(StakesWin {
        year: 1,
        race: "X".into(),
        grade: Grade::G3,
    });
    g.advance_week();
    assert!(g
        .news
        .iter()
        .any(|n| n.contains("目標達成") && n.contains("重賞初制覇")));
    assert!(goal(&g, "重賞初制覇"));
    g.advance_week();
    assert!(!g.news.iter().any(|n| n.contains("重賞初制覇")));
    g.field[i].career.stakes_wins.clear();
    assert!(goal(&g, "重賞初制覇"));
    assert!(Game::from_json(&g.to_json())
        .unwrap()
        .state()
        .goals
        .iter()
        .any(|x| x.name == "重賞初制覇" && x.done));
}

#[test]
fn a_g1_won_by_a_home_bred_horse_is_a_goal_of_its_own() {
    let mut g = Game::with_seed(1);
    breed_default(&mut g).unwrap();
    let i = g
        .field
        .iter()
        .position(|h| h.name == "ニューホープ")
        .unwrap();
    assert!(!goal(&g, "自家生産馬でG1制覇"));
    let founder = (i + 1) % g.field.len();
    g.field[founder].career.stakes_wins.push(StakesWin {
        year: 1,
        race: "X".into(),
        grade: Grade::G1,
    });
    assert!(!goal(&g, "自家生産馬でG1制覇"));
    g.field[i].career.stakes_wins.push(StakesWin {
        year: 1,
        race: "X".into(),
        grade: Grade::G1,
    });
    assert!(goal(&g, "自家生産馬でG1制覇"));
}

// ---- the rivals who race on year after year (ライバル) ----
#[test]
fn a_field_with_rivals_offers_no_ticket_that_pays_well_above_its_chance() {
    let mut worst: f64 = 0.0;
    for seed in [2u64, 3, 4, 5] {
        let mut g = Game::with_seed(seed);
        g.week = 47;
        let race = calendar_race("国際友好ステークス");
        let spec = g.race_spec(Some(&key_of(&race))).unwrap();
        let priced = Game::priced_n(&spec, 1000); // as many as the game itself runs
        let names: Vec<String> = spec.entrants.iter().map(|e| e.name.clone()).collect();
        let mut fav: Vec<usize> = (0..names.len()).collect();
        fav.sort_by(|&a, &b| priced.chances[b].0.total_cmp(&priced.chances[a].0));
        let orders: Vec<Vec<String>> = (0..2000u64)
            .map(|s| {
                Game::run_race(&spec, 7_000_000 + s)
                    .1
                    .iter()
                    .map(|r| r.name.clone())
                    .collect()
            })
            .collect();
        for (kind, idx) in [
            (BetKind::Exacta, vec![fav[0], fav[1]]),
            (BetKind::Exacta, vec![fav[1], fav[0]]),
            (BetKind::Quinella, vec![fav[0], fav[1]]),
            (BetKind::Trio, vec![fav[0], fav[1], fav[2]]),
        ] {
            let picks: Vec<String> = idx.iter().map(|&i| names[i].clone()).collect();
            let hits = orders
                .iter()
                .filter(|o| bets::is_hit(kind, &picks, o))
                .count() as f64;
            if hits < 80.0 {
                continue;
            }
            let back = hits / orders.len() as f64 * Game::ticket_odds(&priced, kind, &idx);
            worst = worst.max(back);
        }
    }
    assert!(worst > 0.0);
    assert!(worst < 1.25, "a ticket returns {worst:.2}");
}

#[test]
fn a_combination_comes_in_by_the_first_home_in_the_right_way() {
    let top = [3usize, 5, 7];
    assert!(Game::combo_hit(BetKind::Exacta, &[3, 5], &top));
    assert!(!Game::combo_hit(BetKind::Exacta, &[5, 3], &top));
    assert!(Game::combo_hit(BetKind::Quinella, &[5, 3], &top));
    assert!(!Game::combo_hit(BetKind::Quinella, &[3, 7], &top));
    assert!(Game::combo_hit(BetKind::Trifecta, &[3, 5, 7], &top));
    assert!(!Game::combo_hit(BetKind::Trifecta, &[5, 3, 7], &top));
    assert!(Game::combo_hit(BetKind::Trio, &[7, 3, 5], &top));
    assert!(!Game::combo_hit(BetKind::Trio, &[3, 5, 8], &top));
}

/// A race on the calendar of this grade this horse could be aimed at, and the star to aim.
fn star_for(race: &CalendarRace) -> Star {
    let mut rng = rivals::Rng::new(5);
    let mut star = rivals::new_star(&mut rng, 4, &Default::default());
    star.name = "ライバルオー".to_string();
    star.sex = if race.eligibility.fillies_only {
        Sex::Female
    } else {
        Sex::Male
    };
    star.age = match race.eligibility.age {
        AgeRule::Two => 2,
        AgeRule::Three => 3,
        _ => 4,
    };
    star.peak_age = star.age;
    star.speed = 95;
    star.apt_min = 1000;
    star.apt_max = 3600;
    star.dirt = 1;
    star.last_run = 0;
    star
}

fn week_with(g: &mut Game, name: &str) -> (CalendarRace, RaceKey) {
    let race = calendar_race(name);
    g.week = race.week;
    let key = key_of(&race);
    (race, key)
}

#[test]
fn a_new_game_has_rivals_of_every_age() {
    let g = Game::new();
    assert_eq!(g.stars.len(), 15);
    for age in 2..=6u8 {
        assert_eq!(g.stars.iter().filter(|s| s.age == age).count(), 3);
    }
    let names: std::collections::HashSet<&String> = g.stars.iter().map(|s| &s.name).collect();
    assert_eq!(names.len(), 15);
    assert!(g
        .stars
        .iter()
        .all(|s| g.field.iter().all(|h| h.name != s.name)));
    assert!(g.stars.iter().all(|s| (76..=96).contains(&s.speed)));
    let v = serde_json::to_value(g.state()).unwrap();
    assert_eq!(v["stars"].as_array().unwrap().len(), 15);
}

#[test]
fn a_rival_runs_in_the_race_it_is_aimed_at_and_no_other() {
    let mut g = Game::new();
    g.money = 100_000;
    let (race, key) = week_with(&mut g, "国際友好ステークス");
    g.stars = vec![star_for(&race)];
    let spec = g.race_spec(Some(&key)).unwrap();
    let star = spec
        .entrants
        .iter()
        .find(|e| e.name == "ライバルオー")
        .unwrap();
    assert!(star.star && !star.mine);
    assert!(star.course_fit.is_some() && star.weight.is_some());
    assert_eq!(spec.entrants.len(), entries::FULL_FIELD);
    let names: std::collections::HashSet<&String> = spec.entrants.iter().map(|e| &e.name).collect();
    assert_eq!(names.len(), spec.entrants.len());
    // it runs nowhere else that week
    let others = calendar::shared()
        .races
        .iter()
        .filter(|r| r.week == race.week && r.name != race.name);
    for r in others {
        let spec = g.race_spec(Some(&key_of(r))).unwrap();
        assert!(
            spec.entrants.iter().all(|e| e.name != "ライバルオー"),
            "{}",
            r.name
        );
    }
    let card = g.race_card(Some(&key)).unwrap();
    assert!(card
        .entrants
        .iter()
        .any(|e| e.star && e.name == "ライバルオー"));
}

#[test]
fn a_rival_goes_only_to_races_it_could_run_in() {
    let mut g = Game::new();
    let (race, _) = week_with(&mut g, "ジュニアティアラ"); // 2歳牝馬
    let mut star = star_for(&calendar_race("国際友好ステークス"));
    g.stars = vec![star.clone()];
    assert!(g.star_target(&star).is_none_or(|r| !(r.day == race.day
        && r.venue == race.venue
        && r.race_no == race.race_no)));
    // a horse without the distance stays away
    let (long, _) = week_with(&mut g, "国際友好ステークス");
    star.apt_min = 3000;
    star.apt_max = 3600;
    assert!(g.star_target(&star).is_none_or(|r| r.name != long.name));
    // nor does it race for anything less than a graded prize
    star.apt_min = 1000;
    for week in 1..=52 {
        g.week = week;
        if let Some(r) = g.star_target(&star) {
            assert!(matches!(r.grade, Grade::G1 | Grade::G2 | Grade::G3));
        }
    }
}

#[test]
fn a_rival_takes_time_between_races() {
    let mut g = Game::new();
    let (race, _) = week_with(&mut g, "国際友好ステークス");
    let mut star = star_for(&race);
    star.last_run = g.absolute_week() - 1;
    assert!(g.star_target(&star).is_none());
    star.last_run = g.absolute_week() - STAR_GAP;
    assert!(g.star_target(&star).is_some());
}

#[test]
fn a_rivals_races_are_run_whether_or_not_the_stable_is_there() {
    let mut g = Game::new();
    g.money = 100_000;
    let (race, _) = week_with(&mut g, "国際友好ステークス");
    g.stars = vec![star_for(&race)];
    g.advance_week();
    let star = &g.stars[0];
    assert_eq!(star.starts, 1);
    assert!(star.last_run > 0);
    assert_eq!(star.met, 0, "none of the stable ran against it");
}

#[test]
fn a_rival_counts_its_wins_and_big_ones_make_the_news() {
    let mut g = Game::new();
    g.money = 100_000;
    let (race, _) = week_with(&mut g, "国際友好ステークス");
    let mut star = star_for(&race);
    star.speed = 400; // unbeatable
    star.stamina = 400;
    g.stars = vec![star];
    g.advance_week();
    let star = &g.stars[0];
    assert_eq!((star.wins, star.g1_wins), (1, 1));
    assert_eq!(
        star.earnings,
        (race.prize as f64 * PRIZE_RATE).round() as u32
    );
    assert!(
        g.news
            .iter()
            .any(|n| n.contains("ライバルオー") && n.contains(&race.name)),
        "{:?}",
        g.news
    );
}

#[test]
fn meetings_with_the_stable_are_counted() {
    let mut g = Game::new();
    g.money = 100_000;
    let (race, key) = week_with(&mut g, "国際友好ステークス");
    let mine = my_runner(&g, &key);
    let i = g.field.iter().position(|h| h.name == mine).unwrap();
    let mut star = star_for(&race);
    star.speed = 400;
    star.stamina = 400;
    g.stars = vec![star];
    let rival = a_rival_in(&g, &key);
    g.play_combo(Some(&key), &[rival], 10, 1, BetKind::Win)
        .unwrap();
    let star = &g.stars[0];
    assert_eq!((star.starts, star.met, star.beaten), (1, 1, 0));
    assert_eq!(g.field[i].record.wins, default_record(&mine).wins);
    // and the other way about
    let mut g = Game::new();
    g.money = 100_000;
    let (race, key) = week_with(&mut g, "国際友好ステークス");
    let mine = my_runner(&g, &key);
    let i = g.field.iter().position(|h| h.name == mine).unwrap();
    g.field[i].speed = 4000;
    g.field[i].stamina = 4000;
    g.stars = vec![star_for(&race)];
    let rival = a_rival_in(&g, &key);
    g.play_combo(Some(&key), &[rival], 10, 1, BetKind::Win)
        .unwrap();
    assert_eq!((g.stars[0].met, g.stars[0].beaten), (1, 1));
}

#[test]
fn rivals_grow_older_and_new_ones_come_along() {
    let mut g = Game::new();
    g.week = calendar::WEEKS_PER_YEAR;
    for s in &mut g.stars {
        s.g1_wins = 1; // only the great are remarked on when they go
    }
    let before: Vec<(String, u8)> = g.stars.iter().map(|s| (s.name.clone(), s.age)).collect();
    g.advance_week();
    assert_eq!(g.year, 2);
    for (name, age) in before {
        match g.stars.iter().find(|s| s.name == name) {
            Some(s) => assert_eq!(s.age, age + 1),
            None => assert!(age + 1 >= STAR_RETIRE_AGE, "{name} {age}"),
        }
    }
    assert_eq!(g.stars.iter().filter(|s| s.age == 2).count(), 3);
    assert_eq!(
        g.stars.iter().filter(|s| s.age >= STAR_RETIRE_AGE).count(),
        0
    );
    assert!(g.news.iter().any(|n| n.contains("引退")), "{:?}", g.news);
}

#[test]
fn a_foal_cannot_take_a_rivals_name() {
    let mut g = Game::new();
    g.money = 100_000;
    let (sire, dam) = stallion_and_mare(&g);
    g.stars[0].name = "ハナミズキ".to_string();
    let slot = g.field[7].name.clone();
    assert_eq!(
        g.breed_now(&sire, &dam, &slot, "ハナミズキ"),
        Err(GameError::DuplicateName)
    );
}

#[test]
fn rivals_are_kept_in_a_save_and_an_old_save_gets_them() {
    let mut g = Game::new();
    g.stars[0].wins = 7;
    let back = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(back.stars, g.stars);
    let mut old = old_save(&Game::new());
    old.as_object_mut().unwrap().remove("stars");
    let old = Game::from_json(&old.to_string()).unwrap();
    assert_eq!(old.stars.len(), 15);
}

// ---- what the stable has achieved (実績) ----

/// Makes one of the stable the winner of a race, unbeatable.
fn win_race(g: &mut Game, i: usize, name: &str) {
    let race = calendar_race(name);
    g.money = 100_000;
    g.week = race.week;
    g.field[i].fatigue = 0;
    g.field[i].away = None;
    g.field[i].resting = false;
    g.field[i].since_race = 99;
    g.field[i].speed = 4000;
    g.field[i].stamina = 4000;
    g.stars.clear();
    let key = key_of(&race);
    let me = g.field[i].name.clone();
    g.set_entries(Some(&key), std::slice::from_ref(&me))
        .unwrap();
    let rival = a_rival_in(g, &key);
    let out = g
        .play_combo(Some(&key), &[rival], 10, 1, BetKind::Win)
        .unwrap();
    assert_eq!(out.results[0].name, me, "should win {name}");
    g.field[i].away = None;
}

fn colt_of_three(g: &mut Game) -> usize {
    let i = (0..g.field.len())
        .find(|&i| g.field[i].sex == Sex::Male)
        .unwrap();
    g.field[i].age = 3;
    g.field[i].record = RaceRecord::default();
    i
}

fn filly_of_three(g: &mut Game) -> usize {
    let i = (0..g.field.len())
        .find(|&i| g.field[i].sex == Sex::Female)
        .unwrap();
    g.field[i].age = 3;
    g.field[i].record = RaceRecord::default();
    i
}

#[test]
fn a_win_goes_down_in_the_horses_career_and_its_trainers() {
    let mut g = Game::new();
    let i = colt_of_three(&mut g);
    let t = g.field[i].trainer as usize;
    win_race(&mut g, i, "疾風賞");
    let c = &g.field[i].career;
    assert_eq!(c.g1_wins(), 1);
    assert_eq!(c.stakes_wins[0].race, "疾風賞");
    assert_eq!(c.stakes_wins[0].year, 1);
    let prize = (calendar_race("疾風賞").prize as f64 * PRIZE_RATE).round() as u32;
    assert_eq!(c.earnings, prize);
    let r = &g.trainer_records[t];
    assert_eq!((r.starts, r.wins, r.g1_wins, r.earnings), (1, 1, 1, prize));
    let v = serde_json::to_value(g.state()).unwrap();
    assert_eq!(v["field"][i]["career"]["earnings"], prize);
    assert_eq!(
        v["trainer_records"].as_array().unwrap().len(),
        TRAINERS.len()
    );
}

#[test]
fn a_lesser_place_only_earns_money() {
    let mut g = Game::new();
    g.money = 100_000;
    g.week = 47;
    let race = calendar_race("国際友好ステークス");
    let spec = g.race_spec(Some(&key_of(&race))).unwrap();
    let mine = spec.entrants.iter().find(|e| e.mine).unwrap().name.clone();
    let i = g.field.iter().position(|h| h.name == mine).unwrap();
    let others: Vec<String> = spec
        .entrants
        .iter()
        .filter(|e| e.name != mine)
        .map(|e| e.name.clone())
        .collect();
    let mut names = vec![others[0].clone(), mine.clone()];
    names.extend(others[1..].iter().cloned());
    let results: Vec<RaceResult> = names
        .iter()
        .enumerate()
        .map(|(n, name)| RaceResult {
            rank: n as u32 + 1,
            name: name.clone(),
            finish_time: n as f64,
        })
        .collect();
    g.note_results(&spec, &results);
    assert!(g.field[i].career.stakes_wins.is_empty());
    assert_eq!(g.field[i].career.earnings, (1500.0 * 0.4) as u32);
}

#[test]
fn winning_all_three_classics_is_the_triple_crown() {
    let mut g = Game::new();
    let i = colt_of_three(&mut g);
    for name in ["疾風賞", "王冠ステークス"] {
        win_race(&mut g, i, name);
        assert!(g.field[i].career.titles.is_empty());
    }
    let horse = g.field[i].name.clone();
    win_race(&mut g, i, "錦秋ロングステークス");
    assert_eq!(g.field[i].career.titles, vec!["三冠".to_string()]);
    let a = g.awards.last().unwrap();
    assert_eq!(
        (a.title.as_str(), a.horse.as_str(), a.mine),
        ("三冠", horse.as_str(), true)
    );
    assert!(
        g.news
            .iter()
            .any(|n| n.contains(&horse) && n.contains("三冠")),
        "{:?}",
        g.news
    );
}

#[test]
fn fillies_have_a_triple_crown_of_their_own() {
    let mut g = Game::new();
    let i = filly_of_three(&mut g);
    for name in ["白梅賞", "月下美人賞", "秋桜ティアラ"] {
        win_race(&mut g, i, name);
    }
    assert_eq!(g.field[i].career.titles, vec!["牝馬三冠".to_string()]);
}

#[test]
fn the_year_turns_with_the_best_horse_as_horse_of_the_year() {
    let mut g = Game::new();
    let i = colt_of_three(&mut g);
    win_race(&mut g, i, "錦秋ロングステークス");
    let horse = g.field[i].name.clone();
    g.money = 100_000;
    g.week = calendar::WEEKS_PER_YEAR;
    g.advance_week();
    let a = g.awards.iter().find(|a| a.title == "年度代表馬").unwrap();
    assert_eq!(
        (a.year, a.horse.as_str(), a.mine),
        (1, horse.as_str(), true)
    );
    assert!(g.field[i].career.titles.contains(&"年度代表馬".to_string()));
    assert_eq!(g.field[i].career.year_g1, 0);
    assert!(g
        .news
        .iter()
        .any(|n| n.contains(&horse) && n.contains("年度代表馬")));
}

#[test]
fn a_rival_can_beat_the_stable_to_the_honour() {
    let mut g = Game::new();
    let i = colt_of_three(&mut g);
    win_race(&mut g, i, "錦秋ロングステークス");
    let mut star = star_for(&calendar_race("国際友好ステークス"));
    star.year_g1 = 3;
    g.stars = vec![star];
    g.money = 100_000;
    g.week = calendar::WEEKS_PER_YEAR;
    g.advance_week();
    let a = g.awards.iter().find(|a| a.title == "年度代表馬").unwrap();
    assert_eq!((a.horse.as_str(), a.mine), ("ライバルオー", false));
    assert_eq!(g.stars[0].year_g1, 0);
}

#[test]
fn no_horse_of_the_year_without_a_race_won() {
    let mut g = Game::new();
    g.stars.clear();
    g.money = 100_000;
    g.week = calendar::WEEKS_PER_YEAR;
    g.advance_week();
    assert!(g.awards.iter().all(|a| a.title != "年度代表馬"));
}

#[test]
fn a_great_horse_enters_the_hall_of_fame_when_it_retires() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    g.field[0].career.earnings = HALL_EARNINGS;
    g.field[0].record = RaceRecord {
        starts: 10,
        wins: 5,
    };
    g.retire(&name).unwrap();
    let h = g.hall.last().unwrap();
    assert_eq!(
        (h.name.as_str(), h.wins, h.earnings),
        (name.as_str(), 5, HALL_EARNINGS)
    );
    let stud = g.studs.iter().find(|s| s.name == name).unwrap();
    assert_eq!(stud.career.earnings, HALL_EARNINGS);
}

#[test]
fn enough_g1s_or_a_crown_are_enough_for_the_hall_and_less_is_not() {
    let win = |n: usize| {
        (0..n)
            .map(|k| StakesWin {
                year: 1,
                race: format!("{k}"),
                grade: Grade::G1,
            })
            .collect::<Vec<_>>()
    };
    let career = |n, titles: Vec<String>| Career {
        stakes_wins: win(n),
        titles,
        ..Career::default()
    };
    assert!(career(HALL_G1_WINS, vec![]).for_the_hall());
    assert!(!career(HALL_G1_WINS - 1, vec![]).for_the_hall());
    assert!(career(0, vec!["三冠".to_string()]).for_the_hall());
    assert!(!career(0, vec!["年度代表馬".to_string()]).for_the_hall());
    assert!(Career {
        earnings: HALL_EARNINGS,
        ..Career::default()
    }
    .for_the_hall());
}

#[test]
fn an_ordinary_horse_does_not_enter_the_hall_and_retiring_with_age_counts_too() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    g.retire(&name).unwrap();
    assert!(g.hall.is_empty());
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    g.field[0].career.earnings = HALL_EARNINGS;
    g.field[0].age = RETIRE_AGE - 1;
    g.money = 100_000;
    g.week = calendar::WEEKS_PER_YEAR;
    g.advance_week();
    assert_eq!(g.hall[0].name, name);
    assert!(
        g.news
            .iter()
            .any(|n| n.contains(&name) && n.contains("殿堂")),
        "{:?}",
        g.news
    );
}

#[test]
fn automatic_runs_count_towards_achievements_too() {
    let (mut g, _) = auto_runner();
    g.stars.clear();
    let t = g.field[0].trainer as usize;
    g.advance_week();
    assert_eq!(g.trainer_records[t].starts, 1);
}

#[test]
fn the_free_race_counts_for_nothing() {
    let mut g = Game::new();
    let name = g.field[0].name.clone();
    g.set_entries(None, std::slice::from_ref(&name)).unwrap();
    g.play_combo(None, &[name], 10, 1, BetKind::Win).unwrap();
    assert!(g.trainer_records.iter().all(|r| r.starts == 0));
    assert_eq!(g.field[0].career.earnings, 0);
}

#[test]
fn a_foal_starts_with_a_clean_career() {
    let mut g = Game::new();
    g.money = 100_000;
    let (sire, dam) = stallion_and_mare(&g);
    let slot = g
        .field
        .iter()
        .position(|h| h.name != sire && h.name != dam)
        .unwrap();
    g.field[slot].career.earnings = 500;
    let replace = g.field[slot].name.clone();
    g.breed_now(&sire, &dam, &replace, "ハナミズキ").unwrap();
    assert_eq!(g.field[slot].career, Career::default());
    g.breed_now(&sire, &dam, "", "サクラソウ").unwrap();
}

#[test]
fn achievements_are_kept_in_a_save_and_old_saves_start_clean() {
    let mut g = Game::new();
    let i = colt_of_three(&mut g);
    win_race(&mut g, i, "疾風賞");
    g.hall.push(HallEntry {
        name: "A".into(),
        year: 1,
        wins: 1,
        g1_wins: 1,
        earnings: 1,
        titles: vec![],
    });
    g.awards.push(Award {
        year: 1,
        title: "三冠".into(),
        horse: "A".into(),
        mine: true,
    });
    let back = Game::from_json(&g.to_json()).unwrap();
    assert_eq!(back.all(|h| h.career.clone()), g.all(|h| h.career.clone()));
    assert_eq!(back.hall, g.hall);
    assert_eq!(back.awards, g.awards);
    assert_eq!(back.trainer_records, g.trainer_records);
    let mut old = old_save(&Game::new());
    let o = old.as_object_mut().unwrap();
    for k in ["careers", "hall", "awards", "trainer_records"] {
        o.remove(k);
    }
    for s in o["studs"].as_array_mut().unwrap() {
        s.as_object_mut().unwrap().remove("career");
    }
    let old = Game::from_json(&old.to_string()).unwrap();
    assert_eq!(old.trainer_records.len(), TRAINERS.len());
}

#[test]
fn years_of_play_with_rivals_stay_consistent() {
    let mut g = Game::new();
    for i in 0..g.field.len() {
        g.field[i].auto_entry = true;
        g.field[i].auto_training = true;
    }
    for _ in 0..(3 * calendar::WEEKS_PER_YEAR as usize) {
        g.advance_week();
        let names: std::collections::HashSet<&String> = g.stars.iter().map(|s| &s.name).collect();
        assert_eq!(names.len(), g.stars.len());
        assert!(g.stars.iter().all(|s| s.age < STAR_RETIRE_AGE));
    }
    assert!(g.stars.iter().map(|s| s.starts).sum::<u32>() > 0);
    assert!(g.awards.iter().any(|a| a.title == "年度代表馬"));
    assert!(g.trainer_records.iter().map(|r| r.starts).sum::<u32>() > 0);
}
