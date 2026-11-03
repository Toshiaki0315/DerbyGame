// Work, rest and racing gear for the stable's horses.

import { DIRT_LABEL, DIRT_WORDS, SEX_LABEL, gainText, recordText } from "../labels";
import { AppView } from "../useGame";
import { FormBadge, HorseIcon } from "../widgets";

export default function TrainTab({ app }: { app: AppView }) {
  const { busy, dirtSessions, game, menus, raceFatigueLimit, racing, recallHorse, restHorse, sendToPasture, setAutoEntry, setAutoTraining, setMenus, toggleGear, train, trainFatigueLimit, trainers, workouts } = app;
  return (
      <main role="tabpanel" aria-label="育成">
        <section className="panel">
          <h2>育成</h2>
          <details className="help">
            <summary>調教・疲労・お任せの説明</summary>
            <p className="muted">お金を払って調教します。どの調教も1回ごとに費用が上がります。</p>
          <p className="muted">
            {`調教やレースで疲労がたまり、1週ごとに少しずつ抜けます。疲労が${raceFatigueLimit}以上だとレースに出られず、${trainFatigueLimit}以上だと調教できません。疲れているとレースでも力を出し切れません。休養させるとその週は休みになり、疲労が大きく抜けます。`}
          </p>
          <p className="muted">
            馬具はレースで付けるものです。メンコは周りの音を遮り、気性の荒い馬が掛かりにくくなります。シャドーロールは前に集中させ、出来のぶれが小さくなります。一度付けると外すまで付けたままです。
          </p>
          <p className="muted">
            {`お任せにした馬は、週が変わるたびに所属厩舎の調教師が調教します。一番低い能力を伸ばし、気性が荒ければ曳き運動、疲れていれば休養させます。どこまで追うかは調教師の傾向(スパルタ〜優しい)で変わります。費用は通常と同じで、所持金を${game.auto_keep_money ?? 0}残してその範囲で行います。結果は週が変わったときにお知らせします。`}
          </p>
          <p className="muted">
            レースや調教のあとに故障することがあり、疲れているほど、脚元が弱いほど起こりやすくなります。故障すると治療費がかかり、治るまで走れません。放牧に出すと4週間は調教もレースもできませんが、疲労がよく抜け、預託料は半額で、好調で戻ってきます。脚元は親から受け継ぎます。
          </p>
          <p className="muted">
            自動出走にした馬は、週の終わりに馬券なしでレースに出ます。どのレースに、どのくらいの間隔で使うかは、所属厩舎の調教師の傾向で決まります。賞金と戦績は通常のレースと同じように付きます。
          </p>
          </details>
          <ul className="workouts" aria-label="調教メニュー">
            {workouts.map((w) => (
              <li key={w.workout}>
                <strong>{w.name}</strong>
                <span className="gains">{gainText(w, dirtSessions)}</span>
                <span className="muted">{w.description}</span>
              </li>
            ))}
          </ul>
          <table className="training">
            <thead>
              <tr>
                <th align="left">馬</th>
                <th>スピード</th><th>スタミナ</th><th>パワー</th><th>気性</th><th>ダート</th>
                <th>脚元</th><th>疲労</th><th>調子</th><th align="left">調教</th>
              </tr>
            </thead>
            <tbody>
              {game.field.map((h) => (
                <tr key={h.name}>
                  <td className="who">
                    <div className="who-name">
                      <HorseIcon name={h.name} coat={h.blood?.coat_color} />
                      <span>{h.name}</span>
                    </div>
                    <div className="who-more">
                      <span>{`${SEX_LABEL[h.sex]}${h.age ?? ""}`}</span>
                      <span>{trainers.find((t) => t.id === h.trainer)?.name ?? "-"}</span>
                      <span>{h.record ? recordText(h.record) : "-"}</span>
                    </div>
                  </td>
                  <td align="center">{h.speed}</td>
                  <td align="center">{h.stamina}</td>
                  <td align="center">{h.power ?? "-"}</td>
                  <td align="center">{h.temper ?? "-"}</td>
                  <td align="center">
                    <span
                      aria-label={`ダート適性: ${DIRT_WORDS[h.dirt ?? 0]}${
                        (h.dirt ?? 0) < 2 ? `(次の段階まで ${h.dirt_progress ?? 0}/${dirtSessions})` : ""
                      }`}
                    >
                      {DIRT_LABEL[h.dirt ?? 0]}
                      {(h.dirt ?? 0) < 2 && <small className="progress">{` ${h.dirt_progress ?? 0}/${dirtSessions}`}</small>}
                    </span>
                  </td>
                  <td align="center">
                    <span aria-label={`${h.name}の脚元: ${h.legs ?? "-"}`}>{h.legs ?? "-"}</span>
                  </td>
                  <td className="fatigue">
                    {(() => {
                      const f = h.fatigue ?? 0;
                      const level = f >= raceFatigueLimit ? "tired" : f > 30 ? "weary" : "fresh";
                      return (
                        <span
                          role="meter"
                          aria-label={`${h.name}の疲労度`}
                          aria-valuenow={f}
                          aria-valuemin={0}
                          aria-valuemax={100}
                          className={`fatigue-meter ${level}`}
                          title={`疲労 ${f}`}
                        >
                          <span style={{ width: `${f}%` }} />
                          <small>{f}</small>
                        </span>
                      );
                    })()}
                  </td>
                  <td align="center"><FormBadge horse={h} /></td>
                  <td className="train">
                    {(() => {
                      const chosen = workouts.find((w) => w.workout === menus[h.name]) ?? workouts[0];
                      if (!chosen) return null;
                      return (
                        <>
                          <div className="train-line">
                          <select
                            aria-label={`${h.name}の調教メニュー`}
                            value={chosen.workout}
                            disabled={racing}
                            onChange={(e) => setMenus({ ...menus, [h.name]: e.target.value })}
                          >
                            {workouts.map((w) => (
                              <option key={w.workout} value={w.workout}>{w.name}</option>
                            ))}
                          </select>
                          <button
                            aria-label={`${h.name}を調教`}
                            title={`${gainText(chosen, dirtSessions)} — ${chosen.description}`}
                            disabled={
                              racing ||
                              game.game_over ||
                              game.money < h.train_cost ||
                              h.resting === true ||
                              !!h.away ||
                              (h.fatigue ?? 0) >= trainFatigueLimit
                            }
                            onClick={() => train(h.name, chosen.workout)}
                          >
                            調教
                          </button>
                          <button
                            aria-label={`${h.name}を休養させる`}
                            disabled={racing || h.resting === true || !!h.away}
                            onClick={() => restHorse(h.name)}
                          >
                            休養
                          </button>
                          {!h.away && (
                            <button aria-label={`${h.name}を放牧に出す`} disabled={busy} onClick={() => sendToPasture(h.name)}>
                              放牧
                            </button>
                          )}
                          {h.away?.kind === "pasture" && (
                            <button aria-label={`${h.name}を帰厩させる`} disabled={busy} onClick={() => recallHorse(h.name)}>
                              帰厩
                            </button>
                          )}
                          <span className="cost">
                            費用 <span aria-label={`${h.name}の調教費用`}>{h.train_cost}</span>
                          </span>
                          </div>
                          <div className="train-line">
                          <label className="auto">
                            <input
                              type="checkbox"
                              aria-label={`${h.name}をお任せ調教`}
                              checked={h.auto_training ?? false}
                              disabled={racing}
                              onChange={(e) => setAutoTraining(h.name, e.target.checked)}
                            />
                            お任せ
                          </label>
                          <label className="auto">
                            <input
                              type="checkbox"
                              aria-label={`${h.name}を自動出走`}
                              checked={h.auto_entry ?? false}
                              disabled={racing}
                              onChange={(e) => setAutoEntry(h.name, e.target.checked)}
                            />
                            自動出走
                          </label>
                          <label className="auto">
                      <input
                        type="checkbox"
                        aria-label={`${h.name}のメンコ`}
                        checked={h.gear?.menko ?? false}
                        disabled={racing}
                        onChange={() => toggleGear(h.name, h.gear, "menko")}
                      />
                      メンコ
                    </label>
                          <label className="auto">
                      <input
                        type="checkbox"
                        aria-label={`${h.name}のシャドーロール`}
                        checked={h.gear?.shadow_roll ?? false}
                        disabled={racing}
                        onChange={() => toggleGear(h.name, h.gear, "shadow_roll")}
                      />
                      シャドーロール
                    </label>
                          {h.resting && <span className="resting">休養中</span>}
                          {h.away && (
                            <span className={`resting ${h.away.kind}`}>
                              {h.away.kind === "pasture"
                                ? `放牧中(残り${h.away.weeks}週)`
                                : h.away.kind === "growing"
                                  ? `牧場で育成中(残り${h.away.weeks}週)`
                                  : h.away.kind === "expedition"
                                    ? `遠征中(残り${h.away.weeks}週)`
                                    : `${h.away.label}で休養中(残り${h.away.weeks}週)`}
                            </span>
                          )}
                          </div>
                        </>
                      );
                    })()}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </section>
      </main>
  );
}
