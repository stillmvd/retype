import type { CSSProperties } from "react";
import type { LevelInfo } from "../api";
import { KEYS } from "./engine";

type Spec = [code: string, units?: number];

const ROWS: { pad: number; keys: Spec[] }[] = [
  { pad: 0, keys: [["Backquote"], ["Digit1"], ["Digit2"], ["Digit3"], ["Digit4"], ["Digit5"], ["Digit6"], ["Digit7"], ["Digit8"], ["Digit9"], ["Digit0"], ["Minus"], ["Equal"], ["Backspace", 1.9]] },
  { pad: 1.45, keys: [["KeyQ"], ["KeyW"], ["KeyE"], ["KeyR"], ["KeyT"], ["KeyY"], ["KeyU"], ["KeyI"], ["KeyO"], ["KeyP"], ["BracketLeft"], ["BracketRight"]] },
  { pad: 1.75, keys: [["KeyA"], ["KeyS"], ["KeyD"], ["KeyF"], ["KeyG"], ["KeyH"], ["KeyJ"], ["KeyK"], ["KeyL"], ["Semicolon"], ["Quote"]] },
  { pad: 0, keys: [["ShiftLeft", 2.3], ["KeyZ"], ["KeyX"], ["KeyC"], ["KeyV"], ["KeyB"], ["KeyN"], ["KeyM"], ["Comma"], ["Period"], ["Slash"], ["ShiftRight", 2.5]] },
  { pad: 4, keys: [["Space", 6.4]] },
];

const FINGER_CODES = [
  "Backquote Digit1 KeyQ KeyA KeyZ ShiftLeft",
  "Digit2 KeyW KeyS KeyX",
  "Digit3 KeyE KeyD KeyC",
  "Digit4 Digit5 KeyR KeyT KeyF KeyG KeyV KeyB",
  "Digit6 Digit7 KeyY KeyU KeyH KeyJ KeyN KeyM",
  "Digit8 KeyI KeyK Comma",
  "Digit9 KeyO KeyL Period",
  "Digit0 Minus Equal Backspace KeyP BracketLeft BracketRight Semicolon Quote Slash ShiftRight",
].map((s) => s.split(" "));

export const fingerOf = (code: string) => {
  const i = FINGER_CODES.findIndex((codes) => codes.includes(code));
  return i < 0 ? 8 : i;
};

const SIGN_CODES: Record<string, string> = { ".": "Slash", ",": "Slash", "?": "Digit7", "!": "Digit1", "-": "Minus", ":": "Digit6" };

export function codeOf(key: string, lang: "ru" | "en"): string | null {
  if (key === " ") return "Space";
  if (key === "⌫") return "Backspace";
  if (key === "⇧L") return "ShiftLeft";
  if (key === "⇧R") return "ShiftRight";
  if (SIGN_CODES[key]) return SIGN_CODES[key];
  const lo = key.toLowerCase();
  const li = lang === "ru" ? 0 : 1;
  return Object.keys(KEYS).find((c) => KEYS[c][li] === lo) ?? null;
}


function levelOf(levels: LevelInfo[], lang: "ru" | "en"): Record<string, number> {
  const m: Record<string, number> = { Space: 1 };
  for (const l of levels) {
    if (l.n === 14) m.ShiftLeft = m.ShiftRight = 14;
    else if (l.n === 15) m.Backspace = 15;
    else if (l.n === 13) for (const ch of l.keys) m[SIGN_CODES[ch]] = 13;
    else if (l.lang === lang) for (const ch of l.keys) { const c = codeOf(ch, lang); if (c) m[c] = l.n; }
  }
  return m;
}

const LABEL: Record<string, string> = { Backspace: "⌫", ShiftLeft: "Shift", ShiftRight: "Shift", Space: "", Minus: "-", Equal: "=", Backquote: "ё" };

function label(code: string, lang: "ru" | "en") {
  if (code.startsWith("Digit")) return code.slice(5);
  if (code === "Slash") return lang === "ru" ? "." : "/";
  if (code === "Backquote" && lang === "en") return "`";
  if (LABEL[code] !== undefined) return LABEL[code];
  return KEYS[code]?.[lang === "ru" ? 0 : 1] ?? "";
}

const width = (u: number) => `calc(var(--ku) * ${u} + 4px * ${u - 1})`;

export function Keyboard({ levels, cur, lang, on = [], fresh, ku }: { levels: LevelInfo[]; cur: number; lang: "ru" | "en"; on?: string[]; fresh?: number; ku?: number }) {
  const lv = levelOf(levels, lang);
  const style = ku ? ({ "--ku": `${ku}px` } as CSSProperties) : undefined;
  return (
    <div className="pf-kb" style={style} aria-hidden="true">
      {ROWS.map((r, i) => (
        <div key={i} className="pf-kr">
          {r.pad > 0 && <span style={{ flex: "none", width: `calc(var(--ku) * ${r.pad - 1} + 4px * ${r.pad - 2})` }} />}
          {r.keys.map(([code, u = 1]) => {
            const l = lv[code];
            let cls = "pf-k";
            if (l === undefined) cls += " dig";
            else if (l > cur) cls += " off";
            else if (fresh && l === fresh) cls += " new";
            if (on.includes(code)) cls += " on";
            if (code === "KeyF" || code === "KeyJ") cls += " bump";
            return <span key={code} className={cls} style={{ width: width(u), "--f": `var(--f${fingerOf(code)})` } as CSSProperties}>{label(code, lang)}</span>;
          })}
        </div>
      ))}
    </div>
  );
}

const HAND = [34, 44, 50, 44, 26];

export function Hand({ side, on }: { side: "l" | "r"; on: number[] }) {
  const fingers = side === "l"
    ? HAND.map((h, i) => (i === 4 ? { h, f: 8 } : { h, f: i }))
    : [...HAND].reverse().map((h, i) => (i === 0 ? { h, f: 8 } : { h, f: i + 3 }));
  return (
    <span className="pf-hand" aria-hidden="true">
      {fingers.map(({ h, f }, i) => (
        <i key={i} className={f === 8 ? "th" : on.includes(f) ? "on" : undefined} style={{ height: h, "--f": `var(--f${f})` } as CSSProperties} />
      ))}
    </span>
  );
}

export const Hands = ({ on }: { on: number[] }) => (
  <div className="pf-hands"><Hand side="l" on={on} /><Hand side="r" on={on} /></div>
);
