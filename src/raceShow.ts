/** What the track panel makes of a race as it is run. */

/** How close the first two must be in the straight for it to be called a close race (metres). */
const CLOSE_METRES = 3;
/** How long before the straight the final corner is called (metres). */
const CORNER_METRES = 400;
/** The gap at the line that makes it a photo finish (seconds). */
const PHOTO_SECONDS = 0.1;
/** How much of a race counts as the start when the course is known. */
const START_SHARE = 0.05;

export interface RaceView {
  /** The runners in the order they are running, and each lane's place in that order. */
  order: string[];
  ranks: number[];
  leader: string;
  /** What the leader has left to run, to the nearest ten metres. */
  remaining: number;
  phase: string;
  /** In the final straight. Only known when the course's straight is. */
  stretch: boolean;
  /** Nothing between the first two in the straight. */
  close: boolean;
}

/** A course's line in a drawing, measured along its length from the goal. */
export interface Trace {
  points: [number, number][];
  /** How far round each point is, ending with the whole way round. */
  lengths: number[];
  total: number;
  middle: [number, number];
}

/** The shape of a course seen from above, and how big to draw it. */
export interface OvalShape {
  lap: number;
  straight: number;
  /** From the goal on to the first corner. */
  afterGoal?: number;
  turn: string;
  width: number;
  height: number;
  /** A course with no corners at all. */
  straightCourse?: boolean;
  /** How many corners a race goes round, when known. */
  corners?: number;
  /** The course's own line; without one it is drawn as two straights and two half-circles. */
  trace?: Trace;
  /** A straight course's start and goal in the drawing. */
  line?: { from: [number, number]; to: [number, number] };
  /** The course a straight course runs alongside, drawn behind it. */
  backdrop?: [number, number][];
  /** A gate known to stand off the line of the course. */
  gate?: { x: number; y: number; label: string };
}

/** The share of a lap that can be straight, however long a course says its straight is. */
const MOST_STRAIGHT = 0.4;
/** The space left around the course in the drawing. */
const MAP_MARGIN = 16;

/** The lengths of a straight and of the pair of corners at each end of a lap. */
export const courseSpan = (lap: number, straight: number, afterGoal = 0) => {
  const side = Math.min(straight + afterGoal, lap * MOST_STRAIGHT);
  return { straight: side - afterGoal, side, afterGoal, corner: (lap - 2 * side) / 2 };
};

/** Where on the course a horse is, by what it has left to run. */
export const courseSection = (toGo: number, lap: number, straight: number, afterGoal = 0): string => {
  const span = courseSpan(lap, straight, afterGoal);
  if (toGo <= span.straight) return "最後の直線";
  const d = toGo % lap;
  if (d <= span.straight) return "正面スタンド前";
  if (d <= span.straight + span.corner) {
    return d - span.straight < span.corner / 2 ? "第4コーナー" : "第3コーナー";
  }
  const backEnd = span.straight + span.corner + span.side;
  if (d <= backEnd) return "向正面";
  if (d <= backEnd + span.corner) {
    return d - backEnd < span.corner / 2 ? "第2コーナー" : "第1コーナー";
  }
  return "正面スタンド前";
};

/** How much of a pocket is drawn, in metres, and how far it runs when nothing says otherwise. */
const POCKET_METRES = 120;
/** How far short of the home straight a gate still counts as on it. */
const HOME_STRAIGHT_LEEWAY = 60;

/** Where a race's gate stands. */
export interface GateSpot {
  label: string;
  /** Off the oval, on a chute of its own. */
  pocket: boolean;
  /** What is left to run where the horses join the course. */
  join: number;
}

/** How far back from a gate off the course its chute can join. */
const LONGEST_CHUTE = 500;

/** What is left to run where a chute from a gate meets the course: the nearest point ahead. */
const joinOf = (distance: number, shape: OvalShape, gate: { x: number; y: number }): number => {
  let best = distance;
  let nearest = Infinity;
  for (let toGo = distance - 1; toGo >= Math.max(0, distance - LONGEST_CHUTE); toGo -= 5) {
    const p = ovalPoint(toGo, shape);
    const gap = Math.hypot(p.x - gate.x, p.y - gate.y);
    if (gap < nearest) {
      nearest = gap;
      best = toGo;
    }
  }
  return best;
};

/** The half-corners a horse goes round with this much left to run. */
const cornersAhead = (toGo: number, lap: number, span: ReturnType<typeof courseSpan>): number => {
  const backEnd = span.straight + span.corner + span.side;
  const mids = [
    span.straight + span.corner / 4,
    span.straight + (span.corner * 3) / 4,
    backEnd + span.corner / 4,
    backEnd + (span.corner * 3) / 4,
  ];
  let count = 0;
  for (let round = 0; round * lap < toGo; round++) {
    count += mids.filter((m) => round * lap + m <= toGo).length;
  }
  return count;
};

/**
 * Where the gate stands for a distance, measured back from the goal. A course does not
 * start a race on a bend: a gate that would fall on one stands on a chute (ポケット) that
 * joins the course further on, where the race has as many corners left as it should.
 */
export const gateSpot = (distance: number, shape: OvalShape, corners = shape.corners): GateSpot => {
  if (shape.straightCourse) return { label: "直線コース", pocket: false, join: distance };
  if (shape.gate && shape.trace) {
    return { label: shape.gate.label, pocket: true, join: joinOf(distance, shape, shape.gate) };
  }
  const span = courseSpan(shape.lap, shape.straight, shape.afterGoal ?? 0);
  const d = distance % shape.lap;
  const backEnd = span.straight + span.corner + span.side;
  const halves: [number, string][] = [
    [span.straight, "4"],
    [span.straight + span.corner / 2, "3"],
    [backEnd, "2"],
    [backEnd + span.corner / 2, "1"],
  ];
  let held = halves.findIndex(([from]) => d > from && d <= from + span.corner / 2);
  // past the goal, yet with a corner fewer to run than from here: it starts in the first corner's pocket
  const pastGoal = d > backEnd + span.corner;
  if (pastGoal && corners !== undefined && cornersAhead(distance, shape.lap, span) > corners) held = 3;
  if (held === 0 && d - span.straight <= HOME_STRAIGHT_LEEWAY) {
    return { label: "正面スタンド前", pocket: false, join: distance };
  }
  // on a course's own line a gate read off no plan stands on the line itself
  if (held >= 0 && shape.trace) {
    return { label: `${halves[held][1]}コーナー付近`, pocket: false, join: distance };
  }
  if (held < 0) {
    const label = d > span.straight && d <= backEnd ? "向正面" : "正面スタンド前";
    return { label, pocket: false, join: distance };
  }
  // the end of each half-corner nearer the goal, from this one back to the straight before it
  let joinD = Math.max(halves[held][0], d - POCKET_METRES);
  if (corners !== undefined) {
    for (const [from] of [...halves.slice(0, held + 1)].reverse()) {
      const at = distance - (d - from);
      if (cornersAhead(at, shape.lap, span) <= corners) {
        joinD = from;
        break;
      }
    }
  }
  const onStraight = joinD === span.straight || joinD === backEnd;
  const number = onStraight ? (held < 2 ? "4" : "2") : halves[held][1];
  const label = number === "4" ? "4コーナー寄り" : `${number}コーナー奥ポケット`;
  return { label, pocket: true, join: distance - (d - joinD) };
};

/** The scale of a drawing of the course, in pixels a metre. */
const mapScale = (shape: OvalShape): number => {
  if (shape.trace) return shape.trace.total / shape.lap;
  if (shape.straightCourse) return (shape.width - 2 * MAP_MARGIN) / shape.lap;
  const span = courseSpan(shape.lap, shape.straight, shape.afterGoal ?? 0);
  const across = (2 * span.corner) / Math.PI;
  return Math.min((shape.width - 2 * MAP_MARGIN) / (span.side + across), (shape.height - 2 * MAP_MARGIN) / across);
};

/** The far end of a pocket in the drawing, along the line it joins the course on. */
export const gatePoint = (distance: number, shape: OvalShape): { x: number; y: number } => {
  if (shape.gate) return { x: shape.gate.x, y: shape.gate.y };
  const spot = gateSpot(distance, shape);
  const join = ovalPoint(spot.join, shape);
  if (!spot.pocket) return join;
  const further = ovalPoint(spot.join + 1, shape);
  const length = Math.hypot(further.x - join.x, further.y - join.y) || 1;
  const drawn = Math.min(distance - spot.join, POCKET_METRES) * mapScale(shape);
  return {
    x: join.x + ((further.x - join.x) / length) * drawn,
    y: join.y + ((further.y - join.y) / length) * drawn,
  };
};

/** Where a horse is in the drawing: on its pocket at first, then on the course. */
export const runnerPoint = (toGo: number, distance: number, shape: OvalShape, outwards = 0): { x: number; y: number } => {
  const spot = gateSpot(distance, shape);
  if (!spot.pocket || toGo <= spot.join) return ovalPoint(toGo, shape, outwards);
  const join = ovalPoint(spot.join, shape, outwards);
  const gate = gatePoint(distance, shape);
  const run = (toGo - spot.join) / (distance - spot.join);
  return { x: join.x + (gate.x - join.x) * run, y: join.y + (gate.y - join.y) * run };
};

/**
 * The point in a drawing of the course a horse has reached, by what it has left to run:
 * two straights and two half-circles, with the home straight along the bottom. On a
 * left-handed course the horses go anticlockwise, so they run it left to right.
 */
export const ovalPoint = (toGo: number, shape: OvalShape, outwards = 0): { x: number; y: number } => {
  if (shape.trace) return tracePoint(toGo, shape.lap, shape.trace, outwards);
  if (shape.line) {
    const share = toGo / shape.lap;
    const { from, to } = shape.line;
    return { x: to[0] + (from[0] - to[0]) * share, y: to[1] + (from[1] - to[1]) * share + outwards };
  }
  if (shape.straightCourse) {
    return {
      x: shape.width - MAP_MARGIN - (toGo / shape.lap) * (shape.width - 2 * MAP_MARGIN),
      y: shape.height / 2 + outwards,
    };
  }
  const span = courseSpan(shape.lap, shape.straight, shape.afterGoal ?? 0);
  const scale = mapScale(shape);
  const radius = (span.corner / Math.PI) * scale;
  const middle = shape.height / 2;
  const leftEnd = shape.width / 2 - (span.side * scale) / 2;
  const rightEnd = shape.width / 2 + (span.side * scale) / 2;
  const goal = rightEnd - span.afterGoal * scale;
  const wide = radius + outwards;
  const backEnd = span.straight + span.corner + span.side;
  const d = ((toGo % shape.lap) + shape.lap) % shape.lap;
  let x: number;
  let y: number;
  if (d <= span.straight) {
    x = goal - d * scale;
    y = middle + wide;
  } else if (d <= span.straight + span.corner) {
    const t = (d - span.straight) / span.corner;
    x = leftEnd - wide * Math.sin(Math.PI * t);
    y = middle + wide * Math.cos(Math.PI * t);
  } else if (d <= backEnd) {
    x = leftEnd + (d - span.straight - span.corner) * scale;
    y = middle - wide;
  } else if (d <= backEnd + span.corner) {
    const t = (d - backEnd) / span.corner;
    x = rightEnd + wide * Math.sin(Math.PI * t);
    y = middle - wide * Math.cos(Math.PI * t);
  } else {
    x = rightEnd - (d - backEnd - span.corner) * scale;
    y = middle + wide;
  }
  return { x: shape.turn === "右回り" ? shape.width - x : x, y };
};

/** The point on a course's own line with this much left to run. */
const tracePoint = (toGo: number, lap: number, trace: Trace, outwards: number): { x: number; y: number } => {
  const along = (((toGo % lap) + lap) % lap) / lap * trace.total;
  let i = 0;
  while (i < trace.points.length - 1 && trace.lengths[i + 1] < along) i++;
  const [ax, ay] = trace.points[i];
  const [bx, by] = trace.points[(i + 1) % trace.points.length];
  const length = trace.lengths[i + 1] - trace.lengths[i] || 1;
  const t = (along - trace.lengths[i]) / length;
  const [x, y] = [ax + (bx - ax) * t, ay + (by - ay) * t];
  if (outwards === 0) return { x, y };
  // square to the line, on the side away from the middle of the course
  let [nx, ny] = [-(by - ay) / length, (bx - ax) / length];
  if (nx * (x - trace.middle[0]) + ny * (y - trace.middle[1]) < 0) [nx, ny] = [-nx, -ny];
  return { x: x + nx * outwards, y: y + ny * outwards };
};

const phaseOf = (ahead: number, distance: number, straight?: number, lap?: number, afterGoal = 0): string => {
  const left = distance - ahead;
  if (left <= 0) return "ゴール";
  // a straight course has no corners to call
  if (lap === 0 && straight !== undefined) {
    if (left <= straight) return "最後の直線";
    const share = ahead / distance;
    return share <= 0.1 ? "スタート" : share < 0.5 ? "序盤" : "中盤";
  }
  if (straight !== undefined && lap !== undefined) {
    return ahead / distance < START_SHARE ? "スタート" : courseSection(left, lap, straight, afterGoal);
  }
  if (straight !== undefined) {
    if (left <= straight) return "最後の直線";
    if (left <= straight + CORNER_METRES) return "最終コーナー";
  }
  const run = ahead / distance;
  if (run < 0.1) return "スタート";
  return run < 0.5 ? "序盤" : "中盤";
};

export const raceView = (
  positions: number[],
  names: string[],
  distance: number,
  straight?: number,
  lap?: number,
  afterGoal?: number,
  finishOrder: string[] = []
): RaceView => {
  // the runners past the post all stand on it, so only the result can tell them apart
  const placed = (i: number) => {
    const place = finishOrder.indexOf(names[i]);
    return place < 0 ? names.length : place;
  };
  const lanes = names
    .map((_, i) => i)
    .sort((a, b) => positions[b] - positions[a] || placed(a) - placed(b) || a - b);
  const ranks = names.map((_, i) => lanes.indexOf(i) + 1);
  const ahead = positions[lanes[0]] ?? 0;
  const left = distance - ahead;
  const stretch = straight !== undefined && left <= straight;
  return {
    order: lanes.map((i) => names[i]),
    ranks,
    leader: names[lanes[0]],
    remaining: Math.max(0, Math.round(left / 10) * 10),
    phase: phaseOf(ahead, distance, straight, lap, afterGoal),
    stretch,
    close: stretch && left > 0 && lanes.length > 1 && ahead - positions[lanes[1]] <= CLOSE_METRES,
  };
};

export interface FinishView {
  winner: string;
  photo: boolean;
  mineWon: boolean;
  /** The best of the stable's horses, when none of them won. */
  minePlaced: { name: string; rank: number } | null;
}

export const finishView = (
  results: { rank: number; name: string; finish_time: number }[],
  mine: string[]
): FinishView => {
  const [first, second] = results;
  const best = results.find((r) => mine.includes(r.name));
  const mineWon = best?.rank === 1;
  return {
    winner: first?.name ?? "",
    photo: second !== undefined && second.finish_time - first.finish_time < PHOTO_SECONDS,
    mineWon,
    minePlaced: best && !mineWon ? { name: best.name, rank: best.rank } : null,
  };
};
