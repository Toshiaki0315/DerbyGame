// The game as the backend last sent it, the open tab, and what the player is told.

import { GameState, TabId } from "../types";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

export function useCore() {
  const [game, setGame] = useState<GameState | null>(null);
  const [tab, setTab] = useState<TabId>("race");
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");

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
  }, []);

  return {
    game,
    setGame,
    tab,
    setTab,
    error,
    setError,
    notice,
    setNotice,
    load,
    attempt,
  };
}

export type Core = ReturnType<typeof useCore>;
