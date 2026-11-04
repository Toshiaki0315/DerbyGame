// This week's races, the one chosen, and its field.

import { CalendarData, CalendarRace, VENUE_LABEL, admits, ageAllows, mayRunOver, racesOfWeek } from "../Calendar";
import { FREE_RACE, raceKey, raceValue } from "../labels";
import { RaceCard } from "../types";
import { Core } from "./useCore";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

export function useRaces(core: Core, autoplay: boolean) {
  const { game, setError } = core;
  const [calendar, setCalendar] = useState<CalendarData | null>(null);
  // "free" or "<day>:<venue>:<race_no>" of a race on this week's card
  const [raceSel, setRaceSel] = useState(FREE_RACE);
  // the field of the selected race; kept with the selection it was loaded for
  const [cardState, setCardState] = useState<{ race: string; card: RaceCard } | null>(null);
  const [cardVersion, setCardVersion] = useState(0); // bump to load the field again
  const card = cardState && cardState.race === raceSel ? cardState.card : null;

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
        (r.surface === "jump" ? mayRunOver(r, h) : admits(r.grade, h.record?.starts ?? 0, h.record?.wins ?? 0))
    );
  useEffect(() => {
    invoke<CalendarData>("get_calendar")
      .then((c) => setCalendar(Array.isArray(c?.races) ? c : null))
      .catch(() => setCalendar(null));
  }, []);

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

  return {
    calendar,
    setCalendar,
    raceSel,
    setRaceSel,
    cardState,
    setCardState,
    cardVersion,
    setCardVersion,
    card,
    weekRaces,
    chosenRace,
    raceGroups,
    canRun,
    toggleIn,
    isRegistered,
    toggleEntry,
  };
}

export type Races = ReturnType<typeof useRaces>;
