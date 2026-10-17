# Contracts: команды Tauri и события (004)

Все команды — в процессе Ghbdtn, без сети.

| Команда | Вход | Выход | Назначение |
|---|---|---|---|
| `fingers_state` | — | `FingersState` | карта, текущий уровень, клавиши, серия, разминка открыта |
| `fingers_material` | — | `Item[]` | занятие текущего уровня (research R4) |
| `fingers_finish` | `FingerSession` | `FingersState` | записать занятие, освоение, откат, открыть уровень |
| `fingers_intro_seen` | — | `FingersState` | экран нового уровня показан |
| `fingers_reset` | — | `FingersState` | удалить `fingers.json` (разминка не трогается) |
| `open_trainer_cmd` | — | — | окно хаба на вкладке «Тренажёр» (было: окно `trainer`) |
| `open_settings_cmd` | — | — | окно хаба на последней вкладке |

```ts
type KeyState = { key: string; finger: number; state: "closed" | "working" | "mastered"; days: number;
                  acc?: number; cpm?: number };
type LevelInfo = { n: number; name: string; keys: string; lang: "ru" | "en"; state: "done" | "current" | "closed" };
type FingersState = {
  level: number; levels: LevelInfo[]; keys: KeyState[];
  warmOpen: boolean; streak: number; today: boolean;
  opened?: number;
  intro: boolean;
};
type FingerSession = { seconds: number; done: boolean; accuracy: number; cpm: number;
                       keys: Record<string, { n: number; miss: number; ms: number }> };
```

- `opened` — номер уровня, открытого этим `fingers_finish` (для итога «Открыт уровень N»).
- `intro` — текущий уровень открыт, но его экран ещё не показан.
- Событие `hub-tab` (`"util" | "trainer"`) — оболочка → окно хаба: переключить вкладку.
- Событие `trainer-section` (`"fingers" | "warm"`) — открыть раздел тренажёра (напоминание, строка меню трея).
- `trainer_state` 003 получает поле `warmOpen` (для строки меню и раздела «Разминка»).
