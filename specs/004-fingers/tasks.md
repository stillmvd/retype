# Tasks: Постановка пальцев (004)

**Input**: `specs/004-fingers/` — spec, plan, research (R1–R7), data-model, contracts/commands.md, quickstart
**Вид**: стенд 005 — `.planning/UI-STANDS.md`, «005 — расшифровка выбора»; разметка и числа —
`.planning/sketches/005-fingers/parts/e-domain.html`, `parts/g-css.html`
**Тесты**: юнит-тесты ядра (quickstart «Автоматически»); UI — ручная приёмка по quickstart.

Каждая фаза: `pnpm check`, `pnpm build`, адверсарий (`adversary`, sonnet), локальный коммит `feat(core|ui): …`.

## Phase 1: Ядро — уровни, освоение, материал

- [x] T001 Модуль `src-tauri/core/src/fingers.rs` (+ `pub mod fingers` в `src-tauri/core/src/lib.rs`): `LEVELS`
  (15: номер, название, клавиши, ru/en — research R2), `finger(key) -> u8` (0–7 по схеме `FINGERS` из
  `src/trainer/engine.ts`, пробел — 8), `open_keys(level)`.
- [x] T002 Там же: модель `Fingers { level, keys: Map<String, KeyStat { days: Vec<String>, mastered, acc, cpm }>,
  sessions, intro }` (serde, data-model) и `finish(&mut Fingers, session, date) -> Option<u8>`: только `done`;
  клавиша учитывается при `n ≥ 10`; порог `acc ≥ 0.95` и `cpm ≥ 100` (cpm = 60000 / (ms / n)); день в `days`
  (уникальные, последние 2); `mastered` при 2 днях; откат освоенной ниже порога — `mastered = false`, остаётся 1 день;
  уровни 14 и 15 — псевдоклавиши `⇧`/`⌫` по итогу занятия; открыть следующий уровень, если все открытые клавиши
  освоены (возврат — номер); `sessions` старше 365 дней удаляются.
- [x] T003 Там же: `material(f: &Fingers, seed) -> Vec<Item>` по research R4: уровни 1–9 — `ru.freq` (первые 50 000),
  все буквы открыты, длина 2–8 (уровень 1 — от 1), вес «в работе» ×3; 10–12 — `en.freq`, только открытые латинские;
  13 — знаки между словами; 14 — каждое третье слово с заглавной; 15 — русские слова; ~1600 знаков; без одного
  слова дважды подряд. Источники freq — переиспользовать `material::words` (сделать `pub(crate)`).
- [x] T004 Там же: `warm_open(f) = level ≥ 13`, `done_dates(f)` — даты завершённых занятий; `src-tauri/core/src/trainer.rs`:
  `streak` и `did_today` принимают дополнительные даты постановки (вызовы в `src-tauri/src/trainer.rs` обновить).
- [x] T005 Юнит-тесты в `fingers.rs`: открытые клавиши по уровню; освоение только в двух разных днях; < 10 нажатий
  не считается; незавершённое занятие не считается; откат и возврат за один день; открытие уровня целиком и номер;
  `warm_open` после 12; материал: 100 % слов из открытых букв на уровнях 1, 4, 10; доля букв «в работе» ≥ 2×
  освоенных (SC-003); 10–12 — только латиница; 13 — есть знаки; 14 — есть заглавные; серия с днями постановки.

## Phase 2: Оболочка — хранилище и команды

- [x] T006 `src-tauri/src/trainer.rs` (сделано в общем `Store` 003 — общий замок и файлы): хранилище над
  `%APPDATA%\Ghbdtn\fingers.json` (чтение/атомарная запись как `read`/`write` в `src-tauri/src/trainer.rs` — вынести
  их в `pub(crate)`), `state() -> FingersState` (contracts), `material()`, `finish(session)`, `intro_seen()`, `reset()`.
- [x] T007 `src-tauri/src/lib.rs`: команды `fingers_state`, `fingers_material`, `fingers_finish`, `fingers_intro_seen`,
  `fingers_reset`; `S.fingers`; `trainer_state` получает `warmOpen` (поле `warm_open` в `ghbdtn_core::trainer::State`).
- [x] T008 Серия: `Store::build` в `src-tauri/src/trainer.rs` передаёт даты постановки в `streak`/`did_today`.
  Напоминание (`remind`, `preview`): при `!warm_open` — «5 минут постановки: уровень N — <название>.».
- [x] T009 `src/api.ts`: типы `FingersState`, `KeyState`, `LevelInfo`, `FingerSession`, поле `warmOpen` в
  `TrainerState`; методы `fingersState`, `fingersMaterial`, `fingersFinish`, `fingersIntroSeen`, `fingersReset`.

## Phase 3: Хаб «Утилита / Тренажёр» (FR-001a)

- [x] T010 `src/Hub.tsx` (новый): титлбар — `AppMark`, «Ghbdtn», капсула `.hub-seg` по центру (стенд: raised r999,
  padding 3; пункты 26, padding 0 14, 12.5; выбранный cosmic 700 `--sh-press`; `role="tablist"`), `UpdateBadge`, кнопки
  окна; тело — `Settings` или `Trainer`. Вкладка — `localStorage` `ghbdtn:hub` (try/catch), `#trainer` в URL и
  событие `hub-tab` переключают. `src/main.tsx`: метка `settings` → `Hub`.
- [x] T011 `src/settings/Settings.tsx`: убрать свой титлбар (остаётся тело с рейлом и `UpdateToast`), убрать
  `TrainerGroup` из «Основного». `src/trainer/Trainer.tsx`: убрать титлбар; рейл «Постановка / Разминка (замок при
  `!warmOpen`) / Прогресс … Настройки» (стенд, `rail`); событие `trainer-section`.
- [x] T012 `src-tauri/src/lib.rs`: `open_trainer` → окно `settings` (URL `index.html#trainer` для нового, `emit_to`
  `hub-tab` для открытого), заголовок «Ghbdtn», 1040×720; окно `trainer` и его ветки удалить; уведомление и
  single-instance открывают хаб. CSS: `src/settings/settings.css`, `src/trainer/trainer.css` без правил титлбара.
- [x] T013 Меню трея `src/flyout/Flyout.tsx`: подвал — `t-btn s32 g` со знаком «Открыть Ghbdtn» (`open_settings_cmd`);
  строка тренажёра — при `!warmOpen` «Постановка» / «Уровень N · <название>» + `t-btn s32 p` «Продолжить», иначе
  «Разминка» 003.
- [x] T014 Раздел «Настройки» тренажёра (`src/trainer/TrainerSettings.tsx`, новый): «Напоминание» (тумблер
  `trainer.remind`, «Пример напоминания» → «Показать»), «Данные» («Считать ошибки печати», «Прогресс постановки» →
  «Сбросить» с подтверждением в строке, «Данные разминки» → «Стереть»).

## Phase 4: User Story 1 — первое занятие (P1)

**Goal**: карта → уровень → занятие 5 минут → итог. **Test**: quickstart п. 3.

- [x] T015 [US1] `src/trainer/Keyboard.tsx` (новый): клавиатура ЙЦУКЕН/QWERTY по `KROWS` стенда (размер клавиши
  `--ku`), зоны пальцев цветом «Две руки» (`--f0…--f7` как `.fc-b` стенда: левая — акцент 40/60/80/100 % к
  `--raised`, правая — `--acc-horizon` зеркально), закрытые клавиши приглушены, новые отмечены, ожидаемая клавиша
  подсвечена; `Hand`/`Hands` — столбики 14 px (высоты 34/44/50/44/26), нужный палец с кольцом. Стили —
  `src/trainer/fingers.css` (из `parts/g-css.html`: `.pf-kb`, `.pf-kr`, `.pf-hand`).
- [x] T016 [US1] `src/trainer/Fingers.tsx` (новый) — экран «карта»: шапка «Постановка» 26/300 + «уровень N из 15» +
  серия; дорожка (стенд `pathMap`: окно из 8 уровней от текущего −3, сдвиги змейкой, узел 46 / текущий 58 с кольцом и
  «Начать»/«Продолжить», пройденный — акцент и галочка); закрытые — туман (`lk-c`: буквы blur 3.5, название и номер
  2.5, замок поверх номера).
- [x] T017 [US1] Там же — экран «уровень» (стенд `levelScreen` A): «N. **Название**», новые буквы плитками 64 r18
  цветом пальца, клавиатура с новыми клавишами, руки с пальцами уровня, пример слов `--dim`, «Начать занятие».
- [x] T018 [US1] `src/trainer/engine.ts`: статистика по клавишам в `Run` (верные нажатия, первый промах на позиции,
  мс от предыдущего верного) и `fingerSession(run, seconds, done): FingerSession`; существующая разминка не меняется.
- [x] T019 [US1] `Fingers.tsx` — экран «занятие» (стенд `lessonScreen` C): полоса времени и лента 003 (вынести
  `Tape`/`Bar` из `Trainer.tsx` в общий компонент), ряд «рука — клавиатура (`ku` 22) — рука» с gap 14, подсветка
  ожидаемой клавиши и пальца; 5 минут, дописать слово, Esc — стоп (незавершённое сохраняется как `done: false`);
  подсказка раскладки 003.
- [x] T020 [US1] Там же — экран «итог» (стенд `resultScreen` B): «Готово: **X %** · **Y** зн/мин», строки `t-setrow`
  48 по буквам занятия (буква в плашке 30 r9 цветом пальца, палец `--dim`, точность, скорость, состояние «освоена» /
  «день 1 из 2» / «в работе»), «Остальные N букв — X %», «Ещё раз» и «К уровням».

## Phase 5: User Story 2 — открытие уровней (P1)

**Test**: quickstart п. 4.

- [x] T021 [US2] Итог: при `opened` — строка «Открыт уровень N — <название>»; карта переходит на новый текущий;
  «Продолжить» при `intro` показывает экран нового уровня и зовёт `fingersIntroSeen`.

## Phase 6: User Story 4 — сначала постановка (P1)

**Test**: quickstart п. 5, 7.

- [x] T022 [US4] `src/trainer/Trainer.tsx` — «Разминка» при `!warmOpen` (стенд `warmLocked` B): замок 64 r22,
  «Сначала **поставь пальцы**», полоса из 12 сегментов 8 px (пройденные — акцент, текущий — `--gb-soft`),
  «Уровень N из 12 — разминка откроется после английского нижнего ряда», «К постановке». Раздел по умолчанию —
  «Постановка» при `!warmOpen`, иначе «Разминка».
- [x] T023 [US4] Напоминание и строка меню трея ведут в раздел по `warmOpen` (`trainer-section`).

## Phase 7: User Story 3 — английский, знаки, Shift, Backspace (P2)

**Test**: quickstart п. 6.

- [x] T024 [US3] `src/trainer/engine.ts`: режим `shift` (код последнего Shift; заглавная левой руки ждёт `ShiftRight`,
  правой — `ShiftLeft`; не тот — промах псевдоклавиши `⇧`) и режим `backspace` (неверная клавиша печатается
  `--danger`, курсор стоит до Backspace; промах — один на позицию). Клавиатура подсвечивает нужный Shift / Backspace.
- [x] T025 [US3] Уровни 10–12: клавиатура в латинице, подсказка раскладки на EN.

## Phase 8: User Story 5 и приёмка

- [x] T026 [US5] «Прогресс»: серия считает занятия постановки (через `trainer_state`); строка «В тренажёре: …» — по разминкам.
- [x] T027 Узкое окно 720×520: клавиатура уменьшается целиком, лента не перекрывается (edge case).
- [ ] T028 Приёмка по `quickstart.md`; `pnpm check`, `pnpm build`, корпус; `HANDOFF.md`; релиз — с согласия.

## Dependencies

Phase 1 → 2 → 3 → 4 → 5/6/7 (независимы друг от друга) → 8. MVP — фазы 1–4.

## Parallel

T015 и T016–T017 — разные файлы; T024 и T022 — разные файлы.
