use retype_core::dict::Dictionaries;
use retype_core::keymap::Lang;

const TOP_WORDS: usize = 50_000;
const MAX_LEN: usize = 8;
const BLOCK: &[&str] = &[
    "ass", "asses", "bitch", "cock", "cunt", "damn", "dick", "fag", "fags", "faggot", "fuck", "nigga", "nigger", "piss", "porn", "pussy", "retard", "sex", "sexy",
    "shit", "slut", "tits", "whore", "сука", "суки", "сучка", "суку", "шлюха", "шлюхи", "шлюху", "жопа", "жопу", "жопе", "говно", "дерьмо", "дерьма", "хрен",
    "хер", "член", "секс", "ублюдок", "ублюдки", "сволочь", "задница", "задницу", "трахать", "черт", "чёрт",
];

fn main() {
    let dicts = Dictionaries::load();
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    for (lang, freq, letters, vowels, singles, name) in [
        (Lang::Ru, include_str!("../data/ru.freq"), "абвгдеёжзийклмнопрстуфхцчшщъыьэюя", "аеёиоуыэюя", "авиксуоя", "ru.fingers"),
        (Lang::En, include_str!("../data/en.freq"), "abcdefghijklmnopqrstuvwxyz", "aeiouy", "a", "en.fingers"),
    ] {
        let mut seen = std::collections::HashSet::new();
        let picked: Vec<&str> = freq
            .lines()
            .take(TOP_WORDS)
            .filter_map(|l| l.split_whitespace().next())
            .filter(|w| {
                let n = w.chars().count();
                n <= MAX_LEN
                    && w.chars().all(|c| letters.contains(c))
                    && w.chars().any(|c| vowels.contains(c))
                    && (n > 1 || singles.contains(*w))
                    && !BLOCK.contains(w)
                    && dicts.certain(lang, w)
            })
            .filter(|w| seen.insert(*w))
            .collect();
        std::fs::write(out.join(name), picked.join("\n") + "\n").expect("write");
        println!("{name}: {}", picked.len());
    }
}
