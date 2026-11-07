// The stable's pedigrees and the whole blood database.

import Bloodline, { inbreedText } from "../Bloodline";
import { DEBUT_AGE, SEX_LABEL, isYoung, mutationText, recordText } from "../labels";
import { AppView } from "../view";
import { Fold, HorseIcon } from "../widgets";

export default function PedigreeTab({ app }: { app: AppView }) {
  const { busy, foldProps, game, pedigreeView, retireHorse, setPedigreeView, studsFull } = app;
  return (
      <>
      <div className="subtabs">
        <button className={pedigreeView === "stable" ? "active" : ""} onClick={() => setPedigreeView("stable")}>
          厩舎の馬
        </button>
        <button className={pedigreeView === "database" ? "active" : ""} onClick={() => setPedigreeView("database")}>
          血統データベース
        </button>
      </div>
      {pedigreeView === "database" ? (
        <main>
          <Bloodline />
        </main>
      ) : (
      <main>
        <p className="muted">配合と繁殖馬の管理は「繁殖」タブで行います。</p>
        <div className="pedigrees">
        {game.field.map((h, i) => (
          <section key={h.name} className="panel pedigree" aria-label={`${h.name}の血統`}>
            <Fold
              {...foldProps(`pedigree:${h.name}`)}
              className="fold-part"
              summary={
                <>
                  <h2>
                    <HorseIcon name={h.name} coat={h.blood?.coat_color} />
                    {h.name}
                    <span className={`sex ${h.sex}`} aria-label={`${h.name}の性別: ${SEX_LABEL[h.sex]}`}>
                      {SEX_LABEL[h.sex]}
                    </span>
                  </h2>
                  <span className="fold-note">
                    {h.pedigree ? `父 ${h.pedigree.sire}・母 ${h.pedigree.dam}` : "血統不明"}
                  </span>
                </>
              }
            >
            {h.pedigree && (
              <dl>
                <div className="gen1"><dt>父</dt><dd>{`父: ${h.pedigree.sire}`}</dd></div>
                <div className="gen1"><dt>母</dt><dd>{`母: ${h.pedigree.dam}`}</dd></div>
                <div className="gen2"><dd>{`父父: ${h.pedigree.sire_sire}`}</dd></div>
                <div className="gen2"><dd>{`父母: ${h.pedigree.sire_dam}`}</dd></div>
                <div className="gen2"><dd>{`母父: ${h.pedigree.dam_sire}`}</dd></div>
                <div className="gen2"><dd>{`母母: ${h.pedigree.dam_dam}`}</dd></div>
              </dl>
            )}
            {h.blood && (
              <ul className="traits">
                {h.record && <li>{`戦績: ${recordText(h.record)}`}</li>}
                <li>{`毛色: ${h.blood.coat_color}`}</li>
                <li>{`系統: ${h.blood.lineage}`}</li>
                <li>{`成長型: ${h.blood.growth_type}(${h.blood.peak_age}歳ごろがピーク)`}</li>
                <li>{`脚質: ${h.blood.running_style}`}</li>
                <li>{`性格: ${h.blood.personality}`}</li>
                {h.blood.jump !== undefined && <li>{`飛越の血: ${Math.round(h.blood.jump)}`}</li>}
                <li>{`距離適性: ${h.blood.aptitude_min}m〜${h.blood.aptitude_max}m`}</li>
                <li>{`近親度: ${inbreedText(h.blood.inbreed_vol)}`}</li>
                {h.blood.mutations.length > 0 && (
                  <li className="mutant">{`突然変異: ${mutationText(h.blood.mutations)}`}</li>
                )}
              </ul>
            )}
            <button
              className="retire"
              aria-label={`${h.name}を引退させる`}
              disabled={busy || game.field.length <= 1 || (studsFull && !h.gelded) || isYoung(h)}
              title={
                isYoung(h)
                  ? `${DEBUT_AGE}歳になるまで引退できません`
                  : studsFull && !h.gelded
                    ? "繁殖馬がいっぱいです"
                    : game.field.length <= 1
                      ? "最後の1頭です"
                      : undefined
              }
              onClick={() => retireHorse(h.name)}
            >
              {h.gelded ? "引退" : "引退(繁殖入り)"}
            </button>
            </Fold>
          </section>
        ))}
        </div>

      </main>
      )}
      </>
  );
}
