// The trainers' stables and what they have to say.

import { ORDER_LABELS, sexLabel, bySide, recordText } from "../labels";
import { AppView } from "../view";
import { HorseIcon, SideOptions } from "../widgets";

export default function StableTab({ app }: { app: AppView }) {
  const { abroadHorse, acceptAdvice, advice, busy, game, geldHorse, goAbroad, goOverToJumps, takeJumpExam, inStable, moveToStable, racing, sellHorse, setAbroadHorse, setJockey, setOrder, stableCapacity, trainers } = app;
  return (
      <main role="tabpanel" aria-label="厩舎">
        <section className="panel">
          <h2>厩舎</h2>
          <details className="help">
            <summary>厩舎・騎手の説明</summary>
            <p className="muted">
            {`馬は調教師の厩舎に預けます。1つの厩舎に預けられるのは${stableCapacity}頭までです。「お任せ」の調教と「自動出走」のレース選び・出走間隔は、預けた調教師の傾向で決まります。調教師の得意距離のレースではスピードとスタミナが${game.trainer_bonus_percent ?? 0}%上がります(万能の調教師にはこの補正はありません)。騎手を選ばなければ、調教師が目標に合わせて手配します(G1志向は腕利きを、堅実派は騎乗料の安い騎手を選びます)。カレンダーのレースでは1頭ごとに騎乗料がかかります。同じ騎手で続けて走るほど息が合います。厩舎は無料でいつでも移せます。毎週、馬1頭ごとに預託料がかかり、払えなくなると一番安い馬から手放します。`}
          </p>
          </details>
          {game.overseas && (
            <section className="panel" aria-label="海外遠征">
              <h2>海外遠征</h2>
              <p className="muted">{`G1を勝った3歳以上の馬(または獲得賞金5000以上の馬)を海外の大レースに送れます。レースの8週前から2週前まで受け付け、費用は${game.expedition_cost}です。遠征中はほかのレースに出られません。`}</p>
              {(game.expeditions ?? []).map((e) => (
                <p key={e.horse} className="season">{`${e.horse}が${e.race}に遠征中です。`}</p>
              ))}
              <ul className="honours">
                {game.overseas.map((r) => {
                  const horse = r.candidates.includes(abroadHorse[r.id]) ? abroadHorse[r.id] : r.candidates[0];
                  return (
                    <li key={r.id}>
                      <strong>{r.name}</strong>
                      <span>{`${r.country}・第${r.week}週・${r.surface === "turf" ? "芝" : "ダート"}${r.distance}m・1着 ${r.first_prize}`}</span>
                      {r.open && horse && (
                        <>
                          <select
                            aria-label={`${r.name}に送る馬`}
                            value={horse}
                            onChange={(e) => setAbroadHorse({ ...abroadHorse, [r.id]: e.target.value })}
                          >
                            {r.candidates.map((c) => (
                              <option key={c} value={c}>{c}</option>
                            ))}
                          </select>
                          <button
                            disabled={busy || game.money <= (game.expedition_cost ?? 0)}
                            onClick={() => goAbroad(horse, r.id)}
                          >
                            {`${r.name}に遠征 (${game.expedition_cost})`}
                          </button>
                        </>
                      )}
                    </li>
                  );
                })}
              </ul>
            </section>
          )}
          {bySide(trainers).map(([side, here]) => (
          <section key={side} aria-label={side ? `${side}の厩舎` : undefined}>
          {side && <h3 className="side">{side}</h3>}
          <div className="stables">
            {here.map((t) => {
              const horses = inStable(t.id);
              return (
                <section key={t.id} className="stable" aria-label={`${t.name}厩舎`}>
                  <header>
                    <h3>{`${t.name}厩舎`}</h3>
                    <span className="count">{`${horses.length}/${stableCapacity}頭`}</span>
                  </header>
                  <ul className="tendencies">
                    <li>{`調教: ${t.training}`}</li>
                    <li>{`距離: ${t.distance}`}</li>
                    <li>{`間隔: ${t.rotation}`}</li>
                    <li>{`目標: ${t.aim}`}</li>
                  </ul>
                  <p className="muted">{t.description}</p>
                  {horses.length === 0 ? (
                    <p className="muted">預けている馬はいません。</p>
                  ) : (
                    <ul className="stable-horses">
                      {horses.map((h) => (
                        <li key={h.name}>
                          <HorseIcon name={h.name} coat={h.blood?.coat_color} />
                          <strong>{h.name}</strong>
                          <span>{`${sexLabel(h)}${h.age ?? ""}`}</span>
                          <span className="record">{h.record ? recordText(h.record) : "-"}</span>
                          {(() => {
                            const said = advice.find((a) => a.horse === h.name);
                            return (
                              said && (
                                <div className="advice">
                                  <span>{said.comment}</span>
                                  {said.proposal && (
                                    <button
                                      aria-label={`${h.name}への提案を受ける`}
                                      disabled={busy || game.game_over}
                                      onClick={() => acceptAdvice(h.name)}
                                    >
                                      {said.proposal}
                                    </button>
                                  )}
                                </div>
                              )
                            );
                          })()}
                          {h.sale_price !== undefined && (
                            <button
                              className="sell"
                              aria-label={`${h.name}を${h.sale_price}で売却`}
                              disabled={busy || game.game_over || game.field.length <= 1}
                              onClick={() => sellHorse(h.name, h.sale_price!)}
                            >
                              {`売却 ${h.sale_price}`}
                            </button>
                            
                          )}
                          {!h.jumper ? (
                            <button
                              aria-label={`${h.name}を障害転向`}
                              disabled={busy || (h.age ?? 0) < 3}
                              onClick={() => goOverToJumps(h.name)}
                            >
                              障害転向
                            </button>
                          ) : !h.jump_license ? (
                            <button
                              aria-label={`${h.name}の障害試験`}
                              disabled={busy || (h.jump ?? 0) < (game.jump_exam_level ?? 0)}
                              title={`飛越${game.jump_exam_level ?? 0}以上で受けられます`}
                              onClick={() => takeJumpExam(h.name)}
                            >
                              {`障害試験 ${game.jump_exam_cost ?? 0}`}
                            </button>
                          ) : (
                            <span className="muted">障害馬</span>
                          )}
                          {h.sex === "male" && !h.gelded && (
                              <button
                                aria-label={`${h.name}を去勢`}
                                disabled={busy || !!h.away}
                                onClick={() => geldHorse(h.name, game.gelding_cost ?? 0)}
                              >
                                {`去勢 ${game.gelding_cost ?? 0}`}
                              </button>
                            )}
                          <label className="pick">騎手
                          <select
                            aria-label={`${h.name}の騎手`}
                            value={h.jockey ?? ""}
                            disabled={racing}
                            onChange={(e) => setJockey(h.name, e.target.value)}
                          >
                            <option value="">調教師におまかせ</option>
                            <SideOptions
                              people={game.jockeys ?? []}
                              option={(j) => (
                                <option key={j.id} value={j.id}>{`${j.name}(騎乗料 ${j.fee})`}</option>
                              )}
                            />
                          </select>
                          </label>
                          <label className="pick">作戦
                          <select
                            aria-label={`${h.name}の作戦`}
                            title="馬本来の脚質と違う作戦を指示すると、気性が荒くなって掛かりやすくなります。"
                            value={h.order ?? ""}
                            disabled={racing}
                            onChange={(e) => setOrder(h.name, e.target.value)}
                          >
                            <option value="">騎手におまかせ</option>
                            {ORDER_LABELS.map(([style, label]) => (
                              <option key={style} value={style}>{label}</option>
                            ))}
                          </select>
                          </label>
                          <label className="pick">厩舎
                          <select
                            aria-label={`${h.name}の厩舎`}
                            value={t.id}
                            disabled={racing}
                            onChange={(e) => {
                              const to = trainers.find((x) => x.id === Number(e.target.value));
                              if (to) moveToStable(h.name, to);
                            }}
                          >
                            <SideOptions
                              people={trainers}
                              option={(o) => (
                                <option
                                  key={o.id}
                                  value={o.id}
                                  disabled={o.id !== t.id && inStable(o.id).length >= stableCapacity}
                                >
                                  {o.name}
                                </option>
                              )}
                            />
                          </select>
                          </label>
                        </li>
                      ))}
                    </ul>
                  )}
                </section>
              );
            })}
          </div>
          </section>
          ))}
        </section>
      </main>
  );
}
