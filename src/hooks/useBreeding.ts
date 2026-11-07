// The pair to be bred, the market in breeding stock, and the yearling sale.

import { AuctionLot, GameState, Outlook, StudOffer } from "../types";
import { Core } from "./useCore";
import { RaceShow } from "./useRaceShow";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

export function useBreeding(core: Core, show: RaceShow) {
  const { game, tab, load, setError, setNotice, attempt } = core;
  const { setOutcome, setFrame } = show;
  const [sire, setSire] = useState("");
  const [dam, setDam] = useState("");
  const [retire, setRetire] = useState("");

  const [foalName, setFoalName] = useState("");
  const [outlook, setOutlook] = useState<Outlook | null>(null);
  const [outlookError, setOutlookError] = useState("");

  // the pair that would be bred, and what the pairing would bring (asked of the backend)
  const parents = [...(game?.field ?? []), ...(game?.studs ?? [])];
  const [market, setMarket] = useState<StudOffer[]>([]);
  const [lots, setLots] = useState<AuctionLot[]>([]);
  const [bids, setBids] = useState<Record<number, { max: string; name: string }>>({});
  const [bidResult, setBidResult] = useState("");

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

  const bidFor = (lot: number) =>
    attempt(async () => {
      const { max, name } = bids[lot];
      const r = await invoke<{ won: boolean; price: number; state: GameState }>("bid_at_auction", {
        lot,
        maxBid: Math.trunc(Number(max)),
        name,
      });
      load(r.state);
      setBidResult(r.won ? `「${name}」を${r.price}で落札しました。` : `競り負けました(落札額 ${r.price})。`);
    }, true);

  const retireHorse = async (name: string) => {
    const gelding = game?.field.find((h) => h.name === name)?.gelded;
    const question = gelding ? `${name}を引退させますか? セン馬は繁殖入りできないので厩舎を去ります。` : `${name}を引退させて繁殖入りさせますか? 引退した馬はレースに戻れません。`;
    if (!window.confirm(question)) return;
    await attempt(async () => {
      load(await invoke<GameState>("retire_horse", { name }));
      setNotice(gelding ? `${name}が引退しました。` : `${name}が引退し、繁殖入りしました。`);
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

  return {
    sire,
    setSire,
    dam,
    setDam,
    retire,
    setRetire,
    foalName,
    setFoalName,
    outlook,
    setOutlook,
    outlookError,
    setOutlookError,
    parents,
    market,
    setMarket,
    lots,
    setLots,
    bids,
    setBids,
    bidResult,
    setBidResult,
    parentNames,
    sires,
    outsideSires,
    maleNames,
    femaleNames,
    outlookSire,
    outlookDam,
    breed,
    bidFor,
    retireHorse,
    buyStud,
    releaseStud,
  };
}
