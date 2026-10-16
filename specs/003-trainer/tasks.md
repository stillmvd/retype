---
description: "Задачи фичи 003-trainer"
---

# Tasks: Тренажёр печати из собственных ошибок

**Input**: `specs/003-trainer/` — plan.md, spec.md, research.md, data-model.md, contracts/commands.md, quickstart.md
**Tests**: юнит-тесты для чистых модулей ядра (`stats.rs`, `trainer.rs`) — по конституции («логика решений — чистые
функции, покрытые `cargo test`»).
**После каждой фазы**: `pnpm check`, `pnpm build`, корпус; адверсарий (sonnet); локальный коммит; отметка здесь;
`HANDOFF.md`.

## Phase 1: Setup

- [ ] T001 `Config.trainer { count: bool = false, remind: bool = true, minutes: u8 = 3 }` («только 3 или 5» —
  проверка в `Config::validate`), serde default для старых конфигов, тест загрузки старого конфига — в
  `src-tauri/core/src/config.rs`; тип в `src/api.ts`
- [ ] T002 [P] Пример `src-tauri/core/examples/phrases.rs`: из выгрузки Tatoeba (`sentences.csv`, путь аргументом)
  отобрать по ~2000 предложений ru и en длиной 20–80 знаков, только буквы своего алфавита, пробел и `, . ? ! - :`,
  без повторов → `src-tauri/core/data/ru.phrases`, `en.phrases`; строка лицензии (Tatoeba, CC BY 2.0 FR) в списке
  `licenses` команды `about` (`src-tauri/src/lib.rs`) и в `src-tauri/core/data/LICENSES.md`

## Phase 2: Foundational

- [ ] T003 `SilentReason::Own`: `Worker::evaluate` молчит, если окно в фокусе — своего процесса (pid из
  `context::current()` против `GetCurrentProcessId`) — `src-tauri/core/src/win/context.rs`, `win/mod.rs`; в журнал
  и меню трея не выводится (`src-tauri/src/lib.rs::reason/state`)
- [ ] T004 Окно `trainer` по требованию: `open_trainer_cmd` и `open_trainer(app)` по образцу `open_settings`
  (`src-tauri/src/lib.rs`), ветка `label === "trainer"` в `src/main.tsx`, заглушка `src/trainer/Trainer.tsx`,
  `api.openTrainer`; пункт «Тренажёр» в меню трея `src/flyout/Flyout.tsx`
- [ ] T005 Модуль оболочки `src-tauri/src/trainer.rs`: чтение/запись `typing.json` и `trainer.json`
  (`%APPDATA%\Ghbdtn`, формат data-model.md), обрезка дней старше 365 при записи, атомарная запись (tmp + rename)

**Checkpoint**: окно открывается из трея, в нём ядро молчит.

## Phase 3: User Story 2 — Счётчики ошибок реальной печати (P1)

**Goal**: по тумблеру ядро копит числа без текста, оболочка пишет их по дням.
**Independent Test**: quickstart п. 4–5.

- [ ] T006 [US2] Чистый автомат `src-tauri/core/src/stats.rs`: события `letter(c)`, `back`, `word_end`, `reset`,
  `typo_fixed(typed, result)`; выравнивание по R2 (пара только при равных сериях и разных буквах; передумал — не
  промах; сброс теряет серию); итог `Tally { words, misses, pairs, presses }`; тесты: замена, две замены, пропуск,
  лишняя, передумал, сброс, исправление ядром одной заменой и не одной
- [ ] T007 [US2] Подключение в `Worker` (`src-tauri/core/src/win/mod.rs`): `key` (буква → `letter`, Backspace →
  `back`), `finish` (`word_end`; слово не в той раскладке и его исправление не промах), `reset`, `Acted { Typo }` →
  `typo_fixed`; только при `cfg.trainer.count` и статусе `Working`; `Tally` в `Arc<Mutex<Tally>>`, доступ из
  `Engine::tally()`; счёт слов за день для напоминания — всегда, только в памяти
- [ ] T008 [US2] Слив `Tally` в `typing.json` раз в минуту и при выходе (`src-tauri/src/trainer.rs`, поток в
  `src-tauri/src/lib.rs::run`), только при `trainer.count`
- [ ] T009 [US2] `trainer_clear` — удалить оба файла и обнулить `Tally` (`src-tauri/src/trainer.rs`, регистрация в
  `lib.rs`)

**Checkpoint**: счётчики пишутся, пароли/«не трогать»/полный экран не считаются.

## Phase 4: Материал разминки (бэкенд US1)

**Goal**: команда отдаёт упражнение из слабых пар, журнала и фраз.

- [ ] T010 [US1] Чистый модуль `src-tauri/core/src/trainer.rs`: слабые/побеждённые пары по R5 (окно 14 дней, доля
  ≥ 2 %, нажатий ≥ 50 — без счётчиков 20 по тренажёру; победа — 7 дней реальной печати, ≥ 50 нажатий верной в день,
  0 промахов; возврат в слабые); тесты на синтетических днях
- [ ] T011 [US1] Там же — материал по R4: частые слова (первые 5000 строк `ru.freq`/`en.freq`), слова под слабые
  пары, верные формы из журнала (только буквы/дефис, только из частотного списка), фразы из `data/*.phrases`; смесь
  40/25/35, без данных — частые слова и фразы; доля ru/en по словам 14 дней (без счётчиков 70/30); `focus` — верные
  буквы слабых пар; тесты: фильтр журнала (цифры, латиница+кириллица, нет в списке — отсекается), доли, детерминизм по
  зерну; время сборки < 50 мс (release)
- [ ] T012 [US1] Чтение журнала за 30 дней в память (`src-tauri/src/journal.rs`) и команда `trainer_material
  { minutes }` (`src-tauri/src/trainer.rs`), контракт — contracts/commands.md

## Phase 5: Стенд 004 (развилка с пользователем)

- [ ] T013 Стенд `.planning/sketches/004-trainer/` по паспорту `CLAUDE.md` (Kockers DS, тексты настоящие): экраны
  старт (предложение «Считать ошибки печати», длина 3/5, серия), разминка (строка, курсор, отметка ошибки, палец для
  слабой клавиши, подсказка раскладки, таймер), итог, прогресс (линия ошибок на 100 слов по неделям, точность и
  скорость по дням, побеждённые пары), раздел «Тренажёр» в настройках; 2–3 варианта; выбор — `AskUserQuestion`;
  запись в `.planning/UI-STANDS.md`

## Phase 6: User Story 1 — Разминка (P1) 🎯 MVP

**Goal**: разминка проходится в окне.
**Independent Test**: quickstart п. 1–3.

- [ ] T014 [US1] Экран разминки `src/trainer/Trainer.tsx` (+ `trainer.css`) по выбранному стенду: стоп на неверной
  букве, повтор слова с ошибкой позже, таймер 3/5 и дописать текущее слово, подсказка раскладки по
  `KeyboardEvent.key`/`code` без ошибки (R7), палец по `code` только для `focus` (R6), итог (точность, знаков в
  минуту, пары) → `trainer_finish`
- [ ] T015 [US1] `trainer_finish` и `trainer_state` (`src-tauri/src/trainer.rs`): запись разминки (`done`, точность,
  cpm, пары, нажатия), пересчёт пар и `won`; стартовый экран с предложением счёта при `firstRun`

## Phase 7: User Story 3 — Прогресс (P2)

**Goal**: видно, что ошибок стало меньше.
**Independent Test**: quickstart п. 7.

- [ ] T016 [US3] `trainer_state`: недели (`words`, `misses`), дни тренажёра, пары со статусом — `src-tauri/src/trainer.rs`
- [ ] T017 [US3] Экран прогресса по стенду — `src/trainer/Trainer.tsx`

## Phase 8: User Story 4 — Напоминание и серия (P3)

**Goal**: одно напоминание в первую паузу дня, серия дней.
**Independent Test**: quickstart п. 6.

- [ ] T018 [US4] WinRT-уведомление с кнопкой «Размяться» (`src-tauri/src/notify.rs`; features `UI_Notifications`,
  `Data_Xml_Dom` в `src-tauri/Cargo.toml`), `Activated` → `open_trainer`; AUMID в dev
- [ ] T019 [US4] Правило напоминания (R8) в `src-tauri/src/trainer.rs` + поток проверки `GetLastInputInfo` раз в
  30 с: ≥ 100 слов за день, простой ≥ 10 мин, первый ввод после него, сегодня не было разминки и напоминания,
  `trainer.remind`, не полный экран; `reminded` в `trainer.json`; чистая функция решения с тестами
- [ ] T020 [US4] Серия в `trainer_state` (подряд дни с `done`, сегодня без разминки серию не рвёт)

## Phase 9: Polish

- [ ] T021 Раздел «Тренажёр» в настройках `src/settings/Settings.tsx` по стенду: «Считать ошибки печати»,
  «Напоминать о разминке», «Стереть данные тренажёра»
- [ ] T022 Приёмка: quickstart вручную (с пользователем), проверка диска SC-005, стенд `rivals` SC-004 (с согласия),
  `quickstart.md` обновить
- [ ] T023 Релиз `pnpm release` — только с согласия пользователя

## Dependencies & Execution Order

- Setup → Foundational → US2 (счётчики) → материал → стенд → US1 UI → US3 → US4 → Polish.
- US1 работает и без US2 (материал без своих данных), но слабые пары появляются только с US2 или с разминками.
- Стенд (T013) блокирует любые задачи UI (T014, T017, T021); бэкенд-фазы идут до него.
- [P]: T002 параллельно T001; остальное — последовательно (общие `lib.rs`, `win/mod.rs`, `trainer.rs`).

## Implementation Strategy

MVP — Phase 1–6 (окно с разминкой из своих данных и счётчики). Затем прогресс, напоминание, раздел настроек,
приёмка, релиз с согласия.
