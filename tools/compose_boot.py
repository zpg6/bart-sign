"""Compose `cargo run --example boot_pair` frames into an animated GIF of
both signs booting, in the Lafayette-gray housing. Run via tools/render_docs.sh.

    python3 -I compose_boot.py <frames-dir> <out.gif>
"""
import sys, os
from PIL import Image, ImageDraw, ImageFont
F, OUT = sys.argv[1], sys.argv[2]
W, H = 480, 320
C = dict(void=(0,0,0), housing=(92,94,98), hood=(14,12,16), hood_edge=(60,60,64), frame=(40,34,36), underside=(58,60,64), label=(240,240,236))
UNIT_H, GAP = 155, 10
def font_for_cap(cap):
    for size in range(6, 60):
        f = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", size, index=1)
        l, t, r, b = f.getbbox("P")
        if b - t >= cap: return f
F13 = font_for_cap(13)
def unit(img, d, y0, band, label):
    d.rectangle([0, y0, W-1, y0+UNIT_H-1], fill=C["housing"])
    d.rectangle([0, y0+1, W-1, y0+2], fill=C["hood"])
    d.line([0, y0, W-1, y0], fill=C["hood_edge"])
    d.rectangle([10, y0+4, 469, y0+130], fill=C["frame"])
    img.paste(band, (11, y0+5))
    l, t, r, b = F13.getbbox(label)
    d.text((13-l, y0+138-t), label, font=F13, fill=C["label"])
    d.rectangle([0, y0+152, W-1, y0+154], fill=C["underside"])
frames, durs = [], []
for line in open(os.path.join(F, "index.txt")):
    i, t, step = line.split()
    img = Image.new("RGB", (W, H), C["void"]); d = ImageDraw.Draw(img)
    unit(img, d, 0, Image.open(f"{F}/{i}_top.png"), "PLATFORM 2")
    unit(img, d, UNIT_H+GAP, Image.open(f"{F}/{i}_bot.png"), "PLATFORM 1")
    frames.append(img); durs.append(int(step))
frames[0].save(OUT, save_all=True, append_images=frames[1:], duration=durs, loop=0)
for k in (len(frames)-1,):
    frames[k].save(OUT.replace(".gif", "_final.png"))
print(len(frames), "frames")
