// Looking after the horses: training, equipment, the trainers, going abroad.

import { GameState, Gear, TrainerInfo } from "../types";
import { Core } from "./useCore";
import { Races } from "./useRaces";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

export function useStable(core: Core, races: Races) {
  const { game, tab, load, setError, setNotice, attempt } = core;
  const { setCardVersion } = races;
  // the training menu chosen for each horse (the first menu until one is picked)
  const [menus, setMenus] = useState<Record<string, string>>({});

  const [abroadHorse, setAbroadHorse] = useState<Record<number, string>>({});
  const [advice, setAdvice] = useState<{ horse: string; comment: string; proposal?: string | null }[]>([]);

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

  // put a メンコ or a シャドーロール on, or take it off (it stays until changed)
  const toggleGear = async (name: string, current: Gear | undefined, part: keyof Gear) => {
    setError("");
    const next = { menko: current?.menko ?? false, shadow_roll: current?.shadow_roll ?? false, blinkers: current?.blinkers ?? false };
    next[part] = !next[part];
    try {
      load(await invoke<GameState>("set_gear", { horse: name, menko: next.menko, shadowRoll: next.shadow_roll, blinkers: next.blinkers }));
      setCardVersion((v) => v + 1);
    } catch (e) {
      setError(String(e));
    }
  };

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

  const setAutoTraining = (name: string, on: boolean) =>
    attempt(async () => {
      load(await invoke<GameState>("set_auto_training", { horse: name, on }));
    });

  const sendToPasture = (name: string) =>
    attempt(async () => {
      load(await invoke<GameState>("pasture_horse", { name }));
      setCardVersion((v) => v + 1);
    }, true);

  const geldHorse = async (name: string, cost: number) => {
    if (!window.confirm(`${name}を${cost}で去勢しますか? 気性は穏やかになりますが、繁殖入りやクラシック出走ができなくなります。元には戻せません。`)) return;
    await attempt(async () => {
      load(await invoke<GameState>("geld_horse", { name }));
      setNotice(`${name}を去勢しました。しばらく放牧で休ませます。`);
      setCardVersion((v) => v + 1);
    }, true);
  };

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

  return {
    menus,
    setMenus,
    abroadHorse,
    setAbroadHorse,
    advice,
    setAdvice,
    toggleGear,
    train,
    setAutoEntry,
    setJockey,
    sellHorse,
    moveToStable,
    setAllAuto,
    setAutoTraining,
    sendToPasture,
    geldHorse,
    recallHorse,
    restHorse,
    goAbroad,
    upgradeFarm,
    acceptAdvice,
    setOrder,
  };
}
