import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { installNow, useShip } from "@stillmvd/tauri-ship";
import { api, plural, type Config, type FingersState, type FlyoutState } from "../api";
import { AppMark, Icon, Mark, Toggle } from "../Icon";
import "./flyout.css";

const FEATURES: [keyof Config["features"], string][] = [["layout", "Раскладка"], ["typos", "Опечатки"], ["replace", "Замены"]];

function statusText(s: FlyoutState) {
  if (s.status === "disabled") return `выключено · ${s.config.hotkeys.toggle}`;
  if (s.status === "working") return "работает";
  switch (s.reason) {
    case "process": return `молчит в ${s.app?.name ?? "этом окне"}`;
    case "password": return "молчит в поле пароля";
    case "fullscreen": return "молчит в полноэкранном окне";
    case "elevated": return "молчит в окне администратора";
    case "noruen": return "нет русской или английской раскладки";
    default: return "молчит: раскладка не RU и не EN";
  }
}

const warm = (streak: number, minutes: number) =>
  streak > 0 ? `${streak} ${plural(streak, "день", "дня", "дней")} подряд` : `${minutes} ${plural(minutes, "минута", "минуты", "минут")} на слабые клавиши`;

export function Flyout() {
  const [s, setS] = useState<FlyoutState | null>(null);
  const [streak, setStreak] = useState(0);
  const [fs, setFs] = useState<FingersState | null>(null);
  const card = useRef<HTMLDivElement>(null);
  const { status } = useShip();
  const ready = status?.phase === "ready" || status?.phase === "installing";
  const load = useCallback(() => {
    api.flyoutState().then(setS).catch(() => {});
    api.trainerState().then((t) => setStreak(t.streak)).catch(() => {});
    api.fingersState().then(setFs).catch(() => {});
  }, []);

  useEffect(() => {
    api.flyoutState().then(setS).catch(() => getCurrentWindow().close());
    api.trainerState().then((t) => setStreak(t.streak)).catch(() => {});
    api.fingersState().then(setFs).catch(() => {});
    const offs = ["status", "config-changed"].map((e) => listen(e, load));
    const esc = (e: KeyboardEvent) => { if (e.key === "Escape") getCurrentWindow().close(); };
    addEventListener("keydown", esc);
    return () => { offs.forEach((p) => p.then((f) => f())); removeEventListener("keydown", esc); };
  }, [load]);

  useLayoutEffect(() => {
    const el = card.current;
    if (!el) return;
    const fit = () => getCurrentWindow().setSize(new LogicalSize(el.offsetWidth + 32, el.offsetHeight + 32)).catch(() => {});
    fit().then(api.flyoutReady);
    const ro = new ResizeObserver(fit);
    ro.observe(el);
    return () => ro.disconnect();
  }, [s !== null]);

  if (!s) return null;
  const cfg = s.config;
  const off = !cfg.enabled;
  const save = (next: Config) => {
    setS({ ...s, config: next });
    api.setConfig(next).then(load, load);
  };
  const appOff = s.app ? cfg.processes.off.includes(s.app.exe) : false;
  const toggleApp = (v: boolean) => {
    if (!s.app) return;
    const exe = s.app.exe;
    const list = v ? [...cfg.processes.off, exe] : cfg.processes.off.filter((p) => p !== exe);
    save({ ...cfg, processes: { ...cfg.processes, off: list } });
  };
  const look = s.status === "disabled" ? "off" : s.status === "silent" ? "mute" : "on";

  return (
    <div className="fl-wrap">
      <div className="fl" ref={card}>
        <div className="fl-head">
          <Mark look={look} ru={s.ru} />
          <span className="fl-title"><b>Retype</b><span>{statusText(s)}</span></span>
          <Toggle on={cfg.enabled} label="Retype" onChange={(v) => save({ ...cfg, enabled: v })} />
        </div>
        <div className="fl-rows">
          {FEATURES.map(([k, l]) => (
            <div key={k} className={`t-setrow${off ? " gb-dis" : ""}`}>
              <span className="tx"><b>{l}</b></span>
              <Toggle on={cfg.features[k]} label={l} onChange={(v) => save({ ...cfg, features: { ...cfg.features, [k]: v } })} />
            </div>
          ))}
        </div>
        {s.app && (
          <div className={`t-setrow${off ? " gb-dis" : ""}`}>
            <span className="tx"><b>Не трогать {s.app.name}</b></span>
            <Toggle on={appOff} label={`Не трогать ${s.app.name}`} onChange={toggleApp} />
          </div>
        )}
        {fs && !fs.warmOpen ? (
          <div className="t-setrow">
            <span className="tx"><b>Постановка</b><span>Уровень {fs.level} · {fs.levels[fs.level - 1]?.name.toLowerCase()}</span></span>
            <button type="button" className="t-btn s32 p" onClick={() => api.openTrainer()}><Icon name="play" />Продолжить</button>
          </div>
        ) : (
          <div className="t-setrow">
            <span className="tx"><b>Разминка</b><span>{warm(streak, cfg.trainer.minutes)}</span></span>
            <button type="button" className="t-btn s32 p" onClick={() => api.openTrainer()}><Icon name="play" />Начать</button>
          </div>
        )}
        {ready ? (
          <div className="fl-foot">
            <span className="fl-up">Готова версия {status?.available?.version}</span>
            <span className="fl-btns">
              <button type="button" className="t-btn s32 p" disabled={status?.phase === "installing"} onClick={() => installNow()}><Icon name="restart" />Перезапустить</button>
              <button type="button" className="t-btn s32 g ib" aria-label="Открыть Retype" title="Открыть Retype" onClick={() => api.openSettings()}><AppMark className="fl-am" /></button>
            </span>
          </div>
        ) : (
          <div className="fl-foot">
            <span>{s.count} {plural(s.count, "исправление", "исправления", "исправлений")} сегодня</span>
            <button type="button" className="t-btn s32 g" onClick={() => api.openSettings()}><AppMark className="fl-am" />Открыть Retype</button>
          </div>
        )}
      </div>
    </div>
  );
}
