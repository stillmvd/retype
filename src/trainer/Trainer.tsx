import { getCurrentWindow } from "@tauri-apps/api/window";

export function Trainer() {
  return (
    <div data-tauri-drag-region style={{ padding: 24 }}>
      <b>Тренажёр</b>
      <button type="button" className="t-btn s32 g" onClick={() => getCurrentWindow().close()}>Закрыть</button>
    </div>
  );
}
