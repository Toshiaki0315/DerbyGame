// The ticket being made, and the list of tickets for the race.

import { KIND_PICKS, MAX_TICKETS, raceKey } from "../labels";
import { BetKind, BuyStyle, SlipTicket } from "../types";
import { Core } from "./useCore";
import { Races } from "./useRaces";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

export function useBetting(core: Core, races: Races) {
  const { game, setError, attempt } = core;
  const { card, chosenRace, raceSel } = races;
  // the horses picked for the ticket, in order (fixed up against the field when used)
  const [picks, setPicks] = useState<string[]>([]);
  const [quote, setQuote] = useState<number | null>(null);
  const [quoteError, setQuoteError] = useState("");
  const [kind, setKind] = useState<BetKind>("win");

  const [amount, setAmount] = useState(100);
  // several tickets at once: the list, its odds, and how new tickets are made (ordinary, box or flow)
  const [slip, setSlip] = useState<SlipTicket[]>([]);
  const [slipOdds, setSlipOdds] = useState<number[]>([]);
  const [style, setStyle] = useState<BuyStyle>("normal");
  const [boxPicks, setBoxPicks] = useState<string[]>([]);
  const [axis, setAxis] = useState("");
  const [partners, setPartners] = useState<string[]>([]);

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

  // change one slot of the ticket, keeping the others as currently shown
  const pick = (i: number, name: string) => {
    const next = [...selection];
    next[i] = name;
    setPicks(next);
  };

  return {
    picks,
    setPicks,
    quote,
    setQuote,
    quoteError,
    setQuoteError,
    kind,
    setKind,
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
    addTickets,
    addExpanded,
    pick,
  };
}

export type Betting = ReturnType<typeof useBetting>;
