// オートプレイ: the weeks go on by themselves until stopped.

import { AUTOPLAY_LOG_LIMIT, AUTOPLAY_RESERVE_WEEKS, FREE_RACE } from "../labels";
import { GameState } from "../types";
import { Core } from "./useCore";
import { RaceShow } from "./useRaceShow";
import { Races } from "./useRaces";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

export function useAutoplay(core: Core, races: Races, show: RaceShow, autoplay: boolean, setAutoplay: (on: boolean) => void) {
  const { game, load, setError, setNotice } = core;
  const { setRaceSel } = races;
  const { setOutcome } = show;
  const [autoplayMs, setAutoplayMs] = useState(500);
  const [stopAtNewYear, setStopAtNewYear] = useState(true);
  const [autoplayLog, setAutoplayLog] = useState<string[]>([]);

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

  const startAutoplay = () => {
    setError("");
    setNotice("");
    setAutoplayLog([]);
    setAutoplay(true);
  };

  return {
    autoplayMs,
    setAutoplayMs,
    stopAtNewYear,
    setStopAtNewYear,
    autoplayLog,
    setAutoplayLog,
    startAutoplay,
  };
}
