// The state of the game on screen, and everything the player can do to it.

import { CalendarData, CalendarRace, VENUE_LABEL, admits, ageAllows, racesOfWeek } from "./Calendar";
import { AUTOPLAY_LOG_LIMIT, AUTOPLAY_RESERVE_WEEKS, FRAME_MS, FREE_RACE, HISTORY_LIMIT, KIND_PICKS, MAX_TICKETS, STRAIGHT_COURSE_FINISH, STRETCH_FRAME_MS, raceKey, raceValue, selectionLabel } from "./labels";
import { isValidHorseName } from "./naming";
import { raceView } from "./raceShow";
import { AuctionLot, BetKind, BuyStyle, CourseInfo, GameState, Gear, HistoryEntry, Outlook, RaceCard, RoundOutcome, Runner, SlipTicket, StudOffer, TabId, TrainerInfo, VenueFit } from "./types";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

export function useGame() {
  const [game, setGame] = useState<GameState | null>(null);
  // the horses picked for the ticket, in order (fixed up against the field when used)
  const [picks, setPicks] = useState<string[]>([]);
  const [quote, setQuote] = useState<number | null>(null);
  const [quoteError, setQuoteError] = useState("");
  const [kind, setKind] = useState<BetKind>("win");
  const [tab, setTab] = useState<TabId>("race");
  const [sire, setSire] = useState("");
  const [dam, setDam] = useState("");
  const [retire, setRetire] = useState("");
  // the training menu chosen for each horse (the first menu until one is picked)
  const [menus, setMenus] = useState<Record<string, string>>({});
  const [foalName, setFoalName] = useState("");
  const [outlook, setOutlook] = useState<Outlook | null>(null);
  const [outlookError, setOutlookError] = useState("");
  // the pedigree tab shows either the stable's horses or the whole pedigree database
  const [pedigreeView, setPedigreeView] = useState<"stable" | "database">("stable");
  const [amount, setAmount] = useState(100);
  // several tickets at once: the list, its odds, and how new tickets are made (ordinary, box or flow)
  const [slip, setSlip] = useState<SlipTicket[]>([]);
  const [slipOdds, setSlipOdds] = useState<number[]>([]);
  const [style, setStyle] = useState<BuyStyle>("normal");
  const [boxPicks, setBoxPicks] = useState<string[]>([]);
  const [axis, setAxis] = useState("");
  const [partners, setPartners] = useState<string[]>([]);
  const [outcome, setOutcome] = useState<(RoundOutcome & { stake: number; horse: string; kind: BetKind; ticketCount: number; mixed: boolean }) | null>(null);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  // number of replay frames shown so far; null when no race is in progress
  const [frame, setFrame] = useState<number | null>(null);
  const [calendar, setCalendar] = useState<CalendarData | null>(null);
  // "free" or "<day>:<venue>:<race_no>" of a race on this week's card
  const [raceSel, setRaceSel] = useState(FREE_RACE);
  // the field of the selected race; kept with the selection it was loaded for
  const [cardState, setCardState] = useState<{ race: string; card: RaceCard } | null>(null);
  const [cardVersion, setCardVersion] = useState(0); // bump to load the field again
  const card = cardState && cardState.race === raceSel ? cardState.card : null;

  const racing = outcome !== null && frame !== null && frame < outcome.frames.length;
  const [ranCard, setRanCard] = useState<RaceCard | null>(null);

  // オートプレイ: the weeks go on by themselves until stopped
  const [autoplay, setAutoplay] = useState(false);
  const [autoplayMs, setAutoplayMs] = useState(500);
  const [stopAtNewYear, setStopAtNewYear] = useState(true);
  const [autoplayLog, setAutoplayLog] = useState<string[]>([]);
  const busy = racing || autoplay;

  const load = (s: GameState) => {
    setGame(s);
  };

  // does something the game may refuse, showing why if it does
  const attempt = async (work: () => Promise<void>, clearNotice = false) => {
    setError("");
    if (clearNotice) setNotice("");
    try {
      await work();
    } catch (e) {
      setError(String(e));
    }
  };

  useEffect(() => {
    invoke<GameState>("get_game_state").then(load);
    invoke<CalendarData>("get_calendar")
      .then((c) => setCalendar(Array.isArray(c?.races) ? c : null))
      .catch(() => setCalendar(null));
  }, []);

  const weekRaces = racesOfWeek(calendar?.races ?? [], game?.week);
  const chosenRace = weekRaces.find((r) => raceValue(r) === raceSel) ?? null;
  // this week's races by day and course, as on a race card ("土 京都")
  const raceGroups: [string, CalendarRace[]][] = [];
  for (const r of weekRaces) {
    const label = `${r.day === "sat" ? "土" : "日"} ${VENUE_LABEL[r.venue]}`;
    const group = raceGroups.find(([l]) => l === label);
    if (group) group[1].push(r);
    else raceGroups.push([label, [r]]);
  }
  // whether one of my horses meets a race's conditions (unknown ages count as not)
  const canRun = (r: CalendarRace) =>
    (game?.field ?? []).some(
      (h) =>
        h.age !== undefined &&
        ageAllows(r, h.age, h.sex === "female") &&
        admits(r.grade, h.record?.starts ?? 0, h.record?.wins ?? 0)
    );

  // the pair that would be bred, and what the pairing would bring (asked of the backend)
  const parents = [...(game?.field ?? []), ...(game?.studs ?? [])];
  const [market, setMarket] = useState<StudOffer[]>([]);
  const [lots, setLots] = useState<AuctionLot[]>([]);
  const [bids, setBids] = useState<Record<number, { max: string; name: string }>>({});
  const [bidResult, setBidResult] = useState("");
  const [abroadHorse, setAbroadHorse] = useState<Record<number, string>>({});
  const [advice, setAdvice] = useState<{ horse: string; comment: string; proposal?: string | null }[]>([]);
  const parentNames = new Set(parents.map((h) => h.name));
  // stallions on the market can be used for a fee, without buying them
  const sires = (game?.studs ?? []).filter((h) => h.sex === "male");
  const outsideSires = market.filter((o) => o.sex === "male" && !parentNames.has(o.name));
  const maleNames = [...sires.map((h) => h.name), ...outsideSires.map((o) => o.name)];
  const femaleNames = (game?.studs ?? []).filter((h) => h.sex === "female").map((h) => h.name);
  const outlookSire = maleNames.includes(sire) ? sire : maleNames[0] ?? "";
  const outlookDam = femaleNames.includes(dam) ? dam : femaleNames[0] ?? "";
  useEffect(() => {
    setOutlook(null);
    setOutlookError("");
    if (tab !== "breeding" || !outlookSire || !outlookDam) return;
    let live = true;
    invoke<Outlook>("breeding_outlook", { sire: outlookSire, dam: outlookDam })
      .then((o) => live && setOutlook(o))
      .catch((e) => live && setOutlookError(String(e)));
    return () => {
      live = false;
    };
  }, [tab, outlookSire, outlookDam, game]);

  // the racecourses, asked for once the information tab is opened; how they suit the chosen horse
  const [courses, setCourses] = useState<CourseInfo[]>([]);
  const [fitHorse, setFitHorse] = useState("");
  const [venueFits, setVenueFits] = useState<VenueFit[]>([]);
  const fitHorseSel = game?.field.some((h) => h.name === fitHorse) ? fitHorse : game?.field[0]?.name ?? "";
  useEffect(() => {
    if (tab !== "info" || courses.length > 0) return;
    invoke<CourseInfo[]>("get_courses")
      .then((c) => setCourses(Array.isArray(c) ? c : []))
      .catch((e) => setError(String(e)));
  }, [tab, courses.length]);
  useEffect(() => {
    if (tab !== "info" || !fitHorseSel) return;
    let live = true;
    invoke<VenueFit[]>("get_course_fits", { horse: fitHorseSel })
      .then((f) => live && setVenueFits(Array.isArray(f) ? f : []))
      .catch(() => live && setVenueFits([]));
    return () => {
      live = false;
    };
  }, [tab, fitHorseSel, game]);

  // what the trainers have to say, asked for whenever the stable tab is open
  useEffect(() => {
    if (tab !== "stable") return;
    let live = true;
    invoke<typeof advice>("get_advice")
      .then((a) => live && setAdvice(Array.isArray(a) ? a : []))
      .catch(() => live && setAdvice([]));
    return () => {
      live = false;
    };
  }, [tab, game]);

  // the lots at the yearling sale, asked for whenever the breeding tab is open during it
  useEffect(() => {
    if (tab !== "breeding" || !game?.auction_open) {
      setLots([]);
      setBidResult("");
      return;
    }
    let live = true;
    invoke<AuctionLot[]>("get_auction")
      .then((l) => live && setLots(Array.isArray(l) ? l : []))
      .catch(() => live && setLots([]));
    return () => {
      live = false;
    };
  }, [tab, game]);

  // the stallions and mares on sale, asked for whenever the breeding tab is open and the stable changes
  useEffect(() => {
    if (tab !== "breeding") return;
    let live = true;
    invoke<StudOffer[]>("get_stud_market")
      .then((m) => live && setMarket(Array.isArray(m) ? m : []))
      .catch(() => live && setMarket([]));
    return () => {
      live = false;
    };
  }, [tab, game]);

  // who can be picked, and what the ticket is made of
  const runnerNames: string[] = card ? card.entrants.map((e) => e.name) : [];
  const pickCount = KIND_PICKS[kind];
  const selection: string[] = Array.from({ length: pickCount }, (_, i) =>
    runnerNames.includes(picks[i]) ? picks[i] : runnerNames[i % Math.max(runnerNames.length, 1)] ?? ""
  );
  const fieldTooSmall = card !== null && runnerNames.length < pickCount;
  const duplicate = card !== null && !fieldTooSmall && new Set(selection).size !== selection.length;
  const isSingle = pickCount === 1;
  const selectionKey = selection.join("|");
  const multiKind = pickCount >= 2;
  const effStyle: BuyStyle = multiKind ? style : "normal";
  const axisSel = runnerNames.includes(axis) ? axis : runnerNames[0] ?? "";
  const effBox = boxPicks.filter((n) => runnerNames.includes(n));
  const effPartners = partners.filter((n) => n !== axisSel && runnerNames.includes(n));
  const slipTotal = slip.reduce((t, x) => t + x.amount, 0);
  // a combination's odds come from the backend; wait for a calendar race's field to load first
  const quotable = !isSingle && !duplicate && !fieldTooSmall && card !== null && effStyle === "normal";

  useEffect(() => {
    setQuote(null);
    setQuoteError("");
    if (!quotable) return;
    let live = true;
    invoke<number>("quote_bet", { horses: selection, kind, race: chosenRace ? raceKey(chosenRace) : null })
      .then((q) => live && setQuote(q))
      .catch((e) => live && setQuoteError(String(e)));
    return () => {
      live = false;
    };
  }, [quotable, kind, selectionKey, raceSel, game?.week]); // eslint-disable-line react-hooks/exhaustive-deps

  // tickets belong to one race: start a new list when the race (or week) changes
  useEffect(() => {
    setSlip([]);
    setBoxPicks([]);
    setPartners([]);
  }, [raceSel, game?.week]);

  // the odds of every ticket in the list, asked for together
  useEffect(() => {
    if (slip.length === 0) {
      setSlipOdds([]);
      return;
    }
    let live = true;
    invoke<number[]>("quote_tickets", { tickets: slip, race: chosenRace ? raceKey(chosenRace) : null })
      .then((o) => live && setSlipOdds(Array.isArray(o) ? o : []))
      .catch((e) => live && setError(String(e)));
    return () => {
      live = false;
    };
  }, [slip, raceSel, game?.week]); // eslint-disable-line react-hooks/exhaustive-deps

  // the field (my entered horses and the rivals) of the selected race, free race included;
  // loaded again whenever the game changes, since stats, conditions and odds do
  useEffect(() => {
    if (!game || autoplay) return; // オートプレイ: asked for again once it stops
    let live = true;
    invoke<RaceCard>("get_race_card", { race: chosenRace ? raceKey(chosenRace) : null })
      .then((c) => live && setCardState(Array.isArray(c?.entrants) ? { race: raceSel, card: c } : null))
      .catch((e) => {
        if (!live) return;
        setError(String(e));
        if (chosenRace) setRaceSel(FREE_RACE);
      });
    return () => {
      live = false;
    };
  }, [raceSel, game, calendar, cardVersion, autoplay]); // eslint-disable-line react-hooks/exhaustive-deps

  // each week brought back starts the wait for the next
  useEffect(() => {
    if (!autoplay || !game) return;
    const id = setTimeout(async () => {
      try {
        const s = await invoke<GameState>("next_week");
        load(s);
        setRaceSel(FREE_RACE);
        setOutcome(null);
        const when = `${s.year}年目 第${s.week}週`;
        const lines = (s.news ?? []).map((n) => `${when}: ${n}`).reverse();
        setAutoplayLog((log) => [...lines, ...log].slice(0, AUTOPLAY_LOG_LIMIT));
        const stopped = s.game_over
          ? "ゲームオーバーのためオートプレイを止めました。"
          : s.field.length === 0
            ? "厩舎に馬がいなくなったのでオートプレイを止めました。"
            : s.money < (s.weekly_upkeep ?? 0) * AUTOPLAY_RESERVE_WEEKS
              ? s.field.every((h) => h.auto_entry)
                ? "所持金が少なくなったのでオートプレイを止めました。"
                : "所持金が少なくなったのでオートプレイを止めました。賞金が入るよう「自動出走」を確認してください。"
              : stopAtNewYear && s.year > game.year
              ? `${s.year}年目に入ったのでオートプレイを止めました。`
              : "";
        if (stopped) {
          setAutoplay(false);
          setNotice(stopped);
        }
      } catch (e) {
        setAutoplay(false);
        setError(String(e));
      }
    }, autoplayMs);
    return () => clearTimeout(id);
  }, [autoplay, autoplayMs, stopAtNewYear, game]);

  const liveNames = outcome?.runners ?? [];
  const liveFrame = outcome && frame ? outcome.frames[Math.min(frame, outcome.frames.length) - 1] : null;
  const finishOrder = (outcome?.results ?? []).map((r) => r.name);
  const live =
    outcome && liveFrame && liveNames.length > 0
      ? ranCard?.course?.straight_course
        ? raceView(liveFrame, liveNames, outcome.distance, STRAIGHT_COURSE_FINISH, 0, undefined, finishOrder)
        : raceView(liveFrame, liveNames, outcome.distance, ranCard?.course?.straight, ranCard?.course?.lap, ranCard?.course?.after_goal, finishOrder)
      : null;
  const inStretch = live?.stretch ?? false;
  useEffect(() => {
    if (!racing) return;
    const id = setInterval(() => setFrame((f) => (f ?? 0) + 1), inStretch ? STRETCH_FRAME_MS : FRAME_MS);
    return () => clearInterval(id);
  }, [racing, inStretch]);

  const finished = outcome !== null && !racing;
  useEffect(() => {
    if (finished && outcome?.race_name) {
      // a calendar race moved the game on a week (and maybe a year): take the server's state
      invoke<GameState>("get_game_state").then((s) => {
        load(s);
        setRaceSel(FREE_RACE);
        if (s.news && s.news.length > 0) setNotice(s.news.join(" "));
      });
      return;
    }
    if (finished && outcome) {
      const entry: HistoryEntry = {
        kind: outcome.kind,
        round: 0,
        horse: outcome.horse,
        stake: outcome.stake,
        won: outcome.won,
        payout: outcome.payout,
        prize: outcome.prize_total ?? 0,
        ticket_count: outcome.ticketCount,
        watched: outcome.ticketCount === 0,
        mixed_kinds: outcome.mixed,
        winner: outcome.results[0]?.name ?? "",
      };
      setGame((g) =>
        g
          ? {
              ...g,
              money: outcome.money,
              round: g.round + 1,
              game_over: outcome.money === 0,
              history: [...g.history, { ...entry, round: g.round + 1 }].slice(-HISTORY_LIMIT),
            }
          : g
      );
      // the race tired the horses that ran
      invoke<GameState>("get_game_state").then((s) => setGame((g) => (g ? { ...g, field: s.field } : g)));
    }
  }, [finished, outcome]);

  // the race panel moves on to the race chosen, leaving the one that was run behind
  const chooseRaceOf = (value: string) => {
    setRaceSel(value);
    setOutcome(null);
  };

  const runRace = () =>
    attempt(async () => {
      const seed = Math.floor(Math.random() * 2 ** 32);
      const race = chosenRace ? raceKey(chosenRace) : null;
      if (slip.length > 0) {
        const result = await invoke<RoundOutcome>("place_bets", { tickets: slip, seed, race });
        const first = slip[0];
        const label = selectionLabel(first.kind, first.horses);
        setOutcome({
          ...result,
          stake: slipTotal,
          horse: slip.length > 1 ? `${label} ほか${slip.length - 1}点` : label,
          kind: first.kind,
          ticketCount: slip.length,
          mixed: slip.some((t) => t.kind !== first.kind),
        });
        setSlip([]);
      } else {
        const result = await invoke<RoundOutcome>("watch_race", { seed, race });
        setOutcome({ ...result, stake: 0, horse: "", kind: "win", ticketCount: 0, mixed: false });
      }
      setFrame(0);
      setRanCard(card);
      document.querySelector(".course-info")?.scrollIntoView?.({ behavior: "smooth", block: "start" });
    });

  // put tickets into the list (one race's tickets, at most MAX_TICKETS)
  const addTickets = (lists: string[][]) => {
    if (slip.length + lists.length > MAX_TICKETS) {
      setError(`一度に買える馬券は${MAX_TICKETS}点までです。`);
      return;
    }
    setError("");
    setSlip([...slip, ...lists.map((horses) => ({ kind, horses, amount }))]);
  };

  // the tickets of a box or flow come from the backend, which knows the combinations
  const addExpanded = () =>
    attempt(async () => {
      const lists = await invoke<string[][]>("expand_tickets", {
        style: effStyle,
        kind,
        axis: effStyle === "flow" ? axisSel : null,
        horses: effStyle === "flow" ? effPartners : effBox,
        race: chosenRace ? raceKey(chosenRace) : null,
      });
      addTickets(lists);
    });

  const toggleIn = (list: string[], name: string) =>
    list.includes(name) ? list.filter((n) => n !== name) : [...list, name];

  const isRegistered = (h: RaceCard["my_horses"][number]) => h.registered ?? h.entered;

  // enter or take out one of my horses (the rest of the field is rivals)
  const toggleEntry = async (name: string) => {
    if (!card) return;
    setError("");
    const next = toggleIn(card.my_horses.filter(isRegistered).map((h) => h.name), name);
    try {
      await invoke("set_entries", { race: chosenRace ? raceKey(chosenRace) : null, horses: next });
      setCardVersion((v) => v + 1);
    } catch (e) {
      setError(String(e));
    }
  };

  // put a メンコ or a シャドーロール on, or take it off (it stays until changed)
  const toggleGear = async (name: string, current: Gear | undefined, part: keyof Gear) => {
    setError("");
    const next = { menko: current?.menko ?? false, shadow_roll: current?.shadow_roll ?? false };
    next[part] = !next[part];
    try {
      load(await invoke<GameState>("set_gear", { horse: name, menko: next.menko, shadowRoll: next.shadow_roll }));
      setCardVersion((v) => v + 1);
    } catch (e) {
      setError(String(e));
    }
  };

  // change one slot of the ticket, keeping the others as currently shown
  const pick = (i: number, name: string) => {
    const next = [...selection];
    next[i] = name;
    setPicks(next);
  };

  const nextWeek = () =>
    attempt(async () => {
      const s = await invoke<GameState>("next_week");
      load(s);
      setRaceSel(FREE_RACE);
      setOutcome(null);
      if (s.news && s.news.length > 0) setNotice(s.news.join(" "));
    }, true);

  const train = (name: string, workout: string) =>
    attempt(async () => {
      const s = await invoke<GameState>("train_horse", { horse: name, workout });
      load(s);
      // the session may have been too much for it
      const hurt = s.field.find((h) => h.name === name)?.away?.kind === "injured";
      if (hurt) setNotice((s.news ?? []).filter((n) => n.includes(name)).join(" "));
    });

  const setAutoEntry = (name: string, on: boolean) =>
    attempt(async () => {
      load(await invoke<GameState>("set_auto_entry", { horse: name, on }));
    });

  const setJockey = (name: string, value: string) =>
    attempt(async () => {
      load(await invoke<GameState>("set_jockey", { horse: name, jockey: value === "" ? null : Number(value) }));
      setCardVersion((v) => v + 1);
    });

  const sellHorse = async (name: string, price: number) => {
    if (!window.confirm(`${name}を${price}で売却しますか? 売却した馬は戻ってきません。`)) return;
    await attempt(async () => {
      load(await invoke<GameState>("sell_horse", { name }));
      setNotice(`${name}を${price}で売却しました。`);
    }, true);
  };

  const moveToStable = (name: string, trainer: TrainerInfo) =>
    attempt(async () => {
      load(await invoke<GameState>("set_trainer", { horse: name, trainer: trainer.id }));
      setNotice(`${name}を${trainer.name}厩舎に移しました。`);
    }, true);

  const setAllAuto = (on: boolean) =>
    attempt(async () => {
      load(await invoke<GameState>("set_all_auto", { on }));
    });

  const startAutoplay = () => {
    setError("");
    setNotice("");
    setAutoplayLog([]);
    setAutoplay(true);
  };

  const setAutoTraining = (name: string, on: boolean) =>
    attempt(async () => {
      load(await invoke<GameState>("set_auto_training", { horse: name, on }));
    });

  const sendToPasture = (name: string) =>
    attempt(async () => {
      load(await invoke<GameState>("pasture_horse", { name }));
      setCardVersion((v) => v + 1);
    }, true);

  const recallHorse = (name: string) =>
    attempt(async () => {
      load(await invoke<GameState>("recall_horse", { name }));
      setCardVersion((v) => v + 1);
    }, true);

  const restHorse = (name: string) =>
    attempt(async () => {
      load(await invoke<GameState>("rest_horse", { horse: name }));
      setNotice(`${name}を今週は休養させます。`);
      setCardVersion((v) => v + 1);
    }, true);

  const save = () =>
    attempt(async () => {
      await invoke("save_game");
      setNotice("セーブしました。");
    }, true);

  const loadSaved = () =>
    attempt(async () => {
      const s = await invoke<GameState>("load_game");
      setRaceSel(FREE_RACE);
      setOutcome(null);
      setFrame(null);
      setGame(s);
      setNotice("ロードしました。");
    }, true);

  const breed = (sireName: string, damName: string, retireName: string) =>
    attempt(async () => {
      const name = foalName;
      const mated = await invoke<GameState>("breed_horses", { sire: sireName, dam: damName, replace: retireName, name });
      load(mated);
      setOutcome(null);
      setFrame(null);
      setFoalName("");
      const weeks = mated.pregnancies?.find((p) => p.foal_name === name)?.weeks_left;
      setNotice(
        weeks === undefined
          ? "不受胎でした。費用は戻りません。シーズン中ならもう一度種付けできます。"
          : `種付けしました。「${name}」は${weeks}週後に生まれます。`
      );
    }, true);

  const goAbroad = (horse: string, race: number) =>
    attempt(async () => {
      load(await invoke<GameState>("go_abroad", { horse, race }));
    });

  const upgradeFarm = (facility: "pasture" | "nursery") =>
    attempt(async () => {
      load(await invoke<GameState>("upgrade_farm", { facility }));
    });

  const acceptAdvice = (name: string) =>
    attempt(async () => {
      const proposal = advice.find((a) => a.horse === name)?.proposal;
      load(await invoke<GameState>("accept_advice", { horse: name }));
      setNotice(`${name}: ${proposal}`);
    });

  const setOrder = (name: string, value: string) =>
    attempt(async () => {
      load(await invoke<GameState>("set_order", { horse: name, order: value === "" ? null : value }));
    });

  const bidFor = async (lot: number) => {
    setError("");
    setNotice("");
    const { max, name } = bids[lot];
    try {
      const r = await invoke<{ won: boolean; price: number; state: GameState }>("bid_at_auction", {
        lot,
        maxBid: Number(max),
        name,
      });
      load(r.state);
      setBidResult(r.won ? `「${name}」を${r.price}で落札しました。` : `競り負けました(落札額 ${r.price})。`);
    } catch (e) {
      setError(String(e));
    }
  };

  const retireHorse = async (name: string) => {
    if (!window.confirm(`${name}を引退させて繁殖入りさせますか? 引退した馬はレースに戻れません。`)) return;
    await attempt(async () => {
      load(await invoke<GameState>("retire_horse", { name }));
      setNotice(`${name}が引退し、繁殖入りしました。`);
    }, true);
  };

  const buyStud = async (o: StudOffer) => {
    if (!window.confirm(`${o.name}を${o.price}で購入しますか?`)) return;
    await attempt(async () => {
      load(await invoke<GameState>("buy_stud", { bloodId: o.blood_id }));
      setNotice(`${o.name}を購入しました。`);
    }, true);
  };

  const releaseStud = async (name: string) => {
    if (!window.confirm(`${name}を繁殖から外しますか? 配合に使えなくなります。`)) return;
    await attempt(async () => {
      load(await invoke<GameState>("release_stud", { name }));
      setNotice(`${name}を繁殖から外しました。`);
    }, true);
  };

  const newGame = async () => {
    setOutcome(null);
    setFrame(null);
    setError("");
    setNotice("");
    setRaceSel(FREE_RACE);
    load(await invoke<GameState>("reset_game"));
  };


  return {
    game,
    setGame,
    picks,
    setPicks,
    quote,
    setQuote,
    quoteError,
    setQuoteError,
    kind,
    setKind,
    tab,
    setTab,
    sire,
    setSire,
    dam,
    setDam,
    retire,
    setRetire,
    menus,
    setMenus,
    foalName,
    setFoalName,
    outlook,
    setOutlook,
    outlookError,
    setOutlookError,
    pedigreeView,
    setPedigreeView,
    amount,
    setAmount,
    slip,
    setSlip,
    slipOdds,
    setSlipOdds,
    style,
    setStyle,
    boxPicks,
    setBoxPicks,
    axis,
    setAxis,
    partners,
    setPartners,
    outcome,
    setOutcome,
    error,
    setError,
    notice,
    setNotice,
    frame,
    setFrame,
    calendar,
    setCalendar,
    raceSel,
    setRaceSel,
    cardState,
    setCardState,
    cardVersion,
    setCardVersion,
    card,
    racing,
    ranCard,
    setRanCard,
    autoplay,
    setAutoplay,
    autoplayMs,
    setAutoplayMs,
    stopAtNewYear,
    setStopAtNewYear,
    autoplayLog,
    setAutoplayLog,
    busy,
    load,
    weekRaces,
    chosenRace,
    raceGroups,
    canRun,
    parents,
    market,
    setMarket,
    lots,
    setLots,
    bids,
    setBids,
    bidResult,
    setBidResult,
    abroadHorse,
    setAbroadHorse,
    advice,
    setAdvice,
    parentNames,
    sires,
    outsideSires,
    maleNames,
    femaleNames,
    outlookSire,
    outlookDam,
    courses,
    setCourses,
    fitHorse,
    setFitHorse,
    venueFits,
    setVenueFits,
    fitHorseSel,
    runnerNames,
    pickCount,
    selection,
    fieldTooSmall,
    duplicate,
    isSingle,
    selectionKey,
    multiKind,
    effStyle,
    axisSel,
    effBox,
    effPartners,
    slipTotal,
    quotable,
    liveNames,
    liveFrame,
    finishOrder,
    live,
    inStretch,
    finished,
    chooseRaceOf,
    runRace,
    addTickets,
    addExpanded,
    toggleIn,
    isRegistered,
    toggleEntry,
    toggleGear,
    pick,
    nextWeek,
    train,
    setAutoEntry,
    setJockey,
    sellHorse,
    moveToStable,
    setAllAuto,
    startAutoplay,
    setAutoTraining,
    sendToPasture,
    recallHorse,
    restHorse,
    save,
    loadSaved,
    breed,
    goAbroad,
    upgradeFarm,
    acceptAdvice,
    setOrder,
    bidFor,
    retireHorse,
    buyStud,
    releaseStud,
    newGame,
  };
}

export type AppModel = ReturnType<typeof useGame>;

/** What the screens work out from the state, once the game has loaded. */
export function viewOf(app: AppModel, game: GameState) {
  const { quote, kind, sire, dam, retire, foalName, outcome, frame, card, racing, outsideSires, maleNames, selection, isSingle } = app;

  // who is running: my entered horses and the rivals
  const runners: Runner[] = card?.entrants ?? [];
  const pickedRunner = runners.find((r) => r.name === selection[0]);
  // the odds shown for the ticket: singles from the table, combinations from the backend
  const ticketOdds = isSingle ? (kind === "win" ? pickedRunner?.odds : pickedRunner?.place_odds) ?? null : quote;
  const laneNames = outcome?.runners ?? runners.map((r) => r.name);
  const laneCoats = outcome?.coats ?? runners.map((r) => r.coat_color);
  const trackDistance = outcome?.distance ?? card?.distance ?? 2400;
  const shown = outcome && frame ? Math.min(frame, outcome.frames.length) : 0;
  const net = game.history.reduce((sum, e) => sum + e.payout - e.stake + (e.prize ?? 0), 0);
  const workouts = game.workouts ?? [];
  const raceFatigueLimit = game.race_fatigue_limit ?? 50;
  const trainFatigueLimit = game.train_fatigue_limit ?? 90;
  const trainers = game.trainers ?? [];
  const stableCapacity = game.stable_capacity ?? 10;
  const inStable = (id: number) => game.field.filter((h) => h.trainer === id);
  const dirtSessions = game.dirt_sessions ?? 3;
  const studs = game.studs ?? [];
  const studNames = new Set(studs.map((s) => s.name));
  const females = studs.filter((h) => h.sex === "female");
  const sireSel = maleNames.includes(sire) ? sire : maleNames[0] ?? "";
  const matingCost = game.breed_cost + (outsideSires.find((o) => o.name === sireSel)?.stud_fee ?? 0);
  const pregnancies = game.pregnancies ?? [];
  const inFoal = (name: string) => pregnancies.some((p) => p.dam === name);
  const freeMares = females.filter((h) => !inFoal(h.name));
  const damSel = freeMares.some((h) => h.name === dam) ? dam : freeMares[0]?.name ?? "";
  // a foal goes into an empty place in the stable if there is one, else takes a horse's place
  const roomInStable = game.stable_size !== undefined && game.field.length < game.stable_size;
  const retireSel =
    retire === "" && roomInStable
      ? ""
      : game.field.some((h) => h.name === retire)
        ? retire
        : roomInStable
          ? ""
          : (game.field.find((h) => h.name !== sireSel && h.name !== damSel) ?? game.field[0])?.name ?? "";
  const studsFull = game.max_studs !== undefined && studs.length >= game.max_studs;
  const parentLabel = (name: string) => (studNames.has(name) ? `${name}(繁殖)` : name);
  const canBreed =
    !racing &&
    !game.game_over &&
    game.breeding_open !== false &&
    game.money >= matingCost &&
    sireSel !== "" &&
    damSel !== "" &&
    isValidHorseName(foalName);
  const bettingOpen = !(game.game_over && !racing);

  return {
    runners,
    pickedRunner,
    ticketOdds,
    laneNames,
    laneCoats,
    trackDistance,
    shown,
    net,
    workouts,
    raceFatigueLimit,
    trainFatigueLimit,
    trainers,
    stableCapacity,
    inStable,
    dirtSessions,
    studs,
    studNames,
    females,
    sireSel,
    matingCost,
    pregnancies,
    inFoal,
    freeMares,
    damSel,
    roomInStable,
    retireSel,
    studsFull,
    parentLabel,
    canBreed,
    bettingOpen,
  };
}

export type AppView = AppModel & ReturnType<typeof viewOf> & { game: GameState };
