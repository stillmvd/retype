import type { Item, Session } from "../api";

const KEYS: Record<string, [string, string]> = {
  Backquote: ["ё", "`"], KeyQ: ["й", "q"], KeyW: ["ц", "w"], KeyE: ["у", "e"], KeyR: ["к", "r"], KeyT: ["е", "t"],
  KeyY: ["н", "y"], KeyU: ["г", "u"], KeyI: ["ш", "i"], KeyO: ["щ", "o"], KeyP: ["з", "p"], BracketLeft: ["х", "["],
  BracketRight: ["ъ", "]"], KeyA: ["ф", "a"], KeyS: ["ы", "s"], KeyD: ["в", "d"], KeyF: ["а", "f"], KeyG: ["п", "g"],
  KeyH: ["р", "h"], KeyJ: ["о", "j"], KeyK: ["л", "k"], KeyL: ["д", "l"], Semicolon: ["ж", ";"], Quote: ["э", "'"],
  KeyZ: ["я", "z"], KeyX: ["ч", "x"], KeyC: ["с", "c"], KeyV: ["м", "v"], KeyB: ["и", "b"], KeyN: ["т", "n"],
  KeyM: ["ь", "m"], Comma: ["б", ","], Period: ["ю", "."],
};

const FINGERS: [string, string[]][] = [
  ["мизинец левой", ["Backquote", "KeyQ", "KeyA", "KeyZ"]],
  ["безымянный левой", ["KeyW", "KeyS", "KeyX"]],
  ["средний левой", ["KeyE", "KeyD", "KeyC"]],
  ["указательный левой", ["KeyR", "KeyT", "KeyF", "KeyG", "KeyV", "KeyB"]],
  ["указательный правой", ["KeyY", "KeyU", "KeyH", "KeyJ", "KeyN", "KeyM"]],
  ["средний правой", ["KeyI", "KeyK", "Comma"]],
  ["безымянный правой", ["KeyO", "KeyL", "Period"]],
  ["мизинец правой", ["KeyP", "BracketLeft", "BracketRight", "Semicolon", "Quote"]],
];

const CYR = /^[а-яё]$/i;
const LAT = /^[a-z]$/i;
const REPEAT_AFTER = 4;
const MIN_CPM_SECS = 5;

export function finger(ch: string): { name: string; index: number } | null {
  const lo = ch.toLowerCase();
  const code = Object.keys(KEYS).find((c) => KEYS[c].includes(lo) && (CYR.test(lo) || LAT.test(lo)));
  const index = code ? FINGERS.findIndex(([, codes]) => codes.includes(code)) : -1;
  return index < 0 ? null : { name: FINGERS[index][0], index };
}

export function wrongLayout(key: string, code: string, exp: string): "ru" | "en" | null {
  const k = KEYS[code];
  const lo = exp.toLowerCase();
  if (CYR.test(lo) && (LAT.test(key) || (k?.[0] === lo && !CYR.test(key)))) return "ru";
  if (LAT.test(lo) && (CYR.test(key) || (k?.[1] === lo && !LAT.test(key)))) return "en";
  return null;
}

const same = (key: string, exp: string) => key === exp || (exp === "ё" && key === "е") || (exp === "Ё" && key === "Е");

export type Token = { text: string; lang: "ru" | "en" };

export type Run = {
  tokens: Token[];
  i: number;
  pos: number;
  err: boolean;
  repeated: boolean;
  hint: "ru" | "en" | null;
  ok: number;
  miss: number;
  pairs: Record<string, number>;
  presses: Record<string, number>;
};

export function newRun(items: Item[]): Run {
  const tokens = items.flatMap((it) => it.text.split(/\s+/).filter(Boolean).map((text) => ({ text, lang: it.lang })));
  return { tokens, i: 0, pos: 0, err: false, repeated: false, hint: null, ok: 0, miss: 0, pairs: {}, presses: {} };
}

export const expected = (r: Run) => {
  const t = r.tokens[r.i];
  if (!t) return null;
  return r.pos < t.text.length ? t.text[r.pos] : " ";
};

export const atBoundary = (r: Run) => !r.tokens[r.i] || r.pos === 0 || r.pos >= r.tokens[r.i].text.length;

export function press(r: Run, key: string, code: string): "ok" | "miss" | "hint" | "end" | null {
  const exp = expected(r);
  if (!exp) return "end";
  if (key.length !== 1) return null;
  if (same(key, exp)) {
    r.ok += 1;
    r.err = false;
    r.hint = null;
    const lo = exp.toLowerCase();
    if (CYR.test(lo) || LAT.test(lo)) r.presses[lo] = (r.presses[lo] ?? 0) + 1;
    if (exp === " ") {
      r.i += 1;
      r.pos = 0;
      r.repeated = false;
      return r.tokens[r.i] ? "ok" : "end";
    }
    r.pos += 1;
    if (r.pos >= r.tokens[r.i].text.length && !r.tokens[r.i + 1]) return "end";
    return "ok";
  }
  const lay = wrongLayout(key, code, exp);
  if (lay) {
    r.hint = lay;
    return "hint";
  }
  r.miss += 1;
  r.err = true;
  r.hint = null;
  const w = key.toLowerCase(), e = exp.toLowerCase();
  if (w !== e && (CYR.test(w) || LAT.test(w)) && (CYR.test(e) || LAT.test(e))) r.pairs[`${w}>${e}`] = (r.pairs[`${w}>${e}`] ?? 0) + 1;
  if (!r.repeated) {
    r.repeated = true;
    r.tokens.splice(Math.min(r.i + 1 + REPEAT_AFTER, r.tokens.length), 0, { ...r.tokens[r.i] });
  }
  return "miss";
}

export function session(r: Run, seconds: number, done: boolean): Session {
  const total = r.ok + r.miss;
  return {
    seconds: Math.round(seconds),
    done,
    accuracy: total ? r.ok / total : 1,
    cpm: seconds >= MIN_CPM_SECS ? Math.round((r.ok * 60) / seconds) : 0,
    pairs: r.pairs,
    presses: r.presses,
  };
}
