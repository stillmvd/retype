import io
import math
import pathlib
import subprocess
import tempfile

import uharfbuzz as hb
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont

HERE = pathlib.Path(__file__).resolve().parent
TRAY = HERE.parent.parent / "src-tauri/icons/tray"
FONT = pathlib.Path("C:/Users/stillmvd/Projects/Kockers DS/fonts/KockersSans-Bold.woff2")
INKSCAPE = "C:/Program Files/Inkscape/bin/inkscape.com"

LIME = "#c6ec5a"
DEEP = "#4f7a00"
INK = "#1a2400"
WHITE = "#ffffff"
INK_LIGHT = "#17171a"
INK_DARK = "#ececef"

KEY_W, KEY_H = 31, 40
BASE = dict(skirt=7, weight=4.6)
SMALL = dict(skirt=6, weight=5.6)


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


def fill(pts, color):
    return f'<path d="{d(pts, True)}" fill="{color}"/>'


def stroke(paths, w, color):
    return "".join(f'<path d="{p}" fill="none" stroke="{color}" stroke-width="{w:.2f}" '
                   f'stroke-linecap="round" stroke-linejoin="round"/>' for p in paths)


def letter_r(x, y, w, h):
    mid = y + h * 0.54
    r = (mid - y) / 2
    bowl = [(x, y), (x + w - r, y)] + arc(x + w - r, y + r, r, -90, 90) + [(x, mid)]
    return [d([(x, y), (x, y + h)]), d(bowl), d([(x + w * 0.42, mid), (x + w, y + h)])]


def letter_ya(x, y, w, h):
    mid = y + h * 0.54
    r = (mid - y) / 2
    right = x + w
    bowl = [(right, y), (x + r, y)] + arc(x + r, y + r, r, -90, -270) + [(right, mid)]
    return [d([(right, y), (right, y + h)]), d(bowl), d([(right - w * 0.42, mid), (x, y + h)])]


def key(x, y, body, face, letter, ink, skirt, weight):
    fw, fh = KEY_W - 6, KEY_H - skirt - 2
    lw, lh = fw * 0.46, fh * 0.6
    glyph = letter(x + 3 + (fw - lw) / 2, y + 2 + (fh - lh) / 2, lw, lh)
    return (fill(rrect(x, y, KEY_W, KEY_H, 10), body) + fill(rrect(x + 3, y + 2, fw, fh, 7), face)
            + stroke(glyph, weight, ink))


def mark_body(skirt, weight, mono=None):
    if mono:
        return fill(rrect(4, 60 - KEY_H, KEY_W, KEY_H, 10), mono) + fill(rrect(60 - KEY_W, 4, KEY_W, KEY_H, 10), mono)
    return (key(4, 60 - KEY_H, DEEP, LIME, letter_ya, INK, skirt, weight)
            + key(60 - KEY_W, 4, INK, WHITE, letter_r, INK, skirt, weight))


def document(body, view="0 0 64 64", width=64, height=64):
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{view}" width="{width}" height="{height}">{body}</svg>'


def word(text, cap, x, baseline):
    font = TTFont(FONT)
    raw = io.BytesIO()
    font.flavor = None
    font.save(raw)
    hbfont = hb.Font(hb.Face(hb.Blob(raw.getvalue())))
    buf = hb.Buffer()
    buf.add_str(text)
    buf.guess_segment_properties()
    hb.shape(hbfont, buf, {"kern": True, "liga": True})
    k = cap / font["OS/2"].sCapHeight
    glyphs = font.getGlyphSet()
    order = font.getGlyphOrder()
    pen = SVGPathPen(glyphs, ntos=lambda v: f"{v:.2f}")
    pen_x = 0
    for info, pos in zip(buf.glyph_infos, buf.glyph_positions):
        tp = TransformPen(pen, (k, 0, 0, -k, x + (pen_x + pos.x_offset) * k, baseline - pos.y_offset * k))
        glyphs[order[info.codepoint]].draw(tp)
        pen_x += pos.x_advance
    return pen.getCommands(), pen_x * k


def lockup(ink):
    cap = 26.0
    gap = 14.0
    d_word, word_w = word("Retype", cap, 64 + gap, 32 + cap / 2)
    width = 64 + gap + word_w + 8
    body = mark_body(**BASE) + f'<path d="{d_word}" fill="{ink}"/>'
    return document(body, f"0 0 {width:.2f} 64", f"{width:.2f}", 64)


TRAY_LOOKS = {
    "dark": {"on": (LIME, None, INK), "mute": ("#a2a2a9", None, "#141416"), "off": (None, "#8a8a92", "#8a8a92"),
             "blink": (WHITE, None, "#141416")},
    "light": {"on": (DEEP, None, WHITE), "mute": ("#3a3a40", None, WHITE), "off": (None, "#6b6b73", "#6b6b73"),
              "blink": (INK_LIGHT, None, WHITE)},
}


def plate(text, plate_fill, outline, ink):
    shape = (fill(rrect(0, 0, 32, 32, 7), plate_fill) if plate_fill
             else f'<path d="{d(rrect(1.4, 1.4, 29.2, 29.2, 5.8), True)}" fill="none" stroke="{outline}" stroke-width="2.8"/>')
    cap = 13.0
    d_word, w = word(text, cap, 0, 0)
    x = (32 - w) / 2
    d_word, _ = word(text, cap, x, 16 + cap / 2)
    return document(shape + f'<path d="{d_word}" fill="{ink}"/>', "0 0 32 32", 32, 32)


def tray():
    with tempfile.TemporaryDirectory() as tmp:
        for theme, looks in TRAY_LOOKS.items():
            for look, colors in looks.items():
                for lang in ("ru", "en"):
                    src = pathlib.Path(tmp) / f"{theme}-{look}-{lang}.svg"
                    src.write_text(plate(lang.upper(), *colors), encoding="utf-8")
                    out = TRAY / f"{theme}-{look}-{lang}.png"
                    subprocess.run([INKSCAPE, str(src),
                                    f"--actions=export-filename:{out};export-width:32;export-height:32;export-do"],
                                   check=True, capture_output=True, timeout=180)


def main():
    files = {
        "mark.svg": document(mark_body(**BASE)),
        "mark-small.svg": document(mark_body(**SMALL)),
        "mark-mono.svg": document(mark_body(**BASE, mono="currentColor")),
        "favicon.svg": document(mark_body(**SMALL)),
        "app-icon.svg": document(mark_body(**BASE), width=1024, height=1024),
        "lockup.svg": lockup(INK_LIGHT),
        "lockup-on-dark.svg": lockup(INK_DARK),
    }
    for name, markup in files.items():
        (HERE / name).write_text(markup, encoding="utf-8")
    tray()
    print("SVG:", ", ".join(files), "· трей:", TRAY)


if __name__ == "__main__":
    main()
