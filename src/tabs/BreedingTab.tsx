// The stud book, matings, the yearling sale, the farm and racing abroad.

import { inbreedText } from "../Bloodline";
import { DEBUT_AGE, RETIRE_AGE, SEX_LABEL, statLabel } from "../labels";
import { NAME_RULE, isValidHorseName } from "../naming";
import { AppView } from "../useGame";
import { HorseIcon } from "../widgets";

export default function BreedingTab({ app }: { app: AppView }) {
  const { bidFor, bidResult, bids, breed, buyStud, canBreed, damSel, females, foalName, game, inFoal, lots, market, matingCost, outlook, outlookError, outsideSires, parentLabel, pregnancies, racing, releaseStud, retireSel, roomInStable, setBids, setDam, setFoalName, setRetire, setSire, sireSel, sires, studs, studsFull, upgradeFarm } = app;
  return (
      <main role="tabpanel" aria-label="繁殖">
        <section className="panel breeding">
          <h2>配合</h2>
          <p className="muted">
            引退した種牡馬(または外部の種牡馬)と繁殖牝馬を種付けします。受胎しないこともあり、その場合も費用はかかります。受胎すると仔馬は約1年後に0歳で生まれ、厩舎の空き枠か入れ替える馬の枠に入り、能力と血統を両親から受け継ぎます。入れ替える馬は生まれるまで走り続けます。仔馬は生まれて1年間は牧場で育ち、そのあと調教を始めて、{DEBUT_AGE}歳からレースに出られます。受胎中の牝馬は、出産するまで次の種付けができません。
          </p>
          {game.breeding_start_week !== undefined && (
            <p className={game.breeding_open ? "season" : "season closed"}>
              {`種付けできるのは第${game.breeding_start_week}週〜第${game.breeding_end_week}週(3月〜5月)です。${
                game.breeding_open ? "今は繁殖シーズンです。" : `今は第${game.week}週でシーズン外です。`
              }`}
            </p>
          )}
          {females.length === 0 && (
            <p className="season closed">母馬にできる引退牝馬がいません。牝馬を引退させるか、繁殖馬を購入してください。</p>
          )}
          <div className="bet">
            <label>
              父馬(牡)
              <select aria-label="父馬(牡)" value={sireSel} onChange={(e) => setSire(e.target.value)}>
                {sires.map((h) => (
                  <option key={h.name} value={h.name}>{parentLabel(h.name)}</option>
                ))}
              {outsideSires.length > 0 && (
                  <optgroup label="外部の種牡馬">
                    {outsideSires.map((o) => (
                      <option key={o.name} value={o.name}>{`${o.name}(種付け料 ${o.stud_fee ?? 0})`}</option>
                    ))}
                  </optgroup>
                )}
              </select>
            </label>
            <label>
              母馬(牝)
              <select aria-label="母馬(牝)" value={damSel} onChange={(e) => setDam(e.target.value)}>
                {females.map((h) => (
                  <option key={h.name} value={h.name} disabled={inFoal(h.name)}>
                    {`${parentLabel(h.name)}${inFoal(h.name) ? "(受胎中)" : ""}`}
                  </option>
                ))}
              </select>
            </label>
            <label>
              入れ替える馬
              <select aria-label="入れ替える馬" value={retireSel} onChange={(e) => setRetire(e.target.value)}>
                {roomInStable && <option value="">空き枠に入れる</option>}
                {game.field.map((h) => (
                  <option key={h.name} value={h.name}>{h.name}</option>
                ))}
              </select>
            </label>
            <label>
              仔馬の名前
              <input
                aria-label="仔馬の名前"
                value={foalName}
                maxLength={12}
                onChange={(e) => setFoalName(e.target.value)}
              />
            </label>
            <button className="primary" disabled={!canBreed} onClick={() => breed(sireSel, damSel, retireSel)}>
              {`配合 (${matingCost})`}
            </button>
          </div>
          <div className="outlook">
            {females.find((h) => h.name === damSel)?.conception !== undefined && (
              <p>{`受胎率: ${females.find((h) => h.name === damSel)?.conception}%`}</p>
            )}
            {outlook && (
              <>
                {outlook.rating && <p><strong>{`配合評価: ${outlook.rating}`}</strong></p>}
                {(outlook.theories ?? []).map((t) => (
                  <p key={t.name}>{`${t.name} +${t.points}`}</p>
                ))}
                <p>{`近親度(配合): ${inbreedText(outlook.inbreed_vol)}`}</p>
                {outlook.penalty > 0 && <p>{`気性への影響: -${outlook.penalty.toFixed(1)}`}</p>}
                <p>
                  {`相性(ニック): ${outlook.nick ? `${statLabel(outlook.nick.stat)}+${outlook.nick.bonus}` : "なし"}`}
                </p>
              </>
            )}
            {outlookError && <p className="error">{outlookError}</p>}
          </div>
          <p className="hint">
            馬名の決まり: <span>{NAME_RULE}</span>
          </p>
        </section>


        {pregnancies.length > 0 && (
          <section className="panel" aria-label="出産待ち">
            <h2>出産待ち</h2>
            <ul className="honours">
              {pregnancies.map((p) => (
                <li key={p.foal_name}>
                  <strong>{p.foal_name}</strong>
                  <span>{`父 ${p.sire}・母 ${p.dam}`}</span>
                  <span>{`あと${p.weeks_left}週`}</span>
                  {p.replace !== "" && <span>{`${p.replace}と入れ替わり`}</span>}
                </li>
              ))}
            </ul>
          </section>
        )}

        {game.farm && (
          <section className="panel" aria-label="牧場">
            <h2>牧場</h2>
            <p className="muted">{`放牧地を広げると仔馬の脚元とスタミナが、育成施設を整えると牧場を出る1歳馬のスピードとパワーが伸びます。1段階ごとに週の預託料が${game.farm.upkeep_per_level}増えます。`}</p>
            <ul className="honours">
              {([["pasture", "放牧地", game.farm.pasture, game.farm.pasture_cost], ["nursery", "育成施設", game.farm.nursery, game.farm.nursery_cost]] as const).map(
                ([facility, label, level, cost]) => (
                  <li key={facility}>
                    <strong>{`${label} Lv${level}/${game.farm!.max_level}`}</strong>
                    {cost != null && (
                      <button disabled={racing || game.game_over || cost >= game.money} onClick={() => upgradeFarm(facility)}>
                        {`${label}を拡張 (${cost})`}
                      </button>
                    )}
                  </li>
                )
              )}
            </ul>
          </section>
        )}

        {game.auction_start_week !== undefined && (
          <section className="panel" aria-label="セリ">
            <h2>セリ(1歳馬)</h2>
            <p className="muted">
              {`セリは7月(第${game.auction_start_week}週〜第${game.auction_end_week}週)に開かれます。${
                game.auction_open ? "今は開催中です。" : `今は第${game.week}週です。`
              }`}
            </p>
            {game.auction_open && (
              <p className="muted">
                上限額と馬名を入れて入札します。入札は1頭につき1回で、他の買い手の上限を超えていれば、その少し上の額で落札できます。見込みは目安で、当たるとは限りません。落札した馬はすぐ調教でき、2歳からレースに出られます。
              </p>
            )}
            {bidResult !== "" && <p className="season">{bidResult}</p>}
            <ul className="honours">
              {lots.map((l) => {
                const bid = bids[l.lot] ?? { max: "", name: "" };
                const maxBid = Number(bid.max);
                const canBid =
                  !racing && maxBid >= l.start_price && maxBid < game.money && isValidHorseName(bid.name);
                const edit = (change: Partial<typeof bid>) => setBids({ ...bids, [l.lot]: { ...bid, ...change } });
                return (
                  <li key={l.lot}>
                    <strong>{`上場${l.lot}`}</strong>
                    <span>{`${SEX_LABEL[l.sex]}1歳`}</span>
                    <span>{`父 ${l.sire}・母 ${l.dam}`}</span>
                    <span>{`見込み ${l.estimate}`}</span>
                    <span>{`開始 ${l.start_price}`}</span>
                    {l.closed ? (
                      <span>終了</span>
                    ) : (
                      <>
                        <input
                          aria-label={`上場${l.lot}の上限額`}
                          type="number"
                          min={l.start_price}
                          step={10}
                          value={bid.max}
                          onChange={(e) => edit({ max: e.target.value })}
                        />
                        <input
                          aria-label={`上場${l.lot}の馬名`}
                          placeholder="馬名"
                          maxLength={12}
                          value={bid.name}
                          onChange={(e) => edit({ name: e.target.value })}
                        />
                        <button
                          aria-label={`上場${l.lot}に入札`}
                          disabled={!canBid}
                          onClick={() => bidFor(l.lot)}
                        >
                          入札
                        </button>
                      </>
                    )}
                  </li>
                );
              })}
            </ul>
          </section>
        )}

        <section className="panel studbook" aria-labelledby="studbook-title">
          <h2 id="studbook-title" aria-label="繁殖馬">{`繁殖馬 (${studs.length}/${game.max_studs ?? 0})`}</h2>
          <p className="muted">{`引退させた馬や購入した馬を繁殖馬として配合に使えます。${RETIRE_AGE}歳になる年明けに自動で引退します。`}</p>
          {studs.length === 0 ? (
            <p className="muted">繁殖馬はいません。</p>
          ) : (
            <div className="stud-groups">
              {([["male", "種牡馬"], ["female", "繁殖牝馬"]] as const).map(([sex, label]) => {
                const group = studs.filter((s) => s.sex === sex);
                return (
                  <div key={sex}>
                    <h3>{`${label} (${group.length})`}</h3>
                    {group.length === 0 ? (
                      <p className="muted">{`${label}はいません。`}</p>
                    ) : (
                      <ul className="studs" aria-label={label}>
                        {group.map((s) => (
                          <li key={s.name}>
                            <HorseIcon name={s.name} coat={s.blood?.coat_color} />
                            <strong>{s.name}</strong>
                            <span className="muted">
                              {s.starter
                                ? "初期の繁殖馬"
                                : s.bought
                                  ? `${s.retired_year}年目に購入`
                                  : `${s.retired_year}年目に${s.retired_age}歳で引退`}
                            </span>
                            <span>{`スピード ${s.speed} / スタミナ ${s.stamina} / パワー ${s.power}`}</span>
                            {s.legs !== undefined && <span>{`脚元 ${s.legs}`}</span>}
                            {s.record && !s.bought && !s.starter && (
                              <span className="muted">{`${s.record.starts}戦${s.record.wins}勝`}</span>
                            )}
                            {s.blood && (
                              <span className="muted">{`系統: ${s.blood.lineage} / 脚質: ${s.blood.running_style}`}</span>
                            )}
                            <button
                              aria-label={`${s.name}を繁殖から外す`}
                              disabled={racing}
                              onClick={() => releaseStud(s.name)}
                            >
                              繁殖から外す
                            </button>
                          </li>
                        ))}
                      </ul>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </section>

        <section className="panel market" aria-labelledby="market-title">
          <h2 id="market-title" aria-label="繁殖馬の購入">繁殖馬の購入</h2>
          <p className="muted">血統データベースの馬を繁殖馬として買えます。販売される馬は年ごとに入れ替わります。</p>
          {market.length === 0 ? (
            <p className="muted">いま販売中の馬はいません。</p>
          ) : (
            <div className="stud-groups">
              {([["male", "種牡馬"], ["female", "繁殖牝馬"]] as const).map(([sex, label]) => {
                const group = market.filter((o) => o.sex === sex);
                return (
                  <div key={sex}>
                    <h3>{`${label} (${group.length})`}</h3>
                    {group.length === 0 ? (
                      <p className="muted">{`販売中の${label}はいません。`}</p>
                    ) : (
                      <ul className="studs" aria-label={`販売中の${label}`}>
                        {group.map((o) => (
                          <li key={o.blood_id}>
                            <HorseIcon name={o.name} coat={o.blood.coat_color} />
                            <strong>{o.name}</strong>
                            <span>{`スピード ${o.speed} / スタミナ ${o.stamina} / パワー ${o.power}`}</span>
                            {o.legs !== undefined && <span>{`脚元 ${o.legs}`}</span>}
                            <span className="muted">
                              {`気性 ${o.temper} / 系統: ${o.blood.lineage} / 脚質: ${o.blood.running_style} / ${o.blood.growth_type}`}
                            </span>
                            <button
                              aria-label={`${o.name}を購入`}
                              disabled={racing || game.game_over || studsFull || game.money < o.price}
                              title={studsFull ? "繁殖馬がいっぱいです" : game.money < o.price ? "所持金が足りません" : undefined}
                              onClick={() => buyStud(o)}
                            >
                              {`購入 (${o.price})`}
                            </button>
                          </li>
                        ))}
                      </ul>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </section>
      </main>
  );
}
