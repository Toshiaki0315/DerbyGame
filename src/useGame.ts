// The state of the game on screen, and everything the player can do to it.

import { useAutoplay } from "./hooks/useAutoplay";
import { useBetting } from "./hooks/useBetting";
import { useBreeding } from "./hooks/useBreeding";
import { useCore } from "./hooks/useCore";
import { useCourses } from "./hooks/useCourses";
import { useRaceShow } from "./hooks/useRaceShow";
import { useRaces } from "./hooks/useRaces";
import { useStable } from "./hooks/useStable";
import { FREE_RACE } from "./labels";
import { GameState } from "./types";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";

export function useGame() {
  const core = useCore();
  const { load, setGame, setError, setNotice, attemptOnce, pending } = core;
  // the pedigree tab shows either the stable's horses or the whole pedigree database
  const [pedigreeView, setPedigreeView] = useState<"stable" | "database">("stable");
  const [autoplay, setAutoplay] = useState(false);
  // which folding parts of the screens the player has opened or closed
  const [folds, setFolds] = useState<Record<string, boolean>>({});
  const foldProps = (key: string, byDefault = false) => {
    const open = folds[key] ?? byDefault;
    return { open, onToggle: () => setFolds((f) => ({ ...f, [key]: !open })) };
  };
  const races = useRaces(core, autoplay);
  const { setRaceSel } = races;
  const betting = useBetting(core, races);
  const show = useRaceShow(core, races, betting);
  const { racing, setOutcome, setFrame } = show;
  const autoplaying = useAutoplay(core, races, show, autoplay, setAutoplay);
  const breeding = useBreeding(core, show);
  const courses = useCourses(core);
  const stable = useStable(core, races);
  const busy = racing || autoplay || pending;

  const nextWeek = () =>
    attemptOnce(async () => {
      const s = await invoke<GameState>("next_week");
      load(s);
      setRaceSel(FREE_RACE);
      setOutcome(null);
      if (s.news && s.news.length > 0) setNotice(s.news.join(" "));
    }, true);

  const save = () =>
    attemptOnce(async () => {
      await invoke("save_game");
      setNotice("セーブしました。");
    }, true);

  const loadSaved = () =>
    attemptOnce(async () => {
      const s = await invoke<GameState>("load_game");
      setRaceSel(FREE_RACE);
      setOutcome(null);
      setFrame(null);
      setGame(s);
      setNotice("ロードしました。");
    }, true);

  const newGame = async () => {
    setOutcome(null);
    setFrame(null);
    setError("");
    setNotice("");
    setRaceSel(FREE_RACE);
    load(await invoke<GameState>("reset_game"));
  };

  return {
    ...core,
    ...races,
    ...betting,
    ...show,
    ...autoplaying,
    ...breeding,
    ...courses,
    ...stable,
    pedigreeView,
    setPedigreeView,
    foldProps,
    autoplay,
    setAutoplay,
    busy,
    nextWeek,
    save,
    loadSaved,
    newGame,
  };
}

export type AppModel = ReturnType<typeof useGame>;
