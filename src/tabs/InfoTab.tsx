// The jockeys and the racecourses.

import CourseMap, { MAP_HEIGHT, MAP_WIDTH } from "../CourseMap";
import { courseShape, shapeKey } from "../courseShapes";
import { FIT_RATING_MARK, STYLE_WORDS, TURN_LABEL, bySide, fitReasons } from "../labels";
import { AppView } from "../view";
import { FoldPart } from "../widgets";

export default function InfoTab({ app }: { app: AppView }) {
  const { courses, fitHorseSel, foldProps, game, setFitHorse, venueFits } = app;
  return (
      <main role="tabpanel" aria-label="情報">
        <FoldPart label="騎手" fold={foldProps("jockeys")} notes={[`${(game.jockeys ?? []).length}人`]}>
          <p className="muted">
            技術が高いほど馬の力を引き出します。得意な脚質や回りでは、さらに伸びます。折り合いが高い騎手は、気性の荒い馬を掛かりにくくします。すべて架空の騎手です。
          </p>
          {bySide(game.jockeys ?? []).map(([side, here]) => (
          <section key={side} aria-label={side ? `${side}の騎手` : undefined}>
          {side && <h3 className="side">{side}</h3>}
          <div className="jockeys">
            {here.map((j) => (
              <div key={j.id} className="jockey">
                <strong>{j.name}</strong>
                <ul className="tendencies">
                  <li>{`技術 ${j.skill}`}</li>
                  <li>{STYLE_WORDS[j.style]}</li>
                  <li>{TURN_LABEL[j.turn]}</li>
                  <li>{`折り合い ${j.tact}`}</li>
                  {j.jump !== undefined && <li>{`障害 ${j.jump}`}</li>}
                  <li>{`騎乗料 ${j.fee}`}</li>
                </ul>
                <p className="muted">{j.description}</p>
              </div>
            ))}
          </div>
          </section>
          ))}
        </FoldPart>
        <FoldPart label="競馬場" fold={foldProps("courses")} notes={[`${courses.length}場`]}>
          <p className="muted">
            数値はJRA公式のコース紹介によるもので、コースが複数ある競馬場はAコースの値です。「相性」は、回り、直線の長さ、坂、洋芝が馬に合うかどうかで決まり、レースでの能力が最大で±3.5%変わります。回りの得手不得手は馬ごとに決まっていて、直線は脚質、坂と洋芝はパワーで決まります。
          </p>
          <label className="fit-horse">
            相性を見る馬
            <select aria-label="相性を見る馬" value={fitHorseSel} onChange={(e) => setFitHorse(e.target.value)}>
              {game.field.map((h) => (
                <option key={h.name} value={h.name}>{h.name}</option>
              ))}
            </select>
          </label>
          <div className="courses">
            {courses.map((c) => {
              const fit = venueFits.find((f) => f.venue === c.venue);
              return (
                <section key={c.venue} className="course" aria-label={`${c.name}競馬場`}>
                  <header>
                    <h3>{`${c.name}競馬場`}</h3>
                    <span className={`turn ${c.turn}`}>{TURN_LABEL[c.turn]}</span>
                    <span className="muted">{c.place}</span>
                  </header>
                  <div className="course-maps">
                    {(c.maps ?? []).map((m) => (
                      <figure key={m.label}>
                        <figcaption>{m.label}</figcaption>
                        <CourseMap
                          runners={[]}
                          label={`${c.name} ${m.label}のコース図`}
                          shapeName={shapeKey(c.name, m.surface, m.layout)}
                          shape={courseShape(
                            { name: c.name, turn: TURN_LABEL[c.turn], straight: m.straight, lap: m.lap, layout: m.layout, after_goal: m.after_goal, straight_course: m.straight_course },
                            m.surface,
                            0,
                            MAP_WIDTH,
                            MAP_HEIGHT
                          )}
                        />
                      </figure>
                    ))}
                  </div>
                  <p className="grass">{c.grass}</p>
                  <table className="course-spec">
                    <thead>
                      <tr><th></th><th>1周</th><th>直線</th><th>高低差</th><th>相性</th></tr>
                    </thead>
                    <tbody>
                      {(["turf", "dirt"] as const).map((surface) => {
                        const label = surface === "turf" ? "芝" : "ダート";
                        const f = fit?.[surface];
                        return (
                          <tr key={surface}>
                            <th>{label}</th>
                            <td>{c[surface].lap}</td>
                            <td>{c[surface].straight}</td>
                            <td>{c[surface].elevation}</td>
                            <td>
                              {f && (
                                <span aria-label={`${label}の相性: ${FIT_RATING_MARK[f.rating]}`}>
                                  {FIT_RATING_MARK[f.rating]}
                                </span>
                              )}
                              {f && f.percent !== 0 && <span className="percent">{`${f.percent > 0 ? "+" : ""}${f.percent}%`}</span>}
                            </td>
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                  {fit && fitReasons(fit.turf) && <p className="fit-reasons">{fitReasons(fit.turf)}</p>}
                  <p>{c.features}</p>
                  <p className="muted">{`向いている馬: ${c.suits}`}</p>
                </section>
              );
            })}
          </div>
        </FoldPart>
      </main>
  );
}
