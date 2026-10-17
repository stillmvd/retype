import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api, plural, type Config, type FingersState, type Item, type Pair, type Session, type TrainerState } from "../api";
import { Icon, Toggle } from "../Icon";
import { Group, Row } from "../settings/Settings";
import { finger, session, type Run } from "./engine";
import { Exercise } from "./Exercise";
import { Fingers } from "./Fingers";
import { Streak } from "./Streak";
import "./trainer.css";
import "./fingers.css";

type Screen = { k: "start" } | { k: "ex"; items: Item[] } | { k: "res"; s: Session; misses: number };
type Sec = "fingers" | "warm" | "prog" | "set";

const win = getCurrentWindow();

const MONTHS = ["января", "февраля", "марта", "апреля", "мая", "июня", "июля", "августа", "сентября", "октября", "ноября", "декабря"];
const dayMonth = (date: string) => `${+date.slice(8, 10)} ${MONTHS[+date.slice(5, 7) - 1]}`;
const ddmm = (date: string) => `${date.slice(8, 10)}.${date.slice(5, 7)}`;
const dec = (v: number) => v.toFixed(1).replace(".", ",");
const arrow = (key: string) => key.replace(">", " → ");

function Pairs({ list }: { list: Pair[] }) {
  return (
    <span className="tr-pairs">
      {list.map((p) => (
        <span key={p.wrong + p.right} className={`t-chip${p.state === "won" ? " won" : ""}`} title={`Нажимаешь «${p.wrong}» вместо «${p.right}»`}>
          <b>{p.wrong} → {p.right}</b>{p.since && <i>с {dayMonth(p.since)}</i>}
        </span>
      ))}
    </span>
  );
}

function Start({ st, cfg, saveCfg, offerSeen, go }: { st: TrainerState; cfg: Config; saveCfg: (c: Config) => void; offerSeen: () => void; go: () => void }) {
  const weak = st.pairs.filter((p) => p.state === "weak");
  const setMinutes = (minutes: 3 | 5) => saveCfg({ ...cfg, trainer: { ...cfg.trainer, minutes } });
  return (
    <div className="tr-main center">
      <h2 className="tr-h">Разминка</h2>
      {st.firstRun ? (
        <div className="tr-offer">
          <span className="tx">
            <b>Считать ошибки печати?</b>
            <span>Только числа: сколько слов набрано и какие клавиши стёрты. Без текста, только на этом компьютере.</span>
          </span>
          <span className="btns">
            <button type="button" className="t-btn s32 p" onClick={() => { saveCfg({ ...cfg, trainer: { ...cfg.trainer, count: true } }); offerSeen(); }}>Включить</button>
            <button type="button" className="t-btn s32 g" onClick={offerSeen}>Не сейчас</button>
          </span>
        </div>
      ) : st.streak > 0 && <div className="tr-row c"><Streak n={st.streak} /></div>}
      <div className="tr-row c">
        <span className="t-seg" role="group" aria-label="Длительность">
          {([3, 5] as const).map((m) => <button key={m} type="button" aria-pressed={cfg.trainer.minutes === m} onClick={() => setMinutes(m)}>{m} мин</button>)}
        </span>
      </div>
      <button type="button" className="t-btn p tr-go" onClick={go}><Icon name="play" />Начать разминку</button>
      {!st.firstRun && weak.length > 0 && <span className="tr-dim">Сегодня: {weak.slice(0, 3).map((p) => `${p.wrong} → ${p.right}`).join(", ")}</span>}
    </div>
  );
}

function WarmLocked({ fs, toFingers }: { fs: FingersState; toFingers: () => void }) {
  return (
    <div className="tr-main center">
      <span className="pf-lockbig"><Icon name="lock" /></span>
      <h2 className="pf-h">Сначала <b>поставь пальцы</b></h2>
      <div className="pf-prog" role="img" aria-label={`Уровень ${fs.level} из 12`}>
        {fs.levels.slice(0, 12).map((l) => <i key={l.n} className={l.state === "done" ? "ok" : l.state === "current" ? "cur" : undefined} />)}
      </div>
      <span className="pf-dim">Уровень {fs.level} из 12 — разминка откроется после английского нижнего ряда</span>
      <button type="button" className="t-btn p" onClick={toFingers}><Icon name="hand" />К постановке</button>
    </div>
  );
}

function Result({ s, misses, again, progress }: { s: Session; misses: number; again: () => void; progress: () => void }) {
  const top = Object.entries(s.pairs).sort((a, b) => b[1] - a[1]).slice(0, 4);
  return (
    <div className="tr-main center">
      <h2 className="tr-h">Готово: <b>{Math.round(s.accuracy * 100)} %</b> точность · <b>{s.cpm}</b> зн/мин</h2>
      {top.length > 0 ? (
        <div className="tr-res-rows">
          {top.map(([k, n]) => {
            const right = k.slice(-1);
            const f = finger(right);
            return (
              <div key={k} className="t-setrow">
                <span className="tx"><b className="k">{arrow(k)}</b>{f && <span>{right} — {f.name}</span>}</span>
                <span className="n">{n} {plural(n, "раз", "раза", "раз")}</span>
              </div>
            );
          })}
        </div>
      ) : (
        <span className="tr-dim">{misses ? `Ошибок: ${misses}` : "Без ошибок"}</span>
      )}
      <div className="tr-row c">
        <button type="button" className="t-btn p" onClick={again}><Icon name="restart" />Ещё раз</button>
        <button type="button" className="t-btn g" onClick={progress}><Icon name="chart" />Прогресс</button>
      </div>
    </div>
  );
}

function Chart({ weeks: all }: { weeks: TrainerState["weeks"] }) {
  const W = 760, H = 180, padL = 8, padB = 22;
  const weeks = all.slice(-8);
  const vals = weeks.map((w) => (w.words > 0 ? (w.misses * 100) / w.words : 0));
  const max = Math.max(1, Math.ceil(Math.max(...vals) * 1.15));
  const step = (W - padL * 2) / Math.max(weeks.length, 7);
  const bw = Math.min(56, step * 0.7);
  const x0 = (W - step * weeks.length) / 2;
  const y = (v: number) => H - padB - (v / max) * (H - padB - 20);
  return (
    <svg className="tr-chart" viewBox={`0 0 ${W} ${H}`} role="img" aria-label={`Ошибки на 100 слов по неделям: ${vals.map(dec).join(", ")}`}>
      {weeks.map((w, i) => {
        const bx = x0 + i * step + (step - bw) / 2;
        return (
          <g key={w.week}>
            <rect className={`bar${i === weeks.length - 1 ? " last" : ""}`} x={bx} y={y(vals[i])} width={bw} height={Math.max(0, H - padB - y(vals[i]))} rx="8" />
            <text x={bx + bw / 2} y={H - 6} textAnchor="middle">{ddmm(w.week)}</text>
            <text className="val" x={bx + bw / 2} y={y(vals[i]) - 6} textAnchor="middle">{dec(vals[i])}</text>
          </g>
        );
      })}
    </svg>
  );
}

function Progress({ st, enable }: { st: TrainerState; enable: () => void }) {
  const weak = st.pairs.filter((p) => p.state === "weak");
  const won = st.pairs.filter((p) => p.state === "won");
  const day = st.days[st.days.length - 1];
  return (
    <div className="tr-main scroll">
      <div className="tr-row"><h2 className="tr-h">Ошибки <b>на 100 слов</b></h2><span className="tr-dim">в обычной печати, по неделям</span></div>
      {st.weeks.length > 0 ? <Chart weeks={st.weeks} /> : (
        <div className="tr-empty">
          {st.counting ? (
            <span className="tr-dim">Данные появятся после первых дней печати.</span>
          ) : (
            <>
              <span className="tr-dim">Ошибки в обычной печати не считаются.</span>
              <button type="button" className="t-btn s32 p" onClick={enable}>Считать ошибки</button>
            </>
          )}
        </div>
      )}
      <div className="tr-grid2">
        <div className="tr-blk"><span className="tr-lh"><span className="mono-label tr-lbl">Слабые пары</span><span className="tr-dim">нажал → нужно было</span></span>{weak.length ? <Pairs list={weak} /> : <span className="tr-dim">Пока нет</span>}</div>
        <div className="tr-blk"><span className="mono-label tr-lbl">Побеждены</span>{won.length ? <Pairs list={won} /> : <span className="tr-dim">Пока нет</span>}</div>
      </div>
      {(st.streak > 0 || day) && (
        <div className="tr-row">
          <Streak n={st.streak} />
          {day && <span className="tr-dim">В тренажёре: {Math.round(day.accuracy * 100)} % точность, {day.cpm} зн/мин</span>}
        </div>
      )}
    </div>
  );
}

function TrainerSettings({ cfg, saveCfg, resetFingers, clearWarm }: { cfg: Config; saveCfg: (c: Config) => void; resetFingers: () => void; clearWarm: () => void }) {
  const [ask, setAsk] = useState(false);
  const [reset, setReset] = useState(false);
  const [cleared, setCleared] = useState(false);
  const t = (k: "count" | "remind") => (v: boolean) => saveCfg({ ...cfg, trainer: { ...cfg.trainer, [k]: v } });
  return (
    <div className="tr-main tr-set">
      <h2 className="tr-h">Настройки</h2>
      <Group label="Напоминание">
        <Row title="Напоминать о занятии" sub="Раз в день, когда возвращаешься к компьютеру"><Toggle on={cfg.trainer.remind} label="Напоминать о занятии" onChange={t("remind")} /></Row>
        <Row title="Пример напоминания" sub="Пришлёт уведомление сейчас">
          <button type="button" className="t-btn s32 g" onClick={() => api.trainerRemindPreview().catch(() => {})}>Показать</button>
        </Row>
      </Group>
      <Group label="Данные">
        <Row title="Считать ошибки печати" sub="Только числа, без текста, на этом компьютере"><Toggle on={cfg.trainer.count} label="Считать ошибки печати" onChange={t("count")} /></Row>
        <Row title="Прогресс постановки" sub={reset ? "Сброшен" : ask ? "Уровни и буквы начнутся заново" : "Уровни и буквы"}>
          {ask ? (
            <span className="tr-row">
              <button type="button" className="t-btn s32 g" onClick={() => setAsk(false)}>Отмена</button>
              <button type="button" className="t-btn s32 p" onClick={() => { resetFingers(); setAsk(false); setReset(true); }}>Сбросить</button>
            </span>
          ) : (
            <button type="button" className="t-btn s32 g" disabled={reset} onClick={() => setAsk(true)}>Сбросить</button>
          )}
        </Row>
        <Row title="Данные разминки" sub={cleared ? "Стёрты" : "Счётчики и история"}>
          <button type="button" className="t-btn s32 g" disabled={cleared} onClick={() => { clearWarm(); setCleared(true); }}>Стереть</button>
        </Row>
      </Group>
    </div>
  );
}

export function Trainer() {
  const [st, setSt] = useState<TrainerState | null>(null);
  const [fs, setFs] = useState<FingersState | null>(null);
  const [cfg, setCfg] = useState<Config | null>(null);
  const [sec, setSec] = useState<Sec | null>(null);
  const [scr, setScr] = useState<Screen>({ k: "start" });
  const reload = useCallback(() => {
    api.trainerState().then(setSt).catch(() => {});
    api.fingersState().then(setFs).catch(() => {});
  }, []);

  useEffect(() => {
    reload();
    api.getConfig().then(setCfg).catch(() => {});
    const offs = [
      listen<Config>("config-changed", (e) => { setCfg(e.payload); reload(); }),
      win.onFocusChanged(({ payload }) => { if (payload) reload(); }),
    ];
    return () => { offs.forEach((p) => p.then((f) => f())); };
  }, [reload]);

  useEffect(() => {
    if (sec === null && fs) setSec(fs.warmOpen ? "warm" : "fingers");
  }, [fs, sec]);

  const saveCfg = (next: Config) => {
    setCfg(next);
    api.setConfig(next).then((c) => { setCfg(c); reload(); }, () => reload());
  };
  const go = () => {
    if (!cfg) return;
    api.trainerMaterial(cfg.trainer.minutes).then((items) => setScr({ k: "ex", items })).catch(() => {});
  };
  const save = useCallback((s: Session) => { api.trainerFinish(s).then(setSt).catch(() => {}); }, []);
  const onEnd = useCallback((r: Run, seconds: number, done: boolean) => {
    const s = session(r, seconds, done);
    setScr({ k: "res", s, misses: r.miss });
    save(s);
  }, [save]);
  const onSave = useCallback((r: Run, seconds: number, done: boolean) => save(session(r, seconds, done)), [save]);
  const onStop = useCallback(() => setScr({ k: "start" }), []);
  const switchTo = (t: Sec) => {
    if (scr.k === "ex") setScr({ k: "start" });
    setSec(t);
  };

  const enable = () => cfg && saveCfg({ ...cfg, trainer: { ...cfg.trainer, count: true } });
  const nav: [Sec, string, string][] = [["fingers", "hand", "Постановка"], ["warm", "target", "Разминка"], ["prog", "chart", "Прогресс"]];

  const body = () => {
    if (!st || !cfg || !fs || !sec) return null;
    if (sec === "fingers") return <Fingers fs={fs} setFs={setFs} reload={reload} />;
    if (sec === "prog") return <Progress st={st} enable={enable} />;
    if (sec === "set") return <TrainerSettings cfg={cfg} saveCfg={saveCfg} resetFingers={() => api.fingersReset().then(setFs).catch(() => {})} clearWarm={() => api.trainerClear().then(setSt).catch(() => {})} />;
    if (!fs.warmOpen) return <WarmLocked fs={fs} toFingers={() => switchTo("fingers")} />;
    if (scr.k === "ex") return <Exercise items={scr.items} minutes={cfg.trainer.minutes} onEnd={onEnd} save={onSave} onStop={onStop} />;
    if (scr.k === "res") return <Result s={scr.s} misses={scr.misses} again={go} progress={() => switchTo("prog")} />;
    return <Start st={st} cfg={cfg} saveCfg={saveCfg} offerSeen={() => api.trainerOfferSeen().then(setSt).catch(() => {})} go={go} />;
  };

  return (
    <div className="tr fc-b">
      <div className="tr-body">
        <nav className="tr-rail" aria-label="Разделы тренажёра">
          {nav.map(([k, icon, l]) => (
            <button key={k} type="button" className="t-row" aria-current={sec === k ? "page" : undefined} onClick={() => switchTo(k)}>
              <Icon name={icon} />{l}{k === "warm" && fs && !fs.warmOpen && <Icon name="lock" className="lk" />}
            </button>
          ))}
          <span className="sp" />
          <button type="button" className="t-row" aria-current={sec === "set" ? "page" : undefined} onClick={() => switchTo("set")}><Icon name="settings" />Настройки</button>
        </nav>
        {body()}
      </div>
    </div>
  );
}
