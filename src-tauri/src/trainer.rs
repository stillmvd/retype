use ghbdtn_core::config::Config;
use ghbdtn_core::trainer::Typing;
use ghbdtn_core::win::Engine;
use serde::{de::DeserializeOwned, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use windows::Win32::System::SystemInformation::GetLocalTime;

pub fn today() -> String {
    let now = unsafe { GetLocalTime() };
    format!("{:04}-{:02}-{:02}", now.wYear, now.wMonth, now.wDay)
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

    pub fn clear(&self, engine: &Engine) {
        let _io = self.io.lock().unwrap();
        engine.take_tally();
        let _ = std::fs::remove_file(&self.typing);
        let _ = std::fs::remove_file(&self.history);
    }
}
