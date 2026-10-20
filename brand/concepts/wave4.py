from wave1 import INK, PAPER, fill, letter_r, letter_ya, rrect, stroke
from wave2 import MID

NUMBER = 4
TITLE = "Волна 4 — пропорции C1 и C6"
PROMPT = ("Довожу C1 (клавиша с гранью, R и Я подписью) и C6 (две клавиши Я и R): глубина клавиши, размер грани, "
          "доля Я, вес букв. Отметь лучшие.")
CHOSEN = ["D6"]


def key(x, y, w, h, body, face, skirt=8, side=3, r=10):
    return fill(rrect(x, y, w, h, r), body) + fill(rrect(x + side, y + 2, w - 2 * side, h - skirt - 2, r - 3), face)


def c1(skirt=8, ya=(39, 33, 8, 11), r_box=(16, 13, 16, 21), wr=5.2, wy=3.4):
    return (key(4, 4, 56, 56, INK, PAPER, skirt, 6, 15) + stroke(letter_r(*r_box), wr, INK)
            + stroke(letter_ya(*ya), wy, INK))


def d1():
    return c1()


def d2():
    return (key(4, 4, 56, 56, INK, PAPER, 14, 6, 15) + stroke(letter_r(15, 11, 15, 19), 5.0, INK)
            + stroke(letter_ya(39, 29, 8, 10), 3.2, INK))


def d3():
    return (key(4, 4, 56, 56, INK, PAPER, 8, 6, 15) + stroke(letter_r(13, 11, 13, 18), 5.0, INK)
            + stroke(letter_ya(36, 29, 12, 16), 4.4, INK))


def d4():
    return (key(4, 4, 56, 56, INK, PAPER, 9, 5, 15) + stroke(letter_r(14, 11, 16, 22), 6.4, INK)
            + stroke(letter_ya(41, 35, 9, 11), 4.0, INK))


def d5():
    return (key(4, 8, 27, 50, "#5c5c63", MID, 7, 3, 9) + key(33, 8, 27, 50, INK, PAPER, 7, 3, 9)
            + stroke(letter_ya(10, 18, 13, 22), 4.6, PAPER) + stroke(letter_r(40, 18, 13, 22), 4.6, INK))


def d6():
    return (key(4, 20, 31, 40, INK, PAPER, 7, 3, 10) + key(29, 4, 31, 40, INK, PAPER, 7, 3, 10)
            + stroke(letter_ya(12, 27, 13, 19), 4.4, INK) + stroke(letter_r(38, 11, 13, 19), 4.4, INK))


VARIANTS = {"d1": d1, "d2": d2, "d3": d3, "d4": d4, "d5": d5, "d6": d6}


def variant_svg(uid, key_name, in_circle):
    return VARIANTS[key_name]()


CARDS = [
    ("C1 — одна клавиша, R и Я подписью", [
        {"code": "D1", "name": "Как C1", "note": "Опорный вариант для сравнения.", "tiles": [("D1", "", "d1", False)]},
        {"code": "D2", "name": "Глубже", "note": "Передний скат вдвое толще — клавиша выше, грань меньше.",
         "tiles": [("D2", "", "d2", False)]},
        {"code": "D3", "name": "Я крупнее", "note": "Я почти в половину R — вторая раскладка читается до 32 px.",
         "tiles": [("D3", "", "d3", False)]},
        {"code": "D4", "name": "Буквы жирнее", "note": "R и Я толще и крупнее — лучше держатся на 24 px.",
         "tiles": [("D4", "", "d4", False)]},
    ]),
    ("C6 — две клавиши", [
        {"code": "D5", "name": "Рядом, во весь рост", "note": "Две клавиши на всю высоту: Я вторым тоном (лайм), "
         "R тёмная с белой гранью.", "tiles": [("D5", "", "d5", False)]},
        {"code": "D6", "name": "Лесенкой", "note": "Клавиши со сдвигом и нахлёстом, как соседние ряды "
         "клавиатуры. Живее, но сложнее на 24 px.", "tiles": [("D6", "", "d6", False)]},
    ]),
]

SUMMARY = ("Моё мнение: D4 — самая крепкая одиночная клавиша: жирные буквы спасают 24 px, Я ещё видна. "
           "Из пар — D5: две клавиши на всю высоту не мельчат, а тона прямо говорят «две раскладки».")
