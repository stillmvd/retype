import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { UpdateBadge, UpdateToast } from "@stillmvd/tauri-ship";
import { AppMark, Icon } from "./Icon";
import { Settings } from "./settings/Settings";
import { Trainer } from "./trainer/Trainer";
import "./hub.css";

type Tab = "util" | "trainer";

const KEY = "ghbdtn:hub";
const TABS: [Tab, string][] = [["util", "Утилита"], ["trainer", "Тренажёр"]];

function initial(): Tab {
  if (location.hash === "#trainer") return "trainer";
  try {
    return localStorage.getItem(KEY) === "trainer" ? "trainer" : "util";
  } catch {
    return "util";
  }
}

export function Hub() {
  const [tab, setTab] = useState<Tab>(initial);
  const [visit, setVisit] = useState(0);
  const win = getCurrentWindow();

  useEffect(() => {
    try { localStorage.setItem(KEY, tab); } catch { return; }
  }, [tab]);

  useEffect(() => {
    const off = listen<Tab>("hub-tab", (e) => {
      setTab(e.payload === "trainer" ? "trainer" : "util");
      setVisit((n) => n + 1);
    });
    return () => { off.then((f) => f()); };
  }, []);

  return (
    <div className="hub">
      <div className="t-titlebar" data-tauri-drag-region>
        <AppMark className="hub-mk" />
        <span className="name" data-tauri-drag-region>Ghbdtn</span>
        <span className="sp" data-tauri-drag-region />
        <div className="hub-seg" role="tablist" aria-label="Раздел Ghbdtn" onKeyDown={(e) => {
          if (e.key !== "ArrowLeft" && e.key !== "ArrowRight") return;
          const next = tab === "util" ? "trainer" : "util";
          setTab(next);
          (e.currentTarget.querySelector(`[data-tab="${next}"]`) as HTMLElement | null)?.focus();
        }}>
          {TABS.map(([k, l]) => (
            <button key={k} type="button" role="tab" data-tab={k} aria-selected={tab === k} tabIndex={tab === k ? 0 : -1} onClick={() => setTab(k)}>{l}</button>
          ))}
        </div>
        <span className="sp" data-tauri-drag-region />
        <UpdateBadge />
        <div className="t-win">
          <button type="button" aria-label="Свернуть" onClick={() => win.minimize()}><Icon name="min" /></button>
          <button type="button" className="close" aria-label="Закрыть" onClick={() => win.close()}><Icon name="x" /></button>
        </div>
      </div>
      {tab === "util" ? <Settings /> : <Trainer key={visit} />}
      <UpdateToast lang="ru" />
    </div>
  );
}
