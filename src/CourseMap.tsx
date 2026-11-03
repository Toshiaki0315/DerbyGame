import { courseSpan, gatePoint, gateSpot, ovalPoint, OvalShape, runnerPoint } from "./raceShow";

interface Runner {
  name: string;
  /** Metres run so far. */
  position: number;
  colour: string;
  mine: boolean;
  leader: boolean;
}

interface Props {
  runners: Runner[];
  /** The distance of the race on it; without one the map has no gate. */
  distance?: number;
  shape: OvalShape;
  label?: string;
  /** Which course's own line this is, if it is one. */
  shapeName?: string;
}

export const MAP_WIDTH = 300;
export const MAP_HEIGHT = 170;
const WIDTH = MAP_WIDTH;
const HEIGHT = MAP_HEIGHT;
/** How many points draw the outline of the course. */
const OUTLINE_POINTS = 96;
/** How far apart the runners spread across the track, by lane. */
const LANE_SPREAD = 3;
const LANES_ACROSS = 4;
/** How far inside the track the corner numbers sit. */
const LABEL_INSIDE = -18;

/** The course seen from above, with a dot for each runner. */
export default function CourseMap({ runners, distance, shape, shapeName, label = "コース図" }: Props) {
  const { lap, straight, turn, afterGoal, straightCourse } = shape;
  const spot = distance === undefined ? null : gateSpot(distance, shape);
  const gate = distance === undefined ? null : gatePoint(distance, shape);
  const joined = spot && ovalPoint(spot.join, shape);
  const outline = Array.from({ length: OUTLINE_POINTS }, (_, i) => {
    const p = ovalPoint((lap * i) / OUTLINE_POINTS, shape);
    return `${p.x.toFixed(1)},${p.y.toFixed(1)}`;
  }).join(" ");
  const span = courseSpan(lap, straight, afterGoal ?? 0);
  const backEnd = span.straight + span.corner + span.side;
  // numbered in the order they are run; each by what is left to run from its middle
  const bends: [string, number][] = straightCourse
    ? []
    : [
        ["1", backEnd + span.corner * 0.75],
        ["2", backEnd + span.corner * 0.25],
        ["3", span.straight + span.corner * 0.75],
        ["4", span.straight + span.corner * 0.25],
      ];
  const finishIn = ovalPoint(0, shape, -11);
  const finishOut = ovalPoint(0, shape, 11);
  const finishLabel = ovalPoint(0, shape, LABEL_INSIDE);
  // the leader last, to be drawn on top
  const inDrawingOrder = runners
    .map((runner, lane) => ({ runner, lane }))
    .sort((a, b) => Number(a.runner.leader) - Number(b.runner.leader) || Number(a.runner.mine) - Number(b.runner.mine));
  return (
    <svg className="course-map" viewBox={`0 0 ${WIDTH} ${HEIGHT}`} role="img" aria-label={label} data-shape={shapeName}>
      {shape.backdrop && (
        <polygon className="map-backdrop" points={shape.backdrop.map(([x, y]) => `${x.toFixed(1)},${y.toFixed(1)}`).join(" ")} />
      )}
      {straightCourse ? (
        <polyline className="map-turf" points={outline} />
      ) : (
        <>
          <polygon className="map-turf" points={outline} />
          <polygon className="map-rail" points={outline} />
        </>
      )}
      {spot?.pocket && joined && gate && (
        <line className="map-pocket" x1={joined.x} y1={joined.y} x2={gate.x} y2={gate.y} />
      )}
      {gate && (
        <>
          <rect className="map-gate" x={gate.x - 3} y={gate.y - 8} width={6} height={16} rx={1} />
          <text className="map-gate-label" x={gate.x} y={gate.y - 11} textAnchor="middle">
            ゲート
          </text>
        </>
      )}
      {bends.map(([label, toGo]) => {
        const p = ovalPoint(toGo, shape, LABEL_INSIDE);
        return (
          <text key={label} className="map-corner" x={p.x} y={p.y} textAnchor="middle" dominantBaseline="middle">
            {label}
          </text>
        );
      })}
      <line className="map-finish" x1={finishIn.x} y1={finishIn.y} x2={finishOut.x} y2={finishOut.y} />
      <text className="map-goal" x={finishLabel.x} y={finishLabel.y} textAnchor="middle" dominantBaseline="middle">
        ゴール
      </text>
      <text className="map-turn" x={WIDTH / 2} y={HEIGHT / 2} textAnchor="middle" dominantBaseline="middle">
        {straightCourse ? "直線" : turn}
      </text>
      {inDrawingOrder.map(({ runner, lane }) => {
        const wide = (lane % LANES_ACROSS) * LANE_SPREAD - LANE_SPREAD;
        const race = distance ?? 0;
        const p = runnerPoint(Math.max(0, race - runner.position), race, shape, wide);
        return (
          <circle
            key={runner.name}
            data-horse={runner.name}
            className={`map-horse${runner.mine ? " mine-horse" : ""}${runner.leader ? " leader-horse" : ""}`}
            cx={p.x.toFixed(1)}
            cy={p.y.toFixed(1)}
            r={runner.mine || runner.leader ? 5.5 : 4}
            fill={runner.colour}
          >
            <title>{runner.name}</title>
          </circle>
        );
      })}
    </svg>
  );
}
