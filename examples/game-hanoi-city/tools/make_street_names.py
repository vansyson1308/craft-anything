"""Draws the faces of Hà Nội's street-name signs for every real street in the city file: white
lettering on the city's blue plates, one cell per street, into assets/tex/street_names.png, with
the list of names (in cell order) in assets/tex/street_names.json.

Only streets ("Phố", "Đường") at least 5 m wide get signs; lanes don't.
Usage (from examples/game-hanoi-city): python3 -I tools/make_street_names.py   (needs Pillow)"""

import json
import math
import os

from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
CELL = (256, 78)  # the sign's face is about 0.97 × 0.30 m
COLS = 8
BLUE = (22, 66, 150)
WHITE = (245, 245, 240)
FONT = os.path.join(ROOT, "assets", "fonts", "DejaVuSans-Bold.ttf")


def street_names(city):
    names = set()
    for r in city.get("roads", []):
        n = r.get("name")
        if n and r.get("w", 0) >= 50 and not r.get("bridge") and n.split(" ")[0] in ("Phố", "Đường"):
            names.add(n)
    return sorted(names)


def fit(draw, text, size, width):
    while size > 9:
        f = ImageFont.truetype(FONT, size)
        if draw.textlength(text, font=f) <= width:
            return f
        size -= 1
    return ImageFont.truetype(FONT, 9)


def main():
    city = json.load(open(os.path.join(ROOT, "assets", "hoankiem.json"), encoding="utf-8"))
    names = street_names(city)
    rows = max(1, math.ceil(len(names) / COLS))
    atlas = Image.new("RGB", (CELL[0] * COLS, CELL[1] * rows), BLUE)
    d = ImageDraw.Draw(atlas)
    for i, name in enumerate(names):
        x, y = (i % COLS) * CELL[0], (i // COLS) * CELL[1]
        d.rectangle([x + 4, y + 4, x + CELL[0] - 5, y + CELL[1] - 5], outline=WHITE, width=3)
        kind, _, rest = name.partition(" ")
        small = ImageFont.truetype(FONT, 13)
        d.text((x + 12, y + 9), kind.upper(), font=small, fill=WHITE)
        big = fit(d, rest.upper(), 30, CELL[0] - 24)
        w = d.textlength(rest.upper(), font=big)
        d.text((x + (CELL[0] - w) / 2, y + 28), rest.upper(), font=big, fill=WHITE)
    out = os.path.join(ROOT, "assets", "tex")
    atlas.save(os.path.join(out, "street_names.png"), optimize=True)
    with open(os.path.join(out, "street_names.json"), "w", encoding="utf-8") as f:
        json.dump({"cell": list(CELL), "cols": COLS, "rows": rows, "names": names}, f, ensure_ascii=False, indent=0)
    print(f"{len(names)} street names, atlas {atlas.size[0]}×{atlas.size[1]}")


if __name__ == "__main__":
    main()
