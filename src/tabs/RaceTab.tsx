// The week's races: the card, who is entered, and the race itself.

import { GRADE_LABEL, SURFACE_LABEL, VENUE_LABEL, conditionText, monthOfWeek } from "../Calendar";
import { DEBUT_AGE, DIRT_LABEL, DIRT_WORDS, FIT_MARK, FIT_RATING_MARK, FIT_WORDS, FREE_RACE, GROWTH_MARK, GROWTH_WORDS, SEX_LABEL, STYLE_WORDS, TRACK_LABEL, WEATHER_LABEL, badTrackMark, fitReasons, gearWords, isYoung, raceValue, recordText, starRecord, temperMark } from "../labels";
import BetPanel from "./BetPanel";
import RacePanel from "./RacePanel";
import { AppView } from "../view";
import { FormBadge, GateBadge } from "../widgets";
import { ReactNode } from "react";

export default function RaceTab({ app, messages }: { app: AppView; messages: ReactNode }) {
  const { busy, canRun, card, chooseRaceOf, chosenRace, game, isRegistered, nextWeek, raceFatigueLimit, raceGroups, raceSel, runners, selection, toggleEntry } = app;
  return (
    <>
    <div className="weekbar">
      <strong>{`第${game.week}週(${monthOfWeek(game.week)}月)・${game.year}年目`}</strong>
      <label>
        レース
        <select aria-label="レース" value={raceSel} disabled={busy} onChange={(e) => chooseRaceOf(e.target.value)}>
          <option value={FREE_RACE}>フリーレース(練習・芝2400m)</option>
          {raceGroups.map(([label, races]) => (
            <optgroup key={label} label={label}>
              {races.map((r) => (
                <option key={raceValue(r)} value={raceValue(r)}>
                  {`${r.race_no}R ${GRADE_LABEL[r.grade]} ${r.name}(${SURFACE_LABEL[r.surface]}${r.distance}m)${
                    canRun(r) ? " ★出走可" : ""
                  }`}
                </option>
              ))}
            </optgroup>
          ))}
        </select>
      </label>
      <button onClick={nextWeek} disabled={busy}>次の週へ</button>
    </div>
    {messages}
    <main className="columns">
      <section className="panel">
        <h2>出走表</h2>
        {chosenRace && (
          <p className="race-info">
            <span className={`grade ${chosenRace.grade}`}>{GRADE_LABEL[chosenRace.grade]}</span>
            <strong>{chosenRace.name}</strong>
            <span>{`${VENUE_LABEL[chosenRace.venue]} ${SURFACE_LABEL[chosenRace.surface]}${chosenRace.distance}m`}</span>
            <span>{conditionText(chosenRace)}</span>
            <span>{`${chosenRace.prize.toLocaleString("en-US")}万円`}</span>
            {card?.weather && <span aria-label={`天候: ${WEATHER_LABEL[card.weather]}`}>{WEATHER_LABEL[card.weather]}</span>}
            {card?.track && (
              <span className={`track-badge ${card.track}`} aria-label={`馬場状態: ${TRACK_LABEL[card.track]}`}>
                {TRACK_LABEL[card.track]}
              </span>
            )}
            {card?.track && card.track !== "good" && <span className="muted">道悪適性の高い馬が有利です。</span>}
          </p>
        )}
        {card && (
          <fieldset className="entries">
            <legend>{`出走登録(最大${card.max_entries}頭)`}</legend>
            <p className="muted">{`自分の馬は最大${card.max_entries}頭まで出走できます。残りは相手馬です。`}</p>
            <p className="muted">
              {(card.trial_for ?? []).map((g1) => (
                <span key={g1} className="trial-note">{`トライアル: 3着までに「${g1}」の優先出走権`}</span>
              ))}
              {(card.first_prize ?? 0) > 0
                ? `1着の馬主賞金: ${card.first_prize}(2〜5着にも順位に応じた賞金)`
                : "このレースに賞金はありません。"}
            </p>
            {chosenRace && card.gate !== undefined && card.nominated !== undefined && (
              <p className="nominations">
                <strong>{`登録${card.nominated}頭 / フルゲート${card.gate}頭`}</strong>
                {card.nominated > card.gate && chosenRace.grade !== "g1" && (
                  <span className="muted">
                    {`${card.nominated - card.gate}頭多いので、優先出走権のない馬は抽選で除外されることがあります。除外された馬は次走で優先されます。`}
                  </span>
                )}
              </p>
            )}
            <div className="entry-list">
              {card.my_horses.map((h) => {
                const full = card.my_horses.filter(isRegistered).length >= card.max_entries;
                const registered = isRegistered(h);
                return (
                  <div key={h.name} className={`entry${h.eligible ? "" : " off"}`}>
                    <label>
                      <input
                        type="checkbox"
                        aria-label={`出走登録: ${h.name}`}
                        checked={registered}
                        disabled={busy || !h.eligible || (!registered && full)}
                        onChange={() => toggleEntry(h.name)}
                      />
                      {h.name}
                      {h.drawn_out && <span className="drawn-out">(抽選で除外)</span>}
                      {!h.eligible && (
                        <span className="muted">
                          {h.away
                            ? h.away.kind === "pasture"
                              ? "(放牧中)"
                              : h.away.kind === "growing"
                                ? "(牧場で育成中)"
                                : h.away.kind === "expedition"
                                  ? "(遠征中)"
                                  : `(${h.away.label}で休養中)`
                            : isYoung(game.field.find((x) => x.name === h.name))
                            ? `(${DEBUT_AGE}歳から)`
                            : h.barred
                            ? `(${h.barred})`
                            : h.resting
                            ? "(休養中)"
                            : (h.fatigue ?? 0) >= raceFatigueLimit
                              ? `(疲労 ${h.fatigue})`
                              : "(条件外)"}
                        </span>
                      )}
                    </label>
                    {(() => {
                      const rec = game.field.find((x) => x.name === h.name)?.record;
                      return rec && <span className="record">{recordText(rec)}</span>;
                    })()}
                  </div>
                );
              })}
            </div>
          </fieldset>
        )}
        <table>
          <thead>
            <tr>
              <th align="left">馬</th><th>性齢</th><th>スピード</th><th>スタミナ</th><th>パワー</th><th>調子</th>
              <th title="レースで背負う重さです。重いほど不利で、1kgで能力が0.3%変わります。別定は勝ち数で増え、ハンデは強い馬ほど重くなります。">斤量</th><th>馬具</th><th>脚質</th><th>気性</th><th>成長</th><th>距離</th><th>場</th><th>騎手</th><th>ダート</th><th>道悪</th><th>単勝</th><th>複勝</th><th title="単勝オッズの低い順です。">人気</th>
            </tr>
          </thead>
          <tbody>
            {runners.map((h, i) => (
              <tr key={h.name} className={selection.includes(h.name) ? "picked" : ""}>
                <td className="name">
                  <GateBadge index={i} count={runners.length} name={h.name} />
                  {h.name}
                  {h.mine && <span className="mine" aria-label="自馬">★</span>}
                  {h.star && (() => {
                    const known = game.stars?.find((x) => x.name === h.name);
                    return (
                      <span className="star-badge" aria-label={`${h.name}はライバル`} title={known ? starRecord(known) : undefined}>
                        ライバル
                      </span>
                    );
                  })()}
                </td>
                <td align="center">{`${SEX_LABEL[h.sex]}${h.age ?? ""}`}</td>
                <td align="center">{h.speed}</td>
                <td align="center">{h.stamina}</td>
                <td align="center">{h.power ?? "-"}</td>
                <td align="center"><FormBadge horse={h} /></td>
                <td align="center">
                  {h.weight ? <span aria-label={`${h.name}の斤量: ${h.weight}kg`}>{h.weight}</span> : "-"}
                </td>
                <td align="center">
                  {gearWords(h.gear) ? (
                    <span className="gear" aria-label={`馬具: ${gearWords(h.gear)}`} title={gearWords(h.gear)}>
                      {h.gear?.menko && <span className="gear-mark menko">メ</span>}
                      {h.gear?.shadow_roll && <span className="gear-mark shadow">シ</span>}
                    </span>
                  ) : (
                    "-"
                  )}
                </td>
                <td align="center">
                  {h.style ? <span aria-label={`脚質: ${STYLE_WORDS[h.style]}`}>{STYLE_WORDS[h.style]}</span> : "-"}
                </td>
                <td align="center">
                  {h.temper === undefined ? (
                    "-"
                  ) : (
                    <span aria-label={`気性: ${h.temper}`} title={h.temper < 50 ? "掛かることがあります" : undefined}>
                      {temperMark(h.temper)}
                    </span>
                  )}
                </td>
                <td align="center">
                  {h.growth === undefined ? (
                    "-"
                  ) : (
                    <span aria-label={`成長期: ${GROWTH_WORDS[h.growth]}`} title={`能力の${Math.round((h.age_form ?? 1) * 100)}%`}>
                      {GROWTH_MARK[h.growth]}
                    </span>
                  )}
                </td>
                <td align="center">
                  {h.distance_fit === undefined ? (
                    "-"
                  ) : (
                    <span
                      aria-label={`距離適性: ${FIT_WORDS[h.distance_fit]}`}
                      title={`得意 ${h.apt_min}〜${h.apt_max}m`}
                    >
                      {FIT_MARK[h.distance_fit]}
                    </span>
                  )}
                  {h.trainer_fit && (
                    <span className="trainer-fit" aria-label="調教師の得意距離" title="調教師の得意距離です。能力が少し上がります。">
                      得
                    </span>
                  )}
                </td>
                <td align="center">
                  {h.course_fit ? (
                    <span
                      aria-label={`${h.name}と競馬場の相性: ${FIT_RATING_MARK[h.course_fit.rating]}`}
                      title={fitReasons(h.course_fit) || "特に有利も不利もありません"}
                    >
                      {FIT_RATING_MARK[h.course_fit.rating]}
                    </span>
                  ) : (
                    "-"
                  )}
                </td>
                <td align="center">
                  {h.jockey ? <span aria-label={`${h.name}の騎手: ${h.jockey}`}>{h.jockey}</span> : "-"}
                </td>
                <td align="center">
                  <span aria-label={`ダート適性: ${DIRT_WORDS[h.dirt ?? 0]}`}>{DIRT_LABEL[h.dirt ?? 0]}</span>
                </td>
                <td align="center">
                  {h.bad_track === undefined ? (
                    "-"
                  ) : (
                    <span aria-label={`道悪適性: ${h.bad_track}`}>{badTrackMark(h.bad_track)}</span>
                  )}
                </td>
                <td align="center">{`x${h.odds.toFixed(1)}`}</td>
                <td align="center">{`x${h.place_odds.toFixed(1)}`}</td>
                <td align="center" aria-label={`${h.name}: ${h.popularity}番人気`}>{h.popularity}</td>
              </tr>
            ))}
          </tbody>
        </table>

        <BetPanel app={app} />
      </section>

      <RacePanel app={app} />
    </main>
    </>
  );
}
