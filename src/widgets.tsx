// Small pieces the screens share.

import { gateForRunner } from "./gates";
import { COAT_COLOURS, CONDITION_LABEL, DEFAULT_COAT, bySide } from "./labels";
import { Condition } from "./types";
import { ReactNode } from "react";

/** A galloping horse in its coat colour, facing right. */
export function HorseIcon({ name, coat }: { name: string; coat?: string | null }) {
  const c = coat && COAT_COLOURS[coat] ? coat : DEFAULT_COAT;
  const { body, mane } = COAT_COLOURS[c];
  return (
    <svg className="horse-icon" viewBox="0 0 40 28" role="img" aria-label={`${name}の毛色: ${c}`}>
      <path d="M8 13 C3 12 2 17 1 21 C4 18 6 16 9 16 Z" fill={mane} />
      <path
        d="M8 11 C11 9 21 9 25 10 L30 4 C32 2 35 2 37 4 L39 8 C39 9 38 10 37 10 L34 9 L31 14 C30 17 27 18 24 18 L13 18 C10 18 8 16 8 13 Z"
        fill={body}
        stroke="rgba(0,0,0,0.45)"
        strokeWidth="0.8"
      />
      <path d="M27 9 L30 3 L32 3 L29 10 Z" fill={mane} />
      <path d="M12 17 L6 24 M15 18 L13 26 M24 18 L30 25 M27 17 L34 22" stroke={body} strokeWidth="2.6" strokeLinecap="round" />
      <path d="M12 17 L6 24 M15 18 L13 26 M24 18 L30 25 M27 17 L34 22" stroke="rgba(0,0,0,0.35)" strokeWidth="0.6" strokeLinecap="round" />
      <circle cx="35" cy="5.5" r="0.9" fill="#fff" />
    </svg>
  );
}

export function GateBadge({ index, count = 8, name }: { index: number; count?: number; name: string }) {
  const g = gateForRunner(index, count);
  return (
    <span className="gate" aria-label={`${name}の枠番: ${g.no}(${g.label})`} style={{ background: g.bg, color: g.fg }}>
      {g.no}
    </span>
  );
}

/** The options of a list of people, grouped by where they are based. */
export function SideOptions<T extends { region?: string }>({ people, option }: { people: T[]; option: (person: T) => ReactNode }) {
  return (
    <>
      {bySide(people).map(([side, here]) =>
        side ? (
          <optgroup key={side} label={side}>
            {here.map(option)}
          </optgroup>
        ) : (
          here.map(option)
        )
      )}
    </>
  );
}

export function FormBadge({ horse }: { horse: { name: string; condition: Condition } }) {
  return (
    <span className={`form ${horse.condition}`} aria-label={`${horse.name}の調子: ${CONDITION_LABEL[horse.condition]}`}>
      {CONDITION_LABEL[horse.condition]}
    </span>
  );
}
