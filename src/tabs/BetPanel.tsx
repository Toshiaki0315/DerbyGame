// Choosing and buying tickets.

import { KIND_LABEL, KIND_OPTIONS, KIND_ORDERED, KIND_PICKS, ORDERED_SLOTS, UNORDERED_SLOTS, selectionLabel } from "../labels";
import { BetKind, BuyStyle } from "../types";
import { AppView } from "../view";

export default function BetPanel({ app }: { app: AppView }) {
  const { addExpanded, addTickets, amount, axisSel, bettingOpen, busy, card, duplicate, effBox, effPartners, effStyle, fieldTooSmall, game, isSingle, kind, multiKind, newGame, outcome, pick, pickCount, quoteError, racing, runRace, runners, selection, setAmount, setAxis, setBoxPicks, setFrame, setKind, setPartners, setSlip, setStyle, slip, slipOdds, slipTotal, ticketOdds, toggleIn } = app;
  return (
    <>
    {bettingOpen ? (
      <>
      <div className="bet">
        <label>
          賭け方
          <select
            aria-label="賭け方"
            value={kind}
            onChange={(e) => {
              const next = e.target.value as BetKind;
              setKind(next);
              if (KIND_PICKS[next] < 2) setStyle("normal"); // a single horse cannot be boxed or flowed
            }}
          >
            {KIND_OPTIONS.map((o) => (
              <option key={o.value} value={o.value}>
                {o.value === "place" ? `複勝(${card?.place_slots ?? 3}着以内)` : o.label}
              </option>
            ))}
          </select>
        </label>
        <label>
          買い方
          <select
            aria-label="買い方"
            value={effStyle}
            disabled={!multiKind}
            onChange={(e) => setStyle(e.target.value as BuyStyle)}
          >
            <option value="normal">通常</option>
            <option value="box">ボックス</option>
            <option value="flow">流し</option>
          </select>
        </label>
        {effStyle === "normal" &&
          (isSingle ? (
            <label>
              馬
              <select aria-label="馬" value={selection[0]} onChange={(e) => pick(0, e.target.value)}>
                {runners.map((h) => (
                  <option key={h.name} value={h.name}>{h.mine ? `${h.name} ★` : h.name}</option>
                ))}
              </select>
            </label>
          ) : (
            selection.map((name, i) => {
              const slot = (KIND_ORDERED[kind] ? ORDERED_SLOTS : UNORDERED_SLOTS)[i];
              return (
                <label key={slot}>
                  {slot}
                  <select aria-label={slot} value={name} onChange={(e) => pick(i, e.target.value)}>
                    {runners.map((h) => (
                      <option key={h.name} value={h.name}>{h.mine ? `${h.name} ★` : h.name}</option>
                    ))}
                  </select>
                </label>
              );
            })
          ))}
        {effStyle === "flow" && (
          <label>
            軸
            <select aria-label="軸" value={axisSel} onChange={(e) => setAxis(e.target.value)}>
              {runners.map((h) => (
                <option key={h.name} value={h.name}>{h.mine ? `${h.name} ★` : h.name}</option>
              ))}
            </select>
          </label>
        )}
        <label>
          賭け金
          <input
            aria-label="賭け金"
            type="number"
            min={1}
            max={game.money}
            value={amount}
            onChange={(e) => setAmount(Number(e.target.value))}
          />
        </label>
        <button
          disabled={
            racing ||
            !card ||
            amount < 1 ||
            (effStyle === "normal" && (duplicate || fieldTooSmall)) ||
            (effStyle === "box" && effBox.length < pickCount) ||
            (effStyle === "flow" && effPartners.length < pickCount - 1)
          }
          onClick={effStyle === "normal" ? () => addTickets([selection]) : addExpanded}
        >
          買い目に追加
        </button>
        <button
          className="primary"
          onClick={runRace}
          disabled={busy || !card || (slip.length > 0 && slipTotal > game.money)}
        >
          レース開始
        </button>
        {racing && <button onClick={() => setFrame(outcome!.frames.length)}>スキップ</button>}
        {effStyle === "normal" && fieldTooSmall && (
          <p className="error">{`この馬券には${pickCount}頭以上の出走馬が必要です。`}</p>
        )}
        {effStyle === "normal" && duplicate && <p className="error">同じ馬は選べません。</p>}
        {effStyle === "normal" && ticketOdds !== null && !fieldTooSmall && !duplicate && (
          <p className="quote">{`オッズ: x${ticketOdds.toFixed(1)}`}</p>
        )}
        {effStyle === "normal" && ticketOdds !== null && !fieldTooSmall && !duplicate && (
          <p className="muted">{`的中時の払い戻し: ${Math.round(amount * ticketOdds)}`}</p>
        )}
        {slip.length === 0 && (
          <p className="muted">馬券を買うときは「買い目に追加」を押してください。押さずに始めると馬券なしで走ります。</p>
        )}
        {quoteError && <p className="error">{quoteError}</p>}
      </div>

      {effStyle === "box" && (
        <fieldset className="entries">
          <legend>ボックスに入れる馬</legend>
          <div className="entry-list">
            {runners.map((h) => (
              <label key={h.name}>
                <input
                  type="checkbox"
                  aria-label={`ボックス: ${h.name}`}
                  checked={effBox.includes(h.name)}
                  onChange={() => setBoxPicks(toggleIn(effBox, h.name))}
                />
                {h.name}
              </label>
            ))}
          </div>
        </fieldset>
      )}
      {effStyle === "flow" && (
        <fieldset className="entries">
          <legend>相手にする馬</legend>
          <div className="entry-list">
            {runners
              .filter((h) => h.name !== axisSel)
              .map((h) => (
                <label key={h.name}>
                  <input
                    type="checkbox"
                    aria-label={`相手: ${h.name}`}
                    checked={effPartners.includes(h.name)}
                    onChange={() => setPartners(toggleIn(effPartners, h.name))}
                  />
                  {h.name}
                </label>
              ))}
          </div>
        </fieldset>
      )}

      {slip.length > 0 && (
        <div className="slip">
          <h3>買い目</h3>
          <ul>
            {slip.map((t, i) => {
              const text = `${KIND_LABEL[t.kind]} ${selectionLabel(t.kind, t.horses)}`;
              return (
                <li key={i}>
                  <span>{`${text} ${t.amount}`}</span>
                  {slipOdds[i] !== undefined && <span className="odds">{`x${slipOdds[i].toFixed(1)}`}</span>}
                  <button aria-label={`削除: ${text}`} onClick={() => setSlip(slip.filter((_, j) => j !== i))}>
                    削除
                  </button>
                </li>
              );
            })}
          </ul>
          <p>{`合計 ${slip.length}点 / ${slipTotal}`}</p>
          {slipTotal > game.money && <p className="error">所持金が足りません。</p>}
          <button onClick={() => setSlip([])}>買い目をすべて削除</button>
        </div>
      )}
      </>
    ) : (
      <div className="gameover">
        <h2>ゲームオーバー</h2>
        <button className="primary" onClick={newGame}>最初から</button>
      </div>
    )}
    </>
  );
}
