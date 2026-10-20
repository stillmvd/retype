from wave1 import INK, PAPER, arc, d, fill, letter_k, letter_r, letter_ya, rrect, stroke

NUMBER = 2
TITLE = "Волна 2 — клавиша и зеркальная пара"
PROMPT = ("Сверху — объёмная клавиша (A3) в пяти вариантах; ниже скрещиваю A2, A3 и B2. Серый здесь — "
          "второй тон будущей пары цветов, не цвет. Отметь, что ближе.")
CHOSEN = ["A4", "A6", "B5"]

MID = "#8a8a92"


def key_ya():
    return (fill(rrect(4, 4, 56, 56, 15), INK) + stroke(letter_r(13, 12, 17, 23), 5.4, PAPER)
            + stroke(letter_ya(39, 38, 10, 13), 3.8, PAPER))


def key_r():
    return fill(rrect(4, 4, 56, 56, 15), INK) + stroke(letter_r(20, 14, 22, 36), 7.0, PAPER)


def face_pair():
    return (fill(rrect(4, 4, 56, 56, 15), INK) + fill(rrect(9, 7, 46, 40, 10), PAPER)
            + stroke(letter_ya(14, 16, 12, 22), 4.4, INK) + stroke(letter_r(38, 16, 12, 22), 4.4, INK))


def split_tones():
    left = [(32, 4)] + arc(19, 17, 13, -90, -180) + arc(19, 47, 13, 180, 90) + [(32, 60)]
    right = [(32, 4)] + arc(45, 17, 13, -90, 0) + arc(45, 47, 13, 0, 90) + [(32, 60)]
    return (fill(left, MID) + fill(right, INK)
            + stroke(letter_ya(11, 20, 13, 24), 5.0, PAPER) + stroke(letter_r(40, 20, 13, 24), 5.0, PAPER))


def split_diag():
    body = rrect(4, 4, 56, 56, 15)
    upper = [p for p in body if p[0] + p[1] <= 64]
    return (fill(body, INK) + fill(upper + [(4, 60)][:0], MID)
            + stroke(letter_ya(11, 10, 12, 19), 4.6, PAPER) + stroke(letter_r(36, 34, 14, 21), 5.0, PAPER))


def split_gap():
    left = [(30, 4)] + arc(19, 17, 13, -90, -180) + arc(19, 47, 13, 180, 90) + [(30, 60)]
    right = [(34, 4)] + arc(45, 17, 13, -90, 0) + arc(45, 47, 13, 0, 90) + [(34, 60)]
    return (fill(left, MID) + fill(right, INK)
            + stroke(letter_ya(10, 20, 12, 24), 5.0, PAPER) + stroke(letter_r(41, 20, 12, 24), 5.0, PAPER))


def deep_rk():
    return (fill(rrect(4, 4, 56, 56, 15), INK) + fill(rrect(10, 7, 44, 36, 10), PAPER)
            + stroke(letter_r(17, 13, 13, 17), 4.4, INK) + stroke(letter_k(40, 29, 7, 9), 3.2, INK))


def face_r():
    return (fill(rrect(4, 4, 56, 56, 15), INK) + fill(rrect(10, 7, 44, 42, 11), PAPER)
            + stroke(letter_r(23, 14, 17, 27), 6.0, INK))


def bevel():
    face = rrect(13, 9, 38, 36, 8)
    corners = [((14, 10), (10, 9)), ((50, 10), (54, 9)), ((14, 44), (10, 54)), ((50, 44), (54, 54))]
    edges = "".join(stroke([d([a, b])], 2.2, MID) for a, b in corners)
    return (fill(rrect(4, 4, 56, 56, 14), INK) + edges + fill(face, PAPER)
            + stroke(letter_r(19, 14, 12, 16), 4.0, INK) + stroke(letter_k(39, 31, 6, 8), 3.0, INK))


def two_tone_face():
    return (fill(rrect(4, 4, 56, 56, 15), INK) + fill(rrect(10, 7, 44, 42, 11), MID)
            + stroke(letter_r(17, 14, 15, 20), 5.0, PAPER) + stroke(letter_k(40, 34, 7, 9), 3.4, PAPER))


def face_split():
    lf = [(31, 7)] + arc(21, 18, 11, -90, -180) + arc(21, 38, 11, 180, 90) + [(31, 49)]
    rt = [(33, 7)] + arc(43, 18, 11, -90, 0) + arc(43, 38, 11, 0, 90) + [(33, 49)]
    return (fill(rrect(4, 4, 56, 56, 15), INK) + fill(lf, MID) + fill(rt, PAPER)
            + stroke(letter_ya(14, 17, 11, 20), 4.2, PAPER) + stroke(letter_r(38, 17, 11, 20), 4.2, INK))


VARIANTS = {"deep_rk": deep_rk, "face_r": face_r, "bevel": bevel, "two_tone_face": two_tone_face,
            "face_split": face_split,"key_ya": key_ya, "key_r": key_r, "face_pair": face_pair, "split_tones": split_tones,
            "split_diag": split_diag, "split_gap": split_gap}


def variant_svg(uid, key, in_circle):
    return VARIANTS[key]()


CARDS = [
    ("A3 — объёмная клавиша", [
        {"code": "A7", "name": "Глубокая клавиша", "note": "Грань выше, передний скат толще — клавиша «стоит», "
         "R и К на грани.", "tiles": [("A7", "", "deep_rk", False)]},
        {"code": "A8", "name": "Грань с одной R", "note": "Объём оставлен, подпись одна и крупная — держится "
         "до 16 px.", "tiles": [("A8", "", "face_r", False)]},
        {"code": "A9", "name": "Скосы по углам", "note": "Грань меньше, от её углов к краям идут рёбра — "
         "клавиша сверху, как на фото клавиатуры. На 24 px рёбра сливаются.", "tiles": [("A9", "", "bevel", False)]},
        {"code": "A10", "name": "Грань вторым тоном", "note": "Грань не белая, а второго тона — в цвете лайм на "
         "тёмном корпусе, буквы светлые.", "tiles": [("A10", "", "two_tone_face", False)]},
        {"code": "A11", "name": "Грань пополам Я | R", "note": "Объём + пара раскладок двумя тонами на грани.",
         "tiles": [("A11", "", "face_split", False)]},
    ]),
    ("A2 — залитая клавиша", [
        {"code": "A4", "name": "R и маленькая Я", "note": "Вместо К — Я: подпись второй раскладки и зеркало R. "
         "На 24 px Я, как и К, превращается в точку.", "tiles": [("A4", "", "key_ya", False)]},
        {"code": "A5", "name": "Одна R", "note": "Только R, крупно. Держится до 16 px; про раскладку молчит — это "
         "скажут цвета и плашка в трее.", "tiles": [("A5", "", "key_r", False)]},
    ]),
    ("A3 + B2 — грань клавиши с парой", [
        {"code": "A6", "name": "Я R на грани", "note": "Светлая верхняя грань, на ней Я и R рядом. Узнаётся "
         "клавиша, пара читается до 32 px.", "tiles": [("A6", "", "face_pair", False)]},
    ]),
    ("B2 — пара Я | R двумя тонами", [
        {"code": "B4", "name": "Пополам, два тона", "note": "Клавиша во весь квадрат, левая половина светлее. "
         "В цвете это лайм и тёмный — две раскладки.", "tiles": [("B4", "", "split_tones", False)]},
        {"code": "B5", "name": "Пополам с щелью", "note": "То же, но половины разведены щелью — как две клавиши "
         "или две плашки у Tolk.", "tiles": [("B5", "", "split_gap", False)]},
        {"code": "B6", "name": "По диагонали", "note": "Я сверху слева, R снизу справа — как подписи на клавише, "
         "но двумя полями.", "tiles": [("B6", "", "split_diag", False)]},
    ]),
]

SUMMARY = ("Моё мнение: B5 — самый сильный. Две половины со щелью роднят с Tolk, на 24 px держится силуэт, Я и R "
           "читаются до 32 px, а в цвете лайм против тёмного прямо говорит «две раскладки». A5 — запасной: самый "
           "крепкий на 16 px, но без идеи раскладки.")
