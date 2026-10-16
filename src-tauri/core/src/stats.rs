use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Day {
    pub words: u32,
    pub misses: u32,
    pub pairs: BTreeMap<String, u32>,
    pub presses: BTreeMap<String, u32>,
}

pub fn pair_key(wrong: char, right: char) -> String {
    format!("{wrong}>{right}")
}

pub fn split_pair(key: &str) -> Option<(char, char)> {
    let mut it = key.chars();
    let (w, sep, r) = (it.next()?, it.next()?, it.next()?);
    (sep == '>' && it.next().is_none()).then_some((w, r))
}

impl Day {
    pub fn is_empty(&self) -> bool {
        self.words == 0 && self.misses == 0 && self.pairs.is_empty() && self.presses.is_empty()
    }

    pub fn merge(&mut self, other: &Day) {
        self.words += other.words;
        self.misses += other.misses;
        for (k, v) in &other.pairs {
            *self.pairs.entry(k.clone()).or_default() += v;
        }
        for (k, v) in &other.presses {
            *self.presses.entry(k.clone()).or_default() += v;
        }
    }
}

#[derive(Default)]
pub struct Counter {
    erased: Vec<char>,
    retyped: Vec<char>,
    letters: bool,
    word: Day,
    pub full: bool,
    pub day: Day,
}

impl Counter {
    pub fn letter(&mut self, c: char) {
        let c = c.to_lowercase().next().unwrap_or(c);
        if !c.is_alphabetic() {
            return;
        }
        self.letters = true;
        if !self.full {
            return;
        }
        *self.word.presses.entry(c.to_string()).or_default() += 1;
        if self.erased.is_empty() {
            return;
        }
        self.retyped.push(c);
        if self.retyped.len() == self.erased.len() {
            self.settle();
        }
    }

    pub fn back(&mut self, removed: Option<char>) {
        if !self.full {
            return;
        }
        let Some(c) = removed.and_then(|c| c.to_lowercase().next()).filter(|c| c.is_alphabetic()) else {
            self.clear();
            return;
        };
        if self.retyped.pop().is_none() {
            self.erased.push(c);
        }
    }

    pub fn word_end(&mut self, counted: bool) {
        if !self.erased.is_empty() && !self.retyped.is_empty() {
            self.word.misses += 1;
        }
        if std::mem::take(&mut self.letters) && counted {
            self.day.words += 1;
        }
        self.keep_word();
    }

    pub fn drop_word(&mut self) {
        self.word = Day::default();
        self.clear();
    }

    pub fn reset(&mut self) {
        self.letters = false;
        self.keep_word();
    }

    fn keep_word(&mut self) {
        let word = std::mem::take(&mut self.word);
        self.day.merge(&word);
        self.clear();
    }

    pub fn typo_fixed(&mut self, typed: &str, result: &str) {
        if !self.full {
            return;
        }
        self.day.misses += 1;
        let (t, r): (Vec<char>, Vec<char>) = (typed.to_lowercase().chars().collect(), result.to_lowercase().chars().collect());
        if t.len() != r.len() {
            return;
        }
        let mut diff = t.iter().zip(&r).filter(|(a, b)| a != b);
        if let (Some((w, ok)), None) = (diff.next(), diff.next()) {
            *self.day.pairs.entry(pair_key(*w, *ok)).or_default() += 1;
        }
    }

    pub fn take(&mut self) -> Day {
        std::mem::take(&mut self.day)
    }

    fn settle(&mut self) {
        let erased = self.erased.iter().rev();
        let mut missed = false;
        for (w, r) in erased.zip(&self.retyped) {
            if w != r {
                missed = true;
                *self.word.pairs.entry(pair_key(*w, *r)).or_default() += 1;
            }
        }
        if missed {
            self.word.misses += 1;
        }
        self.clear();
    }

    fn clear(&mut self) {
        self.erased.clear();
        self.retyped.clear();
    }
}

pub fn day_number(date: &str) -> Option<i64> {
    let mut it = date.splitn(3, '-').map(|p| p.parse::<i64>().ok());
    let (y, m, d) = (it.next()??, it.next()??, it.next()??);
    let (y, m) = if m <= 2 { (y - 1, m + 9) } else { (y, m - 3) };
    Some(365 * y + y / 4 - y / 100 + y / 400 + (153 * m + 2) / 5 + d)
}

pub fn keep_recent<T>(days: &mut BTreeMap<String, T>, today: &str, keep: i64) {
    let Some(t) = day_number(today) else { return };
    days.retain(|d, _| day_number(d).is_some_and(|n| t - n <= keep));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairs_and_dates() {
        assert_eq!(split_pair(&pair_key('р', 'о')), Some(('р', 'о')));
        assert_eq!(split_pair("ab"), None);
        assert_eq!(day_number("2026-03-01").unwrap() - day_number("2026-02-28").unwrap(), 1);
        assert_eq!(day_number("2027-01-01").unwrap() - day_number("2026-01-01").unwrap(), 365);
        let mut m = BTreeMap::from([("2025-10-16".to_string(), 1), ("2025-10-17".to_string(), 2), ("2026-10-17".to_string(), 3)]);
        keep_recent(&mut m, "2026-10-17", 365);
        assert_eq!(m.keys().collect::<Vec<_>>(), ["2025-10-17", "2026-10-17"]);
    }

    fn run(steps: &str) -> Day {
        let mut c = Counter { full: true, ..Counter::default() };
        let mut typed: Vec<char> = Vec::new();
        for ch in steps.chars() {
            match ch {
                '<' => c.back(typed.pop()),
                ' ' => c.word_end(true),
                '#' => c.reset(),
                _ => {
                    typed.push(ch);
                    c.letter(ch);
                }
            }
        }
        c.take()
    }

    fn pairs(list: &[(&str, u32)]) -> BTreeMap<String, u32> {
        list.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn counts_replacements() {
        let d = run("пии<<ривет ");
        assert_eq!((d.words, d.misses), (1, 1));
        assert_eq!(d.pairs, pairs(&[("и>р", 1)]));
        let d = run("ghbdtn ");
        assert_eq!((d.words, d.misses, d.pairs.len()), (1, 0, 0));
        assert_eq!(d.presses["g"], 1);
        let d = run("приы<вет ");
        assert_eq!(d.pairs, pairs(&[("ы>в", 1)]));
        let d = run("пиак<<ривет ");
        assert_eq!(d.misses, 1);
        assert_eq!(d.pairs, pairs(&[("а>р", 1), ("к>и", 1)]));
        let d = run("о<ро<о ");
        assert_eq!((d.misses, d.pairs.clone()), (1, pairs(&[("о>р", 1)])));
    }

    #[test]
    fn same_letters_changed_mind_and_retyped_prefix() {
        let d = run("при<ивет ");
        assert_eq!((d.misses, d.pairs.len()), (0, 0));
        let d = run("привет< ");
        assert_eq!((d.words, d.misses), (1, 0));
        let d = run("привт<<вет ");
        assert_eq!((d.misses, d.pairs.clone()), (1, pairs(&[("т>е", 1)])));
        let d = run("пррр<<<ивет ");
        assert_eq!((d.misses, d.pairs.len()), (1, 3));
        let d = run("пиа<<# ");
        assert_eq!((d.words, d.misses), (0, 0));
    }

    #[test]
    fn wrong_layout_word_and_unknown_back() {
        let mut c = Counter { full: true, ..Counter::default() };
        for ch in "ghbs".chars() {
            c.letter(ch);
        }
        c.back(Some('s'));
        c.letter('d');
        c.drop_word();
        c.word_end(true);
        assert_eq!(c.take(), Day { words: 1, ..Day::default() });
        for ch in "пии".chars() {
            c.letter(ch);
        }
        c.back(Some('и'));
        c.back(Some('и'));
        c.letter('р');
        c.back(None);
        c.letter('и');
        c.word_end(true);
        let d = c.take();
        assert_eq!((d.words, d.misses, d.pairs.len()), (1, 0, 0));
    }

    #[test]
    fn off_counts_only_words() {
        let mut c = Counter::default();
        for ch in "пии".chars() {
            c.letter(ch);
        }
        c.back(Some('и'));
        c.letter('р');
        c.typo_fixed("пнивет", "привет");
        c.word_end(true);
        c.word_end(true);
        assert_eq!(c.take(), Day { words: 1, ..Day::default() });
    }

    #[test]
    fn shorter_retype_is_miss_without_pair() {
        let d = run("приввт<<т ");
        assert_eq!((d.misses, d.pairs.len()), (1, 0));
    }

    #[test]
    fn engine_typo_fix() {
        let mut c = Counter { full: true, ..Counter::default() };
        c.typo_fixed("пнивет", "привет");
        c.typo_fixed("прувет", "привет");
        c.typo_fixed("приет", "привет");
        let d = c.take();
        assert_eq!(d.misses, 3);
        assert_eq!(d.pairs, BTreeMap::from([("н>р".to_string(), 1), ("у>и".to_string(), 1)]));
    }

    #[test]
    fn merge_days() {
        let mut a = Day { words: 2, misses: 1, pairs: BTreeMap::from([("р>о".into(), 1)]), presses: BTreeMap::new() };
        let b = Day { words: 3, misses: 1, pairs: BTreeMap::from([("р>о".into(), 2)]), presses: BTreeMap::from([("о".into(), 5)]) };
        a.merge(&b);
        assert_eq!((a.words, a.misses, a.pairs["р>о"], a.presses["о"]), (5, 2, 3, 5));
    }
}
