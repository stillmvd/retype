from wave1 import fill, letter_r, letter_ya, rrect, stroke
from wave5 import SIZES, SKIRTS, WEIGHTS, glyph

NUMBER = 6
TITLE = "Волна 6 — цвет"
PROMPT = ("Форма зафиксирована: D6, средние клавиши, средний скат, средние буквы. Выбери палитру — сверху она "
          "на светлой и тёмной подложке, как на панели задач Windows.")
CHOSEN = ["P Я лаймовая, R тёмная"]

LIME = "#c6ec5a"
DEEP = "#4f7a00"
INK = "#1a2400"
WHITE = "#ffffff"

PALETTES = [
    ("тёмные, белые грани", (INK, WHITE, INK), (INK, WHITE, INK)),
    ("тёмные, лаймовые грани", (INK, LIME, INK), (INK, LIME, INK)),
    ("Я лаймовая, R тёмная", (DEEP, LIME, INK), (INK, WHITE, INK)),
    ("зелёный корпус, белые грани", (DEEP, WHITE, INK), (DEEP, WHITE, INK)),
    ("зелёный корпус, лаймовые грани", (DEEP, LIME, INK), (DEEP, LIME, INK)),
]
DEFAULT = 3


def key(x, y, w, h, body, face, skirt):
    return fill(rrect(x, y, w, h, 10), body) + fill(rrect(x + 3, y + 2, w - 6, h - skirt - 2, 7), face)


def mark(pi):
    _, kw, kh = SIZES[1]
    skirt = SKIRTS[1][1]
    weight = WEIGHTS[1][1]
    (lb, lf, li), (rb, rf, ri) = PALETTES[pi][1:]
    lx, ly, rx, ry = 4, 60 - kh, 60 - kw, 4
    return (key(lx, ly, kw, kh, lb, lf, skirt) + glyph(letter_ya, lx, ly, kw, kh, skirt, weight, li)
            + key(rx, ry, kw, kh, rb, rf, skirt) + glyph(letter_r, rx, ry, kw, kh, skirt, weight, ri))


def variant_svg(uid, key_name, in_circle):
    return mark(int(key_name))


CARDS = [
    ("Палитра", [
        {"code": "P", "name": "Цвета клавиш", "radio": True,
         "note": "Тёмный корпус на тёмной панели задач почти пропадает — держатся только грани. Зелёный корпус "
                 "виден на обеих подложках.",
         "tiles": [(f"P {label}", label, str(i), False, i == DEFAULT) for i, (label, *_) in enumerate(PALETTES)]},
    ]),
]

CSS = """
.pv.dark{background:#1c1c1f;border-color:#2e2e33}
.pv.dark .size{color:#8a8a92}
"""


def svg(inner, cls="", size=None):
    attrs = f' class="{cls}"' if cls else ""
    if size:
        attrs += f' width="{size}" height="{size}"'
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"{attrs} aria-hidden="true">{inner}</svg>'


def preview():
    rows = []
    for i in range(len(PALETTES)):
        inner = mark(i)
        sizes = "".join(f'<span class="size">{svg(inner, size=px)}{px}</span>' for px in (48, 32, 24, 16))
        cells = "".join(f'<div class="pv {t}">{svg(inner, "big")}<span class="sizes">{sizes}</span></div>'
                        for t in ("light", "dark"))
        hidden = "" if i == DEFAULT else " hidden"
        rows.append(f'<div class="preview-row" data-combo="{i}"{hidden}>{cells}</div>')
    return f'<div class="preview"><h3>Сборка</h3>{"".join(rows)}</div>'


SUMMARY = ("Моё мнение: зелёный корпус с белыми гранями. Это тёмный акцент интерфейса (#4f7a00) — знак виден и на "
           "светлой, и на тёмной панели задач, белые грани держат буквы, а лайм останется плашке RU/EN в трее, чтобы "
           "знак приложения и индикатор раскладки не спорили.")
