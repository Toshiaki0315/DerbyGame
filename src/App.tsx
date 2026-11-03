import Calendar from "./Calendar";
import { AUTOPLAY_SPEEDS, TABS } from "./labels";
import BreedingTab from "./tabs/BreedingTab";
import HonoursTab from "./tabs/HonoursTab";
import InfoTab from "./tabs/InfoTab";
import PedigreeTab from "./tabs/PedigreeTab";
import RaceTab from "./tabs/RaceTab";
import StableTab from "./tabs/StableTab";
import TrainTab from "./tabs/TrainTab";
import { useGame, viewOf } from "./useGame";
import "./App.css";

function App() {
  const model = useGame();
  if (!model.game) return <p>読み込み中...</p>;
  const app = { ...model, ...viewOf(model, model.game), game: model.game };
  const { autoplay, autoplayLog, autoplayMs, busy, calendar, error, game, loadSaved, notice, racing, save, setAllAuto, setAutoplay, setAutoplayMs, setStopAtNewYear, setTab, startAutoplay, stopAtNewYear, tab } = app;

  // below the bars rather than above them, so that the bars do not move as messages come and go
  const messages = (
    <>
      {notice && <p className="notice">{notice}</p>}
      {error && <p className="error">{error}</p>}
    </>
  );

  return (
    <div className="app">
      <header className="topbar">
        <h1>ダービーゲームスタジオ</h1>
        <div className="status">
          <p className="money">所持金: {game.money}</p>
          {game.weekly_upkeep !== undefined && <p className="round">{`週の預託料: ${game.weekly_upkeep}`}</p>}
          <p className="round">ラウンド: {game.round}</p>
        </div>
        <div className="actions">
          <button onClick={save} disabled={busy}>セーブ</button>
          <button onClick={loadSaved} disabled={busy}>ロード</button>
        </div>
      </header>
      <section className="autoplay">
        {autoplay ? (
          <button onClick={() => setAutoplay(false)}>オートプレイ停止</button>
        ) : (
          <button onClick={startAutoplay} disabled={racing || game.game_over || game.field.length === 0}>
            オートプレイ開始
          </button>
        )}
        <label>
          速さ
          <select aria-label="オートプレイの速さ" value={autoplayMs} onChange={(e) => setAutoplayMs(Number(e.target.value))}>
            {AUTOPLAY_SPEEDS.map((o) => (
              <option key={o.ms} value={o.ms}>{o.label}</option>
            ))}
          </select>
        </label>
        <label>
          <input type="checkbox" checked={stopAtNewYear} onChange={(e) => setStopAtNewYear(e.target.checked)} />
          年が変わったら止める
        </label>
        {game.field.some((h) => !h.auto_entry || !h.auto_training) && (
          <button onClick={() => setAllAuto(true)} disabled={racing}>全頭を自動にする</button>
        )}
        {game.field.some((h) => h.auto_entry || h.auto_training) && (
          <button onClick={() => setAllAuto(false)} disabled={racing}>全頭の自動を解除</button>
        )}
        {autoplay && <strong>{`進行中: ${game.year}年目 第${game.week}週`}</strong>}
        {game.field.every((h) => !h.auto_entry && !h.auto_training) && (
          <span className="muted">育成タブで「自動出走」「お任せ」をオンにした馬だけが、週の間に出走・調教します。</span>
        )}
        {autoplayLog.length > 0 && (
          <ul aria-label="オートプレイの記録">
            {autoplayLog.map((line, i) => (
              <li key={i}>{line}</li>
            ))}
          </ul>
        )}
      </section>

      <nav className="tabs" role="tablist">
        {TABS.map((t) => (
          <button
            key={t.id}
            role="tab"
            aria-selected={tab === t.id}
            className={tab === t.id ? "active" : ""}
            onClick={() => setTab(t.id)}
          >
            {t.label}
          </button>
        ))}
      </nav>
      {tab !== "race" && messages}

      {tab === "race" && <RaceTab app={app} messages={messages} />}

      {tab === "train" && <TrainTab app={app} />}

      {tab === "stable" && <StableTab app={app} />}

      {tab === "calendar" && <Calendar data={calendar ?? undefined} />}

      {tab === "honours" && <HonoursTab app={app} />}

      {tab === "info" && <InfoTab app={app} />}

      {tab === "pedigree" && <PedigreeTab app={app} />}

      {tab === "breeding" && <BreedingTab app={app} />}
    </div>
  );
}

export default App;
