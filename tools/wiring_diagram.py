"""Draw the wiring diagram: the ELEGOO ESP32 DevKit (top side) and the XIITIA
3.5" ST7796 display (back side, portrait, header at the top), joined by the
eight jumper wires. Writes docs/assets/wiring.svg, and with --png also
docs/assets/wiring.png (2x, rendered by headless Chrome).

    python3 -I tools/wiring_diagram.py [--png]

The pin map below must match firmware/src/bin/main.rs (`DisplayPins` and the
backlight channel on GPIO32). The script checks every route ends on the pin
the map names and counts wire crossings, so a layout edit can't silently
move a wire.
"""
import os
import shutil
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT_SVG = os.path.join(ROOT, "docs", "assets", "wiring.svg")
OUT_PNG = os.path.join(ROOT, "docs", "assets", "wiring.png")
CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"

W, H = 1200, 1135
FONT = "'Helvetica Neue', Helvetica, Arial, sans-serif"
BLUE = "#0064a8"   # bart.gov blue
INK = "#1b1b1b"
MUTED = "#5a5a5a"
SILK = "#f4f4f2"

# ---------------------------------------------------------------- the boards

# ELEGOO ESP-WROOM-32 30-pin DevKit, silk labels top (antenna) to bottom
# (USB-C), exactly as printed. "UP"/"UN" are VP/VN (GPIO36/39).
ESP_LEFT = ["EN", "UP", "UN", "D34", "D35", "D32", "D33", "D25",
            "D26", "D27", "D14", "D12", "D13", "GND", "VIN"]
ESP_RIGHT = ["D23", "D22", "TX0", "RX0", "D21", "D19", "D18", "D5",
             "TX2", "RX2", "D4", "D2", "D15", "GND", "3V3"]

# Display header, back view, left to right: pin 9 .. pin 1.
HEADER = ["SDA-O", "BL", "CS", "DC", "RST", "SDA", "SCL", "VCC", "GND"]

# display pin -> (ESP32 row, silk label, GPIO or None, role, colour, outline)
WIRES = {
    "GND": ("R", "GND", None, "Ground", "#111111", "#000000"),
    "VCC": ("R", "3V3", None, "Power, 3.3 V", "#D7261E", "#8E1711"),
    "SCL": ("R", "D18", 18, "SPI clock", "#FFD200", "#A88A00"),
    "SDA": ("R", "D23", 23, "SPI data", "#F7931E", "#A65A00"),
    "RST": ("L", "D33", 33, "Reset", "#8A5FBF", "#55327F"),
    "DC":  ("L", "D27", 27, "Data / command", "#0099CC", "#00607F"),
    "CS":  ("R", "D5", 5, "Chip select", "#339933", "#1D5E1D"),
    "BL":  ("L", "D32", 32, "Backlight (PWM)", "#8C8C8C", "#4F4F4F"),
}
LEGEND_ORDER = ["GND", "VCC", "SCL", "SDA", "RST", "DC", "CS", "BL"]

# ---------------------------------------------------------------- geometry

# ESP32, top view: antenna up, USB-C down. 28 x 51 mm, rows 25.4 mm apart,
# drawn at PITCH px per 2.54 mm.
PITCH = 30
MM = PITCH / 2.54
CX = 357.5                                    # board centre line
BX0, BX1, BY0, BY1 = CX - 14 * MM, CX + 14 * MM, 190, 190 + 51 * MM
PIN_LX, PIN_RX = CX - 150, CX + 150
PIN_Y0 = 287


def pin_y(i):
    return PIN_Y0 + PITCH * i


# Display, back view, portrait, header at the top. 59.19 x 94.84 mm.
DX0, DY0 = 800, 516
DX1, DY1 = DX0 + 360, DY0 + 360 * 94.84 / 59.19
HDR_PITCH = 24                     # drawn a little wider than 2.54 mm
HDR_Y = DY0 + 12
HDR_X0 = DX0 + 0.58 * 360 - HDR_PITCH * 4     # header sits centre-right


def hdr_x(name):
    return HDR_X0 + HDR_PITCH * HEADER.index(name)


# Routing. Every wire rises from its header pin to a lane, runs left to a
# column in the channel between the boards, then either drops or climbs to
# its pin on the ESP32's right row, or (left-row pins) passes under the
# board and climbs the far side. Lane order is the reverse of header order,
# so the header end stays crossing-free; the crossings all land in the
# channel, where each one is a clean right angle.
LANE_TOP, LANE_STEP = 302, 27
LANE_ORDER = ["GND", "VCC", "SCL", "SDA", "RST", "DC", "CS", "BL"]   # top to bottom
COL_X0, COL_STEP = 655, 14
COL_ORDER = ["SCL", "GND", "VCC", "CS", "SDA", "DC", "RST", "BL"]    # board side outward
UNDER = {"DC": (824, 62), "RST": (838, 48), "BL": (852, 34)}         # lane y, left column x
RADIUS = 7


def route(name):
    row, silk, *_ = WIRES[name]
    rows = ESP_RIGHT if row == "R" else ESP_LEFT
    ty = pin_y(rows.index(silk))
    x = hdr_x(name)
    ly = LANE_TOP + LANE_STEP * LANE_ORDER.index(name)
    cx = COL_X0 + COL_STEP * COL_ORDER.index(name)
    pts = [(x, HDR_Y), (x, ly), (cx, ly)]
    if row == "R":
        pts += [(cx, ty), (PIN_RX, ty)]
    else:
        uy, lx = UNDER[name]
        pts += [(cx, uy), (lx, uy), (lx, ty), (PIN_LX, ty)]
    return pts


def rounded(pts, r=RADIUS):
    """Orthogonal polyline -> SVG path with rounded corners."""
    d = [f"M{pts[0][0]:g},{pts[0][1]:g}"]
    for (x0, y0), (x1, y1), (x2, y2) in zip(pts, pts[1:], pts[2:]):
        def toward(ax, ay, bx, by, k):
            l = abs(bx - ax) + abs(by - ay)
            k = min(k, l / 2)
            return ax + (bx - ax) / l * k, ay + (by - ay) / l * k
        ax, ay = toward(x1, y1, x0, y0, r)
        bx, by = toward(x1, y1, x2, y2, r)
        d.append(f"L{ax:g},{ay:g}Q{x1:g},{y1:g} {bx:g},{by:g}")
    d.append(f"L{pts[-1][0]:g},{pts[-1][1]:g}")
    return "".join(d)


def crossings():
    segs = {n: list(zip(route(n), route(n)[1:])) for n in WIRES}
    out = []
    names = list(WIRES)
    for i, a in enumerate(names):
        for b in names[i + 1:]:
            for s in segs[a]:
                for t in segs[b]:
                    h, v = (s, t) if s[0][1] == s[1][1] else (t, s)
                    if h[0][1] != h[1][1] or v[0][0] != v[1][0]:
                        continue
                    lo, hi = sorted((h[0][0], h[1][0]))
                    vlo, vhi = sorted((v[0][1], v[1][1]))
                    if lo < v[0][0] < hi and vlo < h[0][1] < vhi:
                        out.append((a, b, v[0][0], h[0][1]))
    return out


def check():
    """Every wire must end on the pin WIRES names, at both ends."""
    for name, (row, silk, *_rest) in WIRES.items():
        pts = route(name)
        assert pts[0] == (hdr_x(name), HDR_Y), name
        rows = ESP_RIGHT if row == "R" else ESP_LEFT
        end = (PIN_RX if row == "R" else PIN_LX, pin_y(rows.index(silk)))
        assert pts[-1] == end, (name, pts[-1], end)
    assert "SDA-O" not in WIRES
    assert len(WIRES) == 8


# ---------------------------------------------------------------- svg helpers

def esc(s):
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace('"', "&quot;")


def text(x, y, s, size, fill=INK, weight="normal", anchor="start", extra=""):
    w = ' font-weight="bold"' if weight == "bold" else ""
    return (f'<text x="{x:g}" y="{y:g}" font-size="{size:g}"{w} fill="{fill}" '
            f'text-anchor="{anchor}"{extra}>{esc(s)}</text>')


def rtext(x, y, s, size, angle, fill=SILK, weight="bold", anchor="middle", extra=""):
    """Rotated text: -90 reads bottom to top, 90 reads top to bottom."""
    return text(x, y, s, size, fill, weight, anchor, f' transform="rotate({angle} {x:g} {y:g})"' + extra)


def rect(x, y, w, h, fill, rx=0, stroke=None, sw=1, extra=""):
    s = f' stroke="{stroke}" stroke-width="{sw}"' if stroke else ""
    return f'<rect x="{x:g}" y="{y:g}" width="{w:g}" height="{h:g}" rx="{rx:g}" fill="{fill}"{s}{extra}/>'


def circle(x, y, r, fill, stroke=None, sw=1, extra=""):
    s = f' stroke="{stroke}" stroke-width="{sw}"' if stroke else ""
    return f'<circle cx="{x:g}" cy="{y:g}" r="{r:g}" fill="{fill}"{s}{extra}/>'


# Text widths for sizing chips and callouts: measured with Helvetica Neue when
# Pillow and the macOS font are present, else a rough Helvetica Bold table.
# A few percent of slack covers Helvetica or Arial on other systems.
_ADV = {c: 0.72 for c in "ABCDEFGHKNORSUVXYZ"}
_ADV.update({c: 0.61 for c in "LPT"}, I=0.28, J=0.56, M=0.83, Q=0.78, W=0.94)
_ADV.update({c: 0.556 for c in "0123456789"}, **{"-": 0.33, " ": 0.28, "/": 0.28, ".": 0.28})
try:
    from PIL import ImageFont
    _HN = "/System/Library/Fonts/HelveticaNeue.ttc"
    _FACES = {True: ImageFont.truetype(_HN, 100, index=1), False: ImageFont.truetype(_HN, 100, index=0)}
except Exception:
    _FACES = None


def tw(s, size, bold=True):
    if _FACES:
        return _FACES[bold].getlength(s) * size / 100 * 1.04
    return sum(_ADV.get(c, 0.56) for c in s) * size * 1.08


def chip(cx, cy, label, size, minw=0, dim=False):
    """A station-sign label: black plate, white Helvetica Bold."""
    w = max(minw, tw(label, size) + size * 0.75)
    h = size * 1.42
    fill, ink = ("#e7e9eb", "#7f868c") if dim else ("#000", "#fff")
    return (rect(cx - w / 2, cy - h / 2, w, h, fill, rx=3)
            + text(cx, cy + size * 0.36, label, size, ink, "bold", "middle")), w, h


def badge(o, x, y, n):
    o.append(circle(x, y, 11.5, BLUE, "#fff", 2))
    o.append(text(x, y + 4.8, str(n), 14, "#fff", "bold", "middle"))


def hole(o, x, y, r=9.5):
    """Plated mounting hole: a bright ring, the page showing through."""
    o.append(circle(x, y, r, "#e2e4e6", "#9aa0a6", 1))
    o.append(circle(x, y, r * 0.55, "#ffffff", "#b9bec3", 0.8))


# ---------------------------------------------------------------- ESP32

def draw_esp32(o):
    o.append('<g id="esp32">')
    o.append('<defs>'
             '<linearGradient id="metal" x1="0" y1="0" x2="0" y2="1">'
             '<stop offset="0" stop-color="#eef0f2"/><stop offset="0.55" stop-color="#c3c7cc"/>'
             '<stop offset="1" stop-color="#9aa0a7"/></linearGradient>'
             '<linearGradient id="gold" x1="0" y1="0" x2="1" y2="1">'
             '<stop offset="0" stop-color="#f0d27a"/><stop offset="1" stop-color="#b8902e"/></linearGradient>'
             '</defs>')
    # PCB with four plated holes
    o.append(rect(BX0 + 2, BY0 + 5, BX1 - BX0, BY1 - BY0, "#000", rx=16, extra=' opacity="0.13"'))
    o.append(rect(BX0, BY0, BX1 - BX0, BY1 - BY0, "#141518", rx=16, stroke="#000", sw=1.5))
    for hx in (BX0 + 18, BX1 - 18):
        for hy in (BY0 + 20, BY1 - 20):
            hole(o, hx, hy)

    # ESP-WROOM-32: antenna PCB at the very top, then the black shield
    mx0, mx1, my0, my1 = CX - 106, CX + 106, BY0, BY0 + 301
    o.append(rect(mx0, my0, mx1 - mx0, my1 - my0, "#0d0e10", rx=3, stroke="#2a2d31"))
    ant = [f"M{mx0 + 22},{my0 + 66} V{my0 + 14}"]
    x = mx0 + 22
    for k in range(8):
        x += 21
        ant.append(f"H{x} V{my0 + (52 if k % 2 == 0 else 14)}")
    ant.append(f"H{x + 21} V{my0 + 66}")
    o.append(f'<path d="{" ".join(ant)}" fill="none" stroke="#3e4248" stroke-width="3" stroke-linejoin="round"/>')
    sx0, sx1, sy0, sy1 = CX - 98, CX + 98, my0 + 80, my1 - 9
    o.append(rect(sx0, sy0, sx1 - sx0, sy1 - sy0, "#1d1f23", rx=4, stroke="#3d4046", sw=1.5))
    o.append(rect(sx0 + 3, sy0 + 3, sx1 - sx0 - 6, sy1 - sy0 - 6, "none", rx=3, stroke="#2b2e33", sw=1))
    # printing on the shield
    o.append(rect(CX - 25, sy0 + 18, 50, 22, "none", rx=11, stroke=SILK, sw=1.6))
    o.append(text(CX, sy0 + 34, "Wi-Fi", 12, SILK, "bold", "middle", ' font-style="italic"'))
    o.append(text(CX, sy0 + 72, "ESP-WROOM-32", 19, SILK, "bold", "middle"))
    o.append(text(CX - 58, sy0 + 108, "CE", 20, SILK, "normal", "middle", ' letter-spacing="-1"'))
    o.append(circle(CX - 18, sy0 + 102, 8, "none", SILK, 1.4))
    o.append(text(CX - 18, sy0 + 106, "R", 10, SILK, "bold", "middle"))
    o.append(text(CX - 6, sy0 + 107, "211-161007", 12, SILK, "normal", "start"))
    o.append(text(CX, sy0 + 146, "FC", 22, SILK, "bold", "middle", ' font-style="italic" letter-spacing="-1"'))
    o.append(text(CX, sy0 + 178, "FCC ID:2AC7Z-ESP32WROOM32", 10.5, SILK, "normal", "middle"))
    # castellated pads along the module's bottom edge
    for k in range(10):
        px = CX - 90 + k * 20
        o.append(rect(px - 5, my1 - 7, 10, 10, "url(#gold)", rx=1.5))

    # small parts below the module
    def res(x, y, label, w=18, h=9):
        o.append(rect(x - w / 2, y - h / 2, w, h, "#0b0b0c", rx=1, stroke="#3a3c40", sw=0.8))
        o.append(rect(x - w / 2, y - h / 2, 3, h, "#b9bdc2"))
        o.append(rect(x + w / 2 - 3, y - h / 2, 3, h, "#b9bdc2"))
        o.append(text(x, y + 2.6, label, 6.5, "#c9ccd0", "bold", "middle"))
    t0 = my1 + 22
    o.append(rect(CX - 98, t0 - 2, 40, 22, "#e6a21a", rx=2, stroke="#9a6a07"))     # tantalum
    o.append(rect(CX - 98, t0 - 2, 8, 22, "#b77908", rx=1))
    o.append(text(CX - 74, t0 + 13, "106C", 9, "#5b3a00", "bold", "middle"))
    for (rx_, ry_, lab) in ((CX - 36, t0 + 4, "103"), (CX - 10, t0 + 4, "102"), (CX + 34, t0 + 4, "103"),
                            (CX + 70, t0 + 4, "103"), (CX + 86, t0 + 92, "102"), (CX - 6, t0 + 124, "103")):
        res(rx_, ry_, lab)
    # AMS1117 (SOT-223) and a small silver part to its left
    rg0 = t0 + 44
    o.append(rect(CX - 72, rg0 - 10, 38, 11, "#aeb3b9", rx=1))
    o.append(rect(CX - 80, rg0, 54, 32, "#0e0f11", rx=2, stroke="#3a3d42"))
    for k in range(3):
        o.append(rect(CX - 74 + k * 18, rg0 + 32, 6, 8, "#aeb3b9"))
    o.append(text(CX - 53, rg0 + 19, "AMS1117", 7, "#6f747a", "bold", "middle"))
    o.append(rect(CX - 104, rg0 + 6, 16, 22, "url(#metal)", rx=2, stroke="#7d838a", sw=0.8))
    # CP2102 (U2), a QFN
    q0 = t0 + 40
    o.append(rect(CX + 20, q0, 46, 46, "#0f1012", rx=2, stroke="#3a3d42"))
    for k in range(6):
        for (qx, qy, qw, qh) in ((CX + 26 + k * 6.5, q0 - 2, 3, 3), (CX + 26 + k * 6.5, q0 + 45, 3, 3),
                                 (CX + 18, q0 + 6 + k * 6.5, 3, 3), (CX + 65, q0 + 6 + k * 6.5, 3, 3)):
            o.append(rect(qx, qy, qw, qh, "#9ca1a7"))
    o.append(circle(CX + 27, q0 + 7, 2.2, "#34373b"))
    o.append(text(CX + 43, q0 + 64, "U2", 10, SILK, "bold", "middle"))
    # two tiny LEDs, the power LED lit
    o.append(circle(CX - 34, BY1 - 92, 9, "#ff3b24", extra=' opacity="0.28"'))
    o.append(rect(CX - 39, BY1 - 95, 10, 6, "#ff4a32", rx=1))
    o.append(rect(CX + 28, BY1 - 95, 10, 6, "#e9e3cf", rx=1, stroke="#9a9580", sw=0.6))

    # USB-C receptacle: metal shell, centred, sticking out past the edge
    ux0, ux1 = CX - 53, CX + 53
    o.append(rect(ux0, BY1 - 70, ux1 - ux0, 84, "url(#metal)", rx=7, stroke="#737a82", sw=1.5))
    o.append(rect(ux0 + 8, BY1 - 62, ux1 - ux0 - 16, 66, "none", rx=5, stroke="#8a9097", sw=1))
    o.append(f'<line x1="{ux0 + 1}" y1="{BY1}" x2="{ux1 - 1}" y2="{BY1}" stroke="#6c737a" stroke-width="1"/>')
    # EN and BOOT buttons
    for bx, label in ((CX - 95, "EN"), (CX + 95, "BOOT.")):
        o.append(rect(bx - 23, BY1 - 64, 46, 46, "#18191b", rx=3, stroke="#4b4e53", sw=1.2))
        for cx_, cy_ in ((bx - 19, BY1 - 60), (bx + 19, BY1 - 60), (bx - 19, BY1 - 22), (bx + 19, BY1 - 22)):
            o.append(circle(cx_, cy_, 2, "#8d9298"))
        o.append(circle(bx, BY1 - 41, 13, "#2a2c30", "#4d5056", 1.2))
        o.append(circle(bx - 3, BY1 - 44, 5, "#3a3d42"))
        o.append(text(bx, BY1 - 70, label, 11, SILK, "bold", "middle"))

    # header pins: gold rings, dark centres; silk labels rotated, just inside
    for row, labels, px in (("L", ESP_LEFT, PIN_LX), ("R", ESP_RIGHT, PIN_RX)):
        for i, lab in enumerate(labels):
            y = pin_y(i)
            o.append(circle(px, y, 9.5, "url(#gold)", "#8a6d22", 1))
            o.append(circle(px, y, 4.4, "#101010"))
            tx = px + 27 if row == "L" else px - 17
            o.append(rtext(tx, y, lab, 13, -90))
    o.append("</g>")


def draw_esp_tags(o):
    """Tags on the used pins only: label plate plus the GPIO number."""
    size, cw, gw = 16, 50, 62
    for n, (row, silk, gpio, role, colour, outline) in WIRES.items():
        rows = ESP_RIGHT if row == "R" else ESP_LEFT
        y = pin_y(rows.index(silk))
        cx = BX0 - 6 - cw / 2 if row == "L" else BX1 + 6 + cw / 2
        gx = cx - cw / 2 - gw if row == "L" else cx + cw / 2
        h = size * 1.42
        o.append(rect(gx, y - h / 2, gw, h, "#fff", rx=3, stroke="#000", sw=1.5))
        edge = gx if row == "L" else gx + gw - 6
        o.append(rect(edge, y - h / 2, 6, h, colour, extra=f' stroke="{outline}" stroke-width="1"'))
        g = f"GPIO{gpio}" if gpio is not None else ("power" if silk == "3V3" else "ground")
        o.append(text(gx + gw / 2 + (3 if row == "L" else -3), y + 5, g, 13.5, INK, "bold", "middle"))
        o.append(chip(cx, y, silk, size, minw=cw)[0])


# ---------------------------------------------------------------- display

FPC16 = ["BL"] + [f"D{k}" for k in range(15, -1, -1)] + ["RST", "RD", "WR", "DC", "CS", "VCC", "GND"]
FPC8 = ["BL"] + [f"D{k}" for k in range(7, -1, -1)] + ["RST", "RD", "WR", "DC", "CS", "VCC", "GND"]


def fpc(o, x0, y0, w, h, labels, label_y, label_dir):
    """White FPC socket with a row of tiny rotated pin labels."""
    o.append(rect(x0, y0, w, h, "#f5f2ea", rx=2, stroke="#c9c2ae", sw=1))
    o.append(rect(x0 + 5, y0 + h * 0.36, w - 10, h * 0.3, "#d8b866"))
    step = (w - 10) / (len(labels) - 1)
    for k, lab in enumerate(labels):
        x = x0 + 5 + k * step
        anchor = "start" if label_dir > 0 else "end"
        o.append(rtext(x - 2, label_y, lab, 5.6, 90, SILK, "bold", anchor, ' opacity="0.8"'))


def draw_display(o):
    o.append('<g id="display">')
    o.append('<defs><linearGradient id="kapton" x1="0" y1="0" x2="0" y2="1">'
             '<stop offset="0" stop-color="#dba33a"/><stop offset="1" stop-color="#bf7c1d"/></linearGradient></defs>')
    w, h = DX1 - DX0, DY1 - DY0

    def X(f):          # fraction of the board width
        return DX0 + w * f

    def Y(f):          # fraction of the board height, top = 0
        return DY0 + h * f

    o.append(rect(DX0 + 2, DY0 + 6, w, h, "#000", rx=12, extra=' opacity="0.13"'))
    o.append(rect(DX0, DY0, w, h, "#1f5fb0", rx=12, stroke="#14457f", sw=1.5))

    # trace bundles from about 45% down into the glass's flex
    tr = []
    for k in range(10):
        xs = X(0.10 + k * 0.035)
        xe = X(0.42 + k * 0.034)
        tr.append(f"M{xs:g},{Y(0.45):g} V{Y(0.50) + k * 2:g} L{xe:g},{Y(0.57) + k * 1.2:g} V{Y(0.62):g}")
    for k in range(8):
        xs = X(0.93 - k * 0.03)
        xe = X(0.74 - k * 0.025)
        tr.append(f"M{xs:g},{Y(0.45):g} V{Y(0.50) + k * 2:g} L{xe:g},{Y(0.57) + k * 1.2:g} V{Y(0.62):g}")
    for k in range(9):               # ZIF to the 8080-8P socket
        xs = X(0.44 + k * 0.035)
        xe = X(0.45 + k * 0.035)
        tr.append(f"M{xs:g},{Y(0.79):g} L{xe:g},{Y(0.83):g}")
    o.append(f'<path d="{" ".join(tr)}" fill="none" stroke="#174c90" stroke-width="1.5" opacity="0.85"/>')

    for hx in (DX0 + 16, DX1 - 16):
        for hy in (DY0 + 16, DY1 - 16):
            hole(o, hx, hy, 8.5)

    # 9-pin header at the very top edge, "9" and "SPI" at its ends
    hx0, hx1 = hdr_x("SDA-O") - 14, hdr_x("GND") + 14
    o.append(rect(hx0, DY0 + 2, hx1 - hx0, 20, "#121212", rx=2))
    for n in HEADER:
        x = hdr_x(n)
        o.append(rect(x - 5, HDR_Y - 5, 10, 10, "url(#gold)", rx=1, stroke="#8a6d22", sw=0.8))
    o.append(text(hx0 - 4, HDR_Y + 4, "9", 11, SILK, "bold", "end"))
    o.append(text(hx1 + 4, Y(0.075), "SPI", 11, SILK, "bold", "start"))
    # silk under each pin, reading top to bottom, down to about 9%
    for n in HEADER:
        o.append(rtext(hdr_x(n) - 3.4, DY0 + 25, n, 9, 90, anchor="start"))

    # 8080-16P socket at 10-13%, its tiny labels just below
    fpc(o, X(0.06), Y(0.10), w * 0.62, Y(0.13) - Y(0.10), FPC16, Y(0.136), 1)
    o.append(text(X(0.70), Y(0.127), "8080-16P", 11, SILK, "bold"))

    # frame tabs at 18% and 77% on both edges; "ST779.." under the top right one
    o.append(text(DX1 - 12, Y(0.18) + 4, "ST7796", 10.5, SILK, "bold", "end", ' opacity="0.85"'))
    for f in (0.18, 0.77):
        o.append(rect(DX0 - 3, Y(f) - 13, 12, 26, "#0f0f10", rx=1.5))
        o.append(rect(DX1 - (26 if f == 0.18 else 9), Y(f) - 13, 29 if f == 0.18 else 12, 26, "#0f0f10", rx=1.5))

    # 3.5"TFT at 21-24%, centred slightly right
    o.append(text(X(0.55), Y(0.24), '3.5"TFT', 28, SILK, "bold", "middle"))

    # IM jumper table, 30-42% on the left half; the SPI column is the setting
    tx0, tx1, ty0, ty1 = X(0.08), X(0.52), Y(0.30), Y(0.42)
    rh = (ty1 - ty0) / 4
    cw = (tx1 - tx0 - 58) / 4
    gx = tx1 - cw * 4
    cols = ["P8", "P16", "SPI", "SPI3"]
    vals = {"IM0": "1011", "IM1": "1110", "IM2": "0011"}
    for k in (1, 2, 3):
        o.append(rect(gx + cw * k + cw / 2 - 3.5, Y(0.29) - 4, 7, 7, SILK))
    spi = gx + cw * 2
    o.append(rect(spi + 1, ty0 + 1, cw - 2, ty1 - ty0 - 2, "#fff", rx=2))
    o.append(text(tx0 + 19, ty0 + rh - 4, "0", 8, SILK, "bold", "middle"))
    o.append(text(tx0 + 29, ty0 + rh - 4, "1", 8, SILK, "bold", "middle"))
    for k, c in enumerate(cols):
        o.append(text(gx + cw * k + cw / 2, ty0 + rh - 4, c, 9.5, BLUE if c == "SPI" else SILK, "bold", "middle"))
    for r, (im, v) in enumerate(vals.items()):
        yb = ty0 + rh * (r + 2) - 4
        o.append(rect(tx0 - 14, yb - 8, 14, 7, "#0b0b0c", rx=1))
        o.append(text(tx0 - 7, yb - 2.6, "103", 4.6, "#c9ccd0", "bold", "middle"))
        o.append(rect(tx0 + 16, yb - 8, 6, 7, "#d8b866", rx=1))
        o.append(rect(tx0 + 26, yb - 8, 6, 7, "#d8b866", rx=1))
        o.append(text(tx0 + 46, yb, im, 9.5, SILK, "bold", "middle"))
        for k, ch in enumerate(v):
            hi = cols[k] == "SPI"
            o.append(text(gx + cw * k + cw / 2, yb, ch, 10.5 if hi else 9.5, BLUE if hi else SILK, "bold", "middle"))
    grid = [f"M{gx + cw * k:g},{ty0:g} V{ty1:g}" for k in range(5)]
    grid += [f"M{tx0 + 34:g},{ty0 + rh * r:g} H{tx1:g}" for r in range(1, 5)]
    o.append(f'<path d="{" ".join(grid)}" fill="none" stroke="{SILK}" stroke-width="0.9" opacity="0.7"/>')

    # U8 (SOT-23-3) at 30-36% and U7 (SOT-23) at 37-42%, at 80-90% of the width
    def sot(x, y, name):
        o.append(rect(x - 9, y - 5.5, 18, 11, "#0e0f11", rx=1.5, stroke="#3a3d42", sw=0.8))
        for lx in (x - 5.5, x + 5.5):
            o.append(rect(lx - 1.5, y + 5.5, 3, 3, "#aeb3b9"))
        o.append(rect(x - 1.5, y - 8.5, 3, 3, "#aeb3b9"))
        o.append(text(x - 12, y + 3, name, 7, SILK, "bold", "end"))

    def part(x, y, name, res=False, side=-1):
        o.append(rect(x - 4.5, y - 2.5, 9, 5, "#0b0b0c" if res else "#c8a27a", rx=0.8, stroke="#3d3a33", sw=0.5))
        o.append(text(x + side * 7, y + 2, name, 5.8, SILK, "bold", "end" if side < 0 else "start"))
    sot(X(0.85), Y(0.33), "U8")
    part(X(0.79), Y(0.305), "C17")
    part(X(0.92), Y(0.305), "C18", side=1)
    part(X(0.79), Y(0.355), "C20")
    part(X(0.92), Y(0.355), "C19", side=1)
    sot(X(0.85), Y(0.398), "U7")
    part(X(0.79), Y(0.376), "C16")
    part(X(0.92), Y(0.376), "R41", True, side=1)
    part(X(0.79), Y(0.42), "R39", True)
    part(X(0.92), Y(0.42), "R40", True, side=1)
    part(X(0.92), Y(0.398), "R18", True, side=1)

    # the glass's flex, 62-74%, into its ZIF (black, white base) at 75-79%
    rx0, rx1 = X(0.38), X(0.77)
    o.append(rect(rx0, Y(0.62), rx1 - rx0, Y(0.755) - Y(0.62), "url(#kapton)", stroke="#8e5a12", sw=1))
    for k in range(22):
        lx = rx0 + 6 + k * (rx1 - rx0 - 12) / 21
        o.append(f'<line x1="{lx:g}" y1="{Y(0.62) + 1:g}" x2="{lx:g}" y2="{Y(0.75):g}" stroke="#a8691a" stroke-width="0.9" opacity="0.55"/>')
    o.append(text(rx0 + 8, Y(0.715), "40", 10, "#5e3906", "bold", "start"))
    o.append(text(rx1 - 8, Y(0.715), "1", 10, "#5e3906", "bold", "end"))
    o.append(rect(rx0 - 10, Y(0.75), rx1 - rx0 + 20, Y(0.775) - Y(0.75), "#111", rx=1.5))
    o.append(rect(rx0 - 8, Y(0.775), rx1 - rx0 + 16, Y(0.79) - Y(0.775), "#f2f0ea", rx=1))

    # 8080-8P socket at the very bottom (88-93%), labels just above, name to its left
    fpc(o, X(0.40), Y(0.88), w * 0.40, Y(0.93) - Y(0.88), FPC8, Y(0.87), -1)
    o.append(text(X(0.38), Y(0.915), "8080-8P", 11, SILK, "bold", "end"))
    o.append("</g>")


def draw_display_tags(o):
    """Readable chips on each wire just after it leaves its pin, stepped
    up the lanes; SDA-O, which has no wire, gets a dim chip on the board."""
    size = 14
    for n in WIRES:
        x = hdr_x(n)
        ly = LANE_TOP + LANE_STEP * LANE_ORDER.index(n)
        colour, outline = WIRES[n][4], WIRES[n][5]
        wch = max(36, tw(n, size) + size * 0.75)
        cx = x - 14 - wch / 2
        s, wc, hc = chip(cx, ly, n, size, minw=36)
        o.append(rect(cx - wc / 2 - 2, ly - hc / 2 - 2, wc + 4, hc + 4, colour, rx=4,
                      extra=f' stroke="{outline}" stroke-width="1"'))
        o.append(s)
    s, wc, hc = chip(DX0 + 52, DY0 + 46, "SDA-O", 12, dim=True)
    o.append(s)
    o.append(f'<path d="M{DX0 + 52 + wc / 2:g},{DY0 + 46} H{hdr_x("SDA-O") - 6:g}" stroke="#fff" stroke-width="1.2" opacity="0.8"/>')


# ---------------------------------------------------------------- wires, legend

def draw_wires(o):
    o.append('<g id="wires" fill="none" stroke-linecap="round" stroke-linejoin="round">')
    # Paint order decides which wire passes on top at a crossing: a wire that
    # crosses a parallel group goes on top, so it stays one unbroken line.
    for n in ["BL", "DC", "RST", "VCC", "GND", "CS", "SCL", "SDA"]:
        colour, outline = WIRES[n][4], WIRES[n][5]
        d = rounded(route(n))
        o.append(f'<g data-wire="{n}">')
        o.append(f'<path d="{d}" stroke="#fff" stroke-width="13"/>')
        o.append(f'<path d="{d}" stroke="{outline}" stroke-width="8"/>')
        o.append(f'<path d="{d}" stroke="{colour}" stroke-width="5.4"/>')
        o.append("</g>")
    o.append("</g>")
    # wire ends: a dot on each pin, the gold ring still showing round it
    for n in WIRES:
        colour, outline = WIRES[n][4], WIRES[n][5]
        for (x, y) in (route(n)[0], route(n)[-1]):
            o.append(circle(x, y, 6, colour, outline, 2))


def callout(o, x, y, n, line):
    size = 17
    w = tw(line, size) + 56
    o.append(rect(x, y, w, 38, "#fff", rx=5, stroke="#d5d9de", sw=1))
    o.append(rect(x, y, 5, 38, BLUE, rx=1))
    badge(o, x + 24, y + 19, n)
    o.append(text(x + 44, y + 25, line, size, INK, "bold"))


def draw_legend(o):
    x0, y0 = 740, 14
    o.append(text(x0, y0 + 14, "CONNECTIONS", 14, BLUE, "bold", "start", ' letter-spacing="1.5"'))
    o.append(text(x0 + 60, y0 + 38, "Display", 13, MUTED, "bold"))
    o.append(text(x0 + 142, y0 + 38, "ESP32", 13, MUTED, "bold"))
    o.append(text(x0 + 284, y0 + 38, "What it does", 13, MUTED, "bold"))
    rh = 25
    for i, n in enumerate(LEGEND_ORDER):
        row, silk, gpio, role, colour, outline = WIRES[n]
        y = y0 + 60 + rh * i
        o.append(f'<line x1="{x0 + 4}" y1="{y}" x2="{x0 + 44}" y2="{y}" stroke="{outline}" stroke-width="8" stroke-linecap="round"/>')
        o.append(f'<line x1="{x0 + 4}" y1="{y}" x2="{x0 + 44}" y2="{y}" stroke="{colour}" stroke-width="5.4" stroke-linecap="round"/>')
        o.append(text(x0 + 60, y + 7, n, 19, INK, "bold"))
        o.append(text(x0 + 124, y + 6, "→", 18, MUTED, "normal", "middle"))
        o.append(text(x0 + 142, y + 7, silk, 19, INK, "bold"))
        if gpio is not None:
            o.append(text(x0 + 190, y + 6, f"GPIO{gpio}", 14, MUTED, "bold"))
        o.append(text(x0 + 284, y + 6.5, role, 16.5, INK))
    y = y0 + 60 + rh * 8
    o.append(f'<line x1="{x0 + 4}" y1="{y}" x2="{x0 + 44}" y2="{y}" stroke="#b9bec3" stroke-width="3" stroke-dasharray="1 6" stroke-linecap="round"/>')
    o.append(text(x0 + 60, y + 7, "SDA-O", 19, "#868c92", "bold"))
    o.append(text(x0 + 142, y + 6.5, "not connected, leave it empty", 16.5, "#868c92"))


def build():
    check()
    xs = crossings()
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}" '
         f'role="img" aria-labelledby="wiring-title wiring-desc" font-family="{FONT}">']
    o.append('<title id="wiring-title">Wiring: ESP32 board to 3.5-inch ST7796S display</title>')
    desc = ("Eight female-to-female jumper wires connect the display's 9-pin SPI header, seen from the back "
            "with the header at the top, to the ESP32 DevKit, seen from the top with the USB-C port at the bottom. "
            + "; ".join(f"display {n} to ESP32 {WIRES[n][1]}"
                        + (f" (GPIO{WIRES[n][2]})" if WIRES[n][2] is not None else "")
                        for n in LEGEND_ORDER)
            + ". Display pin SDA-O is not connected. The SCL and SDA pins are SPI, not I2C. "
              "The IM0, IM1 and IM2 jumpers on the display must all be 1 (the SPI column). "
              "Use a USB-C cable that carries data.")
    o.append(f'<desc id="wiring-desc">{esc(desc)}</desc>')
    o.append(rect(0, 0, W, H, "#ffffff"))
    # title
    o.append(rect(36, 30, 64, 7, BLUE))
    o.append(text(34, 86, "Wiring", 46, INK, "bold"))
    o.append(text(36, 112, "8 female-to-female jumper wires", 19, MUTED))
    draw_esp32(o)
    draw_display(o)
    draw_wires(o)
    draw_display_tags(o)
    draw_esp_tags(o)
    draw_legend(o)
    # board captions
    o.append(text(BX1 + 14, BY0 + 24, "ESP32 board", 16, INK, "bold"))
    o.append(text(BX1 + 14, BY0 + 44, "top side", 15, MUTED))
    o.append(text(DX0, DY1 + 26, "Display", 16, INK, "bold"))
    o.append(text(DX0 + 64, DY1 + 26, "back side", 15, MUTED))
    # numbered markers on the boards, explained in the cards below
    badge(o, hdr_x("GND") + 34, LANE_TOP + LANE_STEP * 2, 1)
    badge(o, DX0 + 16, DY0 + (DY1 - DY0) * 0.285, 2)
    badge(o, CX - 72, BY1 + 10, 3)
    y = UNDER["BL"][0] + 22
    callout(o, 30, y, 1, "SCL/SDA here means SPI, not I2C")
    callout(o, 30, y + 48, 2, "Check IM0–IM2 jumpers are on 1 (SPI)")
    callout(o, 30, y + 96, 3, "USB-C: use a cable that carries data")
    o.append("</svg>")
    return "\n".join(o) + "\n", xs


def render_png(svg_path, png_path):
    tmp = tempfile.mkdtemp()
    html = os.path.join(tmp, "w.html")
    with open(svg_path) as f:
        svg = f.read()
    with open(html, "w") as f:
        f.write(f"<!doctype html><html><head><style>html,body{{margin:0;background:#fff}}"
                f"svg{{display:block}}</style></head><body>{svg}</body></html>")
    subprocess.run([CHROME, "--headless", "--disable-gpu", "--hide-scrollbars",
                    f"--window-size={W},{H}", "--force-device-scale-factor=2",
                    f"--screenshot={png_path}", "file://" + html],
                   check=True, capture_output=True)
    shutil.rmtree(tmp)


if __name__ == "__main__":
    svg, xs = build()
    with open(OUT_SVG, "w") as f:
        f.write(svg)
    print(f"wrote {OUT_SVG} ({len(xs)} crossings, all in the channel: "
          f"{all(x < DX0 for _, _, x, _ in xs)})")
    if "--png" in sys.argv:
        render_png(OUT_SVG, OUT_PNG)
        print(f"wrote {OUT_PNG}")
