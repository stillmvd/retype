use crate::material::{Item, Rng};
use crate::stats::keep_recent;
use crate::trainer::KEEP_DAYS;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub struct Level {
    pub n: u8,
    pub name: &'static str,
    pub keys: &'static str,
    pub show: &'static str,
    pub lang: &'static str,
}

const fn lv(n: u8, name: &'static str, keys: &'static str, show: &'static str, lang: &'static str) -> Level {
    Level { n, name, keys, show, lang }
}

pub const LEVELS: [Level; 15] = [
    lv(1, "Домашний ряд", "фываолдж", "ФЫВА ОЛДЖ", "ru"),
    lv(2, "Указательные к центру", "пр", "П Р", "ru"),
    lv(3, "Указательные вверх", "кенг", "К Е Н Г", "ru"),
    lv(4, "Указательные вниз", "мить", "М И Т Ь", "ru"),
    lv(5, "Средние пальцы", "ушсб", "У Ш С Б", "ru"),
    lv(6, "Безымянные", "цщчю", "Ц Щ Ч Ю", "ru"),
    lv(7, "Левый мизинец", "йя", "Й Я", "ru"),
    lv(8, "Правый мизинец", "зхэ", "З Х Э", "ru"),
    lv(9, "Редкие буквы", "ёъ", "Ё Ъ", "ru"),
    lv(10, "Английский: домашний ряд", "asdfghjkl", "A S D F G H J K L", "en"),
    lv(11, "Английский: верхний ряд", "qwertyuiop", "Q W E R T Y U I O P", "en"),
    lv(12, "Английский: нижний ряд", "zxcvbnm", "Z X C V B N M", "en"),
    lv(13, "Знаки", ".,?!-:", ". , ? ! - :", "ru"),
    lv(14, "Shift", SHIFT, "заглавные другой рукой", "ru"),
    lv(15, "Backspace", BACK, "правым мизинцем", "ru"),
];

pub const SHIFT: &str = "⇧";
pub const BACK: &str = "⌫";
pub const WARM_AFTER: u8 = 12;
pub const MIN_PRESSES: u32 = 10;
pub const PASS_ACC: f32 = 0.95;
pub const PASS_CPM: u32 = 100;
pub const LESSON_CHARS: usize = 1600;
const RU_WORDS: &str = include_str!("../data/ru.fingers");
const EN_WORDS: &str = include_str!("../data/en.fingers");
const POOL: usize = 2_000;
const PER_LETTER: usize = 60;
const FOCUS_SHARE: f32 = 0.6;
const MAX_LEN: usize = 8;
const WORKING_WEIGHT: f32 = 3.0;
const CHUNK: usize = 10;

pub fn finger(key: &str) -> u8 {
    const ZONES: [&str; 8] = [
        "ёйфяqaz`1!⇧",
        "цычwsx2",
        "увсedc3",
        "кеапмиrtfgvb45",
        "нгротьyuhjnm67?:",
        "шлбik8",
        "щдюol9",
        "зхъжэp[];'0-=.,⌫",
    ];
    let Some(c) = key.chars().next().map(|c| c.to_lowercase().next().unwrap_or(c)) else { return 8 };
    ZONES.iter().position(|z| z.contains(c)).map_or(8, |i| i as u8)
}

fn level_keys(l: &Level) -> Vec<String> {
    if l.n >= 14 {
        vec![l.keys.to_string()]
    } else {
        l.keys.chars().map(String::from).collect()
    }
}

pub fn open_keys(level: u8) -> Vec<String> {
    LEVELS.iter().take_while(|l| l.n <= level).flat_map(level_keys).collect()
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyStat {
    pub days: Vec<String>,
    pub mastered: bool,
    pub acc: f32,
    pub cpm: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Lesson {
    pub date: String,
    pub seconds: u32,
    pub done: bool,
    pub level: u8,
    pub accuracy: f32,
    pub cpm: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Fingers {
    pub level: u8,
    pub keys: BTreeMap<String, KeyStat>,
    pub sessions: Vec<Lesson>,
    pub intro: u8,
}

impl Default for Fingers {
    fn default() -> Fingers {
        Fingers { level: 1, keys: BTreeMap::new(), sessions: Vec::new(), intro: 0 }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct KeyCount {
    pub n: u32,
    pub miss: u32,
    pub ms: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct FingerSession {
    pub seconds: u32,
    pub done: bool,
    pub accuracy: f32,
    pub cpm: u32,
    pub keys: BTreeMap<String, KeyCount>,
}

fn level(f: &Fingers) -> u8 {
    f.level.clamp(1, LEVELS.len() as u8)
}

fn grade(key: &str, s: &FingerSession) -> Option<(bool, f32, u32)> {
    if key == BACK {
        return Some((s.accuracy >= PASS_ACC && s.cpm >= PASS_CPM, s.accuracy, s.cpm));
    }
    let c = s.keys.get(key)?;
    if c.n < MIN_PRESSES || c.ms == 0 {
        return None;
    }
    let acc = c.n as f32 / (c.n + c.miss) as f32;
    let cpm = (60_000.0 * c.n as f64 / c.ms as f64).round() as u32;
    Some((acc >= PASS_ACC && cpm >= PASS_CPM, acc, cpm))
}

pub fn finish(f: &mut Fingers, s: &FingerSession, today: &str) -> Option<u8> {
    let lv = level(f);
    f.level = lv;
    f.sessions.push(Lesson { date: today.to_string(), seconds: s.seconds, done: s.done, level: lv, accuracy: s.accuracy.clamp(0.0, 1.0), cpm: s.cpm });
    let mut dated: BTreeMap<String, ()> = f.sessions.iter().map(|l| (l.date.clone(), ())).collect();
    keep_recent(&mut dated, today, KEEP_DAYS);
    f.sessions.retain(|l| dated.contains_key(&l.date));
    if !s.done {
        return None;
    }
    let open = open_keys(lv);
    for key in &open {
        let Some((pass, acc, cpm)) = grade(key, s) else { continue };
        let st = f.keys.entry(key.clone()).or_default();
        st.acc = acc;
        st.cpm = cpm;
        if pass {
            if !st.days.iter().any(|d| d == today) {
                st.days.push(today.to_string());
            }
            let keep = st.days.len().saturating_sub(2);
            st.days.drain(..keep);
            st.mastered = st.mastered || st.days.len() >= 2;
        } else if st.mastered {
            st.mastered = false;
            let keep = st.days.len().saturating_sub(1);
            st.days.drain(..keep);
        }
    }
    let all = open.iter().all(|k| f.keys.get(k).is_some_and(|st| st.mastered));
    if all && (lv as usize) < LEVELS.len() {
        f.level = lv + 1;
        return Some(f.level);
    }
    None
}

pub fn warm_open(f: &Fingers) -> bool {
    level(f) > WARM_AFTER
}

pub fn done_dates(f: &Fingers) -> Vec<String> {
    f.sessions.iter().filter(|l| l.done).map(|l| l.date.clone()).collect()
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LevelInfo {
    pub n: u8,
    pub name: &'static str,
    pub keys: &'static str,
    pub show: &'static str,
    pub lang: &'static str,
    pub state: &'static str,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct KeyView {
    pub key: String,
    pub finger: u8,
    pub state: &'static str,
    pub days: u8,
    pub acc: Option<f32>,
    pub cpm: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub level: u8,
    pub levels: Vec<LevelInfo>,
    pub keys: Vec<KeyView>,
    pub warm_open: bool,
    pub intro: bool,
}

pub fn view(f: &Fingers) -> View {
    let lv = level(f);
    let levels = LEVELS
        .iter()
        .map(|l| LevelInfo { n: l.n, name: l.name, keys: l.keys, show: l.show, lang: l.lang, state: if l.n < lv { "done" } else if l.n == lv { "current" } else { "closed" } })
        .collect();
    let keys = open_keys(lv)
        .into_iter()
        .map(|k| {
            let st = f.keys.get(&k);
            KeyView {
                finger: finger(&k),
                state: if st.is_some_and(|s| s.mastered) { "mastered" } else { "working" },
                days: st.map_or(0, |s| s.days.len().min(2) as u8),
                acc: st.filter(|s| s.cpm > 0).map(|s| s.acc),
                cpm: st.filter(|s| s.cpm > 0).map(|s| s.cpm),
                key: k,
            }
        })
        .collect();
    View { level: lv, levels, keys, warm_open: warm_open(f), intro: f.intro < lv }
}

pub fn intro_seen(f: &mut Fingers) {
    f.intro = level(f);
}

pub fn lang(level: u8) -> &'static str {
    LEVELS[level.clamp(1, LEVELS.len() as u8) as usize - 1].lang
}


fn mastered(f: &Fingers, key: &str) -> bool {
    f.keys.get(key).is_some_and(|s| s.mastered)
}

fn weights(f: &Fingers) -> BTreeMap<char, f32> {
    let lv = level(f);
    let en = lang(lv) == "en";
    open_keys(lv)
        .into_iter()
        .filter_map(|k| {
            let c = k.chars().next()?;
            let letter = if en { c.is_ascii_lowercase() } else { ('а'..='я').contains(&c) || c == 'ё' };
            letter.then(|| (c, if mastered(f, &k) { 1.0 } else { WORKING_WEIGHT }))
        })
        .collect()
}

fn pool(f: &Fingers, w: &BTreeMap<char, f32>) -> Vec<(&'static str, f32)> {
    let lv = level(f);
    let min_len = if lv == 1 { 1 } else { 2 };
    let list = if lang(lv) == "en" { EN_WORDS } else { RU_WORDS };
    let fits: Vec<&'static str> = list
        .lines()
        .filter(|word| {
            let n = word.chars().count();
            (min_len..=MAX_LEN).contains(&n) && word.chars().all(|c| w.contains_key(&c))
        })
        .collect();
    let mut picked: Vec<&'static str> = fits.iter().take(POOL).copied().collect();
    for (&c, &weight) in w {
        if weight > 1.0 {
            let have = picked.iter().filter(|word| word.contains(c)).count();
            let more = fits.iter().skip(POOL).filter(|word| word.contains(c)).take(PER_LETTER.saturating_sub(have));
            picked.extend(more);
        }
    }
    picked
        .into_iter()
        .map(|word| {
            let avg = word.chars().map(|c| w[&c]).sum::<f32>() / word.chars().count() as f32;
            (word, avg.powi(6))
        })
        .collect()
}

fn pick<'a>(pool: &[(&'a str, f32)], total: f32, rng: &mut Rng) -> &'a str {
    let mut x = (rng.next() % 1_000_000) as f32 / 1_000_000.0 * total;
    for (word, w) in pool {
        if x < *w {
            return word;
        }
        x -= w;
    }
    pool[pool.len() - 1].0
}

fn capital(word: &str) -> String {
    let mut cs = word.chars();
    cs.next().map_or_else(String::new, |c| c.to_uppercase().chain(cs).collect())
}

fn least<T: Copy + Ord>(items: &[T], counts: &BTreeMap<T, usize>) -> Option<T> {
    items.iter().copied().min_by_key(|c| counts.get(c).copied().unwrap_or(0))
}

pub fn material(f: &Fingers, seed: u64) -> Vec<Item> {
    let lv = level(f);
    let lang = lang(lv);
    let w = weights(f);
    let pool = pool(f, &w);
    if pool.is_empty() {
        return Vec::new();
    }
    let total: f32 = pool.iter().map(|(_, w)| w).sum();
    let working: Vec<char> = w.iter().filter(|(_, &v)| v > 1.0).map(|(&c, _)| c).collect();
    let open_signs: Vec<char> = if lv >= 13 { LEVELS[12].keys.chars().collect() } else { Vec::new() };
    let working_signs: Vec<char> = open_signs.iter().copied().filter(|c| !mastered(f, &c.to_string())).collect();
    let caps = lv >= 14;
    let mut rng = Rng(seed | 1);
    let mut seen: BTreeMap<char, usize> = BTreeMap::new();
    let mut tokens: Vec<String> = Vec::new();
    let mut chars = 0;
    let mut prev = "";
    let mut sentence = caps;
    let mut n = 0usize;
    while chars < LESSON_CHARS {
        let focus = if rng.chance(FOCUS_SHARE) { least(&working, &seen) } else { None };
        let word = match focus {
            Some(c) => {
                let with: Vec<(&str, f32)> = pool.iter().copied().filter(|(word, _)| word.contains(c) && *word != prev).collect();
                if with.is_empty() { pick(&pool, total, &mut rng) } else { pick(&with, with.iter().map(|(_, w)| w).sum(), &mut rng) }
            }
            None => pick(&pool, total, &mut rng),
        };
        if word == prev && pool.len() > 1 {
            continue;
        }
        prev = word;
        for c in word.chars() {
            *seen.entry(c).or_default() += 1;
        }
        n += 1;
        let mut token = if sentence || (caps && n.is_multiple_of(3)) { capital(word) } else { word.to_string() };
        sentence = false;
        let sign = if open_signs.is_empty() || !rng.chance(if working_signs.is_empty() { 0.3 } else { 0.7 }) {
            None
        } else if !working_signs.is_empty() && rng.chance(FOCUS_SHARE) {
            least(&working_signs, &seen)
        } else {
            Some(open_signs[rng.below(open_signs.len())])
        };
        match sign {
            Some('-') => {
                chars += token.chars().count() + 1;
                tokens.push(token);
                token = "-".to_string();
            }
            Some(s) => {
                token.push(s);
                sentence = caps && matches!(s, '.' | '?' | '!');
            }
            None => {}
        }
        if let Some(s) = sign {
            *seen.entry(s).or_default() += 1;
        }
        chars += token.chars().count() + 1;
        tokens.push(token);
    }
    tokens.chunks(CHUNK).map(|c| Item { text: c.join(" "), lang, kind: "common", focus: Vec::new() }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn session(keys: &[&str], n: u32, miss: u32, ms_per: u64) -> FingerSession {
        FingerSession {
            seconds: 300,
            done: true,
            accuracy: n as f32 / (n + miss) as f32,
            cpm: (60_000 / ms_per) as u32,
            keys: keys.iter().map(|k| (k.to_string(), KeyCount { n, miss, ms: ms_per * n as u64 })).collect(),
        }
    }

    fn all(level: u8) -> Vec<String> {
        open_keys(level)
    }

    fn refs(v: &[String]) -> Vec<&str> {
        v.iter().map(String::as_str).collect()
    }

    #[test]
    fn levels_cover_alphabets() {
        let ru: String = LEVELS[..9].iter().map(|l| l.keys).collect();
        assert_eq!(ru.chars().count(), 33);
        let en: String = LEVELS[9..12].iter().map(|l| l.keys).collect();
        assert_eq!(en.chars().count(), 26);
        assert_eq!(open_keys(1).len(), 8);
        assert_eq!(open_keys(2).len(), 10);
        assert!(open_keys(14).contains(&SHIFT.to_string()));
        assert_eq!(finger("ф"), 0);
        assert_eq!(finger("о"), 4);
        assert_eq!(finger("ж"), 7);
        assert_eq!(finger("J"), 4);
        assert_eq!(finger(" "), 8);
    }

    #[test]
    fn mastery_needs_two_days() {
        let mut f = Fingers::default();
        let keys = all(1);
        assert_eq!(finish(&mut f, &session(&refs(&keys), 40, 1, 400), "2026-10-16"), None);
        assert!(!f.keys["ф"].mastered);
        assert_eq!(finish(&mut f, &session(&refs(&keys), 40, 1, 400), "2026-10-16"), None);
        assert!(!f.keys["ф"].mastered);
        assert_eq!(finish(&mut f, &session(&refs(&keys), 40, 1, 400), "2026-10-17"), Some(2));
        assert!(f.keys["ф"].mastered);
        assert_eq!(view(&f).level, 2);
        assert!(view(&f).intro);
    }

    #[test]
    fn thresholds_and_unfinished() {
        let mut f = Fingers::default();
        let keys = all(1);
        let mut s = session(&refs(&keys), 9, 0, 400);
        finish(&mut f, &s, "2026-10-16");
        assert!(f.keys.is_empty());
        s = session(&refs(&keys), 40, 1, 400);
        s.done = false;
        finish(&mut f, &s, "2026-10-16");
        assert!(f.keys.is_empty());
        assert_eq!(f.sessions.len(), 2);
        finish(&mut f, &session(&refs(&keys), 40, 3, 400), "2026-10-16");
        assert!(f.keys["ф"].days.is_empty());
        finish(&mut f, &session(&refs(&keys), 40, 0, 700), "2026-10-17");
        assert!(f.keys["ф"].days.is_empty());
        assert_eq!(f.keys["ф"].cpm, 86);
    }

    #[test]
    fn regression_and_recovery() {
        let mut f = Fingers::default();
        let keys = all(1);
        finish(&mut f, &session(&refs(&keys), 40, 0, 400), "2026-10-15");
        finish(&mut f, &session(&refs(&keys), 40, 0, 400), "2026-10-16");
        assert_eq!(f.level, 2);
        finish(&mut f, &session(&["ф"], 40, 10, 400), "2026-10-17");
        assert!(!f.keys["ф"].mastered);
        assert_eq!(f.keys["ф"].days.len(), 1);
        assert_eq!(f.level, 2);
        finish(&mut f, &session(&["ф"], 40, 0, 400), "2026-10-17");
        assert!(f.keys["ф"].mastered);
        let v = view(&f);
        assert_eq!(v.keys.iter().find(|k| k.key == "ф").unwrap().state, "mastered");
        assert_eq!(v.keys.iter().find(|k| k.key == "п").unwrap().state, "working");
    }

    #[test]
    fn level_opens_whole_and_warm() {
        let mut f = Fingers { level: 12, ..Fingers::default() };
        assert!(!warm_open(&f));
        let keys = all(12);
        finish(&mut f, &session(&refs(&keys), 40, 0, 400), "2026-10-16");
        assert_eq!(finish(&mut f, &session(&refs(&keys), 40, 0, 400), "2026-10-17"), Some(13));
        assert!(warm_open(&f));
        assert_eq!(view(&f).levels[12].state, "current");
        assert_eq!(view(&f).levels[11].state, "done");
        assert_eq!(view(&f).levels[13].state, "closed");
        let mut f = Fingers { level: 15, ..Fingers::default() };
        let keys = all(15);
        finish(&mut f, &session(&refs(&keys), 40, 0, 400), "2026-10-16");
        assert_eq!(finish(&mut f, &session(&refs(&keys), 40, 0, 400), "2026-10-17"), None);
        assert!(f.keys[BACK].mastered);
        assert_eq!(f.level, 15);
    }

    fn letters_of(items: &[Item]) -> String {
        items.iter().map(|i| i.text.as_str()).collect::<Vec<_>>().join(" ")
    }

    fn at(level: u8) -> Fingers {
        let mut f = Fingers { level, ..Fingers::default() };
        for k in open_keys(level - 1) {
            f.keys.insert(k, KeyStat { days: vec!["2026-10-15".into(), "2026-10-16".into()], mastered: true, acc: 1.0, cpm: 150 });
        }
        f
    }

    fn head(text: &str, n: usize) -> String {
        text.chars().take(n).collect()
    }

    #[test]
    fn material_uses_open_letters_only() {
        for lv in [1, 4, 9, 10, 11, 12] {
            let items = material(&at(lv), 7);
            let open: HashSet<char> = open_keys(lv).iter().filter_map(|k| k.chars().next()).filter(|c| (lv < 10) != c.is_ascii()).collect();
            let text = letters_of(&items);
            assert!(text.chars().count() + 1 >= LESSON_CHARS, "level {lv}");
            assert!(text.chars().all(|c| c == ' ' || open.contains(&c)), "level {lv}: {text}");
            assert!(items.iter().all(|i| i.lang == lang(lv)), "level {lv}");
        }
    }

    #[test]
    fn every_working_letter_early_and_often() {
        for lv in 1..=12 {
            let f = at(lv);
            let keys: Vec<char> = LEVELS[lv as usize - 1].keys.chars().collect();
            let mut calm = at(lv + 1);
            calm.level = lv;
            let base = letters_of(&material(&calm, 3));
            let share = |text: &str| text.chars().filter(|x| keys.contains(x)).count() as f32 / text.chars().count() as f32;
            for seed in [1, 7, 42] {
                let full = letters_of(&material(&f, seed));
                let text = head(&full, 700);
                for &c in &keys {
                    let n = text.chars().filter(|x| *x == c).count();
                    assert!(n >= 10, "level {lv} seed {seed}: «{c}» {n} раз за 700 знаков");
                }
                if keys.len() * 2 <= weights(&f).len() {
                    assert!(share(&full) >= 1.3 * share(&base), "level {lv} seed {seed}: {} vs {}", share(&full), share(&base));
                }
            }
        }
    }

    #[test]
    fn signs_and_capitals() {
        for seed in [1, 5, 42] {
            let text = head(&letters_of(&material(&at(13), seed)), 1000);
            for s in LEVELS[12].keys.chars() {
                let n = text.chars().filter(|c| *c == s).count();
                assert!(n >= 10, "seed {seed}: «{s}» {n} раз: {text}");
            }
            assert!(!text.chars().any(char::is_uppercase));
        }
        let text = letters_of(&material(&at(14), 5));
        let words: Vec<&str> = text.split(' ').filter(|w| *w != "-").collect();
        let caps = words.iter().filter(|w| w.chars().next().is_some_and(char::is_uppercase)).count();
        assert!(caps * 3 >= words.len(), "{caps} из {}", words.len());
        assert!(!text.chars().any(|c| c.is_ascii_alphabetic()));
    }

    #[test]
    fn level_out_of_range_and_shift_contract() {
        assert_eq!(view(&Fingers { level: 0, ..Fingers::default() }).level, 1);
        assert_eq!(view(&Fingers { level: 99, ..Fingers::default() }).level, 15);
        let mut f = at(14);
        let letters: Vec<String> = open_keys(13);
        finish(&mut f, &session(&refs(&letters), 40, 0, 400), "2026-10-16");
        assert_eq!(finish(&mut f, &session(&refs(&letters), 40, 0, 400), "2026-10-17"), None);
        assert!(!f.keys.contains_key(SHIFT));
        let mut both = letters.clone();
        both.push(SHIFT.to_string());
        assert_eq!(finish(&mut f, &session(&refs(&both), 40, 0, 400), "2026-10-18"), None);
        assert_eq!(finish(&mut f, &session(&refs(&both), 40, 0, 400), "2026-10-19"), Some(15));
    }

    #[test]
    fn done_dates_for_streak() {
        let mut f = Fingers::default();
        let mut s = session(&["ф"], 40, 0, 400);
        finish(&mut f, &s, "2026-10-16");
        s.done = false;
        finish(&mut f, &s, "2026-10-17");
        assert_eq!(done_dates(&f), vec!["2026-10-16".to_string()]);
        let h = crate::trainer::History::default();
        assert_eq!(crate::trainer::streak(&h, &done_dates(&f), "2026-10-17"), 1);
        assert!(!crate::trainer::did_today(&h, &done_dates(&f), "2026-10-17"));
    }
}
