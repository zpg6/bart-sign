"""Draw the wiring diagram: ESP32 DevKit (top view) to the ST7796S display
(back view), eight jumper wires. Writes docs/assets/wiring.svg, and with
--png also docs/assets/wiring.png (2x, rendered by headless Chrome).

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

W, H = 1200, 850
FONT = "'Helvetica Neue', Helvetica, Arial, sans-serif"
BLUE = "#0064a8"   # bart.gov blue
INK = "#1b1b1b"
MUTED = "#5a5a5a"

# ---------------------------------------------------------------- the boards

# ELEGOO ESP-WROOM-32 30-pin DevKit, silk labels top (antenna) to bottom (USB-C).
ESP_LEFT = ["EN", "VP", "VN", "D34", "D35", "D32", "D33", "D25",
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

# ESP32, top view: antenna up, USB-C down.
PITCH = 32
PIN_Y0 = 214
BX0, BX1, BY0, BY1 = 190, 440, 122, 722      # board outline
PIN_LX, PIN_RX = 208, 422                     # pin columns


def pin_y(i):
    return PIN_Y0 + PITCH * i


# Display, back view, header along the top edge.
DX0, DX1, DY0, DY1 = 684, 1172, 458, 763      # 94.84 x 59.19 mm, landscape
HDR_PITCH = 52
HDR_Y = DY0 + 16
HDR_X0 = (DX0 + DX1) / 2 + 6 - HDR_PITCH * 4


def hdr_x(name):
    return HDR_X0 + HDR_PITCH * HEADER.index(name)


# Routing. Every wire rises from its header pin to a lane, runs left to a
# column in the channel between the boards, then either drops or climbs to
# its pin on the ESP32's right row, or (left-row pins) passes under the
# board and climbs the far side. Lane order is the reverse of header order,
# so the header end stays crossing-free; the crossings all land in the
# channel, where each one is a clean right angle.
LANE_TOP, LANE_STEP = 330, 14
LANE_ORDER = ["GND", "VCC", "SCL", "SDA", "RST", "DC", "CS", "BL"]   # top to bottom
COL_X0, COL_STEP = 568, 14
COL_ORDER = ["SCL", "GND", "VCC", "CS", "SDA", "DC", "RST", "BL"]    # board side outward
UNDER = {"DC": (758, 62), "RST": (772, 48), "BL": (786, 34)}         # lane y, left column x
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
    return (f'<text x="{x:g}" y="{y:g}" font-size="{size}"{w} fill="{fill}" '
            f'text-anchor="{anchor}"{extra}>{esc(s)}</text>')


def rect(x, y, w, h, fill, rx=0, stroke=None, sw=1, extra=""):
    s = f' stroke="{stroke}" stroke-width="{sw}"' if stroke else ""
    return f'<rect x="{x:g}" y="{y:g}" width="{w:g}" height="{h:g}" rx="{rx:g}" fill="{fill}"{s}{extra}/>'


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


def chip(cx, cy, label, size, dim=False, minw=0):
    """A station-sign label: black plate, white Helvetica Bold."""
    w = max(minw, tw(label, size) + size * 0.75)
    h = size * 1.42
    fill, ink = ("#eceef0", "#a3a8ad") if dim else ("#000", "#fff")
    return (rect(cx - w / 2, cy - h / 2, w, h, fill, rx=3)
            + text(cx, cy + size * 0.36, label, size, ink, "bold", "middle"))


# ---------------------------------------------------------------- drawing

def draw_esp32(o):
    o.append('<g id="esp32">')
    # PCB
    o.append(rect(BX0 + 2, BY0 + 4, BX1 - BX0, BY1 - BY0, "#000", rx=12, extra=' opacity="0.12"'))
    o.append(rect(BX0, BY0, BX1 - BX0, BY1 - BY0, "#1c1e21", rx=12, stroke="#000", sw=1.5))
    # ESP-WROOM-32 module: its own PCB with the antenna meander, then the can.
    mx0, mx1, my0, my1 = 236, 394, 128, 376
    o.append(rect(mx0, my0, mx1 - mx0, my1 - my0, "#111316", rx=3, stroke="#2c2f33"))
    ant = "M248,176 V140 H262 V170 H276 V140 H290 V170 H304 V140 H318 V170 H332 V140 H346 V170 H360 V140 H374 V176"
    o.append(f'<path d="{ant}" fill="none" stroke="#b8963e" stroke-width="3" stroke-linejoin="round" opacity="0.85"/>')
    o.append('<defs><linearGradient id="can" x1="0" y1="0" x2="1" y2="1">'
             '<stop offset="0" stop-color="#e4e6e9"/><stop offset="1" stop-color="#b4b8be"/></linearGradient></defs>')
    o.append(rect(244, 190, 142, 178, "url(#can)", rx=3, stroke="#8f949a"))
    o.append(text(315, 262, "ESP-WROOM-32", 13, "#7d8288", "bold", "middle", ' letter-spacing="0.5"'))
    o.append(text(315, 282, "ESPRESSIF", 10, "#959aa0", "bold", "middle", ' letter-spacing="1.5"'))
    # castellated module pads along the can
    for i in range(10):
        y = 200 + i * 17
        o.append(rect(mx0 + 1, y, 5, 8, "#b8963e", extra=' opacity="0.7"'))
        o.append(rect(mx1 - 6, y, 5, 8, "#b8963e", extra=' opacity="0.7"'))
    # AMS1117 regulator and CP2102 USB-UART, drawn quietly
    o.append(rect(246, 452, 34, 26, "#0b0c0d", rx=2, stroke="#34373b"))
    o.append(rect(253, 446, 20, 6, "#9a9ea4"))
    o.append(rect(292, 560, 46, 46, "#0b0c0d", rx=3, stroke="#34373b"))
    o.append('<circle cx="299" cy="567" r="2.2" fill="#3a3d41"/>')
    # USB-C, buttons, power LED
    o.append(rect(285, 700, 60, 30, "#c3c7cc", rx=11, stroke="#7e848a", sw=1.5))
    o.append(rect(295, 711, 40, 8, "#3a3d41", rx=4))
    for bx, label in ((226, "EN"), (374, "BOOT")):
        o.append(rect(bx, 684, 30, 26, "#d9dcdf", rx=3, stroke="#8f949a"))
        o.append(f'<circle cx="{bx + 15}" cy="697" r="8" fill="#2a2c2f"/>')
        o.append(text(bx + 15, 676, label, 10, "#c9ccd0", "bold", "middle"))
    o.append(rect(268, 690, 9, 6, "#e0322a", rx=1.5))
    o.append(f'<circle cx="272.5" cy="693" r="7" fill="#ff3b24" opacity="0.25"/>')
    # header pins
    used = {(WIRES[n][0], WIRES[n][1]) for n in WIRES}
    for row, labels, px in (("L", ESP_LEFT, PIN_LX), ("R", ESP_RIGHT, PIN_RX)):
        for i, lab in enumerate(labels):
            y = pin_y(i)
            on = (row, lab) in used
            o.append(rect(px - 8, y - 8, 16, 16, "#d6b250" if on else "#8a7a4c", rx=2))
            o.append(f'<circle cx="{px}" cy="{y}" r="3.6" fill="#1c1e21"/>')
    o.append("</g>")


def draw_esp_labels(o):
    """Pin chips just outside the board; GPIO numbers for the used pins."""
    used = {(WIRES[n][0], WIRES[n][1]): n for n in WIRES}
    size, cw, gw = 16, 50, 62
    for row, labels in (("L", ESP_LEFT), ("R", ESP_RIGHT)):
        for i, lab in enumerate(labels):
            y = pin_y(i)
            name = used.get((row, lab))
            cx = BX0 - 6 - cw / 2 if row == "L" else BX1 + 6 + cw / 2
            if name is None:
                o.append(chip(cx, y, lab, size, dim=True, minw=cw))
                continue
            gpio, colour, outline = WIRES[name][2], WIRES[name][4], WIRES[name][5]
            # GPIO plate on the outer side, coloured edge where the wire comes in
            gx = cx - cw / 2 - gw if row == "L" else cx + cw / 2
            h = size * 1.42
            o.append(rect(gx, y - h / 2, gw, h, "#fff", rx=3, stroke="#000", sw=1.5))
            edge = gx if row == "L" else gx + gw - 6
            o.append(rect(edge, y - h / 2, 6, h, colour, extra=f' stroke="{outline}" stroke-width="1"'))
            g = f"GPIO{gpio}" if gpio is not None else ("power" if lab == "3V3" else "ground")
            o.append(text(gx + gw / 2 + (3 if row == "L" else -3), y + 5, g, 13.5, INK, "bold", "middle"))
            o.append(chip(cx, y, lab, size, minw=cw))


def draw_display(o):
    o.append('<g id="display">')
    o.append(rect(DX0 + 2, DY0 + 5, DX1 - DX0, DY1 - DY0, "#000", rx=10, extra=' opacity="0.12"'))
    o.append(rect(DX0, DY0, DX1 - DX0, DY1 - DY0, "#1f5ca8", rx=10, stroke="#123c70", sw=1.5))
    faint = ' opacity="0.42"'
    # silk and parts on the back, faded: they help you recognise the board
    o.append(f'<g{faint}>')
    o.append(text(DX0 + 22, DY1 - 24, '3.5"TFT', 26, "#fff", "bold"))
    # 8080 FPC sockets
    o.append(rect(DX0 + 24, DY1 - 118, 170, 22, "#f3efe2", rx=2))
    o.append(rect(DX0 + 30, DY1 - 112, 158, 6, "#c9a85a"))
    o.append(text(DX0 + 24, DY1 - 124, "8080-16P", 13, "#fff", "bold"))
    o.append(rect(DX0 + 196, DY1 - 118, 78, 22, "#f3efe2", rx=2))
    o.append(rect(DX0 + 202, DY1 - 112, 66, 6, "#c9a85a"))
    o.append(text(DX0 + 196, DY1 - 124, "8080-8P", 13, "#fff", "bold"))
    o.append("</g>")
    # IM0-IM2 jumper table (SPI row is the one that matters)
    tx, ty = DX1 - 182, DY1 - 146
    cols = ["", "IM0", "IM1", "IM2"]
    rows = [("P8", "1", "1", "0"), ("P16", "0", "1", "0"), ("SPI", "1", "1", "1"), ("SPI3", "1", "0", "1")]
    cw, rh = 40, 22
    o.append(rect(tx - 6, ty - 18, cw * 4 + 12, rh * 5 + 6, "none", rx=3, stroke="#fff", sw=1, extra=' opacity="0.5"'))
    for j, c in enumerate(cols):
        o.append(text(tx + cw * j + cw / 2, ty, c, 12, "#fff", "bold", "middle", ' opacity="0.55"'))
    for i, r in enumerate(rows):
        y = ty + rh * (i + 1)
        hi = r[0] == "SPI"
        if hi:
            o.append(rect(tx - 4, y - 15, cw * 4 + 8, rh - 1, "#fff", rx=3))
        for j, v in enumerate(r):
            o.append(text(tx + cw * j + cw / 2, y, v, 13 if hi else 12, "#123c70" if hi else "#fff",
                          "bold", "middle", "" if hi else ' opacity="0.5"'))
    # header: black plastic strip, gold pins
    x0, x1 = hdr_x("SDA-O") - HDR_PITCH / 2 + 4, hdr_x("GND") + HDR_PITCH / 2 - 4
    o.append(rect(x0, HDR_Y - 11, x1 - x0, 22, "#141414", rx=2))
    for n in HEADER:
        x = hdr_x(n)
        o.append(rect(x - 6, HDR_Y - 6, 12, 12, "#e2bf5c" if n in WIRES else "#9c8650", rx=1.5))
    o.append(text(x0 + 6, HDR_Y + 4, "9", 11, "#9a9a9a", "bold", "middle"))
    o.append(text(x1 - 6, HDR_Y + 4, "1", 11, "#9a9a9a", "bold", "middle"))
    # pin chips, staggered in two rows so the labels can be big
    size = 18
    for i, n in enumerate(HEADER):
        x = hdr_x(n)
        cy = HDR_Y + 32 if i % 2 == 0 else HDR_Y + 66
        o.append(f'<line x1="{x:g}" y1="{HDR_Y + 7}" x2="{x:g}" y2="{cy - 10:g}" stroke="#fff" stroke-width="1.5" opacity="0.7"/>')
        if n in WIRES:
            colour, outline = WIRES[n][4], WIRES[n][5]
            w = max(56, tw(n, size) + size * 0.75)
            h = size * 1.42
            o.append(chip(x, cy, n, size, minw=56))
            o.append(rect(x - w / 2, cy + h / 2 - 1, w, 5, colour, extra=f' stroke="{outline}" stroke-width="1"'))
        else:
            o.append(chip(x, cy, n, size, dim=True, minw=56))
    o.append("</g>")


def draw_wires(o):
    o.append('<g id="wires" fill="none" stroke-linecap="round" stroke-linejoin="round">')
    # Draw in channel order so crossings layer consistently.
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
    # wire ends: a dot on each pin
    for n in WIRES:
        colour, outline = WIRES[n][4], WIRES[n][5]
        for (x, y) in (route(n)[0], route(n)[-1]):
            o.append(f'<circle cx="{x:g}" cy="{y:g}" r="6.5" fill="{colour}" stroke="{outline}" stroke-width="2"/>')


def callout(o, x, y, lines, anchor="start"):
    size = 17
    w = max(tw(s, size, i == 0) for i, s in enumerate(lines)) + 34
    h = 12 + 22 * len(lines)
    x0 = x if anchor == "start" else x - w
    o.append(rect(x0, y, w, h, "#fff", rx=4, stroke="#d5d9de", sw=1))
    o.append(rect(x0, y, 5, h, BLUE, rx=1))
    for i, s in enumerate(lines):
        o.append(text(x0 + 17, y + 23 + 22 * i, s, size, INK, "bold" if i == 0 else "normal"))
    return x0, w, h


def draw_legend(o):
    x0, y0 = DX0, 20
    o.append(text(x0, y0 + 14, "CONNECTIONS", 14, BLUE, "bold", "start", ' letter-spacing="1.5"'))
    o.append(text(x0 + 60, y0 + 40, "Display", 13, MUTED, "bold"))
    o.append(text(x0 + 142, y0 + 40, "ESP32", 13, MUTED, "bold"))
    o.append(text(x0 + 284, y0 + 40, "What it does", 13, MUTED, "bold"))
    rh = 26
    for i, n in enumerate(LEGEND_ORDER):
        row, silk, gpio, role, colour, outline = WIRES[n]
        y = y0 + 62 + rh * i
        o.append(f'<line x1="{x0 + 4}" y1="{y}" x2="{x0 + 44}" y2="{y}" stroke="{outline}" stroke-width="8" stroke-linecap="round"/>')
        o.append(f'<line x1="{x0 + 4}" y1="{y}" x2="{x0 + 44}" y2="{y}" stroke="{colour}" stroke-width="5.4" stroke-linecap="round"/>')
        o.append(text(x0 + 60, y + 7, n, 20, INK, "bold"))
        o.append(text(x0 + 124, y + 6, "→", 18, MUTED, "normal", "middle"))
        o.append(text(x0 + 142, y + 7, silk, 20, INK, "bold"))
        if gpio is not None:
            o.append(text(x0 + 192, y + 6, f"GPIO{gpio}", 14, MUTED, "bold"))
        o.append(text(x0 + 284, y + 6.5, role, 17, INK))
    y = y0 + 62 + rh * 8
    o.append(f'<line x1="{x0 + 4}" y1="{y}" x2="{x0 + 44}" y2="{y}" stroke="#b9bec3" stroke-width="3" stroke-dasharray="1 6" stroke-linecap="round"/>')
    o.append(text(x0 + 60, y + 7, "SDA-O", 20, "#868c92", "bold"))
    o.append(text(x0 + 142, y + 6.5, "not connected, leave it empty", 17, "#868c92"))


def build():
    check()
    xs = crossings()
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}" '
         f'role="img" aria-labelledby="wiring-title wiring-desc" font-family="{FONT}">']
    o.append('<title id="wiring-title">Wiring: ESP32 board to 3.5-inch ST7796S display</title>')
    desc = ("Eight female-to-female jumper wires connect the display's 9-pin SPI header, seen from the back, "
            "to the ESP32 DevKit, seen from the top with the USB-C port at the bottom. "
            + "; ".join(f"display {n} to ESP32 {WIRES[n][1]}"
                        + (f" (GPIO{WIRES[n][2]})" if WIRES[n][2] is not None else "")
                        for n in LEGEND_ORDER)
            + ". Display pin SDA-O is not connected. The SCL and SDA pins are SPI, not I2C. "
              "The IM0, IM1 and IM2 jumpers on the display must all be 1 (SPI). "
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
    draw_esp_labels(o)
    draw_legend(o)
    # board captions
    o.append(text(BX1 + 14, BY0 + 26, "ESP32 board", 16, INK, "bold"))
    o.append(text(BX1 + 14, BY0 + 46, "top view", 15, MUTED))
    o.append(text(DX0, DY1 + 26, "Display", 16, INK, "bold"))
    o.append(text(DX0, DY1 + 46, "back view, “SPI” header on top", 15, MUTED))
    # callouts
    callout(o, DX0 + 18, DY0 + 106, ["SCL/SDA here means SPI, not I2C"])
    cx0, cw, ch = callout(o, DX1, DY1 + 14, ["Check IM0–IM2 jumpers", "are on 1 (SPI)"], anchor="end")
    ax = DX1 - 92
    o.append(f'<path d="M{ax},{DY1 + 14} V{DY1 - 44}" stroke="#fff" stroke-width="6"/>')
    o.append(f'<path d="M{ax},{DY1 + 14} V{DY1 - 44}" stroke="{BLUE}" stroke-width="2.5"/>')
    o.append(f'<path d="M{ax - 9},{DY1 - 28} L{ax},{DY1 - 45} L{ax + 9},{DY1 - 28}Z" fill="{BLUE}" '
             f'stroke="#fff" stroke-width="2" stroke-linejoin="round"/>')
    ux = (BX0 + BX1) / 2
    callout(o, ux - 160, UNDER["BL"][0] + 16, ["USB-C: use a cable that carries data"])
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
