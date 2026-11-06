// Awards, the hall of fame, rivals and the trainers' records.

import { GRADE_LABEL } from "../Calendar";
import { SEX_LABEL, starRecord } from "../labels";
import { AppView } from "../view";
import { FoldPart } from "../widgets";

export default function HonoursTab({ app }: { app: AppView }) {
  const { foldProps, game, trainers } = app;
  const goals = game.goals ?? [];
  const lines = game.sire_lines ?? [];
  // this year's leader of each table, kept on the heading while the tables are folded
  const leaders = ([["騎手", game.leading?.jockeys], ["調教師", game.leading?.trainers], ["種牡馬", game.leading?.sires]] as const)
    .filter(([, rows]) => (rows ?? []).length > 0)
    .map(([title, rows]) => `${title} ${rows![0].name}`)
    .join("・");
  return (
      <main role="tabpanel" aria-label="実績">
        {game.owner_rank !== undefined && (
          <FoldPart label="目標" fold={foldProps("goals", true)} notes={[`${goals.filter((x) => x.done).length}/${goals.length} 達成`]}>
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
          </FoldPart>
        )}
        {(game.sire_lines ?? []).length > 0 && (
          <FoldPart label="系統" fold={foldProps("sire-lines")} notes={[`${lines.length}系統(確立${lines.filter((l) => l.established).length})`]}>
            <p className="muted">{`自分で使った種牡馬の産駒が${game.line_foals}頭以上、G1を${game.line_g1_wins}勝以上すると、その系統が確立します。確立した系統の種牡馬とその息子は配合評価が上がります。`}</p>
            <ul className="honours">
              {(game.sire_lines ?? []).map((l) => (
                <li key={l.sire}>
                  <strong>{l.established ? `${l.sire}系(確立)` : l.sire}</strong>
                  <span>{`産駒${l.foals}頭・G1 ${l.g1_wins}勝`}</span>
                </li>
              ))}
            </ul>
          </FoldPart>
        )}
        <FoldPart label="受賞" fold={foldProps("awards")} notes={[`${(game.awards ?? []).length}件`]}>
          {(game.awards ?? []).length === 0 ? (
            <p className="muted">まだありません。</p>
          ) : (
            <ul className="honours">
              {[...(game.awards ?? [])].reverse().map((a, i) => (
                <li key={i}>{`${a.year}年目 ${a.title}: ${a.horse}${a.mine ? "" : "(ライバル)"}`}</li>
              ))}
            </ul>
          )}
        </FoldPart>
        <FoldPart label="殿堂" fold={foldProps("hall")} notes={[`${(game.hall ?? []).length}頭`]}>
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
        </FoldPart>
        <FoldPart label="現役馬の実績" fold={foldProps("careers")} notes={[`${game.field.length}頭`]}>
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
        </FoldPart>
        <FoldPart label="ライバル" fold={foldProps("rivals")} notes={[`${(game.stars ?? []).length}頭`]}>
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
        </FoldPart>
        <FoldPart label="リーディング" className="panel leading" fold={foldProps("leading")} notes={[leaders]}>
          <p className="muted">今年ここまでの勝利数です。カッコ内は自分の馬での勝利数です。</p>
          <div className="leading-tables">
            {([["騎手", game.leading?.jockeys], ["調教師", game.leading?.trainers], ["種牡馬", game.leading?.sires]] as const).map(([title, rows]) => (
              <table key={title} aria-label={`${title}リーディング`}>
                <thead>
                  <tr><th>順位</th><th align="left">{title}</th><th>勝利</th></tr>
                </thead>
                <tbody>
                  {(rows ?? []).slice(0, 5).map((r, i) => (
                    <tr key={r.name}>
                      <td align="center">{i + 1}</td>
                      <td>{r.name}</td>
                      <td align="center">{r.own_wins > 0 ? `${r.wins}(${r.own_wins})` : r.wins}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            ))}
          </div>
          {(game.leading_years?.length ?? 0) > 0 && (
            <ul className="leading-history" aria-label="過去のリーディング">
              {[...game.leading_years!].reverse().slice(0, 5).map((y) => (
                <li key={y.year}>{`${y.year}年目: 騎手 ${y.jockey} / 調教師 ${y.trainer} / 種牡馬 ${y.sire}`}</li>
              ))}
            </ul>
          )}
        </FoldPart>
        <FoldPart label="調教師の成績" fold={foldProps("trainer-records")} notes={[`${trainers.length}人`]}>
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
        </FoldPart>
      </main>
  );
}
