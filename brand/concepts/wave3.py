from wave1 import INK, PAPER, arc, d, fill, letter_r, letter_ya, rrect, stroke
from wave2 import MID

NUMBER = 3
TITLE = "Волна 3 — сочетания A4 · A6 · B5"
PROMPT = ("Скрещиваю выбранное: R с маленькой Я (A4), грань клавиши (A6), две половины со щелью (B5). "
          "Серый — место лайма. Отметь ближе всего.")
CHOSEN = ["C1", "C6"]


def two_keys():
    out = ""
    for x, tone, letter, ink in ((4, MID, letter_ya, PAPER), (33, INK, letter_r, PAPER)):
        out += fill(rrect(x, 10, 27, 44, 8), INK if tone == INK else "#5c5c63")
        out += fill(rrect(x + 3, 12, 21, 34, 6), tone if tone != INK else "#3a3a40")
        out += stroke(letter(x + 8, 20, 11, 18), 4.2, ink)
    return out


def a4_face():
    return (fill(rrect(4, 4, 56, 56, 15), INK) + fill(rrect(10, 7, 44, 42, 11), PAPER)
            + stroke(letter_r(16, 13, 16, 21), 5.2, INK) + stroke(letter_ya(39, 33, 8, 11), 3.4, INK))


def a4_chip():
    chip = rrect(33, 33, 23, 23, 10)
    return (fill(rrect(4, 4, 56, 56, 15), INK) + fill(chip, MID)
            + stroke(letter_r(13, 12, 17, 23), 5.4, PAPER) + stroke(letter_ya(39.5, 38.5, 10, 13), 3.6, PAPER))


def b5_big_small():
    left = [(22, 4)] + arc(17, 17, 13, -90, -180)[0:0] + rrect(4, 4, 18, 56, 9)
    return (fill(rrect(4, 4, 18, 56, 9), MID) + fill(rrect(26, 4, 34, 56, 15), INK)
            + stroke(letter_ya(8, 24, 10, 16), 3.8, PAPER) + stroke(letter_r(34, 16, 17, 31), 6.0, PAPER))


def a6_two_tone():
    return (fill(rrect(4, 4, 56, 56, 15), INK) + fill(rrect(9, 7, 46, 42, 10), MID)
            + stroke(letter_ya(14, 16, 12, 23), 4.8, PAPER) + stroke(letter_r(38, 16, 12, 23), 4.8, PAPER))


def b5_face():
    out = ""
    for x, w, tone, letter in ((4, 26, MID, letter_ya), (34, 26, INK, letter_r)):
        out += fill(rrect(x, 4, w, 56, 12), tone)
    out += fill(rrect(7, 7, 20, 40, 8), "#a9a9b0") + fill(rrect(37, 7, 20, 40, 8), "#3a3a40")
    return out + stroke(letter_ya(11, 16, 12, 22), 4.6, PAPER) + stroke(letter_r(41, 16, 12, 22), 4.6, PAPER)


VARIANTS = {"two_keys": two_keys, "a4_face": a4_face, "a4_chip": a4_chip, "b5_big_small": b5_big_small,
            "a6_two_tone": a6_two_tone, "b5_face": b5_face}


def variant_svg(uid, key, in_circle):
    return VARIANTS[key]()


CARDS = [
    ("От A4 — R главная, Я подписью", [
        {"code": "C1", "name": "A4 с гранью", "note": "Объём A6 и подпись A4: R крупно, Я в углу грани.",
         "tiles": [("C1", "", "a4_face", False)]},
        {"code": "C2", "name": "Я на своей плашке", "note": "Я в углу на отдельной плашке второго тона — "
         "в цвете лаймовый уголок. Уголок виден даже на 16 px, хоть Я там и не прочесть.",
         "tiles": [("C2", "", "a4_chip", False)]},
        {"code": "C3", "name": "Узкая Я, широкая R", "note": "B5 с неравными половинами: R главная, Я — "
         "узкая полоса слева.", "tiles": [("C3", "", "b5_big_small", False)]},
    ]),
    ("От A6 и B5 — пара на равных", [
        {"code": "C4", "name": "Грань вторым тоном", "note": "A6, но грань серая (лайм), буквы светлые — "
         "контраст сильнее, чем у белой грани.", "tiles": [("C4", "", "a6_two_tone", False)]},
        {"code": "C5", "name": "Две половины с гранями", "note": "B5, у каждой половины своя грань — "
         "две клавиши, сросшиеся в одну иконку.", "tiles": [("C5", "", "b5_face", False)]},
        {"code": "C6", "name": "Две клавиши", "note": "Две отдельные объёмные клавиши Я и R. Самая буквальная "
         "клавиатура, но мельче всех на 24 px.", "tiles": [("C6", "", "two_keys", False)]},
    ]),
]

SUMMARY = ("Моё мнение: C2 — лучший компромисс. R держит знак на любом размере, а лаймовый уголок с Я "
           "даёт идею второй раскладки даже там, где букву уже не прочесть. Если хочется пары на равных — C4.")
