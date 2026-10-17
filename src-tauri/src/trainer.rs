use crate::journal::Journal;
use ghbdtn_core::config::Config;
use ghbdtn_core::material::{self, Item};
use ghbdtn_core::trainer::{self, History, Pause, Session, State, Typing};
use ghbdtn_core::win::Engine;
use serde::{de::DeserializeOwned, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use windows::Win32::System::StationsAndDesktops::{CloseDesktop, OpenInputDesktop, DESKTOP_CONTROL_FLAGS, DESKTOP_SWITCHDESKTOP};
use windows::Win32::System::SystemInformation::{GetLocalTime, GetTickCount};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

pub fn today() -> String {
    let now = unsafe { GetLocalTime() };
    format!("{:04}-{:02}-{:02}", now.wYear, now.wMonth, now.wDay)
}

pub fn idle_secs() -> Option<u64> {
    let mut info = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
    unsafe { GetLastInputInfo(&mut info) }.as_bool().then(|| (unsafe { GetTickCount() }.wrapping_sub(info.dwTime) / 1000) as u64)
}

pub fn locked() -> bool {
    match unsafe { OpenInputDesktop(DESKTOP_CONTROL_FLAGS(0), false, DESKTOP_SWITCHDESKTOP) } {
        Ok(h) => {
            let _ = unsafe { CloseDesktop(h) };
            false
        }
        Err(_) => true,
    }
}

fn path(name: &str) -> PathBuf {
    Config::default_path().with_file_name(name)
}

fn read<T: DeserializeOwned + Default>(p: &Path) -> T {
    std::fs::read_to_string(p).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

fn write<T: Serialize>(p: &Path, value: &T) -> std::io::Result<()> {
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let text = serde_json::to_string(value).map_err(std::io::Error::other)?;
    let tmp = p.with_extension("json.tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(tmp, p)
}

pub struct Store {
    typing: PathBuf,
    history: PathBuf,
    io: Mutex<()>,
}

impl Store {
    pub fn new() -> Store {
        Store { typing: path("typing.json"), history: path("trainer.json"), io: Mutex::new(()) }
    }

    pub fn flush(&self, engine: &Engine) {
        let _io = self.io.lock().unwrap();
        let day = engine.take_tally();
        if !engine.config().trainer.count || day.is_empty() {
            return;
        }
        let mut t = self.typing();
        t.add(&today(), &day);
        let _ = self.save_typing(&t);
    }

    fn typing(&self) -> Typing {
        read(&self.typing)
    }

    fn save_typing(&self, t: &Typing) -> std::io::Result<()> {
        write(&self.typing, t)
    }

    fn history(&self) -> History {
        read(&self.history)
    }

    fn save_history(&self, h: &History) -> std::io::Result<()> {
        write(&self.history, h)
    }

    fn build(&self, engine: &Engine, history: &mut History, date: &str) -> State {
        let cfg = engine.config();
        let typing = self.typing();
        let before = history.clone();
        let pairs = trainer::pairs(&typing, history, date);
        if *history != before {
            let _ = self.save_history(history);
        }
        State {
            counting: cfg.trainer.count,
            remind: cfg.trainer.remind,
            minutes: cfg.trainer.minutes,
            first_run: !cfg.trainer.count && !history.offered,
            has_journal: cfg.journal,
            streak: trainer::streak(history, date),
            today: trainer::did_today(history, date),
            weeks: trainer::weeks(&typing),
            days: trainer::days(history),
            pairs,
        }
    }

    pub fn state(&self, engine: &Engine) -> State {
        self.flush(engine);
        let _io = self.io.lock().unwrap();
        let mut history = self.history();
        self.build(engine, &mut history, &today())
    }

    pub fn finish(&self, engine: &Engine, session: Session) -> State {
        self.flush(engine);
        let _io = self.io.lock().unwrap();
        let date = today();
        let mut history = self.history();
        trainer::finish(&mut history, session, &date);
        let _ = self.save_history(&history);
        self.build(engine, &mut history, &date)
    }

    pub fn offer_seen(&self, engine: &Engine) -> State {
        let _io = self.io.lock().unwrap();
        let mut history = self.history();
        if !history.offered {
            history.offered = true;
            let _ = self.save_history(&history);
        }
        self.build(engine, &mut history, &today())
    }

    pub fn remind(&self, engine: &Engine, away_secs: u64, locked_secs: u64) -> Option<String> {
        let cfg = engine.config();
        if !cfg.trainer.remind {
            return None;
        }
        self.flush(engine);
        let _io = self.io.lock().unwrap();
        let date = today();
        let history = self.history();
        let pause = Pause { away_secs, locked_secs, fullscreen: ghbdtn_core::win::context::current().fullscreen, enabled: true };
        if !trainer::should_remind(&history, &date, &pause) {
            return None;
        }
        Some(self.text(engine, history, &date))
    }

    pub fn preview(&self, engine: &Engine) -> String {
        let _io = self.io.lock().unwrap();
        self.text(engine, self.history(), &today())
    }

    fn text(&self, engine: &Engine, mut history: History, date: &str) -> String {
        let weak = trainer::weak_keys(&trainer::pairs(&self.typing(), &mut history, date));
        reminder_text(engine.config().trainer.minutes, &weak)
    }

    pub fn reminded(&self) {
        let _io = self.io.lock().unwrap();
        let mut history = self.history();
        history.reminded = today();
        let _ = self.save_history(&history);
    }

    pub fn material(&self, engine: &Engine, journal: &Journal, minutes: u8) -> Vec<Item> {
        let minutes = if minutes == 5 { 5 } else { 3 };
        self.flush(engine);
        let _io = self.io.lock().unwrap();
        let date = today();
        let typing = self.typing();
        let mut history = self.history();
        let pairs = trainer::pairs(&typing, &mut history, &date);
        let _ = self.save_history(&history);
        let weak = trainer::weak_keys(&pairs);
        let words = if engine.config().journal { material::journal_words(&journal.typo_words()) } else { Vec::new() };
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64);
        material::build(&material::Sources { weak: &weak, journal: &words, ru_share: trainer::ru_share(&typing, &date), minutes, seed })
    }

    pub fn clear(&self, engine: &Engine) -> State {
        let _io = self.io.lock().unwrap();
        engine.take_tally();
        let _ = std::fs::remove_file(&self.typing);
        let _ = std::fs::remove_file(&self.history);
        self.build(engine, &mut History::default(), &today())
    }
}

fn reminder_text(minutes: u8, weak: &[(char, char, f32)]) -> String {
    let len = if minutes == 5 { "5 минут" } else { "3 минуты" };
    if weak.is_empty() {
        return format!("{len} разминки печати.");
    }
    let keys: Vec<String> = weak.iter().take(2).map(|(w, r, _)| format!("{w} → {r}")).collect();
    format!("{len} на твои слабые клавиши: {}.", keys.join(", "))
}

#[cfg(test)]
mod tests {
    use super::reminder_text;

    #[test]
    fn reminder_lines() {
        assert_eq!(reminder_text(3, &[]), "3 минуты разминки печати.");
        assert_eq!(reminder_text(5, &[('б', 'ю', 0.05), ('р', 'о', 0.03), ('ш', 'щ', 0.02)]), "5 минут на твои слабые клавиши: б → ю, р → о.");
    }
}
