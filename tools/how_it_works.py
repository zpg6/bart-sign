"""Draw the "How it works" illustration: the sign, your Wi-Fi, the internet
and BART's live departures, plus first-time setup from a phone. Writes
docs/assets/how-it-works.svg, and with --png also docs/assets/how-it-works.png
(2x, rendered by headless Chrome).

    python3 -I tools/how_it_works.py [--png]

Everything is vector. The sign on the screen is rebuilt LED by LED from the
firmware's own render, docs/assets/screens/LAFY-EVENING-0.png: each of the
176 x 48 LEDs per sign is sampled at its centre and drawn as a dot, so the
picture is the real sign, not a lookalike. The ESP32 is the drawing from
tools/wiring_diagram.py, scaled down. Facts in the captions follow the
firmware: a fetch every 60 s (every 5 min once service has ended for the
night), HTTPS to api.bart.gov, and the setup network BART-SIGN-XXXX with its
page at 192.168.4.1.
"""
import importlib.util
import os
import subprocess
import sys

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
OUT_SVG = os.path.join(ROOT, "docs", "assets", "how-it-works.svg")
OUT_PNG = os.path.join(ROOT, "docs", "assets", "how-it-works.png")
SCREEN_PNG = os.path.join(ROOT, "docs", "assets", "screens", "LAFY-EVENING-0.png")

_spec = importlib.util.spec_from_file_location("wiring_diagram", os.path.join(HERE, "wiring_diagram.py"))
wd = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(wd)
text, rect, circle, esc, tw, chip = wd.text, wd.rect, wd.circle, wd.esc, wd.tw, wd.chip
BLUE, INK, MUTED, FONT, CHROME = wd.BLUE, wd.INK, wd.MUTED, wd.FONT, wd.CHROME

W, H = 1100, 1010
YELLOW, YELLOW_D = "#FFD200", "#A88A00"
ORANGE, ORANGE_D = "#F7931E", "#A65A00"
LINES = [("#D7261E", "Red"), ("#F7931E", "Orange"), ("#FFD200", "Yellow"), ("#339933", "Green"), ("#0099CC", "Blue")]


# ---------------------------------------------------------------- the sign, LED by LED

def sign_svg(o):
    """The 480 x 320 screen in its own coordinates, from the firmware render.
    Layout constants mirror tools/compose_mockups.py; LED geometry mirrors
    sign-core (176 x 48 LEDs, dot 200/256 of the pitch, Style::BART)."""
    im = Image.open(SCREEN_PNG).convert("RGB")
    px = im.load()
    cols, rows, bw, bh = 176, 48, 458, 125
    sx, sy = bw / cols, bh / rows
    r = sx * 200 / 256 / 2
    o.append(rect(0, 0, 480, 320, "#000"))
    lit_paths = []
    for slot, y0 in enumerate((0, 165)):
        o.append(rect(0, y0, 480, 155, "rgb(92,94,98)"))
        o.append(rect(0, y0, 480, 1, "rgb(60,60,64)"))
        o.append(rect(0, y0 + 1, 480, 2, "rgb(14,12,16)"))
        o.append(rect(10, y0 + 4, 460, 127, "rgb(40,34,36)"))
        fx, fy = 11, y0 + 5
        o.append(f'<pattern id="unlit{slot}" patternUnits="userSpaceOnUse" x="{fx}" y="{fy}" '
                 f'width="{sx:.4f}" height="{sy:.4f}">'
                 f'<rect width="{sx:.4f}" height="{sy:.4f}" fill="rgb(6,2,2)"/>'
                 f'<circle cx="{sx / 2:.4f}" cy="{sy / 2:.4f}" r="{r:.3f}" fill="rgb(40,8,8)"/></pattern>')
        o.append(rect(fx, fy, bw, bh, f"url(#unlit{slot})"))
        for j in range(rows):
            for i in range(cols):
                cx, cy = fx + (i + 0.5) * sx, fy + (j + 0.5) * sy
                if px[int(cx), int(cy)][0] > 150:
                    lit_paths.append(f"M{cx:.2f},{cy:.2f}h.01")
        label = "PLATFORM 2" if slot == 0 else "PLATFORM 1"
        o.append(text(13, y0 + 151, label, 18.1, "rgb(240,240,236)", "bold"))
        o.append(rect(0, y0 + 152, 480, 3, "rgb(58,60,64)"))
    d = "".join(lit_paths)
    o.append(f'<path d="{d}" stroke="rgb(235,22,14)" stroke-width="{2 * r + 2:.2f}" stroke-linecap="round" opacity="0.22"/>')
    o.append(f'<path d="{d}" stroke="rgb(240,40,26)" stroke-width="{2 * r:.2f}" stroke-linecap="round"/>')
    o.append(f'<path d="{d}" stroke="rgb(255,120,90)" stroke-width="{r:.2f}" stroke-linecap="round"/>')
    return len(lit_paths)


# ---------------------------------------------------------------- pieces

def display_front(o, x0, y0, k):
    """The display module from the front, `k` px per mm, sign on the glass."""
    w, h = 94.84 * k, 59.19 * k
    o.append(rect(x0 + 3, y0 + 6, w, h, "#000", rx=10, extra=' opacity="0.14"'))
    o.append(rect(x0, y0, w, h, "#1f5fb0", rx=10, stroke="#14457f", sw=1.5))
    for hx in (x0 + 4 * k, x0 + w - 4 * k):
        for hy in (y0 + 4 * k, y0 + h - 4 * k):
            wd.hole(o, hx, hy, 2.6 * k)
    # glass and black surround, then the active area (73.44 x 48.96 mm)
    gx0, gy0, gw, gh = x0 + 7 * k, y0 + 2.4 * k, 83 * k, 54.4 * k
    o.append(rect(gx0, gy0, gw, gh, "#0b0b0d", rx=3, stroke="#2a2b2f", sw=1))
    ax0, ay0 = gx0 + 4.6 * k, gy0 + 2.7 * k
    s = 73.44 * k / 480
    o.append(f'<g transform="translate({ax0:g},{ay0:g}) scale({s:.5f})">')
    n = sign_svg(o)
    o.append("</g>")
    # a soft reflection across the glass
    o.append(f'<path d="M{gx0 + gw * 0.55:g},{gy0:g} L{gx0 + gw * 0.72:g},{gy0:g} L{gx0 + gw * 0.42:g},{gy0 + gh:g} '
             f'L{gx0 + gw * 0.25:g},{gy0 + gh:g}Z" fill="#fff" opacity="0.05"/>')
    return w, h, n


def esp32_small(o, x0, y0, s):
    """The wiring diagram's ESP32 drawing, scaled by `s`, top-left at x0, y0."""
    tx, ty = x0 - s * wd.BX0, y0 - s * wd.BY0
    o.append(f'<g transform="translate({tx:g},{ty:g}) scale({s:g})">')
    wd.draw_esp32(o)
    o.append("</g>")
    return lambda x, y: (tx + s * x, ty + s * y)


def mini_wires(o, to, dx1, dy0, dh):
    """The eight jumpers, display edge to the right ESP32 pins: the three
    left-row pins straight across, the five right-row pins round the USB end."""
    def wire(d, colour, outline):
        o.append(f'<path d="{d}" fill="none" stroke="#fff" stroke-width="6" stroke-linecap="round"/>')
        o.append(f'<path d="{d}" fill="none" stroke="{outline}" stroke-width="4.2" stroke-linecap="round"/>')
        o.append(f'<path d="{d}" fill="none" stroke="{colour}" stroke-width="2.6" stroke-linecap="round"/>')
    left = [n for n in ("BL", "RST", "DC")]
    right = ["SDA", "SCL", "CS", "GND", "VCC"]
    ends = []
    for k, n in enumerate(left):
        row, silk, _g, _r, colour, outline = wd.WIRES[n]
        px, py = to(wd.PIN_LX, wd.pin_y(wd.ESP_LEFT.index(silk)))
        sy = dy0 + dh * (0.30 + 0.07 * k)
        wire(f"M{dx1 - 6:g},{sy:g} C{dx1 + 14:g},{sy:g} {px - 16:g},{py:g} {px:g},{py:g}", colour, outline)
        ends.append((px, py, colour, outline))
    bx1 = to(wd.BX1, 0)[0]
    by1 = to(0, wd.BY1)[1]
    for k, n in enumerate(right):
        row, silk, _g, _r, colour, outline = wd.WIRES[n]
        px, py = to(wd.PIN_RX, wd.pin_y(wd.ESP_RIGHT.index(silk)))
        sy = dy0 + dh * (0.56 + 0.06 * k)
        under = by1 + 22 + (4 - k) * 5        # outermost wire goes to the top pin
        side = bx1 + 10 + (4 - k) * 5
        d = (f"M{dx1 - 6:g},{sy:g} C{dx1 + 20:g},{sy:g} {dx1 + 10:g},{under:g} {dx1 + 40:g},{under:g} "
             f"L{side - 12:g},{under:g} Q{side:g},{under:g} {side:g},{under - 12:g} "
             f"L{side:g},{py + 8:g} Q{side:g},{py:g} {side - 8:g},{py:g} L{px:g},{py:g}")
        wire(d, colour, outline)
        ends.append((px, py, colour, outline))
    for px, py, colour, outline in ends:
        o.append(circle(px, py, 3, colour, outline, 1))


def router(o, cx, cy):
    o.append(f'<line x1="{cx - 48}" y1="{cy - 4}" x2="{cx - 60}" y2="{cy - 64}" stroke="#2b2d31" stroke-width="7" stroke-linecap="round"/>')
    o.append(f'<line x1="{cx + 48}" y1="{cy - 4}" x2="{cx + 60}" y2="{cy - 64}" stroke="#2b2d31" stroke-width="7" stroke-linecap="round"/>')
    o.append(rect(cx - 82, cy - 6, 164, 52, "#ffffff", rx=12, stroke="#2b2d31", sw=3))
    o.append(rect(cx - 82, cy + 28, 164, 18, "#2b2d31", rx=0, extra=f' clip-path="none"'))
    o.append(rect(cx - 82, cy + 22, 164, 24, "#2b2d31", rx=12))
    o.append(rect(cx - 82, cy + 22, 164, 10, "#2b2d31"))
    for k in range(4):
        o.append(circle(cx - 48 + k * 18, cy + 11, 4, "#3fbf5a" if k < 3 else "#d8dbde"))
    for k, r in enumerate((22, 40, 58)):
        o.append(f'<path d="M{cx - r * 0.72:g},{cy - 22 - r * 0.7:g} A{r},{r} 0 0 1 {cx + r * 0.72:g},{cy - 22 - r * 0.7:g}" '
                 f'fill="none" stroke="{BLUE}" stroke-width="6" stroke-linecap="round" opacity="{1 - k * 0.22:g}"/>')
    o.append(circle(cx, cy - 26, 6, BLUE))


def cloud(o, cx, cy, label):
    bumps = [(-62, 8, 34), (-26, -18, 42), (22, -26, 46), (62, 0, 36), (0, 14, 40), (-40, 20, 28), (40, 20, 30)]
    for dx, dy, r in bumps:
        o.append(circle(cx + dx, cy + dy, r + 3, "#b9c6d6"))
    for dx, dy, r in bumps:
        o.append(circle(cx + dx, cy + dy, r, "#eef3f9"))
    o.append(text(cx, cy + 6, label, 21, INK, "bold", "middle"))


def bart_card(o, x0, y0, w, h):
    o.append(rect(x0 + 3, y0 + 6, w, h, "#000", rx=12, extra=' opacity="0.10"'))
    o.append(rect(x0, y0, w, h, "#fff", rx=12, stroke="#cfd5db", sw=1.5))
    o.append(f'<path d="M{x0},{y0 + 54} V{y0 + 12} Q{x0},{y0} {x0 + 12},{y0} H{x0 + w - 12} '
             f'Q{x0 + w},{y0} {x0 + w},{y0 + 12} V{y0 + 54}Z" fill="{BLUE}"/>')
    o.append(text(x0 + 20, y0 + 25, "BART", 15, "#cfe3f3", "bold", "start", ' letter-spacing="2"'))
    o.append(text(x0 + 20, y0 + 45, "api.bart.gov", 18, "#fff", "bold"))
    o.append(text(x0 + 20, y0 + 86, "Lafayette", 21, INK, "bold"))
    rows = [("SF Airport", "4, 12"), ("SFO/Millbrae", "32, 52"), ("Pittsburg", "6, 26"), ("Antioch", "14, 34")]
    for k, (dest, mins) in enumerate(rows):
        y = y0 + 118 + k * 30
        o.append(circle(x0 + 28, y - 6, 7, YELLOW, YELLOW_D, 1.5))
        o.append(text(x0 + 44, y, dest, 17, INK))
        o.append(text(x0 + w - 18, y, mins, 17, INK, "bold", "end"))
    o.append(text(x0 + w - 18, y0 + 86, "min", 14, MUTED, "bold", "end"))
    y = y0 + h - 26
    for k, (c, _n) in enumerate(LINES):
        o.append(circle(x0 + w / 2 + (k - 2) * 26, y, 8.5, c, "#00000033", 1))


def phone(o, x0, y0):
    w, h = 132, 236
    o.append(rect(x0 + 2, y0 + 5, w, h, "#000", rx=20, extra=' opacity="0.14"'))
    o.append(rect(x0, y0, w, h, "#1b1c1f", rx=20))
    sx0, sy0, sw, sh = x0 + 7, y0 + 16, w - 14, h - 28
    o.append(rect(sx0, sy0, sw, sh, "#fff", rx=12))
    o.append(rect(x0 + w / 2 - 18, y0 + 6, 36, 5, "#3a3c40", rx=2.5))
    o.append(text(sx0 + 8, sy0 + 22, "Set up your sign", 11.5, INK, "bold"))
    fields = [("Wi-Fi network", "Choose a network"), ("Station", "Lafayette"), ("Line", "All lines")]
    for k, (lab, val) in enumerate(fields):
        y = sy0 + 42 + k * 42
        o.append(text(sx0 + 8, y, lab, 9.5, MUTED, "bold"))
        o.append(rect(sx0 + 8, y + 5, sw - 16, 20, "#f3f5f7", rx=4, stroke="#c9cfd5", sw=1))
        o.append(text(sx0 + 14, y + 19, val, 9.5, INK))
        o.append(f'<path d="M{sx0 + sw - 20},{y + 13} l4,4 l4,-4" fill="none" stroke="{MUTED}" stroke-width="1.4"/>')
    o.append(rect(sx0 + 8, sy0 + sh - 38, sw - 16, 24, BLUE, rx=5))
    o.append(text(sx0 + sw / 2, sy0 + sh - 22, "Save and connect", 9.5, "#fff", "bold", "middle"))


def arrow(o, d, colour, outline, end, angle, width=7):
    """A thick wire-style arrow along `d` with a head at `end` pointing `angle` deg."""
    o.append(f'<path d="{d}" fill="none" stroke="#fff" stroke-width="{width + 6}" stroke-linecap="round"/>')
    o.append(f'<path d="{d}" fill="none" stroke="{outline}" stroke-width="{width + 2.6}" stroke-linecap="round"/>')
    o.append(f'<path d="{d}" fill="none" stroke="{colour}" stroke-width="{width}" stroke-linecap="round"/>')
    x, y = end
    o.append(f'<path d="M0,0 L-20,-12 L-20,12Z" transform="translate({x:g},{y:g}) rotate({angle:g})" '
             f'fill="{colour}" stroke="{outline}" stroke-width="2" stroke-linejoin="round"/>')


def tag(o, x, y, label, colour, outline, size=16):
    """Black station-sign chip with a coloured edge, for the arrows."""
    w = tw(label, size) + 26
    h = size * 1.6
    o.append(rect(x - w / 2 - 2, y - h / 2 - 2, w + 4, h + 4, colour, rx=5, stroke=outline, sw=1))
    o.append(rect(x - w / 2, y - h / 2, w, h, "#000", rx=4))
    o.append(text(x, y + size * 0.36, label, size, "#fff", "bold", "middle"))


def caption(o, x, y, n, head, lines, anchor="start"):
    size = 19
    if anchor == "middle":
        wmax = max([tw(head, 24)] + [tw(s, size, False) for s in lines]) + 40
        x = x - wmax / 2
    wd.badge(o, x + 13, y - 8, n)
    o.append(text(x + 34, y, head, 24, INK, "bold"))
    for k, s in enumerate(lines):
        o.append(text(x + 34, y + 28 + k * 24, s, size, MUTED))


# ---------------------------------------------------------------- page

def build():
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}" '
         f'role="img" aria-labelledby="hiw-title hiw-desc" font-family="{FONT}">']
    o.append('<title id="hiw-title">How it works</title>')
    desc = ("Your sign, an ESP32 board wired to a 3.5-inch screen, joins your home Wi-Fi (2.4 GHz). "
            "Once a minute, every 5 minutes overnight, it asks BART's live departures service at api.bart.gov "
            "for your station's trains over HTTPS. It draws them in the platform sign's own font and counts "
            "the minutes down between updates. The first time, the sign makes its own Wi-Fi network, "
            "BART-SIGN-XXXX; join it on your phone, open 192.168.4.1 if the page doesn't appear, and pick "
            "your Wi-Fi network, station and line.")
    o.append(f'<desc id="hiw-desc">{esc(desc)}</desc>')
    o.append(rect(0, 0, W, H, "#ffffff"))
    o.append(rect(36, 30, 64, 7, BLUE))
    o.append(text(34, 86, "How it works", 46, INK, "bold"))
    o.append(text(36, 118, "From BART's data to your desk", 21, MUTED))

    # 1: the sign, front of the display plus the ESP32 behind it
    k = 4.6
    dx0, dy0 = 36, 452
    dw, dh, n_lit = display_front(o, dx0, dy0, k)
    to = esp32_small(o, dx0 + dw + 30, dy0 + 12, 0.36)
    mini_wires(o, to, dx0 + dw, dy0, dh)
    ex0, ey0 = to(wd.BX0, wd.BY0)
    ex1, ey1 = to(wd.BX1, wd.BY1)
    ecx = (ex0 + ex1) / 2
    # Wi-Fi from the ESP32's antenna end
    for kk, r in enumerate((14, 26)):
        o.append(f'<path d="M{ecx - r * 0.7:g},{ey0 - 10 - r * 0.6:g} A{r},{r} 0 0 1 {ecx + r * 0.7:g},{ey0 - 10 - r * 0.6:g}" '
                 f'fill="none" stroke="{BLUE}" stroke-width="4.5" stroke-linecap="round" opacity="{0.9 - kk * 0.25:g}"/>')

    # 2: your Wi-Fi, 3: the internet and BART
    rcx, rcy = 300, 286
    router(o, rcx, rcy)
    ccx, ccy = 610, 236
    cloud(o, ccx, ccy, "The internet")
    bx0, by0, bw, bh = 820, 150, 244, 300
    bart_card(o, bx0, by0, bw, bh)

    # yellow request out: sign -> router -> internet -> BART
    arrow(o, f"M{ecx:g},{ey0 - 50:g} C{ecx:g},{rcy + 110:g} {rcx + 140:g},{rcy + 100:g} {rcx + 96:g},{rcy + 46:g}",
          YELLOW, YELLOW_D, (rcx + 92, rcy + 40), 228)
    arrow(o, f"M{rcx + 70:g},{rcy - 78:g} C{rcx + 120:g},{rcy - 130:g} {ccx - 150:g},{ccy - 40:g} {ccx - 108:g},{ccy - 16:g}",
          YELLOW, YELLOW_D, (ccx - 102, ccy - 13), 24)
    arrow(o, f"M{ccx + 104:g},{ccy - 22:g} C{ccx + 150:g},{ccy - 70:g} {bx0 - 60:g},{by0 + 40:g} {bx0 - 20:g},{by0 + 70:g}",
          YELLOW, YELLOW_D, (bx0 - 14, by0 + 74), 40)
    tag(o, (ccx + bx0) / 2 + 6, by0 - 22, "Trains at my station?", YELLOW, YELLOW_D)

    # orange data back: BART -> the sign
    arrow(o, f"M{bx0 + bw / 2:g},{by0 + bh + 18:g} C{bx0 + bw / 2:g},{by0 + bh + 150:g} {ex1 + 160:g},{(ey0 + ey1) / 2:g} {ex1 + 54:g},{(ey0 + ey1) / 2 + 10:g}",
          ORANGE, ORANGE_D, (ex1 + 46, (ey0 + ey1) / 2 + 11), 175)
    tag(o, bx0 + bw / 2 - 4, by0 + bh + 92, "Departures, in minutes", ORANGE, ORANGE_D)

    # captions
    caption(o, 36, dy0 + dh + 50, 1, "Your sign", ['An ESP32 and a 3.5" screen'])
    caption(o, 36, 196, 2, "Your Wi-Fi", ["Joins your home network", "(2.4 GHz)"])
    caption(o, 470, 352, 3, "BART's live departures", ["Asks for your station's trains", "once a minute, over HTTPS", "(every 5 min overnight)"])
    caption(o, 700, 672, 4, "On your desk", ["Draws them in the platform", "sign's own font, counting", "down between updates"])

    # first-time setup
    py0 = 790
    o.append(rect(36, py0, W - 72, 196, "#f3f5f7", rx=14))
    phone(o, 64, py0 - 20)
    wx, wy = 250, py0 + 70
    for kk, r in enumerate((12, 24, 36)):
        o.append(f'<path d="M{wx - r * 0.72:g},{wy - r * 0.7:g} A{r},{r} 0 0 1 {wx + r * 0.72:g},{wy - r * 0.7:g}" '
                 f'fill="none" stroke="{BLUE}" stroke-width="5" stroke-linecap="round" opacity="{1 - kk * 0.2:g}"/>')
    o.append(circle(wx, wy, 5, BLUE))
    o.append(text(wx, wy + 30, "BART-SIGN-XXXX", 13, INK, "bold", "middle"))
    x = 330
    o.append(text(x, py0 + 46, "First time", 24, INK, "bold"))
    lines = [("The sign makes its own Wi-Fi, ", "BART-SIGN-XXXX", "."),
             ("Join it on your phone; the setup page opens, or go to ", "192.168.4.1", "."),
             ("Pick your Wi-Fi network, your station and your line.", "", "")]
    for kk, (a, b, c) in enumerate(lines):
        y = py0 + 82 + kk * 32
        o.append(f'<text x="{x}" y="{y}" font-size="19" fill="{INK}">{esc(a)}'
                 f'<tspan font-weight="bold">{esc(b)}</tspan>{esc(c)}</text>')
    o.append(text(x, py0 + 176, "The sign restarts, joins your network and starts showing trains.", 16, MUTED))
    o.append("</svg>")
    return "\n".join(o) + "\n", n_lit


def render_png(svg_path, png_path):
    subprocess.run([CHROME, "--headless", "--disable-gpu", "--hide-scrollbars",
                    f"--window-size={W},{H}", "--force-device-scale-factor=2",
                    f"--screenshot={png_path}", "file://" + svg_path],
                   check=True, capture_output=True)


if __name__ == "__main__":
    svg, n = build()
    with open(OUT_SVG, "w") as f:
        f.write(svg)
    print(f"wrote {OUT_SVG} ({n} lit LEDs, {len(svg) // 1024} KB)")
    if "--png" in sys.argv:
        render_png(OUT_SVG, OUT_PNG)
        print(f"wrote {OUT_PNG}")
