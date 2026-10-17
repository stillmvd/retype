use crate::keymap::{script_of, Lang};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

const CHARS_PER_MINUTE: usize = 700;
const WORD_LEN: std::ops::RangeInclusive<usize> = 3..=10;
const PAIR_PHRASES: usize = 20;
const NO_REPEAT: usize = 6;

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

const RU_FREQ: &str = include_str!("../data/ru.freq");
const EN_FREQ: &str = include_str!("../data/en.freq");

fn words(freq: &'static str) -> impl Iterator<Item = &'static str> {
    freq.lines().filter_map(|l| l.split_whitespace().next())
}

fn load() -> [Vec<&'static str>; 2] {
    let phrases = |p: &'static str| p.lines().filter(|l| !l.is_empty()).collect();
    [phrases(include_str!("../data/ru.phrases")), phrases(include_str!("../data/en.phrases"))]
}

fn phrases() -> &'static [Vec<&'static str>; 2] {
    static DATA: OnceLock<[Vec<&'static str>; 2]> = OnceLock::new();
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

fn letter_phrases(lang: Lang, right: char) -> Vec<&'static str> {
    let list = &phrases()[idx(lang)];
    let count = |p: &str| p.chars().flat_map(char::to_lowercase).filter(|&c| c == right).count();
    let rich: Vec<&str> = list.iter().copied().filter(|p| count(p) >= 2).collect();
    if rich.len() >= PAIR_PHRASES { rich } else { list.iter().copied().filter(|p| count(p) >= 1).collect() }
}

fn by_word() -> &'static [HashMap<String, Vec<&'static str>>; 2] {
    static INDEX: OnceLock<[HashMap<String, Vec<&'static str>>; 2]> = OnceLock::new();
    INDEX.get_or_init(|| {
        phrases().clone().map(|list| {
            let mut map: HashMap<String, Vec<&str>> = HashMap::new();
            for p in list {
                let lower = p.to_lowercase();
                let words: HashSet<&str> = lower.split(|c: char| !(c.is_alphabetic() || c == '-')).filter(|w| !w.is_empty()).collect();
                for w in words {
                    map.entry(w.to_string()).or_default().push(p);
                }
            }
            map
        })
    })
}

fn word_phrases(word: &str) -> Option<(Lang, Vec<&'static str>)> {
    let lang = script_of(word)?;
    by_word()[idx(lang)].get(word).map(|list| (lang, list.clone()))
}

struct Weak {
    lang: Lang,
    right: char,
    weight: f32,
    phrases: Vec<&'static str>,
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
    let all = phrases();
    let mut rng = Rng(src.seed | 1);
    let weak: Vec<Weak> = src
        .weak
        .iter()
        .filter_map(|&(_, r, rate)| {
            let lang = script_of(&r.to_string())?;
            let phrases = letter_phrases(lang, r);
            (!phrases.is_empty()).then_some(Weak { lang, right: r, weight: rate.max(0.001), phrases })
        })
        .collect();
    let journal: Vec<(Lang, Vec<&str>)> = src.journal.iter().filter_map(|w| word_phrases(w)).collect();
    let target = src.minutes.clamp(1, 5) as usize * CHARS_PER_MINUTE;
    let mut out: Vec<Item> = Vec::new();
    let mut total = 0;
    while total < target {
        let roll = rng.below(100);
        let item = if roll < 40 && !weak.is_empty() {
            let w = pick_weak(&weak, &mut rng);
            Item { text: w.phrases[rng.below(w.phrases.len())].to_string(), lang: name(w.lang), kind: "weak", focus: vec![w.right.to_string()] }
        } else if (40..65).contains(&roll) && !journal.is_empty() {
            let (lang, list) = &journal[rng.below(journal.len())];
            Item { text: list[rng.below(list.len())].to_string(), lang: name(*lang), kind: "journal", focus: Vec::new() }
        } else {
            let lang = if rng.chance(src.ru_share) { Lang::Ru } else { Lang::En };
            let list = &all[idx(lang)];
            Item { text: list[rng.below(list.len())].to_string(), lang: name(lang), kind: "phrase", focus: Vec::new() }
        };
        if out.iter().rev().take(NO_REPEAT).any(|l| l.text == item.text) {
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
        let journal = journal_words(&["хочу".to_string(), "сделать".to_string()]);
        let src = Sources { weak: &[('б', 'ю', 0.03), ('р', 'о', 0.02)], journal: &journal, ru_share: 0.7, minutes: 5, seed: 42 };
        let items = build(&src);
        let chars: usize = items.iter().map(|i| i.text.chars().count() + 1).sum();
        assert!(chars >= 5 * CHARS_PER_MINUTE);
        assert!((0.3..0.5).contains(&share(&items, "weak")), "{}", share(&items, "weak"));
        assert!((0.15..0.35).contains(&share(&items, "journal")), "{}", share(&items, "journal"));
        assert!(share(&items, "phrase") > 0.15);
        for i in items.iter().filter(|i| i.kind == "weak") {
            assert!(i.text.to_lowercase().contains(i.focus[0].as_str()), "{i:?}");
        }
        for i in items.iter().filter(|i| i.kind == "journal") {
            let lower = i.text.to_lowercase();
            assert!(lower.contains("хочу") || lower.contains("сделать"), "{i:?}");
        }
        let yu = items.iter().filter(|i| i.kind == "weak" && i.focus[0] == "ю").count();
        let o = items.iter().filter(|i| i.kind == "weak" && i.focus[0] == "о").count();
        assert!(yu > o, "{yu} {o}");
        assert_eq!(build(&src), items);
        assert!(items.windows(2).all(|w| w[0].text != w[1].text));
        assert!(items.iter().all(|i| i.text.split_whitespace().count() >= 2), "одиночное слово");
    }

    #[test]
    fn journal_word_without_phrase_is_skipped() {
        let journal = journal_words(&["привет".to_string()]);
        let items = build(&Sources { weak: &[], journal: &journal, ru_share: 0.7, minutes: 5, seed: 3 });
        assert_eq!(share(&items, "journal"), 0.0);
        assert_eq!(share(&items, "weak"), 0.0);
    }

    #[test]
    fn without_own_data_and_cold_load() {
        let src = Sources { weak: &[], journal: &[], ru_share: 0.7, minutes: 5, seed: 7 };
        let items = build(&src);
        assert!(items.iter().all(|i| i.kind == "phrase"));
        assert!(items.iter().any(|i| i.lang == "en"));
        let t = std::time::Instant::now();
        let fresh = load();
        assert!(!fresh[0].is_empty() && !fresh[1].is_empty());
        assert!(t.elapsed().as_millis() < 50 || cfg!(debug_assertions), "{:?}", t.elapsed());
    }
}
