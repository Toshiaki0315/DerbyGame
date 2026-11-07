// What the screens work out from the state, once the game has loaded.

import { isValidHorseName } from "./naming";
import { GameState, Runner } from "./types";
import { AppModel } from "./useGame";

/** What the screens work out from the state, once the game has loaded. */
export function viewOf(app: AppModel, game: GameState) {
  const { quote, kind, sire, dam, retire, foalName, outcome, frame, card, racing, busy, outsideSires, maleNames, selection, isSingle } = app;

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
    !busy &&
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
