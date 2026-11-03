import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface BloodStats {
  speed: number;
  stamina: number;
  temper: number;
  power: number;
  turf: number;
  dirt: number;
}

export interface BloodHorse {
  id: number;
  name: string;
  sire_id: number | null;
  dam_id: number | null;
  gender: "male" | "female";
  lineage: string;
  family: string;
  gen: number;
  coat_color: string;
  genes: { G: string; E: string; A: string };
  growth_factor: number;
  growth_type: string;
  peak_age: number;
  aptitude_min: number;
  aptitude_max: number;
  running_style: string;
  personality: string;
  inbreed_vol: number;
  stats: BloodStats;
  /** Stats that mutated (突然変異), by name. Absent in data from before mutations. */
  mutations?: (keyof BloodStats)[];
  /** One of the horses of my own stable (or a retired one). Absent in older data. */
  owned?: boolean;
}

interface ConditionEffect {
  condition: "good" | "soft" | "yielding" | "heavy";
  speed: number;
  stamina: number;
}

/** From the backend: bad-track aptitude and the stats on each kind of track. */
interface TrackEffects {
  aptitude: number;
  conditions: ConditionEffect[];
}

const TRACK_LABEL = { good: "良", soft: "稍重", yielding: "重", heavy: "不良" } as const;

const PAGE_SIZE = 15;
const PEDIGREE_GENERATIONS = 4;
const SEX_LABEL = { male: "牡", female: "牝" } as const;
const COAT_SWATCH: Record<string, string> = {
  芦毛: "#c9c9c9",
  栗毛: "#a8552a",
  鹿毛: "#6b3e26",
  青毛: "#1b1b1b",
};
export const STAT_LABELS: { key: keyof BloodStats; label: string }[] = [
  { key: "speed", label: "スピード" },
  { key: "stamina", label: "スタミナ" },
  { key: "temper", label: "気性" },
  { key: "power", label: "パワー" },
  { key: "turf", label: "芝適性" },
  { key: "dirt", label: "ダート適性" },
];

const MUTANT_MARK = "✦";
const isMutant = (h: BloodHorse) => (h.mutations?.length ?? 0) > 0;

/** How wide to draw a bar for a 0-100 value; mutations can go past 100 but the bar stops at its end. */
const barWidth = (v: number) => `${Math.min(Math.max(v, 0), 100)}%`;

/** How strongly a horse is inbred, in words. */
export function inbreedText(vol: number): string {
  if (!vol || vol <= 0) return "なし";
  const level = vol < 0.1 ? "弱い" : vol < 0.25 ? "やや強い" : "強い";
  return `${vol.toFixed(3)}(${level})`;
}

interface Ancestor {
  path: string;
  horse: BloodHorse | null;
}

/** Ancestors generation by generation: 父 母 / 父父 父母 母父 母母 / ... (2, 4, 8, 16). */
function pedigreeOf(horse: BloodHorse, byId: Map<number, BloodHorse>): Ancestor[] {
  const out: Ancestor[] = [];
  let level: Ancestor[] = [{ path: "", horse }];
  for (let g = 0; g < PEDIGREE_GENERATIONS; g++) {
    const next: Ancestor[] = [];
    for (const node of level) {
      const sire = node.horse?.sire_id != null ? byId.get(node.horse.sire_id) ?? null : null;
      const dam = node.horse?.dam_id != null ? byId.get(node.horse.dam_id) ?? null : null;
      next.push({ path: `${node.path}父`, horse: sire }, { path: `${node.path}母`, horse: dam });
    }
    out.push(...next);
    level = next;
  }
  return out;
}

function Bloodline() {
  const [horses, setHorses] = useState<BloodHorse[] | null>(null);
  const [error, setError] = useState("");
  const [gen, setGen] = useState("all");
  const [sex, setSex] = useState("all");
  const [lineage, setLineage] = useState("all");
  const [query, setQuery] = useState("");
  const [mutantsOnly, setMutantsOnly] = useState(false);
  const [ownedOnly, setOwnedOnly] = useState(false);
  const [page, setPage] = useState(0);
  const [selected, setSelected] = useState<number | null>(null);
  const [effects, setEffects] = useState<TrackEffects | null>(null);
  const [effectsError, setEffectsError] = useState("");

  useEffect(() => {
    invoke<BloodHorse[]>("get_bloodline")
      .then(setHorses)
      .catch((e) => setError(String(e)));
  }, []);

  const byId = useMemo(() => new Map((horses ?? []).map((h) => [h.id, h])), [horses]);
  const filtered = useMemo(
    () =>
      (horses ?? []).filter(
        (h) =>
          (gen === "all" || h.gen === Number(gen)) &&
          (sex === "all" || h.gender === sex) &&
          (lineage === "all" || h.lineage === lineage) &&
          (query === "" || h.name.includes(query)) &&
          (!mutantsOnly || isMutant(h)) &&
          (!ownedOnly || h.owned === true)
      ),
    [horses, gen, sex, lineage, query, mutantsOnly, ownedOnly]
  );

  const chosen = selected != null ? byId.get(selected) ?? null : null;

  // how the chosen horse copes with a soft or heavy track (calculated by the backend)
  useEffect(() => {
    setEffects(null);
    setEffectsError("");
    if (!chosen) return;
    let live = true;
    const { speed, stamina, power, dirt } = chosen.stats;
    invoke<TrackEffects>("get_track_effects", { speed, stamina, power, dirt })
      .then((e) => live && setEffects(e))
      .catch((e) => live && setEffectsError(String(e)));
    return () => {
      live = false;
    };
  }, [chosen?.id]); // eslint-disable-line react-hooks/exhaustive-deps

  if (error) return <p className="error">{error}</p>;
  if (!horses) return <p>読み込み中...</p>;

  const gens = [...new Set(horses.map((h) => h.gen))].sort((a, b) => a - b);
  const mutantCount = horses.filter(isMutant).length;
  const ownedCount = horses.filter((h) => h.owned).length;
  const lineages = [...new Set(horses.map((h) => h.lineage))].sort();
  const pages = Math.max(1, Math.ceil(filtered.length / PAGE_SIZE));
  const current = Math.min(page, pages - 1);
  const rows = filtered.slice(current * PAGE_SIZE, (current + 1) * PAGE_SIZE);
  // changing a filter always starts again from the first page
  const filter = (set: (v: string) => void) => (e: { target: { value: string } }) => {
    set(e.target.value);
    setPage(0);
  };

  return (
    <section className="panel bloodline">
      <h2>血統データベース</h2>
      <p className="muted">全{horses.length}頭</p>
      {mutantCount > 0 && <p className="muted">{`突然変異: ${mutantCount}頭`}</p>}
      {ownedCount > 0 && <p className="muted">{`厩舎の馬: ${ownedCount}頭`}</p>}

      <div className="bloodline-filters">
        <label>
          世代
          <select aria-label="世代" value={gen} onChange={filter(setGen)}>
            <option value="all">すべて</option>
            {gens.map((g) => (
              <option key={g} value={g}>{`第${g}世代`}</option>
            ))}
          </select>
        </label>
        <label>
          性別
          <select aria-label="性別" value={sex} onChange={filter(setSex)}>
            <option value="all">すべて</option>
            <option value="male">牡</option>
            <option value="female">牝</option>
          </select>
        </label>
        <label>
          系統
          <select aria-label="系統" value={lineage} onChange={filter(setLineage)}>
            <option value="all">すべて</option>
            {lineages.map((l) => (
              <option key={l} value={l}>{l}</option>
            ))}
          </select>
        </label>
        <label>
          名前
          <input aria-label="名前で検索" value={query} placeholder="名前の一部" onChange={filter(setQuery)} />
        </label>
        <label className="check">
          <input
            type="checkbox"
            aria-label="突然変異のみ"
            checked={mutantsOnly}
            onChange={(e) => {
              setMutantsOnly(e.target.checked);
              setPage(0);
            }}
          />
          突然変異のみ
        </label>
        <label className="check">
          <input
            type="checkbox"
            aria-label="厩舎の馬のみ"
            checked={ownedOnly}
            onChange={(e) => {
              setOwnedOnly(e.target.checked);
              setPage(0);
            }}
          />
          厩舎の馬のみ
        </label>
      </div>

      <div className="bloodline-body">
        <div>
          <p className="muted">{`該当 ${filtered.length}頭`}</p>
          {filtered.length === 0 ? (
            <p className="muted">該当する馬がいません。</p>
          ) : (
            <table aria-label="馬の一覧" className="bloodline-list">
              <thead>
                <tr>
                  <th align="left">名前</th><th>性別</th><th>系統</th><th>牝系</th><th>世代</th><th>毛色</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((h) => (
                  <tr key={h.id} className={h.id === selected ? "picked" : ""}>
                    <td className="name">
                      <button className="link" onClick={() => setSelected(h.id)}>{h.name}</button>
                      {isMutant(h) && <span className="mutant-mark" aria-label="突然変異あり">{MUTANT_MARK}</span>}
                      {h.owned && <span className="owned-mark" aria-label="厩舎の馬">厩</span>}
                    </td>
                    <td align="center">{SEX_LABEL[h.gender]}</td>
                    <td align="center">{h.lineage}</td>
                    <td align="center">{h.family}</td>
                    <td align="center">{h.gen}</td>
                    <td align="center">{h.coat_color}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
          <div className="pager">
            <button disabled={current <= 0} onClick={() => setPage(current - 1)}>前のページ</button>
            <span>{`${current + 1} / ${pages} ページ`}</span>
            <button disabled={current >= pages - 1} onClick={() => setPage(current + 1)}>次のページ</button>
          </div>
        </div>

        {chosen ? (
          <section aria-label="馬の詳細" className="horse-detail">
            <h3>
              {chosen.name}
              <span className={`sex ${chosen.gender}`} aria-label={`${chosen.name}の性別: ${SEX_LABEL[chosen.gender]}`}>
                {SEX_LABEL[chosen.gender]}
              </span>
            </h3>
            {chosen.owned && <p className="owned-note">厩舎の馬です。</p>}
            <dl className="profile">
              <dt>毛色</dt>
              <dd>
                <span className="swatch" style={{ background: COAT_SWATCH[chosen.coat_color] ?? "#888" }} />
                {chosen.coat_color}
              </dd>
              <dt>系統</dt><dd>{chosen.lineage}</dd>
              <dt>牝系</dt><dd>{chosen.family}</dd>
              <dt>世代</dt><dd>{`第${chosen.gen}世代`}</dd>
              <dt>成長型</dt><dd>{`${chosen.growth_type}(${chosen.peak_age}歳ごろがピーク)`}</dd>
              <dt>距離適性</dt><dd>{`${chosen.aptitude_min}m〜${chosen.aptitude_max}m`}</dd>
              <dt>脚質</dt><dd>{chosen.running_style}</dd>
              <dt>性格</dt><dd>{chosen.personality}</dd>
              <dt>近親度</dt><dd>{inbreedText(chosen.inbreed_vol)}</dd>
              {isMutant(chosen) && (
                <>
                  <dt>突然変異</dt>
                  <dd className="mutant-text">
                    {STAT_LABELS.filter(({ key }) => chosen.mutations?.includes(key))
                      .map(({ label }) => label)
                      .join("・")}
                  </dd>
                </>
              )}
            </dl>

            <div className="stat-meters">
              {STAT_LABELS.map(({ key, label }) => {
                const v = chosen.stats[key];
                const mutated = chosen.mutations?.includes(key) ?? false;
                return (
                  <div key={key} className="stat-row">
                    <span>
                      {label}
                      {mutated && (
                        <span className="mutant-mark" aria-label={`${label}は突然変異`}>{MUTANT_MARK}</span>
                      )}
                    </span>
                    <div
                      role="meter"
                      aria-label={`${label}: ${v}`}
                      aria-valuenow={v}
                      aria-valuemin={0}
                      aria-valuemax={Math.max(100, v)}
                      className={mutated ? "meter mutated" : "meter"}
                    >
                      <div style={{ width: barWidth(v) }} />
                    </div>
                    <span className="stat-value">{v}</span>
                  </div>
                );
              })}
            </div>

            <h4>道悪適性</h4>
            {effectsError && <p className="error">{effectsError}</p>}
            {effects && (
              <>
                <div className="stat-row">
                  <span>道悪適性</span>
                  <div
                    role="meter"
                    aria-label={`道悪適性: ${effects.aptitude}`}
                    aria-valuenow={effects.aptitude}
                    aria-valuemin={0}
                    aria-valuemax={Math.max(100, effects.aptitude)}
                    className="meter"
                  >
                    <div style={{ width: barWidth(effects.aptitude) }} />
                  </div>
                  <span className="stat-value">{effects.aptitude}</span>
                </div>
                <p className="muted">
                  パワー70%・ダート適性30%で決まります。重・不良では、適性が高いほどスピードの低下とスタミナの消耗が小さくなります。
                </p>
                <table aria-label="馬場状態別の能力" className="track-effects">
                  <thead>
                    <tr><th align="left">馬場</th><th>スピード</th><th>スタミナ</th></tr>
                  </thead>
                  <tbody>
                    {effects.conditions.map((c) => (
                      <tr key={c.condition}>
                        <td>{TRACK_LABEL[c.condition]}</td>
                        <td align="center">{c.speed}</td>
                        <td align="center">{c.stamina}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </>
            )}

            <h4>血統表(4代)</h4>
            <ul className="pedigree-tree">
              {pedigreeOf(chosen, byId).map(({ path, horse }) => (
                <li
                  key={path}
                  aria-label={`${path}: ${horse ? horse.name : "不明"}`}
                  className={`gen${path.length}${horse && isMutant(horse) ? " mutant" : ""}`}
                >
                  <span className="path">{path}</span>
                  {horse ? (
                    <button className="link" onClick={() => setSelected(horse.id)}>{horse.name}</button>
                  ) : (
                    <span className="muted">不明</span>
                  )}
                  {horse && isMutant(horse) && (
                    <span className="mutant-mark" aria-label="突然変異あり">{MUTANT_MARK}</span>
                  )}
                </li>
              ))}
            </ul>
          </section>
        ) : (
          <p className="muted">馬を選ぶと、詳しい情報と血統表が表示されます。</p>
        )}
      </div>
    </section>
  );
}

export default Bloodline;
