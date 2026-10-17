import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api, plural, type Config, type Item, type Pair, type Session, type TrainerState } from "../api";
import { AppMark, Icon, Kbd } from "../Icon";
import { atBoundary, finger, newRun, press, session, type Run } from "./engine";
import "./trainer.css";

type Screen = { k: "start" } | { k: "ex"; items: Item[] } | { k: "res"; s: Session; misses: number };

const win = getCurrentWindow();

const MONTHS = ["января", "февраля", "марта", "апреля", "мая", "июня", "июля", "августа", "сентября", "октября", "ноября", "декабря"];
const dayMonth = (date: string) => `${+date.slice(8, 10)} ${MONTHS[+date.slice(5, 7) - 1]}`;
const ddmm = (date: string) => `${date.slice(8, 10)}.${date.slice(5, 7)}`;
const dec = (v: number) => v.toFixed(1).replace(".", ",");
const arrow = (key: string) => key.replace(">", " → ");

function Streak({ n }: { n: number }) {
  if (n <= 0) return null;
  return <span className="tr-streak"><Icon name="flame" /><b>{n} {plural(n, "день", "дня", "дней")}</b> подряд</span>;
}

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

function Exercise({ items, minutes, onEnd, save, onStop }: { items: Item[]; minutes: number; onEnd: (s: Session, misses: number) => void; save: (s: Session) => void; onStop: () => void }) {
  const [first] = useState<Run>(() => newRun(items));
  const run = useRef(first);
  const started = useRef<number | null>(null);
  const ended = useRef(false);
  const [, redraw] = useState(0);
  const [left, setLeft] = useState(minutes * 60);
  const total = minutes * 60;

  const elapsed = () => (started.current === null ? 0 : (performance.now() - started.current) / 1000);
  const end = useCallback((done: boolean) => {
    if (ended.current) return;
    ended.current = true;
    onEnd(session(run.current, Math.min(elapsed(), total + 60), done), run.current.miss);
  }, [onEnd, total]);

  useEffect(() => {
    const t = setInterval(() => {
      if (started.current === null || ended.current) return;
      const l = Math.max(0, total - elapsed());
      setLeft(l);
      if (l === 0 && atBoundary(run.current)) end(true);
    }, 200);
    return () => clearInterval(t);
  }, [end, total]);

  useEffect(() => () => {
    if (ended.current || run.current.ok === 0) return;
    ended.current = true;
    save(session(run.current, elapsed(), false));
  }, [save]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (ended.current || e.metaKey || e.ctrlKey !== e.altKey) return;
      if (e.key === "Escape") {
        e.preventDefault();
        ended.current = true;
        if (run.current.ok > 0) save(session(run.current, elapsed(), false));
        onStop();
        return;
      }
      if (e.key.length !== 1) return;
      e.preventDefault();
      if (e.repeat) return;
      if (started.current === null) started.current = performance.now();
      const r = press(run.current, e.key, e.code);
      redraw((n) => n + 1);
      if (r === "end") end(true);
      else if (r === "ok" && elapsed() >= total && atBoundary(run.current)) end(true);
    };
    addEventListener("keydown", onKey);
    return () => removeEventListener("keydown", onKey);
  }, [end, onStop, save, total]);

  const r = run.current;
  const cur = r.tokens[r.i];
  const exp = cur ? (r.pos < cur.text.length ? cur.text[r.pos] : " ") : "";
  const mark = (ch: string) => <span className={r.err ? "tr-ea" : "tr-cur"}>{ch}</span>;
  const done = r.tokens.slice(Math.max(0, r.i - 8), r.i).map((t) => t.text + " ").join("") + (cur?.text.slice(0, r.pos) ?? "");
  const next = r.tokens.slice(r.i + 1, r.i + 12).map((t) => t.text).join(" ");
  const spent = total - left;

  return (
    <div className="tr-main center">
      <span className="tr-bar" role="progressbar" aria-label="Время разминки" aria-valuemin={0} aria-valuemax={total} aria-valuenow={Math.round(spent)}>
        <i style={{ width: `${(spent / total) * 100}%` }} />
      </span>
      <div className="tr-ex">
        <div className="tr-tape">
          <span className="done">{done}</span>
          {cur && (
            <span className="todo">
              {r.pos < cur.text.length ? <>{mark(exp)}{cur.text.slice(r.pos + 1)} {next}</> : <>{mark(" ")}{next}</>}
            </span>
          )}
        </div>
      </div>
      <div className="tr-hint" aria-live="polite">
        {r.hint && <span className="tr-layout">Переключи раскладку на {r.hint === "ru" ? "RU" : "EN"} <Kbd keys="Win+Пробел" /></span>}
      </div>
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

export function Trainer() {
  const [st, setSt] = useState<TrainerState | null>(null);
  const [cfg, setCfg] = useState<Config | null>(null);
  const [tab, setTab] = useState<"warm" | "prog">("warm");
  const [scr, setScr] = useState<Screen>({ k: "start" });
  const reload = useCallback(() => { api.trainerState().then(setSt).catch(() => {}); }, []);

  useEffect(() => {
    reload();
    api.getConfig().then(setCfg).catch(() => {});
    const offs = [
      listen<Config>("config-changed", (e) => { setCfg(e.payload); reload(); }),
      win.onFocusChanged(({ payload }) => { if (payload) reload(); }),
    ];
    return () => { offs.forEach((p) => p.then((f) => f())); };
  }, [reload]);

  const saveCfg = (next: Config) => {
    setCfg(next);
    api.setConfig(next).then((c) => { setCfg(c); reload(); }, () => reload());
  };
  const go = () => {
    if (!cfg) return;
    api.trainerMaterial(cfg.trainer.minutes).then((items) => setScr({ k: "ex", items })).catch(() => {});
  };
  const save = useCallback((s: Session) => { api.trainerFinish(s).then(setSt).catch(() => {}); }, []);
  const onEnd = useCallback((s: Session, misses: number) => {
    setScr({ k: "res", s, misses });
    save(s);
  }, [save]);
  const onStop = useCallback(() => setScr({ k: "start" }), []);
  const switchTab = (t: "warm" | "prog") => {
    if (scr.k === "ex") setScr({ k: "start" });
    setTab(t);
  };

  const enable = () => cfg && saveCfg({ ...cfg, trainer: { ...cfg.trainer, count: true } });

  return (
    <div className="tr">
      <div className="t-titlebar" data-tauri-drag-region>
        <AppMark className="tr-mk" />
        <span className="name" data-tauri-drag-region>Тренажёр</span>
        <span className="sp" data-tauri-drag-region />
        <div className="t-win">
          <button type="button" aria-label="Свернуть" onClick={() => win.minimize()}><Icon name="min" /></button>
          <button type="button" className="close" aria-label="Закрыть" onClick={() => win.close()}><Icon name="x" /></button>
        </div>
      </div>
      <div className="tr-body">
        <nav className="tr-rail" aria-label="Разделы">
          <button type="button" className="t-row" aria-current={tab === "warm" ? "page" : undefined} onClick={() => switchTab("warm")}><Icon name="target" />Разминка</button>
          <button type="button" className="t-row" aria-current={tab === "prog" ? "page" : undefined} onClick={() => switchTab("prog")}><Icon name="chart" />Прогресс</button>
        </nav>
        {st && cfg && (tab === "prog" ? <Progress st={st} enable={enable} />
          : scr.k === "ex" ? <Exercise items={scr.items} minutes={cfg.trainer.minutes} onEnd={onEnd} save={save} onStop={onStop} />
          : scr.k === "res" ? <Result s={scr.s} misses={scr.misses} again={go} progress={() => switchTab("prog")} />
          : <Start st={st} cfg={cfg} saveCfg={saveCfg} offerSeen={() => api.trainerOfferSeen().then(setSt).catch(() => {})} go={go} />)}
      </div>
    </div>
  );
}
