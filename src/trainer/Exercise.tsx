import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import type { Item } from "../api";
import { Kbd } from "../Icon";
import { atBoundary, newRun, press, type Mode, type Run } from "./engine";

export type Done = (run: Run, seconds: number, done: boolean) => void;

export function Exercise({ items, minutes, mode = "plain", onEnd, save, onStop, below, label = "Время разминки" }: {
  items: Item[];
  minutes: number;
  mode?: Mode;
  onEnd: Done;
  save: Done;
  onStop: () => void;
  below?: (r: Run) => ReactNode;
  label?: string;
}) {
  const [first] = useState<Run>(() => newRun(items, mode));
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
    onEnd(run.current, Math.min(elapsed(), total + 60), done);
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
    save(run.current, elapsed(), false);
  }, [save]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (ended.current || e.metaKey || e.ctrlKey !== e.altKey) return;
      if (e.key === "Escape") {
        e.preventDefault();
        ended.current = true;
        if (run.current.ok > 0) save(run.current, elapsed(), false);
        onStop();
        return;
      }
      if (e.key === "Shift") {
        run.current.shift = e.code;
        return;
      }
      if (e.key.length !== 1 && !(e.key === "Backspace" && run.current.bad !== null)) return;
      e.preventDefault();
      if (e.repeat) return;
      if (started.current === null) started.current = performance.now();
      const r = press(run.current, e.key, e.code);
      redraw((n) => n + 1);
      if (r === "end") end(true);
      else if (r === "ok" && elapsed() >= total && atBoundary(run.current)) end(true);
    };
    const onUp = (e: KeyboardEvent) => { if (e.key === "Shift") run.current.shift = null; };
    addEventListener("keydown", onKey);
    addEventListener("keyup", onUp);
    return () => { removeEventListener("keydown", onKey); removeEventListener("keyup", onUp); };
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
      <span className="tr-bar" role="progressbar" aria-label={label} aria-valuemin={0} aria-valuemax={total} aria-valuenow={Math.round(spent)}>
        <i style={{ width: `${(spent / total) * 100}%` }} />
      </span>
      <div className="tr-ex">
        <div className="tr-tape">
          <span className="done">{done}{r.bad !== null && <span className="tr-bad">{r.bad}</span>}</span>
          {cur && (
            <span className="todo">
              {r.pos < cur.text.length ? <>{mark(exp)}{cur.text.slice(r.pos + 1)} {next}</> : <>{mark(" ")}{next}</>}
            </span>
          )}
        </div>
      </div>
      {below?.(r)}
      <div className="tr-hint" aria-live="polite">
        {r.hint && <span className="tr-layout">Переключи раскладку на {r.hint === "ru" ? "RU" : "EN"} <Kbd keys="Win+Пробел" /></span>}
        {r.bad !== null && <span className="tr-layout">Сотри ошибку <Kbd keys="Backspace" /></span>}
      </div>
    </div>
  );
}
