use std::collections::HashSet;

const TARGET: usize = 2000;
const TOP_WORDS: usize = 30_000;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [input, lang, output] = args.as_slice() else {
        eprintln!("phrases <tatoeba.tsv> <ru|en> <out>");
        std::process::exit(2);
    };
    let (freq, letters, extra) = match lang.as_str() {
        "ru" => (include_str!("../data/ru.freq"), "абвгдеёжзийклмнопрстуфхцчшщъыьэюя", ""),
        _ => (include_str!("../data/en.freq"), "abcdefghijklmnopqrstuvwxyz", "'"),
    };
    let known: HashSet<&str> = freq.lines().take(TOP_WORDS).filter_map(|l| l.split_whitespace().next()).collect();
    let text = std::fs::read_to_string(input).expect("input");
    let mut seen = HashSet::new();
    let mut picked: Vec<String> = Vec::new();
    for line in text.lines() {
        let Some(s) = line.splitn(3, '\t').nth(2) else { continue };
        let s = s.trim().replace(" — ", " - ").replace(" – ", " - ");
        let len = s.chars().count();
        if !(20..=80).contains(&len) || !s.ends_with(['.', '?', '!']) || !s.chars().next().is_some_and(char::is_uppercase) {
            continue;
        }
        let ok_char = |c: char| letters.contains(c.to_lowercase().next().unwrap_or(c)) || " ,.?!-:".contains(c) || extra.contains(c);
        if !s.chars().all(ok_char) || s.contains("  ") || s.contains("--") {
            continue;
        }
        let words_ok = s
            .split(|c: char| !(c.is_alphabetic() || extra.contains(c)))
            .filter(|w| !w.is_empty())
            .all(|w| known.contains(w.to_lowercase().as_str()));
        if words_ok && seen.insert(s.to_lowercase()) {
            picked.push(s);
        }
    }
    let step = (picked.len() / TARGET).max(1);
    let out: Vec<&str> = picked.iter().step_by(step).take(TARGET).map(String::as_str).collect();
    std::fs::write(output, out.join("\n") + "\n").expect("write");
    eprintln!("{lang}: {} подходящих, записано {}", picked.len(), out.len());
}
