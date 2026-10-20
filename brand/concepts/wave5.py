import itertools

from wave1 import INK, PAPER, fill, letter_r, letter_ya, rrect, stroke
from wave2 import MID

NUMBER = 5
TITLE = "Волна 5 — доводка D6"
PROMPT = ("Клавиши лесенкой. В каждой карточке выбери одно значение — сверху сразу видно сочетание в крупном "
          "размере и в 48 / 32 / 24 / 16 px. Отправь, когда понравится.")
CHOSEN = ["K средние", "S средний", "W средние", "T обе белые"]

SIZES = [("малые", 28, 37), ("средние", 31, 40), ("крупные", 34, 43)]
SKIRTS = [("мелкий", 5), ("средний", 7), ("глубокий", 10)]
WEIGHTS = [("тонкие", 3.8), ("средние", 4.6), ("жирные", 5.4)]
TONES = [("обе белые", "white"), ("Я вторым тоном", "pair")]
DEFAULT = (1, 1, 1, 0)


def key(x, y, w, h, face, skirt):
    return fill(rrect(x, y, w, h, 10), INK) + fill(rrect(x + 3, y + 2, w - 6, h - skirt - 2, 7), face)


def glyph(letter, x, y, w, h, skirt, weight, color, right=None):
    fw, fh = w - 6, h - skirt - 2
    lw, lh = fw * 0.46, fh * 0.6
    x1 = x + 3 + fw if right is None else min(x + 3 + fw, right - 2)
    cx = (x + 3 + x1) / 2
    return stroke(letter(cx - lw / 2, y + 2 + (fh - lh) / 2, lw, lh), weight, color)


def d6(si, ki, wi, ti):
    _, kw, kh = SIZES[si]
    skirt = SKIRTS[ki][1]
    weight = WEIGHTS[wi][1]
    pair = TONES[ti][1] == "pair"
    lx, ly = 4, 60 - kh
    rx, ry = 60 - kw, 4
    left_face, left_ink = (MID, PAPER) if pair else (PAPER, INK)
    return (key(lx, ly, kw, kh, left_face, skirt) + glyph(letter_ya, lx, ly, kw, kh, skirt, weight, left_ink, rx)
            + key(rx, ry, kw, kh, PAPER, skirt) + glyph(letter_r, rx, ry, kw, kh, skirt, weight, INK))


def variant_svg(uid, key_name, in_circle):
    return d6(*(int(v) for v in key_name.split("|")))


def code(idx):
    return "|".join(str(v) for v in idx)


def with_value(pos, value):
    idx = list(DEFAULT)
    idx[pos] = value
    return code(idx)


def card(letter, name, note, options, pos):
    tiles = [(f"{letter} {label}", label, with_value(pos, i), False, i == DEFAULT[pos])
             for i, (label, *_) in enumerate(options)]
    return {"code": letter, "name": name, "note": note, "radio": True, "tiles": tiles}


CARDS = [
    ("Параметры", [
        card("K", "Размер клавиш", "Крупнее — нахлёст больше, буквы крупнее на 24 px, но лесенка слабее.", SIZES, 0),
        card("S", "Глубина ската", "Толщина переднего ската — насколько клавиша «стоит».", SKIRTS, 1),
        card("W", "Вес букв", "Жирнее — держится на 16–24 px, тоньше — легче на крупных размерах.", WEIGHTS, 2),
        card("T", "Тон грани", "Обе белые — строже; Я вторым тоном — в цвете станет лаймовой клавишей.", TONES, 3),
    ]),
]

CSS = """
.preview{display:grid;gap:12px}
.preview-row{display:grid;gap:10px;grid-template-columns:repeat(auto-fit,minmax(min(100%,300px),1fr))}
.preview-row[hidden]{display:none}
.pv{background:#fff;border:1px solid var(--paper-line);border-radius:22px;padding:24px 16px 16px;display:grid;
  gap:18px;justify-items:center}
.pv .big{width:168px;height:168px}
"""


def svg(inner, cls="", size=None):
    attrs = f' class="{cls}"' if cls else ""
    if size:
        attrs += f' width="{size}" height="{size}"'
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"{attrs} aria-hidden="true">{inner}</svg>'


def preview():
    rows = []
    for idx in itertools.product(*(range(len(o)) for o in (SIZES, SKIRTS, WEIGHTS, TONES))):
        inner = d6(*idx)
        sizes = "".join(f'<span class="size">{svg(inner, size=px)}{px}</span>' for px in (48, 32, 24, 16))
        combo = "-".join(str(v) for v in idx)
        hidden = "" if idx == DEFAULT else " hidden"
        rows.append(f'<div class="preview-row" data-combo="{combo}"{hidden}>'
                    f'<div class="pv">{svg(inner, "big")}</div><div class="pv"><span class="sizes">{sizes}</span></div></div>')
    return f'<div class="preview"><h3>Сборка</h3>{"".join(rows)}</div>'


SUMMARY = ("Моё мнение: крупные клавиши, средний скат, жирные буквы, Я вторым тоном — на 24 px лесенка ещё "
           "видна, а лаймовая Я в цвете сразу скажет про вторую раскладку. Цвет и подложку проверим следующим шагом.")
