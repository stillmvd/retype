use crate::keymap::{script_of, Lang};
use serde::Serialize;
use std::collections::HashSet;
use std::sync::OnceLock;

const COMMON: usize = 5000;
const CHARS_PER_MINUTE: usize = 700;
const WORD_LEN: std::ops::RangeInclusive<usize> = 3..=10;
const PAIR_WORDS: usize = 20;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Item {
    pub text: String,
    pub lang: &'static str,
    pub kind: &'static str,
    pub focus: Vec<String>,
}

pub struct Sources<'a> {
    pub weak: &'a [(char, char, f32)],
    pub journal: &'a [String],
    pub ru_share: f32,
    pub minutes: u8,
    pub seed: u64,
}

struct Data {
    common: [Vec<&'static str>; 2],
    phrases: [Vec<&'static str>; 2],
}

const RU_FREQ: &str = include_str!("../data/ru.freq");
const EN_FREQ: &str = include_str!("../data/en.freq");

fn words(freq: &'static str) -> impl Iterator<Item = &'static str> {
    freq.lines().filter_map(|l| l.split_whitespace().next())
}

fn load() -> Data {
    let common = |f| words(f).take(COMMON).filter(|w| WORD_LEN.contains(&w.chars().count())).collect();
    let phrases = |p: &'static str| p.lines().filter(|l| !l.is_empty()).collect();
    Data {
        common: [common(RU_FREQ), common(EN_FREQ)],
        phrases: [phrases(include_str!("../data/ru.phrases")), phrases(include_str!("../data/en.phrases"))],
    }
}

fn data() -> &'static Data {
    static DATA: OnceLock<Data> = OnceLock::new();
    DATA.get_or_init(load)
}

fn known() -> &'static [HashSet<&'static str>; 2] {
    static KNOWN: OnceLock<[HashSet<&'static str>; 2]> = OnceLock::new();
    KNOWN.get_or_init(|| [words(RU_FREQ).collect(), words(EN_FREQ).collect()])
}

fn idx(lang: Lang) -> usize {
    match lang {
        Lang::Ru => 0,
        Lang::En => 1,
    }
}

fn name(lang: Lang) -> &'static str {
    match lang {
        Lang::Ru => "ru",
        Lang::En => "en",
    }
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }

    fn chance(&mut self, p: f32) -> bool {
        (self.next() % 10_000) as f32 / 10_000.0 < p
    }
}

pub fn journal_words(raw: &[String]) -> Vec<String> {
    let known = known();
    let mut seen = HashSet::new();
    raw.iter()
        .map(|w| w.trim().to_lowercase())
        .filter(|w| WORD_LEN.contains(&w.chars().count()) && w.chars().all(|c| c.is_alphabetic() || c == '-'))
        .filter(|w| {
            let lang = script_of(w);
            lang.is_some_and(|l| w.chars().all(|c| c == '-' || l.owns_letter(c)) && known[idx(l)].contains(w.as_str()))
        })
        .filter(|w| seen.insert(w.clone()))
        .collect()
}

fn pair_words(lang: Lang, wrong: char, right: char) -> Vec<&'static str> {
    let list = &data().common[idx(lang)];
    let twice = |w: &str| w.chars().filter(|&c| c == right).count() >= 2;
    let close: Vec<&str> = list.iter().copied().filter(|w| w.contains(right) && (w.contains(wrong) || twice(w))).collect();
    if close.len() >= PAIR_WORDS { close } else { list.iter().copied().filter(|w| w.contains(right)).collect() }
}

struct Weak {
    lang: Lang,
    right: char,
    weight: f32,
    words: Vec<&'static str>,
}

fn pick_weak<'a>(weak: &'a [Weak], rng: &mut Rng) -> &'a Weak {
    let total: f32 = weak.iter().map(|w| w.weight).sum();
    let mut x = (rng.next() % 10_000) as f32 / 10_000.0 * total;
    for w in weak {
        if x < w.weight {
            return w;
        }
        x -= w.weight;
    }
    &weak[weak.len() - 1]
}

pub fn build(src: &Sources) -> Vec<Item> {
    let d = data();
    let mut rng = Rng(src.seed | 1);
    let weak: Vec<Weak> = src
        .weak
        .iter()
        .filter_map(|&(w, r, rate)| {
            let lang = script_of(&r.to_string())?;
            let words = pair_words(lang, w, r);
            (!words.is_empty()).then_some(Weak { lang, right: r, weight: rate.max(0.001), words })
        })
        .collect();
    let target = src.minutes.clamp(1, 5) as usize * CHARS_PER_MINUTE;
    let mut out: Vec<Item> = Vec::new();
    let mut total = 0;
    while total < target {
        let roll = rng.below(100);
        let lang = if rng.chance(src.ru_share) { Lang::Ru } else { Lang::En };
        let item = if roll < 40 && !weak.is_empty() {
            let w = pick_weak(&weak, &mut rng);
            Item { text: w.words[rng.below(w.words.len())].to_string(), lang: name(w.lang), kind: "weak", focus: vec![w.right.to_string()] }
        } else if (40..65).contains(&roll) && !src.journal.is_empty() {
            let w = &src.journal[rng.below(src.journal.len())];
            Item { text: w.clone(), lang: name(script_of(w).unwrap_or(Lang::Ru)), kind: "journal", focus: Vec::new() }
        } else if roll >= 65 || rng.chance(0.5) {
            let list = &d.phrases[idx(lang)];
            Item { text: list[rng.below(list.len())].to_string(), lang: name(lang), kind: "phrase", focus: Vec::new() }
        } else {
            let list = &d.common[idx(lang)];
            Item { text: list[rng.below(list.len())].to_string(), lang: name(lang), kind: "common", focus: Vec::new() }
        };
        if out.last().is_some_and(|l| l.text == item.text) {
            continue;
        }
        total += item.text.chars().count() + 1;
        out.push(item);
    }
    let focus: Vec<String> = weak.iter().map(|w| w.right.to_string()).collect();
    for item in &mut out {
        if item.focus.is_empty() {
            let lower = item.text.to_lowercase();
            item.focus = focus.iter().filter(|f| lower.contains(f.as_str())).cloned().collect();
            item.focus.sort();
            item.focus.dedup();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn share(items: &[Item], k: &str) -> f32 {
        items.iter().filter(|i| i.kind == k).count() as f32 / items.len() as f32
    }

    #[test]
    fn journal_filter() {
        let raw: Vec<String> = ["привет", "Привет", "nastya0708055", "pixel", "вася_п", "пиксель", "aбв", "фывапролдж", "ок", "google"].map(String::from).to_vec();
        assert_eq!(journal_words(&raw), ["привет", "pixel", "пиксель", "google"]);
    }

    #[test]
    fn mix_and_focus() {
        let journal = journal_words(&["привет".to_string(), "сделать".to_string()]);
        let src = Sources { weak: &[('б', 'ю', 0.03), ('р', 'о', 0.02)], journal: &journal, ru_share: 0.7, minutes: 5, seed: 42 };
        let items = build(&src);
        let chars: usize = items.iter().map(|i| i.text.chars().count() + 1).sum();
        assert!(chars >= 5 * CHARS_PER_MINUTE);
        assert!((0.3..0.5).contains(&share(&items, "weak")), "{}", share(&items, "weak"));
        assert!((0.15..0.35).contains(&share(&items, "journal")), "{}", share(&items, "journal"));
        assert!(share(&items, "phrase") > 0.15);
        for i in items.iter().filter(|i| i.kind == "weak") {
            assert!(i.text.contains(i.focus[0].as_str()), "{i:?}");
        }
        let yu = items.iter().filter(|i| i.kind == "weak" && i.focus[0] == "ю").count();
        let o = items.iter().filter(|i| i.kind == "weak" && i.focus[0] == "о").count();
        assert!(yu > o, "{yu} {o}");
        assert_eq!(build(&src), items);
        assert!(items.windows(2).all(|w| w[0].text != w[1].text));
    }

    #[test]
    fn journal_without_weak_keeps_its_share() {
        let journal = journal_words(&["привет".to_string(), "сделать".to_string()]);
        let items = build(&Sources { weak: &[], journal: &journal, ru_share: 0.7, minutes: 5, seed: 3 });
        assert!(share(&items, "journal") < 0.35, "{}", share(&items, "journal"));
        assert_eq!(share(&items, "weak"), 0.0);
    }

    #[test]
    fn without_own_data_and_cold_load() {
        let src = Sources { weak: &[], journal: &[], ru_share: 0.7, minutes: 5, seed: 7 };
        let items = build(&src);
        assert!(items.iter().all(|i| i.kind == "phrase" || i.kind == "common"));
        assert!(items.iter().any(|i| i.lang == "en"));
        let t = std::time::Instant::now();
        let fresh = load();
        assert!(!fresh.common[0].is_empty() && !fresh.phrases[1].is_empty());
        assert!(t.elapsed().as_millis() < 50 || cfg!(debug_assertions), "{:?}", t.elapsed());
    }
}
