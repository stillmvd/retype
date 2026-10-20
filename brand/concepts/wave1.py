import math

NUMBER = 1
TITLE = "Волна 1 — направления"
PROMPT = ("Монохромом, без цвета. A — клавиша с двумя символами, как ты просил; B — другие метафоры: две "
          "раскладки, переключатель, исправленная опечатка. Отметь всё, что цепляет, можно несколько.")
CHOSEN = ["A2", "A3", "B2"]

INK = "#17171a"
PAPER = "#ffffff"


def arc(cx, cy, r, a0, a1, step=3):
    n = max(2, int(abs(a1 - a0) / step) + 1)
    return [(cx + r * math.cos(math.radians(a0 + (a1 - a0) * i / (n - 1))),
             cy + r * math.sin(math.radians(a0 + (a1 - a0) * i / (n - 1)))) for i in range(n)]


def d(pts, closed=False):
    return "M" + " L".join(f"{x:.2f} {y:.2f}" for x, y in pts) + (" Z" if closed else "")


def rrect(x, y, w, h, r):
    r = min(r, w / 2, h / 2)
    return (arc(x + w - r, y + r, r, -90, 0) + arc(x + w - r, y + h - r, r, 0, 90)
            + arc(x + r, y + h - r, r, 90, 180) + arc(x + r, y + r, r, 180, 270))


def stroke(paths, w, color):
    return "".join(f'<path d="{p}" fill="none" stroke="{color}" stroke-width="{w:.2f}" '
                   f'stroke-linecap="round" stroke-linejoin="round"/>' for p in paths)


def fill(pts, color):
    return f'<path d="{d(pts, True)}" fill="{color}"/>'


def letter_r(x, y, w, h):
    mid = y + h * 0.54
    r = (mid - y) / 2
    bowl = [(x, y), (x + w - r, y)] + arc(x + w - r, y + r, r, -90, 90) + [(x, mid)]
    return [d([(x, y), (x, y + h)]), d(bowl), d([(x + w * 0.42, mid), (x + w, y + h)])]


def letter_ya(x, y, w, h):
    mid = y + h * 0.54
    r = (mid - y) / 2
    right = x + w
    bowl =[(right, y), (x + r, y)] + arc(x + r, y + r, r, -90, -270) + [(right, mid)]
    return [d([(right, y), (right, y + h)]), d(bowl), d([(right - w * 0.42, mid), (x, y + h)])]


def letter_k(x, y, w, h):
    joint = (x + w * 0.12, y + h * 0.52)
    return [d([(x, y), (x, y + h)]), d([(x + w, y), joint, (x + w, y + h)])]


def a1():
    body = d(rrect(6, 6, 52, 52, 13), True)
    return (stroke([body], 4.5, INK) + stroke(letter_r(15, 14, 15, 20), 4.6, INK)
            + stroke(letter_k(40, 39, 8, 10), 3.4, INK))


def a2():
    return (fill(rrect(4, 4, 56, 56, 15), INK) + stroke(letter_r(14, 13, 16, 21), 5.2, PAPER)
            + stroke(letter_k(40, 39, 9, 11), 3.8, PAPER))


def a3():
    top = rrect(10, 7, 44, 42, 10)
    return (fill(rrect(4, 4, 56, 56, 15), INK) + fill(top, PAPER)
            + stroke(letter_r(17, 14, 13, 17), 4.2, INK) + stroke(letter_k(40, 33, 7, 9), 3.2, INK))


def b1():
    back = d(rrect(22, 6, 36, 36, 10), True)
    return (stroke([back], 4.2, INK) + fill(rrect(4, 20, 40, 40, 11), INK)
            + stroke(letter_r(15, 29, 16, 22), 5.0, PAPER))


def b2():
    half = [(32, 4)] + arc(45, 17, 13, -90, 0) + arc(45, 47, 13, 0, 90) + [(32, 60)]
    body = d(rrect(6, 6, 52, 52, 13), True)
    return (stroke([body], 4.5, INK) + fill(half, INK)
            + stroke(letter_ya(13, 21, 12, 22), 4.4, INK) + stroke(letter_r(39, 21, 12, 22), 4.4, PAPER))


def b3():
    pts = []
    for i in range(0, 91):
        t = i / 90
        x = 4 + 36 * t
        amp = 9 * (1 - t) ** 1.2
        pts.append((x, 34 + amp * math.sin(t * 5 * math.pi)))
    pts.append((44, 34))
    return (stroke([d(pts)], 6, INK)
            + stroke([d([(54, 10), (54, 54)]), d([(48, 10), (60, 10)]), d([(48, 54), (60, 54)])], 5, INK))


VARIANTS = {"a1": a1, "a2": a2, "a3": a3, "b1": b1, "b2": b2, "b3": b3}


def variant_svg(uid, key, in_circle):
    return VARIANTS[key]()


CARDS = [
    ("A — клавиша с двумя символами", [
        {"code": "A1", "name": "Контур клавиши", "note": "R крупно сверху слева, К мелко снизу справа — как подписи "
         "на настоящей клавише. Лёгкий, но на 24 px К уже точка.", "tiles": [("A1", "", "a1", False)]},
        {"code": "A2", "name": "Залитая клавиша", "note": "То же светом по тёмному, клавиша во весь квадрат, как "
         "плашка у Tolk. Самый плотный силуэт.", "tiles": [("A2", "", "a2", False)]},
        {"code": "A3", "name": "Клавиша с верхней гранью", "note": "Объём через светлую верхнюю грань — узнаётся "
         "как кейкап даже без букв.", "tiles": [("A3", "", "a3", False)]},
    ]),
    ("B — другие метафоры", [
        {"code": "B1", "name": "Две раскладки", "note": "Две карточки внахлёст: задняя — другая раскладка, передняя "
         "с R. Про переключение, без клавиатуры.", "tiles": [("B1", "", "b1", False)]},
        {"code": "B2", "name": "Я | R пополам", "note": "Клавиша разделена: Я светлой половиной, R — тёмной. Буквы зеркальны, как две раскладки. "
         "Тумблер с R в круге отбросил — читался как ®.", "tiles": [("B2", "", "b2", False)]},
        {"code": "B3", "name": "Опечатка выпрямлена", "note": "Волнистое подчёркивание спелчекера переходит в прямую "
         "и упирается в курсор. Про исправление, не про раскладку.", "tiles": [("B3", "", "b3", False)]},
    ]),
]

SUMMARY = ("Моё мнение: сильнее всего A2 — плотный силуэт во весь квадрат держится на 24 px и роднит с Tolk; "
           "вторая буква К на мелких размерах пропадает в любом варианте A, поэтому для иконки ≤ 32 px, скорее, "
           "оставим одну R. B2 — самая «наша» идея (Я и R зеркальны), но на 24 px буквы мельчают; B3 — самый смысловой, но без буквы безликий.")
