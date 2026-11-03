// The course, the race as it is run, the result and the record.

import { SURFACE_LABEL } from "../Calendar";
import CourseMap, { MAP_HEIGHT, MAP_WIDTH } from "../CourseMap";
import { COURSE_SHAPES, courseShape, shapeKey } from "../courseShapes";
import { gateForRunner } from "../gates";
import { HISTORY_SHOWN, KIND_LABEL, TRACK_LABEL, signed } from "../labels";
import { finishView, gateSpot } from "../raceShow";
import { AppView } from "../view";
import { HorseIcon } from "../widgets";

export default function RacePanel({ app }: { app: AppView }) {
  const { card, finished, game, laneCoats, laneNames, live, net, outcome, racing, ranCard, shown, trackDistance } = app;
  // the race that was run, once one has been; until then the one chosen
  const shownCard = outcome ? ranCard ?? card : card;
  const course = shownCard?.course;
  return (
    <section className="panel">
      <h2>コース</h2>
      {(() => {
        return (
          course && (
            <p className="course-info" aria-label="コース情報">
              {[
                course.layout ? `${course.name}(${course.layout})` : course.name,
                `${SURFACE_LABEL[shownCard.surface]}${shownCard.distance}m`,
                !course.straight_course && course.turn,
                shownCard.track && TRACK_LABEL[shownCard.track],
              ]
                .filter(Boolean)
                .join("・")}
            </p>
          )
        );
      })()}
      {(() => {
        if (!course?.lap || !shownCard) return null;
        const spot = gateSpot(
          shownCard.distance,
          courseShape({ ...course, lap: course.lap }, shownCard.surface, shownCard.distance, MAP_WIDTH, MAP_HEIGHT)
        );
        return (
          <p className="gate-info" aria-label="ゲート位置">
            {[
              `ゲート: ${spot.label}`,
              course.turf_start ? "芝スタート" : null,
              course.corners === undefined ? null : course.corners === 0 ? "コーナーなし" : `コーナー${course.corners}回`,
            ]
              .filter(Boolean)
              .join("・")}
          </p>
        );
      })()}
      {outcome && (live || finished) && (() => {
        const mine = (ranCard?.entrants ?? []).filter((e) => e.mine).map((e) => e.name);
        const end = finished ? finishView(outcome.results, mine) : null;
        return (
          <section
            className={`race-board${live?.stretch && !finished ? " stretch" : ""}${end?.mineWon ? " victory" : ""}`}
            aria-label="レース状況"
          >
            {end ? (
              <>
                <strong className="board-winner">{`1着 ${end.winner}!`}</strong>
                {end.photo && <span className="board-call">写真判定の大接戦!</span>}
                {end.mineWon && <span className="board-call">自分の馬が勝ちました!</span>}
                {end.minePlaced && <span>{`自分の馬 ${end.minePlaced.name} ${end.minePlaced.rank}着`}</span>}
              </>
            ) : (
              live && (
                <>
                  <strong className="board-phase">{live.phase}</strong>
                  <span className="board-left">{`残り ${live.remaining}m`}</span>
                  {live.close && <span className="board-call">大接戦!</span>}
                  <span className="board-order">
                    {live.order.slice(0, 3).map((name, i) => (
                      <span key={name}>{`${i + 1}番手 ${name}`}</span>
                    ))}
                  </span>
                  {mine.map((name) => (
                    <span key={name} className="board-mine">
                      {`自分の馬 ${name} ${live.order.indexOf(name) + 1}番手`}
                    </span>
                  ))}
                </>
              )
            )}
          </section>
        );
      })()}
      <div className="race-view">
      {(() => {
        if (!course?.lap) return null;
        const places = outcome && shown > 0 ? outcome.frames[shown - 1] : [];
        return (
          <CourseMap
            distance={trackDistance}
            shape={courseShape({ ...course, lap: course.lap }, shownCard!.surface, trackDistance, MAP_WIDTH, MAP_HEIGHT)}
            shapeName={
              COURSE_SHAPES[shapeKey(course.name, shownCard!.surface, course.layout)] && !course.straight_course
                ? shapeKey(course.name, shownCard!.surface, course.layout)
                : undefined
            }
            runners={laneNames.map((name, i) => ({
              name,
              position: places[i] ?? 0,
              colour: gateForRunner(i, laneNames.length).bg,
              mine: (shownCard?.entrants ?? []).some((e) => e.mine && e.name === name),
              leader: live?.leader === name,
            }))}
          />
        );
      })()}
      <div className={`track${racing ? " running" : ""}${racing && live?.stretch ? " stretch" : ""}`}>
        {laneNames.map((name, i) => {
          const pos = outcome && shown > 0 ? outcome.frames[shown - 1][i] : 0;
          const g = gateForRunner(i, laneNames.length);
          return (
            <div
              key={name}
              className={`lane${live?.leader === name ? " leader" : ""}${
                (ranCard?.entrants ?? []).some((e) => e.mine && e.name === name) && outcome ? " my-lane" : ""
              }`}
            >
              <span className="gate small" style={{ background: g.bg, color: g.fg }}>{g.no}</span>
              <span className="lane-name">{name}</span>
              <span className="lane-rank">{live ? live.ranks[i] : ""}</span>
              <div className="lane-bar">
                <div
                  role="progressbar"
                  aria-label={name}
                  aria-valuenow={pos}
                  aria-valuemin={0}
                  aria-valuemax={trackDistance}
                  style={{ width: `${(pos / trackDistance) * 100}%`, background: g.bg }}
                />
                <span className="runner" style={{ left: `${(pos / trackDistance) * 100}%` }}>
                  <HorseIcon name={name} coat={laneCoats[i]} />
                </span>
              </div>
            </div>
          );
        })}
        <span className="goal" aria-label="ゴール" />
      </div>
      </div>

      {outcome && (
        <div className="commentary">
          <h2>実況</h2>
          <ul>
            {outcome.commentary
              .filter((l) => l.frame < Math.max(shown, 1))
              .reverse()
              .map((l) => (
                <li key={l.frame + l.text}>{l.text}</li>
              ))}
          </ul>
        </div>
      )}

      {finished && outcome && (
        <div className="results">
          <h2>結果</h2>
          {outcome.ticketCount === 0 ? (
            <p>馬券は買っていません。</p>
          ) : (
            <p className={outcome.won ? "won" : "lost"}>
              {outcome.won ? `的中! 配当 ${outcome.payout}` : `はずれ。-${outcome.stake}`}
            </p>
          )}
          <ol>
            {outcome.results.map((r) => (
              <li key={r.name} className={r.rank === 1 ? "first" : ""}>
                {`${r.rank}着 ${r.name} (${r.finish_time.toFixed(2)}秒)`}
              </li>
            ))}
          </ol>
          {(outcome.tickets?.length ?? 0) > 1 && (
            <ul className="ticket-results">
              {outcome.tickets!.map((t, i) => (
                <li key={i} className={t.won ? "won" : "lost"}>
                  {`${KIND_LABEL[t.kind]} ${t.label} ${t.amount} → ${t.won ? `的中 配当 ${t.payout}` : "はずれ"}`}
                </li>
              ))}
            </ul>
          )}
          {(outcome.prize_total ?? 0) > 0 && (
            <div className="prizes">
              <p className="won">{`賞金: +${outcome.prize_total}`}</p>
              <ul>
                {(outcome.prizes ?? []).map((p) => (
                  <li key={p.name}>{`${p.name} ${p.rank}着 +${p.amount}`}</li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}

      <div className="history">
        <h2>成績</h2>
        {game.history.length === 0 ? (
          <p className="muted">まだレースがありません。</p>
        ) : (
          <>
            <p className={net >= 0 ? "won" : "lost"}>{`収支: ${signed(net)}`}</p>
            <ul>
              {[...game.history].reverse().slice(0, HISTORY_SHOWN).map((e) => (
                <li key={e.round} className={e.won ? "won" : e.watched ? "" : "lost"}>
                  {e.watched
                    ? [`第${e.round}R`, e.race, "馬券なし", e.horse, e.prize ? `賞金+${e.prize}` : "", `(優勝: ${e.winner})`].filter(Boolean).join(" ")
                    : e.ticket_count === 0
                    ? `第${e.round}R ${e.race} 自動出走 ${e.horse}${e.prize ? ` 賞金+${e.prize}` : ""} (優勝: ${e.winner})`
                    : `第${e.round}R ${e.race ? `${e.race} ` : ""}${e.mixed_kinds ? "複数券" : KIND_LABEL[e.kind ?? "win"]} ${e.horse} ${e.stake} → ${e.won ? "的中" : "はずれ"} ${signed(e.payout - e.stake)}${e.prize ? ` 賞金+${e.prize}` : ""} (優勝: ${e.winner})`}
                </li>
              ))}
            </ul>
          </>
        )}
      </div>
    </section>
  );
}
