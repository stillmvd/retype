use crate::stats::{keep_recent, Day};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const KEEP_DAYS: i64 = 365;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Typing {
    pub days: BTreeMap<String, Day>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Session {
    pub date: String,
    pub seconds: u32,
    pub done: bool,
    pub accuracy: f32,
    pub cpm: u32,
    pub pairs: BTreeMap<String, u32>,
    pub presses: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct History {
    pub sessions: Vec<Session>,
    pub won: BTreeMap<String, String>,
    pub reminded: String,
}

impl Typing {
    pub fn add(&mut self, date: &str, day: &Day) {
        if !day.is_empty() {
            self.days.entry(date.to_string()).or_default().merge(day);
        }
        keep_recent(&mut self.days, date, KEEP_DAYS);
    }
}

impl History {
    pub fn prune(&mut self, today: &str) {
        let mut dated: BTreeMap<String, ()> = self.sessions.iter().map(|s| (s.date.clone(), ())).collect();
        keep_recent(&mut dated, today, KEEP_DAYS);
        self.sessions.retain(|s| dated.contains_key(&s.date));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_add_and_prune() {
        let mut t = Typing::default();
        t.add("2025-01-01", &Day { words: 5, ..Day::default() });
        t.add("2026-10-17", &Day { words: 3, ..Day::default() });
        t.add("2026-10-17", &Day { words: 2, ..Day::default() });
        t.add("2026-10-17", &Day::default());
        assert_eq!(t.days.len(), 1);
        assert_eq!(t.days["2026-10-17"].words, 5);
        let mut h = History { sessions: vec![Session { date: "2025-01-01".into(), ..Session::default() }, Session { date: "2026-10-16".into(), ..Session::default() }], ..History::default() };
        h.prune("2026-10-17");
        assert_eq!(h.sessions.len(), 1);
    }
}
