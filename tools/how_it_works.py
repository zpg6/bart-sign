"""Draw the "How it works" illustration: the sign, your Wi-Fi, the internet
and BART's live departures, plus first-time setup from a phone. Writes
docs/assets/how-it-works.svg, and with --png also docs/assets/how-it-works.png
(2x, rendered by headless Chrome).

    python3 -I tools/how_it_works.py [--png] [--out DIR]

Everything is vector. The sign on the screen is rebuilt LED by LED from the
firmware's own render, docs/assets/screens/LAFY-EVENING-0.png: each of the
176 x 48 LEDs per sign is sampled at its centre and drawn as a dot, so the
picture is the real sign, not a lookalike. The ESP32 is the drawing from
tools/wiring_diagram.py at the screen's mm scale, turned on its side so the
Wi-Fi lanes leave its antenna end. Facts in the captions follow the firmware: a fetch every 60 s (every 5 min once service has ended for the
night), HTTPS to api.bart.gov, and the setup network BART-SIGN-XXXX with its
page at 192.168.4.1.
"""
import importlib.util
import os
import subprocess
import sys

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
# In the repo this file lives in tools/, so the repo root is one level up.
# Anywhere else, point BART_SIGN_ROOT at a checkout. --out DIR overrides
# where the SVG and PNG go (default: <root>/docs/assets).
ROOT = os.environ.get("BART_SIGN_ROOT") or os.path.dirname(HERE)
OUT_DIR = os.path.join(ROOT, "docs", "assets")
if "--out" in sys.argv:
    OUT_DIR = os.path.abspath(sys.argv[sys.argv.index("--out") + 1])
OUT_SVG = os.path.join(OUT_DIR, "how-it-works.svg")
OUT_PNG = os.path.join(OUT_DIR, "how-it-works.png")
SCREEN_PNG = os.path.join(ROOT, "docs", "assets", "screens", "LAFY-EVENING-0.png")

_spec = importlib.util.spec_from_file_location("wiring_diagram", os.path.join(HERE, "wiring_diagram.py"))
wd = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(wd)
text, rect, circle, esc, tw = wd.text, wd.rect, wd.circle, wd.esc, wd.tw
BLUE, INK, MUTED, FONT, CHROME = wd.BLUE, wd.INK, wd.MUTED, wd.FONT, wd.CHROME

W, H = 720, 1290
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


# ---------------------------------------------------------------- the grid
#
# One left margin and one right margin for everything: the title's blue bar,
# the BART card, the First time panel and the display all sit on them.
ML, MR = 20, W - 20
CX = 110                         # the spine's centre line
LANE = 20                        # each lane sits this far either side of it
YX, OX = CX - LANE, CX + LANE    # yellow asks go down, orange answers come up
RX = 228                         # caption column: badge left edge
TX = RX + 40                     # caption column: heading and body
CW = MR - TX                     # caption measure
CLEAR = 10                       # every arrow stops this far short of a thing
HEAD, BODY, LEAD = 25, 20, 26    # caption type
LABEL = 18                       # labels: the hardware under the display, the lane key
C1_CLEAR = 25                    # that label's baseline to caption 1's cap top
CAP = 0.714                      # Helvetica Neue cap height, em
H2B = 30                         # heading baseline to first body baseline


# ---------------------------------------------------------------- spine objects
# Each returns (or has a helper for) its outline where the two lanes cross
# it, so the arrows can all stop the same distance short.

ROUTER_S = 1.1


def router(o, cx, top):
    """The router with `top` at the crown of its outer wave (stroke included).
    Returns (bottom, centre y, top at the lanes)."""
    s = ROUTER_S
    cy = top + s * 81                       # local crown is cy - 81
    o.append(f'<g transform="translate({cx:g},{cy:g}) scale({s:g})">')
    for sx in (-1, 1):
        o.append(f'<line x1="{sx * 50}" y1="0" x2="{sx * 62}" y2="-62" stroke="#2b2d31" '
                 f'stroke-width="7" stroke-linecap="round"/>')
    o.append(rect(-82, -6, 164, 54, "#2b2d31", rx=12))
    o.append(rect(-78, -2, 156, 30, "#ffffff", rx=9))
    for k in range(4):
        o.append(circle(-42 + k * 18, 13, 4, "#3fbf5a" if k < 3 else "#d8dbde"))
    for k, r in enumerate((20, 36, 52)):
        o.append(f'<path d="M{-r * 0.72:g},{-26 - r * 0.7:g} A{r},{r} 0 0 1 {r * 0.72:g},{-26 - r * 0.7:g}" '
                 f'fill="none" stroke="{BLUE}" stroke-width="6" stroke-linecap="round" opacity="{1 - k * 0.22:g}"/>')
    o.append(circle(0, -30, 6, BLUE))
    o.append("</g>")
    import math
    dx = LANE / s
    lane_top = cy + s * (-26 - math.sqrt(55 ** 2 - dx ** 2))
    bottom = cy + s * 48
    return bottom, cy + s * 21, lane_top     # 21: the middle of the router's box


def router_body_mid():
    """Crown of the waves to the middle of the box: the waves are its signal,
    like the Wi-Fi mark under the ESP32, so captions centre on the box."""
    return ROUTER_S * (81 + 21)


# A cloud that is symmetric where the lanes cross it (one dome in the middle,
# the same bump under each lane), so both arrows of a hop are equally long.
BUMPS = [(-64, 10, 30), (-36, -10, 36), (0, -22, 46), (38, -8, 38), (66, 12, 30),
         (0, 16, 38), (-38, 22, 28), (38, 22, 28)]
CLOUD_F = 0.8


def cloud_edge(cx, cy, f, x, top=True):
    """Where a vertical line at x meets the cloud's outer outline."""
    import math
    best = None
    for dx, dy, r in BUMPS:
        bx, by, rr = cx + dx * f, cy + dy * f, (r + 3) * f
        if abs(x - bx) < rr:
            h = math.sqrt(rr * rr - (x - bx) ** 2)
            y = by - h if top else by + h
            if best is None or (y < best if top else y > best):
                best = y
    return best


def cloud_box(f=CLOUD_F):
    top = min(dy - r - 3 for dx, dy, r in BUMPS) * f
    bot = max(dy + r + 3 for dx, dy, r in BUMPS) * f
    return top, bot


def cloud(o, cx, cy, label, f=CLOUD_F):
    for dx, dy, r in BUMPS:
        o.append(circle(cx + dx * f, cy + dy * f, (r + 3) * f, "#b9c6d6"))
    for dx, dy, r in BUMPS:
        o.append(circle(cx + dx * f, cy + dy * f, r * f, "#eef3f9"))
    o.append(text(cx, cy + 19 * CAP / 2 + 2, label, 19, INK, "bold", "middle"))


CARD_W, CARD_H = MR - ML - 500, 178  # 180 wide: the same span as the router


def bart_card(o, x0, y0, w=CARD_W, h=CARD_H):
    o.append(rect(x0 + 3, y0 + 6, w, h, "#000", rx=10, extra=' opacity="0.10"'))
    o.append(rect(x0, y0, w, h, "#fff", rx=10, stroke="#cfd5db", sw=1.5))
    o.append(f'<path d="M{x0},{y0 + 42} V{y0 + 10} Q{x0},{y0} {x0 + 10},{y0} H{x0 + w - 10} '
             f'Q{x0 + w},{y0} {x0 + w},{y0 + 10} V{y0 + 42}Z" fill="{BLUE}"/>')
    pad = 14
    o.append(text(x0 + pad, y0 + 17, "BART", 11, "#cfe3f3", "bold", "start", ' letter-spacing="2"'))
    o.append(text(x0 + pad, y0 + 34, "api.bart.gov", 15, "#fff", "bold"))
    o.append(text(x0 + pad, y0 + 66, "Lafayette", 17, INK, "bold"))
    o.append(text(x0 + w - pad, y0 + 66, "min", 11.5, MUTED, "bold", "end"))
    rows = [("SF Airport", "4, 12"), ("SFO/Millbrae", "32, 52"), ("Pittsburg", "6, 26"), ("Antioch", "14, 34")]
    for k, (dest, mins) in enumerate(rows):
        y = y0 + 88 + k * 20
        o.append(circle(x0 + pad + 5, y - 5, 5, YELLOW, YELLOW_D, 1.2))
        o.append(text(x0 + pad + 16, y, dest, 14, INK))
        o.append(text(x0 + w - pad, y, mins, 14, INK, "bold", "end"))
    y = y0 + h - 15
    for k, (c, _n) in enumerate(LINES):
        o.append(circle(x0 + w / 2 + (k - 2) * 18, y, 6, c, "#00000033", 1))


PHONE_S = 0.82
PHONE_W, PHONE_H = 116 * PHONE_S, 188 * PHONE_S


def phone(o, x0, y0):
    o.append(f'<g transform="translate({x0:g},{y0:g}) scale({PHONE_S:g})">')
    w, h = 116, 188
    o.append(rect(2, 5, w, h, "#000", rx=20, extra=' opacity="0.14"'))
    o.append(rect(0, 0, w, h, "#1b1c1f", rx=20))
    sx0, sy0, sw, sh = 7, 16, w - 14, h - 28
    o.append(rect(sx0, sy0, sw, sh, "#fff", rx=12))
    o.append(rect(w / 2 - 18, 6, 36, 5, "#3a3c40", rx=2.5))
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
    o.append("</g>")


# ---------------------------------------------------------------- arrows

SHAFT = 6
HEAD_L, HEAD_W = 20, 12          # arrowhead length and half-width, everywhere
CAP_R = (SHAFT + 2.6) / 2        # visible radius of a shaft's round tail


def arrow(o, x, y_tail, y_tip, colour, outline, width=SHAFT):
    """A thick wire-style vertical arrow, white halo, dark edge, head at y_tip."""
    s = 1 if y_tip > y_tail else -1
    d = f"M{x:g},{y_tail:g} L{x:g},{y_tip - s * (HEAD_L - 6):g}"
    o.append(f'<path d="{d}" fill="none" stroke="#fff" stroke-width="{width + 6}" stroke-linecap="round"/>')
    o.append(f'<path d="{d}" fill="none" stroke="{outline}" stroke-width="{width + 2.6}" stroke-linecap="round"/>')
    o.append(f'<path d="{d}" fill="none" stroke="{colour}" stroke-width="{width}" stroke-linecap="round"/>')
    o.append(f'<path d="M0,0 L{-HEAD_L},{-HEAD_W} L{-HEAD_L},{HEAD_W}Z" '
             f'transform="translate({x:g},{y_tip:g}) rotate({90 if s > 0 else -90})" '
             f'fill="{colour}" stroke="{outline}" stroke-width="2" stroke-linejoin="round"/>')


def hop(o, y_from, y_to, lengths):
    """Both lanes between two things: `y_from` is the upper thing's lower
    outline at the lanes, `y_to` the lower thing's upper outline. Each arrow
    stops CLEAR short of both, so a hop's two arrows match."""
    a, b = y_from + CLEAR + CAP_R, y_to - CLEAR - 1
    arrow(o, YX, a, b, YELLOW, YELLOW_D)
    arrow(o, OX, y_to - CLEAR - CAP_R, y_from + CLEAR + 1, ORANGE, ORANGE_D)
    lengths.append(round(b + 1 - a + CAP_R, 1))


# ---------------------------------------------------------------- type

def rich_lines(parts, size, width):
    """Wrap [(text, bold)] runs into lines of runs; a NBSP keeps words together.
    Captions pass an unbounded width: their lines break where the text says."""
    words, prev_space = [], True
    for t, b in parts:
        for k, w_ in enumerate(t.split(" ")):
            if w_ == "":
                continue
            words.append((w_, b, k == 0 and not t.startswith(" ") and not prev_space))
        prev_space = t.endswith(" ")
    lines, cur, cw = [], [], 0.0
    for w_, b, glued in words:
        add = tw(w_, size, b) + (0 if (glued and cur) or not cur else tw(" ", size, False))
        if cur and not glued and cw + add > width:
            lines.append(cur)
            cur, cw = [], 0.0
            add = tw(w_, size, b)
        cur.append((w_, b, glued and bool(cur)))
        cw += add
    lines.append(cur)
    return lines


def rich_text(o, x, y, line, size, fill, bold_fill=None):
    """One line of (word, bold, glued) runs; bold runs can take their own ink."""
    spans, first = [], True
    bf = f' fill="{bold_fill}"' if bold_fill else ""
    for w_, b, glued in line:
        sep = "" if first or glued else " "
        first = False
        spans.append(sep + (f'<tspan font-weight="bold"{bf}>{esc(w_)}</tspan>' if b else esc(w_)))
    o.append(f'<text x="{x:g}" y="{y:g}" font-size="{size:g}" fill="{fill}">{"".join(spans)}</text>')


def num(o, x, y, n, r=15):
    o.append(circle(x, y, r, BLUE, "#fff", 2))
    o.append(text(x, y + r * 0.42, str(n), r * 1.2, "#fff", "bold", "middle"))


GAP = 8                          # extra space between two thoughts in a caption


def fits(line, size, width):
    """A line of (text, bold) runs, measured with the real face (tw pads 4%)."""
    return sum(tw(t, size, b) for t, b in line) / 1.04 <= width


def caption_height(body):
    """Heading cap top to the last body baseline. `body` is a list of lines,
    each a list of (text, bold) runs, with None for a break between thoughts."""
    n = sum(1 for ln in body if ln is not None)
    return HEAD * CAP + H2B + (n - 1) * LEAD + GAP * body.count(None)


def caption(o, top, n, head, body):
    """Badge, heading and body in the caption column; `top` is the heading's
    cap top. The badge is centred on the heading's capitals. Lines break
    where the text says, never by the wrapper: bold leads in ink, the rest
    muted."""
    hb = top + HEAD * CAP
    num(o, RX + 15, top + HEAD * CAP / 2, n)
    o.append(text(TX, hb, head, HEAD, INK, "bold"))
    y = hb + H2B
    for ln in body:
        if ln is None:
            y += GAP
            continue
        assert fits(ln, BODY, CW), ln
        (words,) = rich_lines(ln, BODY, 10 ** 6)
        rich_text(o, TX, y, words, BODY, MUTED, INK)
        y += LEAD
    return y - LEAD


KEY_SIZE = LABEL                 # lane key chips and their verbs
KEY_PITCH = 46                   # chip centre to chip centre
KEY_H = KEY_SIZE * 1.6
KEY_NOTE = 36                    # note baseline to the first chip's centre
KEY_NOTE_TEXT = [("Out over the internet", True), (" and back the same way", False)]


def lane_key_height():
    """Note cap top to the lower chip's bottom edge (its coloured rim)."""
    return BODY * CAP + KEY_NOTE + KEY_PITCH + KEY_H / 2 + 2


def lane_key(o, top):
    """The trip named once, beside the cloud: a one-line note, then a key.
    Each key row is a short lane arrow on the badge axis, the lane's verb on
    the text edge and a station-sign chip with what it carries."""
    nb = top + BODY * CAP
    assert fits(KEY_NOTE_TEXT, BODY, CW)
    (words,) = rich_lines(KEY_NOTE_TEXT, BODY, 10 ** 6)
    rich_text(o, TX, nb, words, BODY, MUTED, INK)
    rows = [("Asks", "Trains at my station?", YELLOW, YELLOW_D, 1),
            ("Answers", "Departures, in minutes", ORANGE, ORANGE_D, -1)]
    size, h = KEY_SIZE, KEY_H
    w = max(tw(r[1], size) for r in rows) / 1.04 + 24   # one width: it is a key
    cx0 = TX + max(tw(r[0], size) for r in rows) / 1.04 + 14
    for k, (verb, label, colour, outline, s) in enumerate(rows):
        y = nb + KEY_NOTE + k * KEY_PITCH
        half = 18
        arrow(o, RX + 15, y - s * (half - CAP_R), y + s * (half - 1), colour, outline)
        o.append(text(TX, y + size * CAP / 2, verb, size, INK, "bold"))
        o.append(rect(cx0 - 2, y - h / 2 - 2, w + 4, h + 4, colour, rx=5, stroke=outline, sw=1))
        o.append(rect(cx0, y - h / 2, w, h, "#000", rx=4))
        o.append(text(cx0 + 12, y + size * CAP / 2, label, size, "#fff", "bold"))
    assert cx0 + w + 2 <= MR


# ---------------------------------------------------------------- ESP32

def esp32_landscape(o, x0, y0, s):
    """The wiring diagram's ESP32 at scale `s`, turned a quarter turn
    counter-clockwise: antenna end on the left, USB-C on the right. Its own
    drop shadow is dropped (it would fall sideways) and redrawn below."""
    bw = s * (wd.BY1 - wd.BY0)            # landscape width  = 51 mm
    bh = s * (wd.BX1 - wd.BX0)            # landscape height = 28 mm
    bcx, bcy = (wd.BX0 + wd.BX1) / 2, (wd.BY0 + wd.BY1) / 2
    o.append(rect(x0 + 2, y0 + 5, bw, bh, "#000", rx=16 * s, extra=' opacity="0.13"'))
    g = []
    wd.draw_esp32(g)
    g = [e for e in g if 'opacity="0.13"' not in e]
    o.append(f'<g transform="translate({x0 + bw / 2:g},{y0 + bh / 2:g}) rotate(-90) scale({s:g}) '
             f'translate({-bcx:g},{-bcy:g})">')
    o.extend(g)
    o.append("</g>")
    return bw, bh


def wifi_mark(o, cx, top, r=12, colour=BLUE):
    """A small Wi-Fi mark: two arcs and a dot, `top` is the outer arc's crown.
    Returns its bottom."""
    for k, rr in enumerate((r, r * 0.55)):
        cy = top + r
        o.append(f'<path d="M{cx - rr * 0.72:g},{cy - rr * 0.7:g} A{rr:g},{rr:g} 0 0 1 {cx + rr * 0.72:g},{cy - rr * 0.7:g}" '
                 f'fill="none" stroke="{colour}" stroke-width="3.2" stroke-linecap="round" opacity="{1 - k * 0.2:g}"/>')
    o.append(circle(cx, top + r, 2.8, colour))
    return top + r + 2.8


# ---------------------------------------------------------------- the picture

NB = "\u00a0"
HEAD_1, TEXT_1 = "Your sign asks", [
    [("Once a minute", True), (" over HTTPS,", False)],
    [(f"every 5{NB}min overnight", False)],
    None,
    [("Draws the answer", True), (" in the platform sign's font,", False)],
    [("counting down between updates", False)]]
HEAD_2, TEXT_2 = "Your Wi-Fi", [
    [("Joins your ", False), (f"2.4{NB}GHz", True), (" home network", False)]]
HEAD_3, TEXT_3 = "BART answers", [
    [("Live departures", True), (" for your station,", False)],
    [("from api.bart.gov", False)]]


def build():
    global H
    o = ["<svg>"]
    o.append('<title id="hiw-title">How it works</title>')
    desc = ("Your sign, an ESP32 board wired to a 3.5-inch screen, joins your home Wi-Fi (2.4 GHz). "
            "Once a minute, every 5 minutes overnight, the ESP32 asks BART's live departures service at "
            "api.bart.gov for your station's trains over HTTPS: the question goes from the ESP32 through your "
            "router and the internet to BART, and the answer comes back the same way to the ESP32, which draws "
            "it on the screen in the platform sign's own font and counts the minutes down between updates. "
            "The first time, the sign makes its own Wi-Fi network, BART-SIGN-XXXX; join it on your phone, "
            "open 192.168.4.1 if the page doesn't appear, and pick your Wi-Fi network, station and line.")
    o.append(f'<desc id="hiw-desc">{esc(desc)}</desc>')
    bg = "<bg>"
    o.append(bg)
    top0 = 24                                  # the blue bar, and the display's top edge
    o.append(rect(ML, top0, 56, 6, BLUE))
    o.append(text(ML - 2.6, 72, "How it works", 40, INK, "bold"))
    o.append(text(ML - 1, 102, "From BART's data to your desk", 20, MUTED))

    # ---- the sign: the screen is the hero, top right beside the title
    k = 352 / 94.84
    dx0, dy0 = MR - 352, top0
    dw, dh, n_lit, pins = display_front(o, dx0, dy0, k)
    hx, hy0, hy1 = pins
    # The ESP32 at the same mm scale, on its side with the antenna end over
    # the spine, as low as a straight bundle into the header allows.
    s_esp = k / wd.MM
    bw_e = s_esp * (wd.BY1 - wd.BY0)
    bh_e = s_esp * (wd.BX1 - wd.BX0)
    ex0 = CX - 40
    ex1 = ex0 + bw_e
    bnd = 20
    off = (bh_e / 2 - s_esp * 53) / 2      # bundle centre below the board's top edge
    ey0 = min(hy1 - bnd / 2 - 4 - off, 168)
    ey1 = ey0 + bh_e
    by = ey0 + off
    x_a, x_b = ex1 - 40, hx
    o.append(rect(x_a, by - bnd / 2 - 3, x_b - x_a, bnd + 6, "#fff"))
    o.append(rect(x_a, by - bnd / 2, x_b - x_a, bnd, "#2b2d31"))
    for kk in range(8):
        yy = by - bnd / 2 + 2.5 + kk * (bnd - 5) / 7
        o.append(f'<line x1="{x_a:g}" y1="{yy:g}" x2="{x_b:g}" y2="{yy:g}" stroke="#5b5f66" stroke-width="1"/>')
    o.append(rect(dx0 + 1.2 * k, hy0, 3.6 * k, hy1 - hy0, "#151515", rx=2))
    esp32_landscape(o, ex0, ey0, s_esp)
    # the bundle's label, right-aligned to the gap the cable crosses
    lx = dx0 - 12
    o.append(text(lx, ey0 - 31, "8 jumper wires", 15, INK, "bold", "end"))
    o.append(text(lx, ey0 - 13, "see the wiring diagram", 14, MUTED, "normal", "end"))

    # the hardware, named under the screen in the wiring diagram's
    # "Display  back side" style: on the display's left edge
    lb = dy0 + dh + 23                       # cap top 10 below the board, 4 below its shadow
    o.append(text(dx0, lb, "The hardware", LABEL, INK, "bold"))
    o.append(text(dx0 + tw("The hardware", LABEL) / 1.04 + 7, lb, 'an ESP32 and a 3.5" screen', LABEL, MUTED))
    assert dx0 + (tw("The hardware", LABEL) + tw('an ESP32 and a 3.5" screen', LABEL, False)) / 1.04 + 7 <= MR

    # ---- 1  your sign: hangs under the screen it describes
    c1_top = max(lb + C1_CLEAR, ey1 + 20)    # badge clears the board's corner
    c1_end = caption(o, c1_top, 1, HEAD_1, TEXT_1)

    # ---- the spine. Every hop has the same gap between things, measured at
    # the lanes; it is set by the first caption, so caption 2 can sit on the
    # router's centre line.
    e_bot = wifi_mark(o, CX, ey1 + 6, 16)
    c2_top = c1_end + 32
    c2_mid = c2_top + caption_height(TEXT_2) / 2
    r_top = c2_mid - router_body_mid()
    # the router's lane outline is a little below its crown; hop gap at the lanes
    tmp = []
    _, _, r_lane = router(tmp, CX, r_top)
    G = r_lane - e_bot
    lengths = []

    r_bot, r_mid, r_lane = router(o, CX, r_top)
    caption(o, r_mid - caption_height(TEXT_2) / 2, 2, HEAD_2, TEXT_2)
    hop(o, e_bot, r_lane, lengths)

    # the internet: its lane outline G below the router's bottom
    cy0 = 0
    c_top_rel = cloud_edge(CX, cy0, CLOUD_F, YX, True)
    cy_ = r_bot + G - c_top_rel
    cloud(o, CX, cy_, "The internet")
    ct, cb = cloud_box()
    c_mid = cy_ + (ct + cb) / 2
    c_lane_top = cloud_edge(CX, cy_, CLOUD_F, YX, True)
    c_lane_bot = cloud_edge(CX, cy_, CLOUD_F, YX, False)
    assert abs(c_lane_top - cloud_edge(CX, cy_, CLOUD_F, OX, True)) < 0.01
    assert abs(c_lane_bot - cloud_edge(CX, cy_, CLOUD_F, OX, False)) < 0.01
    hop(o, r_bot, c_lane_top, lengths)
    lane_key(o, c_mid - lane_key_height() / 2)

    # ---- 3  BART answers
    card_y = c_lane_bot + G
    bart_card(o, ML, card_y)
    hop(o, c_lane_bot, card_y, lengths)
    caption(o, card_y + CARD_H / 2 - caption_height(TEXT_3) / 2, 3, HEAD_3, TEXT_3)

    # ---- first time: the phone on the spine's axis; the heading on the
    # badges' left edge, the step numbers on the badge axis, the steps on the
    # caption text edge
    py0 = card_y + CARD_H + 36
    pad = 18
    steps = [[("Join ", False), ("BART-SIGN-XXXX", True), (" on your phone", False)],
             [("The setup page opens (or go to ", False), ("192.168.4.1", True), (")", False)],
             [("Pick your Wi-Fi network, station and line", False)]]
    foot = [("Then the sign restarts and shows your trains.", False)]
    for ln in steps:
        assert fits(ln, BODY, MR - pad - TX), ln
    assert fits(foot, BODY, MR - pad - RX)
    S_H2B, S_LEAD, S_FOOT = 40, 34, 38      # heading to step 1, step pitch, last step to note
    text_h = HEAD * CAP + S_H2B + (len(steps) - 1) * S_LEAD + S_FOOT
    ph = max(PHONE_H + 5, text_h) + 2 * pad
    o.append(rect(ML, py0, MR - ML, ph, "#f3f5f7", rx=14))
    phone(o, CX - PHONE_W / 2, py0 + (ph - PHONE_H - 5) / 2)
    t = py0 + (ph - text_h) / 2
    hb = t + HEAD * CAP
    o.append(text(RX, hb, "First time", HEAD, INK, "bold"))
    o.append(text(RX + tw("First time", HEAD) / 1.04 + 12, hb, "the sign makes its own Wi-Fi", BODY, MUTED))
    y = hb + S_H2B
    for k, ln in enumerate(steps):
        # ink, not BART blue, and smaller: they can't be read as steps 4-6
        o.append(circle(RX + 15, y - BODY * CAP / 2, 12, INK))
        o.append(text(RX + 15, y - BODY * CAP / 2 + 14 * CAP / 2, str(k + 1), 14, "#fff", "bold", "middle"))
        (words,) = rich_lines(ln, BODY, 10 ** 6)
        rich_text(o, TX, y, words, BODY, INK)
        y += S_LEAD
    y += S_FOOT - S_LEAD
    o.append(text(RX, y, foot[0][0], BODY, MUTED))
    Hn = py0 + ph + 20
    H = round(Hn)
    o[0] = (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}" '
            f'role="img" aria-labelledby="hiw-title hiw-desc" font-family="{FONT}">')
    o[o.index(bg)] = rect(0, 0, W, H, "#ffffff")
    o.append("</svg>")
    print("hop gap G", round(G, 1), "arrow lengths", lengths)
    return "\n".join(o) + "\n", n_lit, Hn


def render_png(svg_path, png_path):
    subprocess.run([CHROME, "--headless", "--disable-gpu", "--hide-scrollbars",
                    f"--window-size={W},{H}", "--force-device-scale-factor=2",
                    f"--screenshot={png_path}", "file://" + svg_path],
                   check=True, capture_output=True)


if __name__ == "__main__":
    svg, n, Hn = build()
    print("content height", Hn)
    with open(OUT_SVG, "w") as f:
        f.write(svg)
    print(f"wrote {OUT_SVG} ({n} lit LEDs, {len(svg) // 1024} KB)")
    if "--png" in sys.argv:
        render_png(OUT_SVG, OUT_PNG)
        print(f"wrote {OUT_PNG}")
