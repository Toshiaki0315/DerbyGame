// The racecourses, and how they suit the chosen horse.

import { CourseInfo, VenueFit } from "../types";
import { Core } from "./useCore";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

export function useCourses(core: Core) {
  const { game, tab, setError } = core;
  // the racecourses, asked for once the information tab is opened; how they suit the chosen horse
  const [courses, setCourses] = useState<CourseInfo[]>([]);
  const [fitHorse, setFitHorse] = useState("");
  const [venueFits, setVenueFits] = useState<VenueFit[]>([]);
  const fitHorseSel = game?.field.some((h) => h.name === fitHorse) ? fitHorse : game?.field[0]?.name ?? "";
  useEffect(() => {
    if (tab !== "info" || courses.length > 0) return;
    invoke<CourseInfo[]>("get_courses")
      .then((c) => setCourses(Array.isArray(c) ? c : []))
      .catch((e) => setError(String(e)));
  }, [tab, courses.length]);
  useEffect(() => {
    if (tab !== "info" || !fitHorseSel) return;
    let live = true;
    invoke<VenueFit[]>("get_course_fits", { horse: fitHorseSel })
      .then((f) => live && setVenueFits(Array.isArray(f) ? f : []))
      .catch(() => live && setVenueFits([]));
    return () => {
      live = false;
    };
  }, [tab, fitHorseSel, game]);

  return {
    courses,
    setCourses,
    fitHorse,
    setFitHorse,
    venueFits,
    setVenueFits,
    fitHorseSel,
  };
}
