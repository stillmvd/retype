# Contracts: команды Tauri и события (003)

Все команды — в процессе Ghbdtn, без сети.

| Команда | Вход | Выход | Назначение |
|---|---|---|---|
| `open_trainer_cmd` | — | — | создать/показать окно `trainer` (как `open_settings_cmd`) |
| `trainer_state` | — | `TrainerState` | всё для стартового экрана и прогресса |
| `trainer_material` | `{ minutes: 3 \| 5 }` | `Item[]` | упражнение (R4) |
| `trainer_finish` | `Session` | `TrainerState` | записать разминку, пересчитать пары и серию |
| `trainer_clear` | — | `TrainerState` | стереть `typing.json` и `trainer.json` |

```ts
type Pair = { wrong: string; right: string; rate: number; state: "weak" | "won"; since?: string };
type TrainerState = {
  counting: boolean; remind: boolean; minutes: 3 | 5;
  firstRun: boolean;
  hasJournal: boolean;
  streak: number; today: boolean;
  weeks: { week: string; words: number; misses: number }[];
  days: { date: string; accuracy: number; cpm: number }[];
  pairs: Pair[];
};
type Item = { text: string; lang: "ru" | "en"; kind: "weak" | "journal" | "phrase" | "common"; focus: string[] };
type Session = { seconds: number; done: boolean; accuracy: number; cpm: number;
                 pairs: Record<string, number>; presses: Record<string, number> };
```

- Тумблеры `count` / `remind` / `minutes` меняются через существующий `set_config` (`Config.trainer`).
- Уведомление «Размяться» → `open_trainer_cmd` в оболочке.
- Меню трея: пункт «Тренажёр» → `open_trainer_cmd`.
