import { useCallback, useEffect, useState, type CSSProperties } from "react";
import { api, type FingerSession, type FingersState, type Item, type LevelInfo } from "../api";
import { Icon } from "../Icon";
import { expected, fingerSession, shiftFor, type Mode, type Run } from "./engine";
import { Exercise } from "./Exercise";
import { codeOf, fingerOf, Hand, Hands, Keyboard } from "./Keyboard";
import { Streak } from "./Streak";

type Scr = { k: "map" } | { k: "lvl"; items: Item[] } | { k: "ex"; items: Item[] } | { k: "res"; s: FingerSession; opened: number | null };

const FNAME = ["мизинец левой", "безымянный левой", "средний левой", "указательный левой", "указательный правой", "средний правой", "безымянный правой", "мизинец правой", "большие"];
const SNAKE = [0, 40, 70, 40, 0, -30, -50, -30];
const MINUTES = 5;
const SHIFTED = ",?!:";
const fname = (k: string, f: number) => (k === "⇧" ? "мизинец другой руки" : FNAME[f]);

const keysOf = (l: LevelInfo) => (l.n >= 14 ? [l.keys] : [...l.keys]);
const tileOf = (k: string) => (k === "⇧" ? "Shift" : k === "⌫" ? "⌫" : k.toUpperCase());
const fingerKey = (k: string, lang: "ru" | "en") => (k === "⇧" ? 0 : k === "⌫" ? 7 : fingerOf(codeOf(k, lang) ?? ""));
const modeOf = (level: number): Mode => (level >= 15 ? "back" : level >= 14 ? "shift" : "plain");
const fvar = (f: number) => ({ "--f": `var(--f${f})` }) as CSSProperties;

function LevelMap({ fs, open }: { fs: FingersState; open: () => void }) {
  const cur = fs.level;
  const from = Math.max(1, cur - 3);
  const fresh = fs.keys.every((k) => k.cpm === null);
  return (
    <div className="tr-main pf-main">
      <div className="pf-head">
        <h2 className="pf-h">Постановка</h2>
        <span className="pf-dim">уровень {cur} из {fs.levels.length}</span>
        <Streak n={fs.streak} />
      </div>
      <div className="pf-path pf-fade">
        {fs.levels.slice(from - 1, from + 7).map((l) => {
          const st = l.state === "done" ? "done" : l.state === "current" ? "cur" : "pf-lock";
          return (
            <div key={l.n} className={`pf-node ${st}`} style={{ marginLeft: 120 + SNAKE[(l.n - 1) % 8] }} aria-label={st === "pf-lock" ? `Уровень ${l.n} закрыт` : undefined}>
              <span className="pf-dot">{st === "done" ? <Icon name="check" /> : <><span className="num">{l.n}</span>{st === "pf-lock" && <Icon name="lock" />}</>}</span>
              <span className="pf-nt" aria-hidden={st === "pf-lock" || undefined}><b>{l.name}</b><span className="lt">{l.show}</span></span>
              {st === "cur" && <button type="button" className="t-btn s32 p" onClick={open}><Icon name="play" />{fresh ? "Начать" : "Продолжить"}</button>}
            </div>
          );
        })}
      </div>
    </div>
  );
}

function Level({ fs, items, start, back }: { fs: FingersState; items: Item[]; start: () => void; back: () => void }) {
  const l = fs.levels[fs.level - 1];
  const keys = keysOf(l);
  const fingers = [...new Set(keys.map((k) => fingerKey(k, l.lang)))];
  const sample = items.flatMap((i) => i.text.split(" ")).slice(0, 6).join(" ");
  useEffect(() => {
    const esc = (e: KeyboardEvent) => { if (e.key === "Escape") back(); };
    addEventListener("keydown", esc);
    return () => removeEventListener("keydown", esc);
  }, [back]);
  return (
    <div className="tr-main center pf-main">
      <h2 className="pf-h">{l.n}. <b>{l.name}</b></h2>
      <div className="pf-big">{keys.map((k) => <span key={k} style={fvar(fingerKey(k, l.lang))}>{tileOf(k)}</span>)}</div>
      <div className="pf-fit"><Keyboard levels={fs.levels} cur={fs.level} lang={l.lang} fresh={l.n} /></div>
      <Hands on={fingers} />
      {sample && <span className="pf-dim">{sample}</span>}
      <button type="button" className="t-btn p tr-go" onClick={start}><Icon name="play" />Начать занятие</button>
    </div>
  );
}

function Lesson({ fs, items, done, saved, stop }: { fs: FingersState; items: Item[]; done: (s: FingerSession) => void; saved: (f: FingersState) => void; stop: () => void }) {
  const l = fs.levels[fs.level - 1];
  const onEnd = useCallback((r: Run, seconds: number, ok: boolean) => done(fingerSession(r, seconds, ok)), [done]);
  const save = useCallback((r: Run, seconds: number) => { api.fingersFinish(fingerSession(r, seconds, false)).then(saved).catch(() => {}); }, [saved]);
  const below = (r: Run) => {
    const exp = expected(r);
    const codes: string[] = [];
    if (r.bad !== null) codes.push("Backspace");
    else if (exp) {
      const c = codeOf(exp, r.tokens[r.i]?.lang ?? l.lang);
      if (c) codes.push(c);
      if (exp !== exp.toLowerCase()) codes.push(shiftFor(exp));
      else if (c && SHIFTED.includes(exp)) codes.push(fingerOf(c) <= 3 ? "ShiftRight" : "ShiftLeft");
    }
    const on = codes.map(fingerOf);
    return (
      <div className="pf-side">
        <Hand side="l" on={on} />
        <Keyboard levels={fs.levels} cur={fs.level} lang={l.lang} on={codes} ku={28} />
        <Hand side="r" on={on} />
      </div>
    );
  };
  return <Exercise items={items} minutes={MINUTES} mode={modeOf(fs.level)} onEnd={onEnd} save={save} onStop={stop} below={below} label="Время занятия" />;
}

function Result({ fs, s, opened, again, toMap }: { fs: FingersState; s: FingerSession; opened: number | null; again: () => void; toMap: () => void }) {
  const l = fs.levels[(opened ?? fs.level + 1) - 2] ?? fs.levels[fs.level - 1];
  const own = keysOf(l);
  const pct = (n: number, miss: number) => Math.round((n / Math.max(1, n + miss)) * 100);
  const rest = Object.entries(s.keys).filter(([k]) => !own.includes(k) && k !== "⇧");
  const restN = rest.reduce((a, [, c]) => a + c.n, 0);
  const restMiss = rest.reduce((a, [, c]) => a + c.miss, 0);
  const restMastered = rest.every(([k]) => fs.keys.find((x) => x.key === k)?.state === "mastered");
  const opener = opened ? fs.levels[opened - 1] : null;
  return (
    <div className="tr-main center pf-main">
      <h2 className="pf-h">Готово: <b>{Math.round(s.accuracy * 100)} %</b> · <b>{s.cpm}</b> зн/мин</h2>
      {opener && <span className="pf-opened"><Icon name="check" />Открыт уровень {opener.n} — {opener.name}</span>}
      <div className="pf-rows">
        {own.map((k) => {
          const c = k === "⌫" ? null : s.keys[k];
          const ks = fs.keys.find((x) => x.key === k);
          const f = fingerKey(k, l.lang);
          const state = ks?.state === "mastered" ? "освоена" : ks?.days === 1 ? "день 1 из 2" : "в работе";
          return (
            <span key={k} className="t-setrow" style={fvar(f)}>
              <span className="tx"><span className="k">{tileOf(k)}</span><span className="pf-dim">{fname(k, f)}</span></span>
              <span className="r">
                {k === "⌫" || (c && c.ms > 0) ? (
                  <>
                    <span><b>{c ? pct(c.n, c.miss) : Math.round(s.accuracy * 100)} %</b></span>
                    <span><b>{c ? Math.round((60000 * c.n) / c.ms) : s.cpm}</b> зн/мин</span>
                  </>
                ) : <span>не встретилась</span>}
                <span className={`pf-st ${ks?.state === "mastered" ? "ok" : "w"}`}>{state}</span>
              </span>
            </span>
          );
        })}
      </div>
      {rest.length > 0 && <span className="pf-rest">Остальные {rest.length} — {pct(restN, restMiss)} %{restMastered ? ", все освоены" : ""}</span>}
      <div className="tr-row c">
        <button type="button" className="t-btn p" onClick={again}><Icon name="restart" />Ещё раз</button>
        <button type="button" className="t-btn g" onClick={toMap}><Icon name="hand" />К уровням</button>
      </div>
    </div>
  );
}

export function Fingers({ fs, setFs, reload }: { fs: FingersState; setFs: (f: FingersState) => void; reload: () => void }) {
  const [scr, setScr] = useState<Scr>({ k: "map" });
  const toMap = useCallback(() => setScr({ k: "map" }), []);
  const saved = useCallback((next: FingersState) => { setFs(next); reload(); }, [setFs, reload]);
  const load = (k: "lvl" | "ex") => api.fingersMaterial().then((items) => setScr({ k, items })).catch(() => {});
  const open = () => {
    if (fs.intro) api.fingersIntroSeen().then(setFs).catch(() => {});
    load("lvl");
  };
  const done = useCallback((s: FingerSession) => {
    api.fingersFinish(s).then((next) => {
      setFs(next);
      setScr({ k: "res", s, opened: next.opened });
      reload();
    }).catch(() => setScr({ k: "map" }));
  }, [setFs, reload]);

  if (scr.k === "lvl") return <Level fs={fs} items={scr.items} start={() => setScr({ k: "ex", items: scr.items })} back={toMap} />;
  if (scr.k === "ex") return <Lesson fs={fs} items={scr.items} done={done} saved={saved} stop={toMap} />;
  if (scr.k === "res") return <Result fs={fs} s={scr.s} opened={scr.opened} again={() => (scr.opened ? open() : load("ex"))} toMap={toMap} />;
  return <LevelMap fs={fs} open={open} />;
}
