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

  // does something the game may refuse, showing why if it does; one at a time, so a press
  // while another request is on its way is ignored (and the buttons are off meanwhile)
  const attempt = async (work: () => Promise<void>, clearNotice = false) => {
    if (pendingRef.current) return;
    pendingRef.current = true;
    setPending(true);
    setError("");
    if (clearNotice) setNotice("");
    try {
      await work();
    } catch (e) {
      setError(String(e));
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
    pending,
  };
}

export type Core = ReturnType<typeof useCore>;
