// The race being run on screen, frame by frame, and what comes of it.

import { FRAME_MS, FREE_RACE, HISTORY_LIMIT, STRAIGHT_COURSE_FINISH, STRETCH_FRAME_MS, raceKey, selectionLabel } from "../labels";
import { raceView } from "../raceShow";
import { BetKind, GameState, HistoryEntry, RaceCard, RoundOutcome } from "../types";
import { Betting } from "./useBetting";
import { Core } from "./useCore";
import { Races } from "./useRaces";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

export function useRaceShow(core: Core, races: Races, betting: Betting) {
  const { setGame, load, setNotice, setError, attempt } = core;
  const { card, chosenRace, setRaceSel } = races;
  const { slip, setSlip, slipTotal } = betting;
  const [outcome, setOutcome] = useState<(RoundOutcome & { stake: number; horse: string; kind: BetKind; ticketCount: number; mixed: boolean }) | null>(null);

  // number of replay frames shown so far; null when no race is in progress
  const [frame, setFrame] = useState<number | null>(null);

  const racing = outcome !== null && frame !== null && frame < outcome.frames.length;
  const [ranCard, setRanCard] = useState<RaceCard | null>(null);

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
      invoke<GameState>("get_game_state")
        .then((s) => {
          load(s);
          setRaceSel(FREE_RACE);
          if (s.news && s.news.length > 0) setNotice(s.news.join(" "));
        })
        .catch((e) => setError(String(e)));
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

  return {
    outcome,
    setOutcome,
    frame,
    setFrame,
    racing,
    ranCard,
    setRanCard,
    liveNames,
    liveFrame,
    finishOrder,
    live,
    inStretch,
    finished,
    chooseRaceOf,
    runRace,
  };
}

export type RaceShow = ReturnType<typeof useRaceShow>;
