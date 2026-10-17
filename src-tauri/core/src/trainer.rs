use crate::keymap::Lang;
use crate::stats::{day_number, keep_recent, split_pair, Day};
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
    pub weak: BTreeMap<String, String>,
    pub won: BTreeMap<String, String>,
    pub reminded: String,
    pub offered: bool,
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

pub const WINDOW: i64 = 14;
pub const CLEAN_DAYS: i64 = 7;
pub const WEAK_RATE: f32 = 0.02;
pub const MIN_PRESSES: u32 = 50;
pub const MIN_PRESSES_TRAINER: u32 = 20;
pub const DAY_PRESSES: u32 = 50;
pub const MIN_PAIR_MISSES: u32 = 3;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Pair {
    pub wrong: String,
    pub right: String,
    pub rate: f32,
    pub state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
}

fn ago(today: i64, date: &str) -> Option<i64> {
    day_number(date).map(|d| today - d)
}

pub fn pairs(typing: &Typing, history: &mut History, today: &str) -> Vec<Pair> {
    let Some(t) = day_number(today) else { return Vec::new() };
    let sum = |days: std::ops::Range<i64>| {
        let mut d = Day::default();
        for (date, day) in &typing.days {
            if ago(t, date).is_some_and(|a| days.contains(&a)) {
                d.merge(day);
            }
        }
        for s in &history.sessions {
            if ago(t, &s.date).is_some_and(|a| days.contains(&a)) {
                d.merge(&Day { pairs: s.pairs.clone(), presses: s.presses.clone(), ..Day::default() });
            }
        }
        d
    };
    let long = sum(0..WINDOW);
    let rate = |d: &Day, key: &str| {
        let r = split_pair(key)?.1;
        let p = d.presses.get(&r.to_string()).copied().unwrap_or(0);
        let m = d.pairs.get(key).copied().unwrap_or(0);
        (p > 0 && m > 0).then(|| (m as f32 / p as f32, p))
    };
    let min = if typing.days.is_empty() { MIN_PRESSES_TRAINER } else { MIN_PRESSES };
    let real = |a: i64| typing.days.iter().find(|(d, _)| ago(t, d) == Some(a)).map(|(_, d)| d);
    let clean = |key: &str, right: char| {
        real(0).is_none_or(|d| d.pairs.get(key).copied().unwrap_or(0) == 0)
            && (1..=CLEAN_DAYS).all(|a| {
                real(a).is_some_and(|d| d.presses.get(&right.to_string()).copied().unwrap_or(0) >= DAY_PRESSES && d.pairs.get(key).copied().unwrap_or(0) == 0)
            })
    };
    for key in long.pairs.keys() {
        let weak = if history.won.contains_key(key) {
            (0..CLEAN_DAYS).any(|a| real(a).is_some_and(|d| rate(d, key).is_some_and(|(x, _)| x >= WEAK_RATE)))
        } else {
            rate(&long, key).is_some_and(|(x, p)| p >= min && x >= WEAK_RATE) && long.pairs[key] >= MIN_PAIR_MISSES
        };
        if weak {
            history.won.remove(key);
            history.weak.entry(key.clone()).or_insert_with(|| today.to_string());
        }
    }
    let watched: Vec<String> = history.weak.keys().cloned().collect();
    for key in watched {
        if split_pair(&key).is_some_and(|(_, r)| clean(&key, r)) {
            history.weak.remove(&key);
            history.won.insert(key, today.to_string());
        }
    }
    let rate = |key: &str| rate(&long, key);
    let mut out: Vec<Pair> = history
        .weak
        .keys()
        .filter_map(|key| {
            let (w, r) = split_pair(key)?;
            Some(Pair { wrong: w.to_string(), right: r.to_string(), rate: rate(key).map_or(0.0, |x| x.0), state: "weak", since: None })
        })
        .collect();
    out.sort_by(|a, b| b.rate.total_cmp(&a.rate));
    for (key, since) in &history.won {
        if let Some((w, r)) = split_pair(key) {
            out.push(Pair { wrong: w.to_string(), right: r.to_string(), rate: 0.0, state: "won", since: Some(since.clone()) });
        }
    }
    out
}

pub const DEFAULT_RU: f32 = 0.7;

pub fn ru_share(typing: &Typing, today: &str) -> f32 {
    let Some(t) = day_number(today) else { return DEFAULT_RU };
    let (mut ru, mut all) = (0u32, 0u32);
    for (date, day) in &typing.days {
        if ago(t, date).is_some_and(|a| (0..WINDOW).contains(&a)) {
            for (k, n) in &day.presses {
                all += n;
                if k.chars().all(|c| Lang::Ru.owns_letter(c)) {
                    ru += n;
                }
            }
        }
    }
    if all < MIN_PRESSES { DEFAULT_RU } else { (ru as f32 / all as f32).clamp(0.1, 0.9) }
}

pub fn weak_keys(pairs: &[Pair]) -> Vec<(char, char, f32)> {
    pairs.iter().filter(|p| p.state == "weak").filter_map(|p| Some((p.wrong.chars().next()?, p.right.chars().next()?, p.rate))).collect()
}

pub const REMIND_LOCK_SECS: u64 = 20;
pub const REMIND_IDLE_SECS: u64 = 600;

#[derive(Clone, Copy)]
pub struct Pause {
    pub away_secs: u64,
    pub locked_secs: u64,
    pub fullscreen: bool,
    pub enabled: bool,
}

pub fn did_today(history: &History, today: &str) -> bool {
    history.sessions.iter().any(|s| s.date == today && s.done)
}

pub fn should_remind(history: &History, today: &str, p: &Pause) -> bool {
    let back = p.locked_secs >= REMIND_LOCK_SECS || p.away_secs >= REMIND_IDLE_SECS;
    p.enabled && !p.fullscreen && back && history.reminded != today && !did_today(history, today)
}

pub fn streak(history: &History, today: &str) -> u32 {
    let Some(t) = day_number(today) else { return 0 };
    let done: std::collections::HashSet<i64> = history.sessions.iter().filter(|s| s.done).filter_map(|s| day_number(&s.date)).collect();
    let start = if done.contains(&t) { t } else { t - 1 };
    (0..).take_while(|i| done.contains(&(start - i))).count() as u32
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Week {
    pub week: String,
    pub words: u32,
    pub misses: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DayResult {
    pub date: String,
    pub accuracy: f32,
    pub cpm: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub counting: bool,
    pub remind: bool,
    pub minutes: u8,
    pub first_run: bool,
    pub has_journal: bool,
    pub streak: u32,
    pub today: bool,
    pub weeks: Vec<Week>,
    pub days: Vec<DayResult>,
    pub pairs: Vec<Pair>,
}

pub fn weeks(typing: &Typing) -> Vec<Week> {
    let mut by: BTreeMap<String, Week> = BTreeMap::new();
    for (date, day) in &typing.days {
        let Some(w) = crate::stats::week_start(date) else { continue };
        let e = by.entry(w.clone()).or_insert(Week { week: w, words: 0, misses: 0 });
        e.words += day.words;
        e.misses += day.misses;
    }
    by.into_values().filter(|w| w.words > 0).collect()
}

pub fn days(history: &History) -> Vec<DayResult> {
    let mut by: BTreeMap<String, (f32, u32, u32)> = BTreeMap::new();
    for s in history.sessions.iter().filter(|s| s.done) {
        let e = by.entry(s.date.clone()).or_default();
        e.0 += s.accuracy;
        e.1 += s.cpm;
        e.2 += 1;
    }
    by.into_iter().map(|(date, (a, c, n))| DayResult { date, accuracy: a / n as f32, cpm: c / n }).collect()
}

pub fn finish(history: &mut History, mut session: Session, today: &str) {
    session.date = today.to_string();
    session.accuracy = session.accuracy.clamp(0.0, 1.0);
    history.sessions.push(session);
    history.prune(today);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(n: i64) -> String {
        let base = day_number("2026-10-17").unwrap() - n;
        (2026..=2026)
            .flat_map(|y| (1..=12).flat_map(move |m| (1..=31).map(move |d| format!("{y:04}-{m:02}-{d:02}"))))
            .find(|s| day_number(s) == Some(base))
            .unwrap()
    }

    fn day(o: u32, misses: u32) -> Day {
        Day { words: 300, misses, pairs: if misses > 0 { BTreeMap::from([("р>о".to_string(), misses)]) } else { BTreeMap::new() }, presses: BTreeMap::from([("о".to_string(), o)]) }
    }

    #[test]
    fn weak_then_won_then_weak() {
        let today = "2026-10-17";
        let mut t = Typing::default();
        for n in 1..=3 {
            t.days.insert(date(n), day(100, 3));
        }
        let mut h = History::default();
        let p = pairs(&t, &mut h, today);
        assert_eq!(p.len(), 1);
        assert_eq!((p[0].wrong.as_str(), p[0].right.as_str(), p[0].state), ("р", "о", "weak"));
        assert!((p[0].rate - 0.03).abs() < 1e-4);

        let mut t = Typing::default();
        for n in 8..=10 {
            t.days.insert(date(n), day(100, 5));
        }
        for n in 1..=7 {
            t.days.insert(date(n), day(100, 0));
        }
        let p = pairs(&t, &mut h, today);
        assert_eq!(p.iter().map(|p| p.state).collect::<Vec<_>>(), ["won"]);
        assert_eq!(h.won["р>о"], today);

        t.days.insert(date(1), day(100, 6));
        let p = pairs(&t, &mut h, today);
        assert_eq!(p.iter().map(|p| p.state).collect::<Vec<_>>(), ["weak"]);
        assert!(h.won.is_empty());
    }

    #[test]
    fn won_date_kept_and_slip_today() {
        let today = "2026-10-17";
        let mut t = Typing::default();
        for n in 8..=10 {
            t.days.insert(date(n), day(100, 30));
        }
        for n in 1..=7 {
            t.days.insert(date(n), day(100, 0));
        }
        let mut h = History { weak: BTreeMap::from([("р>о".to_string(), date(10))]), ..History::default() };
        h.won.clear();
        pairs(&t, &mut h, today);
        assert!(h.won.contains_key("р>о"));
        h.won.insert("р>о".to_string(), date(3));
        let p = pairs(&t, &mut h, today);
        assert_eq!((p[0].state, p[0].since.as_deref()), ("won", Some(date(3).as_str())));
        t.days.insert(date(0), day(100, 5));
        let p = pairs(&t, &mut h, today);
        assert_eq!(p.iter().map(|p| p.state).collect::<Vec<_>>(), ["weak"]);
    }

    #[test]
    fn single_miss_is_not_weak() {
        let today = "2026-10-17";
        let s = |m: u32| Session { date: today.into(), done: true, pairs: BTreeMap::from([("и>т".to_string(), m)]), presses: BTreeMap::from([("т".to_string(), 25)]), ..Session::default() };
        let mut h = History { sessions: vec![s(1)], ..History::default() };
        assert!(pairs(&Typing::default(), &mut h, today).is_empty());
        h.sessions.push(s(2));
        assert_eq!(pairs(&Typing::default(), &mut h, today).len(), 1);
    }

    #[test]
    fn remind_streak_weeks() {
        let today = "2026-10-17";
        let mut h = History::default();
        let p = Pause { away_secs: 700, locked_secs: 0, fullscreen: false, enabled: true };
        assert!(should_remind(&h, today, &p));
        assert!(!should_remind(&h, today, &Pause { away_secs: 300, ..p }));
        assert!(should_remind(&h, today, &Pause { away_secs: 0, locked_secs: 25, ..p }));
        assert!(!should_remind(&h, today, &Pause { away_secs: 0, locked_secs: 5, ..p }));
        assert!(!should_remind(&h, today, &Pause { fullscreen: true, ..p }));
        assert!(!should_remind(&h, today, &Pause { enabled: false, ..p }));
        h.reminded = today.into();
        assert!(!should_remind(&h, today, &p));
        h.reminded.clear();
        let s = |n: i64, done: bool| Session { date: date(n), done, accuracy: 0.9, cpm: 200, ..Session::default() };
        h.sessions = vec![s(3, true), s(2, true), s(1, true), s(0, false)];
        assert!(should_remind(&h, today, &p));
        assert_eq!(streak(&h, today), 3);
        finish(&mut h, s(0, true), today);
        assert!(!should_remind(&h, today, &p));
        assert_eq!(streak(&h, today), 4);
        h.sessions.retain(|x| x.date != date(2));
        assert_eq!(streak(&h, today), 2);
        assert_eq!(days(&h).len(), 3);
        let mut t = Typing::default();
        t.days.insert("2026-10-12".into(), Day { words: 100, misses: 4, ..Day::default() });
        t.days.insert("2026-10-17".into(), Day { words: 100, misses: 2, ..Day::default() });
        t.days.insert("2026-10-11".into(), Day { words: 50, misses: 5, ..Day::default() });
        assert_eq!(weeks(&t), [Week { week: "2026-10-05".into(), words: 50, misses: 5 }, Week { week: "2026-10-12".into(), words: 200, misses: 6 }]);
    }

    #[test]
    fn share_and_keys() {
        let today = "2026-10-17";
        let mut t = Typing::default();
        assert_eq!(ru_share(&t, today), DEFAULT_RU);
        t.days.insert(date(1), Day { presses: BTreeMap::from([("о".to_string(), 60), ("e".to_string(), 40)]), ..Day::default() });
        assert!((ru_share(&t, today) - 0.6).abs() < 1e-4);
        t.days.insert(date(1), Day { presses: BTreeMap::from([("e".to_string(), 100)]), ..Day::default() });
        assert!((ru_share(&t, today) - 0.1).abs() < 1e-4);
        let p = [
            Pair { wrong: "б".into(), right: "ю".into(), rate: 0.05, state: "weak", since: None },
            Pair { wrong: "ш".into(), right: "щ".into(), rate: 0.0, state: "won", since: Some(today.into()) },
        ];
        assert_eq!(weak_keys(&p), [('б', 'ю', 0.05)]);
    }

    #[test]
    fn thresholds() {
        let today = "2026-10-17";
        let mut h = History::default();
        let mut t = Typing::default();
        t.days.insert(date(1), day(40, 2));
        assert!(pairs(&t, &mut h, today).is_empty());
        t.days.insert(date(1), day(1000, 2));
        assert!(pairs(&t, &mut h, today).is_empty());
        t.days.insert(date(20), day(1000, 900));
        assert!(pairs(&t, &mut h, today).is_empty());
        let mut h = History { sessions: vec![Session { date: date(0), pairs: BTreeMap::from([("б>ю".to_string(), 3)]), presses: BTreeMap::from([("ю".to_string(), 25)]), ..Session::default() }], ..History::default() };
        let p = pairs(&Typing::default(), &mut h, today);
        assert_eq!((p.len(), p[0].right.as_str()), (1, "ю"));
    }

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
