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

W, H = 1376, 872
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
    for hx in (x0 + 3.3 * k, x0 + w - 3.3 * k):
        for hy in (y0 + 3.3 * k, y0 + h - 3.3 * k):
            wd.hole(o, hx, hy, 2.1 * k)
    # glass and black surround, then the active area (73.44 x 48.96 mm)
    gx0, gy0, gw, gh = x0 + 7.6 * k, y0 + 2.4 * k, 80.4 * k, 54.4 * k
    o.append(rect(gx0, gy0, gw, gh, "#0b0b0d", rx=3, stroke="#2a2b2f", sw=1))
    ax0, ay0 = gx0 + 3.4 * k, gy0 + 2.7 * k
    s = 73.44 * k / 480
    o.append(f'<g transform="translate({ax0:g},{ay0:g}) scale({s:.5f})">')
    n = sign_svg(o)
    o.append("</g>")
    # a soft reflection across the glass
    o.append(f'<path d="M{gx0 + gw * 0.55:g},{gy0:g} L{gx0 + gw * 0.72:g},{gy0:g} L{gx0 + gw * 0.42:g},{gy0 + gh:g} '
             f'L{gx0 + gw * 0.25:g},{gy0 + gh:g}Z" fill="#fff" opacity="0.05"/>')
    # The 9-pin header sits on the left short edge (the back view's
    # "centre-right" header, seen from the front and turned landscape: about
    # 39-77% of the edge). Drawn as a plain strip: this picture isn't about pins.
    hy0, hy1 = y0 + h * 0.36, y0 + h * 0.80
    o.append(rect(x0 + 1.2 * k, hy0, 3.6 * k, hy1 - hy0, "#151515", rx=2))
    pins = (x0 + 3 * k, hy0, hy1)
    return w, h, n, pins


def router(o, cx, cy):
    for sx in (-1, 1):
        o.append(f'<line x1="{cx + sx * 50}" y1="{cy}" x2="{cx + sx * 62}" y2="{cy - 62}" stroke="#2b2d31" '
                 f'stroke-width="7" stroke-linecap="round"/>')
    o.append(rect(cx - 82, cy - 6, 164, 54, "#2b2d31", rx=12))
    o.append(rect(cx - 78, cy - 2, 156, 30, "#ffffff", rx=9))
    for k in range(4):
        o.append(circle(cx - 42 + k * 18, cy + 13, 4, "#3fbf5a" if k < 3 else "#d8dbde"))
    for k, r in enumerate((20, 36, 52)):
        o.append(f'<path d="M{cx - r * 0.72:g},{cy - 26 - r * 0.7:g} A{r},{r} 0 0 1 {cx + r * 0.72:g},{cy - 26 - r * 0.7:g}" '
                 f'fill="none" stroke="{BLUE}" stroke-width="6" stroke-linecap="round" opacity="{1 - k * 0.22:g}"/>')
    o.append(circle(cx, cy - 30, 6, BLUE))


def cloud(o, cx, cy, label, f=1.0):
    bumps = [(-62, 8, 34), (-26, -18, 42), (22, -26, 46), (62, 0, 36), (0, 14, 40), (-40, 20, 28), (40, 20, 30)]
    for dx, dy, r in bumps:
        o.append(circle(cx + dx * f, cy + dy * f, r * f + 3, "#b9c6d6"))
    for dx, dy, r in bumps:
        o.append(circle(cx + dx * f, cy + dy * f, r * f, "#eef3f9"))
    o.append(text(cx, cy + 6, label, 19, INK, "bold", "middle"))


def bart_card(o, x0, y0, w, h):
    o.append(rect(x0 + 3, y0 + 6, w, h, "#000", rx=11, extra=' opacity="0.10"'))
    o.append(rect(x0, y0, w, h, "#fff", rx=11, stroke="#cfd5db", sw=1.5))
    o.append(f'<path d="M{x0},{y0 + 46} V{y0 + 11} Q{x0},{y0} {x0 + 11},{y0} H{x0 + w - 11} '
             f'Q{x0 + w},{y0} {x0 + w},{y0 + 11} V{y0 + 46}Z" fill="{BLUE}"/>')
    o.append(text(x0 + 16, y0 + 20, "BART", 12.5, "#cfe3f3", "bold", "start", ' letter-spacing="2"'))
    o.append(text(x0 + 16, y0 + 38, "api.bart.gov", 16, "#fff", "bold"))
    o.append(text(x0 + 16, y0 + 72, "Lafayette", 18, INK, "bold"))
    o.append(text(x0 + w - 14, y0 + 72, "min", 12, MUTED, "bold", "end"))
    rows = [("SF Airport", "4, 12"), ("SFO/Millbrae", "32, 52"), ("Pittsburg", "6, 26"), ("Antioch", "14, 34")]
    for k, (dest, mins) in enumerate(rows):
        y = y0 + 98 + k * 24
        o.append(circle(x0 + 22, y - 5, 5.5, YELLOW, YELLOW_D, 1.2))
        o.append(text(x0 + 34, y, dest, 14.5, INK))
        o.append(text(x0 + w - 14, y, mins, 14.5, INK, "bold", "end"))
    y = y0 + h - 18
    for k, (c, _n) in enumerate(LINES):
        o.append(circle(x0 + w / 2 + (k - 2) * 20, y, 6.5, c, "#00000033", 1))


def phone(o, x0, y0):
    w, h = 116, 188
    o.append(rect(x0 + 2, y0 + 5, w, h, "#000", rx=20, extra=' opacity="0.14"'))
    o.append(rect(x0, y0, w, h, "#1b1c1f", rx=20))
    sx0, sy0, sw, sh = x0 + 7, y0 + 16, w - 14, h - 28
    o.append(rect(sx0, sy0, sw, sh, "#fff", rx=12))
    o.append(rect(x0 + w / 2 - 18, y0 + 6, 36, 5, "#3a3c40", rx=2.5))
    o.append(text(sx0 + 7, sy0 + 17, "Set up your sign", 10.5, INK, "bold"))
    fields = [("Wi-Fi network", "Choose a network"), ("Station", "Lafayette"), ("Line", "All lines")]
    for k, (lab, val) in enumerate(fields):
        y = sy0 + 32 + k * 33
        o.append(text(sx0 + 7, y, lab, 8.5, MUTED, "bold"))
        o.append(rect(sx0 + 7, y + 4, sw - 14, 17, "#f3f5f7", rx=3.5, stroke="#c9cfd5", sw=1))
        o.append(text(sx0 + 12, y + 16, val, 8.5, INK))
        o.append(f'<path d="M{sx0 + sw - 17},{y + 10} l3.5,3.5 l3.5,-3.5" fill="none" stroke="{MUTED}" stroke-width="1.3"/>')
    o.append(rect(sx0 + 7, sy0 + sh - 30, sw - 14, 20, BLUE, rx=4))
    o.append(text(sx0 + sw / 2, sy0 + sh - 16.5, "Save and connect", 8.5, "#fff", "bold", "middle"))


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


def caption(o, x, y, n, head, lines):
    wd.badge(o, x + 12, y - 7, n)
    o.append(text(x + 31, y, head, 21, INK, "bold"))
    for k, s in enumerate(lines):
        o.append(text(x + 31, y + 26 + k * 22, s, 17.5, MUTED))


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

    # One row, left to right: the ESP32 (antenna up), the display it drives,
    # your router, the internet, BART. Yellow asks along the top, orange
    # answers along the bottom; only the ESP32 touches the screen.
    yc = 302                                     # the row's centre line
    s_esp = 0.36
    ex0, ey0 = 40, yc - s_esp * (wd.BY1 - wd.BY0) / 2
    etx, ety = ex0 - s_esp * wd.BX0, ey0 - s_esp * wd.BY0
    ex1, ey1 = etx + s_esp * wd.BX1, ety + s_esp * wd.BY1
    ecx = (ex0 + ex1) / 2
    k = 3.4
    dw, dh = 94.84 * k, 59.19 * k
    dx0, dy0 = ex1 + 124, yc - dh / 2
    dw, dh, n_lit, pins = display_front(o, dx0, dy0, k)
    # the eight jumpers as one neutral bundle: from under the ESP32 into the
    # header strip; which wire goes where is the wiring diagram's job
    hx, hy0, hy1 = pins
    by = (hy0 + hy1) / 2
    bw_ = 26
    x_a, x_b = ex1 - 40, hx
    o.append(rect(x_a, by - bw_ / 2 - 3, x_b - x_a, bw_ + 6, "#fff"))
    o.append(rect(x_a, by - bw_ / 2, x_b - x_a, bw_, "#2b2d31"))
    for kk in range(8):
        yy = by - bw_ / 2 + 3 + kk * (bw_ - 6) / 7
        o.append(f'<line x1="{x_a:g}" y1="{yy:g}" x2="{x_b:g}" y2="{yy:g}" stroke="#5b5f66" stroke-width="1.2"/>')
    o.append(rect(dx0 + 1.2 * k, hy0, 3.6 * k, hy1 - hy0, "#151515", rx=2))     # header strip on top
    mx = (ex1 + dx0) / 2
    o.append(rect(mx - 52, by - 19, 104, 38, "#000", rx=4, stroke="#fff", sw=1.5))
    o.append(text(mx, by - 2, "8 jumper wires", 12.5, "#fff", "bold", "middle"))
    o.append(text(mx, by + 12, "see Wiring", 10.5, "#c9ccd0", "normal", "middle"))
    o.append(f'<g transform="translate({etx:g},{ety:g}) scale({s_esp:g})">')
    wd.draw_esp32(o)
    o.append("</g>")
    for kk, r in enumerate((10, 19)):     # Wi-Fi from the antenna end
        o.append(f'<path d="M{ecx - r * 0.7:g},{ey0 - 6 - r * 0.6:g} A{r},{r} 0 0 1 {ecx + r * 0.7:g},{ey0 - 6 - r * 0.6:g}" '
                 f'fill="none" stroke="{BLUE}" stroke-width="3.5" stroke-linecap="round" opacity="{0.9 - kk * 0.25:g}"/>')

    hop = 66
    rs = 0.72
    r_l = dx0 + dw + 90
    rcx = r_l + 82 * rs
    o.append(f'<g transform="translate({rcx:g},{yc:g}) scale({rs:g}) translate(0,-21)">')
    router(o, 0, 0)
    o.append("</g>")
    r_r = rcx + 82 * rs
    cf = 0.74
    ccx = r_r + hop + 99 * cf
    cloud(o, ccx, yc, "The internet", cf)
    c_l, c_r = ccx - 99 * cf, ccx + 99 * cf
    bx0, bw, bh = c_r + hop, 196, 222
    by0 = yc - bh / 2
    bart_card(o, bx0, by0, bw, bh)

    # yellow: up from the antenna, over the display, into the router; then on
    ytop, ybot, r = dy0 - 44, dy1 + 44 if (dy1 := dy0 + dh) else 0, 10
    up = (f"M{ecx:g},{ey0 - 34:g} L{ecx:g},{ytop + r:g} Q{ecx:g},{ytop:g} {ecx + r:g},{ytop:g} "
          f"L{r_l - 40 - r:g},{ytop:g} Q{r_l - 40:g},{ytop:g} {r_l - 40:g},{ytop + r:g} "
          f"L{r_l - 40:g},{yc - 14 - r:g} Q{r_l - 40:g},{yc - 14:g} {r_l - 40 + r:g},{yc - 14:g} L{r_l - 16:g},{yc - 14:g}")
    arrow(o, up, YELLOW, YELLOW_D, (r_l - 6, yc - 14), 0, 6)
    # orange: out of the router, under the display, up into the ESP32
    back_x = ecx + 22
    down = (f"M{r_l - 6:g},{yc + 14:g} L{r_l - 40 + r:g},{yc + 14:g} Q{r_l - 40:g},{yc + 14:g} {r_l - 40:g},{yc + 14 + r:g} "
            f"L{r_l - 40:g},{ybot - r:g} Q{r_l - 40:g},{ybot:g} {r_l - 40 - r:g},{ybot:g} "
            f"L{back_x + r:g},{ybot:g} Q{back_x:g},{ybot:g} {back_x:g},{ybot - r:g} L{back_x:g},{ey1 + 30:g}")
    arrow(o, down, ORANGE, ORANGE_D, (back_x, ey1 + 22), -90, 6)
    for a_, b_ in ((r_r, c_l), (c_r, bx0)):
        arrow(o, f"M{a_ + 6:g},{yc - 14:g} L{b_ - 16:g},{yc - 14:g}", YELLOW, YELLOW_D, (b_ - 6, yc - 14), 0, 6)
        arrow(o, f"M{b_ - 6:g},{yc + 14:g} L{a_ + 16:g},{yc + 14:g}", ORANGE, ORANGE_D, (a_ + 6, yc + 14), 180, 6)
    # each lane labelled once, on the lane itself
    tag(o, dx0 + dw / 2, ytop, "Trains at my station?", YELLOW, YELLOW_D, 15)
    tag(o, dx0 + dw / 2, ybot, "Departures, in minutes", ORANGE, ORANGE_D, 15)
    hx = (r_r + c_l) / 2
    o.append(text(hx, yc + 52, "your home", 13, MUTED, "bold", "middle"))
    o.append(text(hx, yc + 68, "internet", 13, MUTED, "bold", "middle"))

    # three steps, left to right on one baseline
    cy_ = ybot + 64
    caption(o, 30, cy_, 1, "Your sign", ['An ESP32 and a 3.5" screen. Once a minute it asks',
                                          "BART for your station's trains over HTTPS (every",
                                          "5 min overnight), then draws the answer in the",
                                          "platform sign's font and counts down between updates."])
    caption(o, rcx - 72, cy_, 2, "Your Wi-Fi", ["Joins your home", "network (2.4 GHz)"])
    caption(o, bx0 - 6, cy_, 3, "BART answers", ["Live departures for", "your station, from", "api.bart.gov"])

    # first-time setup
    py0 = 642
    o.append(rect(36, py0, W - 72, 200, "#f3f5f7", rx=14))
    phone(o, 64, py0 + 6)
    wx, wy = 252, py0 + 92
    for kk, r in enumerate((12, 24, 36)):
        o.append(f'<path d="M{wx - r * 0.72:g},{wy - r * 0.7:g} A{r},{r} 0 0 1 {wx + r * 0.72:g},{wy - r * 0.7:g}" '
                 f'fill="none" stroke="{BLUE}" stroke-width="5" stroke-linecap="round" opacity="{1 - kk * 0.2:g}"/>')
    o.append(circle(wx, wy, 5, BLUE))
    o.append(text(wx, wy + 28, "BART-SIGN-XXXX", 13, INK, "bold", "middle"))
    x = 340
    o.append(text(x, py0 + 48, "First time", 24, INK, "bold"))
    lines = [("The sign makes its own Wi-Fi, ", "BART-SIGN-XXXX", "."),
             ("Join it on your phone; the setup page opens, or go to ", "192.168.4.1", "."),
             ("Pick your Wi-Fi network, your station and your line.", "", "")]
    for kk, (a, b, c) in enumerate(lines):
        y = py0 + 84 + kk * 32
        o.append(f'<text x="{x}" y="{y}" font-size="20" fill="{INK}">{esc(a)}'
                 f'<tspan font-weight="bold">{esc(b)}</tspan>{esc(c)}</text>')
    o.append(text(x, py0 + 178, "The sign restarts, joins your network and starts showing trains.", 16, MUTED))
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
