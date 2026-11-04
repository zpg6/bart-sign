"""Compose firmware-rendered LED bands (from `cargo run --example mockups`)
into the Lafayette-gray twin-sign housing: one PNG per screen, plus the
all-stations and rotation slides. Run via tools/render_docs.sh.

    python3 -I compose_mockups.py <bands-dir> <stations.json> <out-dir>
"""
import json, os, sys
from PIL import Image, ImageDraw, ImageFont

BANDS, STATIONS_JSON, OUT = sys.argv[1], sys.argv[2], sys.argv[3]
os.makedirs(os.path.join(OUT, "screens"), exist_ok=True)
W, H = 480, 320
C = dict(void=(0, 0, 0), housing=(92, 94, 98), hood=(14, 12, 16), hood_edge=(60, 60, 64),
         frame=(40, 34, 36), underside=(58, 60, 64), label=(240, 240, 236))
HELV = "/System/Library/Fonts/Helvetica.ttc"


def font(size, bold=True):
    return ImageFont.truetype(HELV, size, index=1 if bold else 0)


def font_for_cap(cap):
    for size in range(6, 60):
        f = font(size)
        l, t, r, b = f.getbbox("P")
        if b - t >= cap:
            return f


F13 = font_for_cap(13)


def screen(code, tick, labels):
    img = Image.new("RGB", (W, H), C["void"])
    d = ImageDraw.Draw(img)
    for slot, y0 in enumerate((0, 165)):
        d.rectangle([0, y0, W - 1, y0 + 154], fill=C["housing"])
        d.rectangle([0, y0 + 1, W - 1, y0 + 2], fill=C["hood"])
        d.line([0, y0, W - 1, y0], fill=C["hood_edge"])
        d.rectangle([10, y0 + 4, 469, y0 + 130], fill=C["frame"])
        img.paste(Image.open(f"{BANDS}/{code}_{tick}_{slot}.png"), (11, y0 + 5))
        # An empty sign keeps its last label on the device; show the default.
        number = labels[slot] or (2, 1)[slot]
        if number:
            text = f"PLATFORM {number}"
            l, t, r, b = F13.getbbox(text)
            d.text((13 - l, y0 + 138 - t), text, font=F13, fill=C["label"])
        d.rectangle([0, y0 + 152, W - 1, y0 + 154], fill=C["underside"])
    return img


names = {s["abbr"]: s["name"] for s in json.load(open(STATIONS_JSON))["root"]["stations"]["station"]}
rows = [l.rstrip("\n").split("\t") for l in open(f"{BANDS}/index.tsv")]
by = {}
for code, tick, n, l0, l1, p0, p1 in rows:
    by.setdefault(code, []).append((int(tick), int(n), int(l0), int(l1), p0, p1))

BG, INK, SUB = (250, 250, 248), (20, 20, 24), (110, 110, 118)
BLUE = (0, 100, 168)


def title(d, w, text, sub):
    d.rectangle([0, 0, w, 86], fill=BLUE)
    d.text((40, 18), text, font=font(34), fill=(255, 255, 255))
    d.text((40, 58), sub, font=font(17, False), fill=(220, 235, 245))


# Slide 1: every station's first screen.
codes = sorted(c for c in by if c in names)
cols, tw, th, cap = 7, 240, 160, 26
rows_n = -(-len(codes) // cols)
sw, sh = 40 * 2 + cols * tw + (cols - 1) * 14, 110 + rows_n * (th + cap + 14) + 30
s1 = Image.new("RGB", (sw, sh), BG)
d = ImageDraw.Draw(s1)
title(d, sw, "All 49 BART stations, every line", "Live data 10:20 PM Mon Nov 2 2026, rendered by the sign's own firmware code. First screen of each; rotating stations marked.")
for i, code in enumerate(codes):
    x = 40 + (i % cols) * (tw + 14)
    y = 110 + (i // cols) * (th + cap + 14)
    t0 = by[code][0]
    s1.paste(screen(code, 0, (t0[2], t0[3])).resize((tw, th), Image.LANCZOS), (x, y))
    label = names.get(code, code)
    limit = 18 if t0[1] > 1 else 26
    if len(label) > limit:
        label = label[:limit - 1].rstrip(" ./") + "…"
    d.text((x, y + th + 5), label, font=font(14), fill=INK)
    if t0[1] > 1:
        d.text((x + tw - 62, y + th + 5), f"{t0[1]} screens", font=font(12), fill=BLUE)
s1.save(os.path.join(OUT, "all-stations.png"))

# Slide 2: stations that rotate, every screen in order.
rot = [c for c in ["MCAR", "12TH", "19TH", "LAKE", "MONT-RUSH"] if c in by]
names.setdefault("MONT-RUSH", "Montgomery St.")
tw2, th2 = 360, 240
maxn = max(len(by[c]) for c in rot)
sw2 = 40 * 2 + 300 + maxn * (tw2 + 40)
sh2 = 110 + len(rot) * (th2 + 40) + 20
s2 = Image.new("RGB", (sw2, sh2), BG)
d = ImageDraw.Draw(s2)
title(d, sw2, "Stations with more platforms rotate", "Each sign shows one platform, like the real ones: southbound on top, northbound below, every 6 s.")
for r, code in enumerate(rot):
    y = 110 + r * (th2 + 40)
    t = by[code]
    d.text((40, y + 70), names.get(code, code).split(" (")[0], font=font(22), fill=INK)
    d.text((40, y + 102), f"top: platforms {t[0][4]}", font=font(15, False), fill=SUB)
    d.text((40, y + 124), f"bottom: platforms {t[0][5]}", font=font(15, False), fill=SUB)
    if code == "MONT-RUSH":
        d.text((40, y + 146), "7:17 PM data: 4 destinations", font=font(15, False), fill=SUB)
        d.text((40, y + 166), "northbound page in pairs", font=font(15, False), fill=SUB)
    for k, (tick, n, l0, l1, *_rest) in enumerate(t):
        x = 340 + k * (tw2 + 40)
        s2.paste(screen(code, tick, (l0, l1)).resize((tw2, th2), Image.LANCZOS), (x, y))
        d.text((x, y + th2 + 4), f"screen {k + 1} of {n}", font=font(13, False), fill=SUB)
        if k + 1 < len(t):
            ax, ay = x + tw2 + 10, y + th2 // 2
            d.line([ax, ay, ax + 18, ay], fill=BLUE, width=3)
            d.polygon([(ax + 22, ay), (ax + 13, ay - 7), (ax + 13, ay + 7)], fill=BLUE)
s2.save(os.path.join(OUT, "rotation.png"))

# Every screen on its own, at 1x (480 x 320, the panel's real pixels).
for code, ticks in by.items():
    for tick, n, l0, l1, *_ in ticks:
        screen(code, tick, (l0, l1)).save(os.path.join(OUT, "screens", f"{code}-{tick}.png"))
print("ok", s1.size, s2.size, sum(len(t) for t in by.values()), "screens")
