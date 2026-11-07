// The game as the backend last sent it, the open tab, and what the player is told.

import { GameState, TabId } from "../types";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

export function useCore() {
  const [game, setGame] = useState<GameState | null>(null);
  const [tab, setTab] = useState<TabId>("race");
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  // a request that moves the game on (a race, the next week) is on its way; the ref also stops
  // a second click that comes before the screen has redrawn with the buttons turned off
  const [pending, setPending] = useState(false);
  const pendingRef = useRef(false);

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

  // like attempt, but only one at a time: presses while one is on its way are ignored
  const attemptOnce = async (work: () => Promise<void>, clearNotice = false) => {
    if (pendingRef.current) return;
    pendingRef.current = true;
    setPending(true);
    try {
      await attempt(work, clearNotice);
    } finally {
      pendingRef.current = false;
      setPending(false);
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
    attemptOnce,
    pending,
  };
}

export type Core = ReturnType<typeof useCore>;
