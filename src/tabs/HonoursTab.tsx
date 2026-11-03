// Awards, the hall of fame, rivals and the trainers' records.

import { GRADE_LABEL } from "../Calendar";
import { SEX_LABEL, starRecord } from "../labels";
import { AppView } from "../useGame";

export default function HonoursTab({ app }: { app: AppView }) {
  const { game, trainers } = app;
  return (
      <main role="tabpanel" aria-label="実績">
        {game.owner_rank !== undefined && (
          <section className="panel" aria-label="目標">
            <h2>目標</h2>
            <p><strong>{`馬主の称号: ${game.owner_rank}`}</strong></p>
            <p className="muted">{`目標 ${(game.goals ?? []).filter((x) => x.done).length}/${(game.goals ?? []).length} 達成`}</p>
            <ul className="honours">
              {(game.goals ?? []).map((x) => (
                <li key={x.name}>
                  <strong>{x.done ? `✓ ${x.name}` : x.name}</strong>
                  <span>{x.description}</span>
                </li>
              ))}
            </ul>
          </section>
        )}
        {(game.sire_lines ?? []).length > 0 && (
          <section className="panel" aria-label="系統">
            <h2>系統</h2>
            <p className="muted">{`自分で使った種牡馬の産駒が${game.line_foals}頭以上、G1を${game.line_g1_wins}勝以上すると、その系統が確立します。確立した系統の種牡馬とその息子は配合評価が上がります。`}</p>
            <ul className="honours">
              {(game.sire_lines ?? []).map((l) => (
                <li key={l.sire}>
                  <strong>{l.established ? `${l.sire}系(確立)` : l.sire}</strong>
                  <span>{`産駒${l.foals}頭・G1 ${l.g1_wins}勝`}</span>
                </li>
              ))}
            </ul>
          </section>
        )}
        <section className="panel" aria-label="受賞">
          <h2>受賞</h2>
          {(game.awards ?? []).length === 0 ? (
            <p className="muted">まだありません。</p>
          ) : (
            <ul className="honours">
              {[...(game.awards ?? [])].reverse().map((a, i) => (
                <li key={i}>{`${a.year}年目 ${a.title}: ${a.horse}${a.mine ? "" : "(ライバル)"}`}</li>
              ))}
            </ul>
          )}
        </section>
        <section className="panel" aria-label="殿堂">
          <h2>殿堂</h2>
          <p className="muted">引退時にG1を5勝以上、三冠、または獲得賞金15000以上の馬が殿堂入りします。</p>
          {(game.hall ?? []).length === 0 ? (
            <p className="muted">まだいません。</p>
          ) : (
            <ul className="honours">
              {(game.hall ?? []).map((h) => (
                <li key={h.name}>
                  <strong>{h.name}</strong>
                  <span>{[`${h.wins}勝(G1 ${h.g1_wins}勝)`, `獲得賞金 ${h.earnings}`, ...h.titles].join("・")}</span>
                </li>
              ))}
            </ul>
          )}
        </section>
        <section className="panel" aria-label="現役馬の実績">
          <h2>現役馬の実績</h2>
          <ul className="honours">
            {game.field.map((h) => (
              <li key={h.name}>
                <strong>{h.name}</strong>
                <span>{`獲得賞金 ${h.career?.earnings ?? 0}`}</span>
                {(h.career?.titles ?? []).map((title, i) => (
                  <span key={i} className="title">{title}</span>
                ))}
                {(h.career?.stakes_wins ?? []).length > 0 && (
                  <span>
                    {h.career!.stakes_wins.map((w) => `${GRADE_LABEL[w.grade]} ${w.race}(${w.year}年目)`).join("・")}
                  </span>
                )}
              </li>
            ))}
          </ul>
        </section>
        <section className="panel" aria-label="ライバル">
          <h2>ライバル</h2>
          <p className="muted">重賞を走り続ける相手です。出走表に「ライバル」の印が付きます。</p>
          <table className="training">
            <thead>
              <tr><th align="left">馬</th><th>性齢</th><th align="left">戦績</th><th>獲得賞金</th><th align="left">自分の馬との対戦</th></tr>
            </thead>
            <tbody>
              {[...(game.stars ?? [])]
                .sort((a, b) => b.earnings - a.earnings)
                .map((r) => (
                  <tr key={r.name}>
                    <td className="name">{r.name}</td>
                    <td align="center">{`${SEX_LABEL[r.sex]}${r.age}`}</td>
                    <td>{starRecord(r)}</td>
                    <td align="center">{r.earnings}</td>
                    <td>{`対戦${r.met}回・先着${r.beaten}回`}</td>
                  </tr>
                ))}
            </tbody>
          </table>
        </section>
        <section className="panel" aria-label="調教師の成績">
          <h2>調教師の成績</h2>
          <table className="training">
            <thead>
              <tr><th align="left">調教師</th><th>所属</th><th>出走</th><th>勝利</th><th>勝率</th><th>G1</th><th>獲得賞金</th></tr>
            </thead>
            <tbody>
              {trainers.map((t) => {
                const rec = game.trainer_records?.[t.id] ?? { starts: 0, wins: 0, g1_wins: 0, earnings: 0 };
                return (
                  <tr key={t.id}>
                    <td className="name">{t.name}</td>
                    <td align="center">{t.region}</td>
                    <td align="center">{rec.starts}</td>
                    <td align="center">{rec.wins}</td>
                    <td align="center">{rec.starts > 0 ? `${((rec.wins / rec.starts) * 100).toFixed(1)}%` : "-"}</td>
                    <td align="center">{rec.g1_wins}</td>
                    <td align="center">{rec.earnings}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </section>
      </main>
  );
}
