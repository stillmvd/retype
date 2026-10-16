# Implementation Plan: Тренажёр печати из собственных ошибок

**Branch**: `main` | **Date**: 2026-10-17 | **Spec**: [spec.md](spec.md)
**Input**: `specs/003-trainer/spec.md`, бриф `.planning/TRAINER-BRIEF.md`

## Summary

Окно «Тренажёр» по требованию из трея: разминка 3/5 мин из своих слабых пар клавиш, реальных опечаток журнала и
фраз ru/en; стоп на неверной букве, палец только для слабых клавиш. Ядро по включённому тумблеру считает в памяти
слова, промахи и пары «стёрто → набрано» без текста; оболочка сливает числа в `typing.json`. Прогресс — ошибки на
100 слов по неделям, серия, побеждённые пары. Напоминание — уведомление Windows в первую паузу дня.

## Technical Context

**Language/Version**: Rust (workspace `src-tauri`, крейт `ghbdtn-core`), TypeScript + React 19
**Primary Dependencies**: `windows` (+ `UI_Notifications`, `Data_Xml_Dom` для уведомления), Tauri 2, `serde_json`
**Storage**: `config.toml` (`trainer`), `typing.json`, `trainer.json` в `%APPDATA%\Ghbdtn`; фразы `core/data/*.phrases`
**Testing**: `cargo test` (юнит `stats.rs`, `trainer.rs`; корпус без изменений), `pnpm check`, стенд `rivals`
**Target Platform**: Windows 11 x64
**Performance Goals**: окно ≤ 1 с; материал ≤ 50 мс; счётчики +≤ 1 мс к исправлению; колбэк хука не меняется
**Constraints**: конституция I (только числа, по явному включению), II (работа в рабочем потоке), окна по
требованию, UI — Kockers DS, вид окна — стенд 004 до кода
**Scale/Scope**: 2 новых модуля ядра, команды оболочки, уведомление, окно React, раздел настроек

## Constitution Check

| Принцип | Как соблюдаем |
|---|---|
| I. Приватность | счётчики выкл. по умолчанию; на диске только числа и пары букв; слова журнала читаются только в память окна; фразы — встроенные |
| II. Мгновенный хук | колбэк не меняется; подсчёт в `Worker` (рабочий поток); мьютекс `Tally` берётся только там и в оболочке раз в минуту |
| III. Не ломать ввод | решения ядра не меняются; в своих окнах ядро молчит (`Own`) |
| IV. Отменяемость | не затронута; отмена ядра не считается промахом человека дважды (R2) |
| V. Исключения | «не трогать», пароли, полный экран, повышенные права — счётчики стоят; напоминание не в полном экране |

Отступлений нет. Сеть не используется (фразы Tatoeba скачиваются один раз при разработке, лицензия в «О программе»).

## Project Structure

```text
specs/003-trainer/          spec, plan, research, data-model, contracts/commands.md, quickstart, tasks
src-tauri/core/src/         stats.rs (новый: выравнивание и Tally), trainer.rs (новый: пары, материал),
                            config.rs (Trainer), win/mod.rs (Own, вызовы stats), win/context.rs (свой процесс)
src-tauri/core/data/        ru.phrases, en.phrases (новые)
src-tauri/core/examples/    phrases.rs (сборка фраз из выгрузки Tatoeba)
src-tauri/src/              trainer.rs (новый: файлы, команды, слив Tally, напоминание), notify.rs (новый: WinRT),
                            lib.rs (окно trainer, команды), tray.rs
src/                        trainer/Trainer.tsx (+ css), api.ts, main.tsx, flyout (пункт), settings (раздел)
.planning/sketches/004-trainer/   стенд вида окна
```

## Подход по фазам

1. **Ядро: тишина в своих окнах и подсчёт** — `SilentReason::Own`; `stats.rs` (чистый автомат + тесты: замена,
   пропуск, лишняя, сброс, передумал, исправление ядром); вызовы из `Worker::key/back/finish/reset` и `Acted`;
   `Config.trainer`; `Tally` в `Arc<Mutex>` из `Engine`.
2. **Оболочка: хранение и команды** — `typing.json`/`trainer.json`, слив раз в минуту и при выходе, обрезка года,
   `trainer_state/finish/clear`, `open_trainer_cmd`, окно `trainer` по требованию.
3. **Материал** — `trainer.rs`: частые слова из `*.freq`, слабые/побеждённые пары (тесты на синтетике), слова
   журнала (фильтр), фразы Tatoeba (`examples/phrases.rs` → `data/*.phrases`, лицензия), смесь и доля ru/en;
   `trainer_material`.
4. **Напоминание** — простой по `GetLastInputInfo`, правило «первая пауза дня», WinRT-уведомление с кнопкой,
   `Activated` → окно; AUMID в dev.
5. **Стенд 004** (развилка с пользователем) — экраны: старт (предложение счёта), разминка (строка, курсор, палец,
   подсказка раскладки, таймер), итог, прогресс; выбор пользователя.
6. **UI** — окно по выбранному стенду, пункт трея «Тренажёр», раздел настроек «Тренажёр» (тумблеры, «Стереть»).
7. **Приёмка** — quickstart вручную, `rivals` (SC-004, с согласия), проверка диска (SC-005), релиз — с согласия.

## Complexity Tracking

Нет.
