"""Builds the game's assets from free (CC0 / CC-BY) sources and the OpenStreetMap city file.

usage: python3 -I tools/make_assets.py <sources dir>

<sources dir> holds what tools/fetch_sources.sh downloads:
  base/      Quaternius "Universal Base Characters" (CC0)
  anim/      Quaternius "Universal Animation Library" (CC0)
  acg/       ambientCG textures (CC0): Asphalt031, PavingStones070, Plaster003, RoofingTiles006,
             Grass004, Bark012, Concrete034
  Fox.glb    Khronos glTF sample "Fox" (CC0 model; CC-BY 4.0 rig and animation by tomkranis)

Writes into assets/: people/ (clothed characters), hair/, anim.glb, dog.glb, tex/ (ground, roofs,
facades of tube houses, shop signs with the real names of places from the map).
"""

import glob
import json
import os
import random
import shutil
import struct
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFilter, ImageFont

SRC = sys.argv[1] if len(sys.argv) > 1 else "sources"
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets")
FONT = "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"
rng = random.Random(1010)


def out(*p):
    path = os.path.join(OUT, *p)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    return path


def one(pattern):
    hits = sorted(glob.glob(os.path.join(SRC, pattern), recursive=True))
    if not hits:
        raise SystemExit(f"missing source: {pattern}")
    return hits[0]


# ---------------------------------------------------------------------------------------------
# glTF helpers


def accessor(g, buf, i):
    a = g["accessors"][i]
    v = g["bufferViews"][a["bufferView"]]
    n = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}[a["type"]]
    dt = {5126: np.float32, 5123: np.uint16, 5125: np.uint32, 5121: np.uint8}[a["componentType"]]
    off = v.get("byteOffset", 0) + a.get("byteOffset", 0)
    return np.frombuffer(buf, dtype=dt, count=a["count"] * n, offset=off).reshape(a["count"], n)


def save_jpg(img, path, size, quality=82):
    img.convert("RGB").resize(size, Image.LANCZOS).save(path, quality=quality, optimize=True)


# ---------------------------------------------------------------------------------------------
# People: paint clothes onto the base body's skin, by where each triangle sits on the body.

PEOPLE = {
    # name: skin, shirt, long sleeves, trousers, shoes, hair
    "minh": ("Ligh", (43, 76, 126), True, (52, 73, 94), (235, 235, 235), "Hair_SimpleParted"),
    "trom": ("Dark", (30, 31, 35), True, (45, 45, 51), (20, 20, 20), "Hair_Buzzed"),
    "trom2": ("Ligh", (88, 38, 30), False, (40, 40, 44), (25, 25, 25), "Hair_Buzzed"),
    "dai_ca": ("Dark", (120, 20, 24), True, (18, 18, 22), (10, 10, 10), "Hair_Beard"),
    "co_hang": ("Ligh", (196, 120, 160), True, (60, 60, 70), (120, 80, 60), "Hair_Buns"),
    "nguoi_di_duong": ("Ligh", (214, 180, 96), False, (70, 80, 96), (90, 70, 50), "Hair_SimpleParted"),
}

def classify(y, ax):
    """Body part of a point on the T-posed base body (metres; ax = |x|)."""
    if y < 0.09:
        return "shoes"
    if y < 0.95:
        return "trousers"
    if ax > 0.73:
        return "skin"  # hands
    if ax > 0.42:
        return "forearm"
    if ax > 0.17:
        return "sleeve"
    if y > 1.50:
        return "skin"  # neck and head
    return "shirt"


def paint_people():
    root = one("base/**/Base Characters/Godot - UE/Superhero_Male_FullBody.gltf")
    d = os.path.dirname(root)
    g0 = json.load(open(root))
    buf = open(os.path.join(d, g0["buffers"][0]["uri"]), "rb").read()
    body = next(p for m in g0["meshes"] for p in m["primitives"] if g0["materials"][p["material"]]["name"] == "MI_Superhero_Male")
    pos = accessor(g0, buf, body["attributes"]["POSITION"])
    uv = accessor(g0, buf, body["attributes"]["TEXCOORD_0"])
    idx = accessor(g0, buf, body["indices"]).reshape(-1, 3)
    size = 1024
    masks = {k: Image.new("L", (size, size), 0) for k in ["shoes", "trousers", "shirt", "sleeve", "forearm"]}
    draws = {k: ImageDraw.Draw(m) for k, m in masks.items()}
    for tri in idx:
        c = pos[tri].mean(0)
        part = classify(c[1], abs(c[0]))
        if part in draws:
            draws[part].polygon([(float(u) * size, float(v) * size) for u, v in uv[tri]], fill=255)
    masks = {k: np.asarray(m.filter(ImageFilter.MaxFilter(5)), dtype=np.float32) / 255.0 for k, m in masks.items()}
    yy = np.arange(size, dtype=np.float32)[:, None]
    for name, (skin, shirt, long_sleeves, trousers, shoes, hair) in PEOPLE.items():
        src = Image.open(one(f"base/**/Base Characters/Textures/T_Superhero_*_{skin}*.png")).convert("RGB").resize((size, size))
        if "Male" not in os.path.basename(src.filename if hasattr(src, "filename") else "Male"):
            pass
        img = np.asarray(src, dtype=np.float32)
        lum = img.mean(2, keepdims=True)
        shade = np.clip(lum / max(1.0, float(lum.mean())), 0.75, 1.15)
        noise = np.random.default_rng(len(name)).normal(1.0, 0.035, (size, size, 1)).astype(np.float32)
        denim = 1.0 + 0.05 * np.sin(yy * 1.7)[:, :, None] * np.ones((1, size, 1), np.float32)

        def cloth(rgb, extra=1.0):
            return np.clip(np.array(rgb, np.float32)[None, None, :] * shade * noise * extra, 0, 255)

        layers = [("shirt", cloth(shirt)), ("sleeve", cloth(shirt)), ("trousers", cloth(trousers, denim)), ("shoes", cloth(shoes))]
        if long_sleeves:
            layers.append(("forearm", cloth(shirt)))
        res = img.copy()
        for part, col in layers:
            m = masks[part][:, :, None]
            res = res * (1 - m) + col * m
        d_out = out("people", name, "x")[:-2]
        os.makedirs(d_out, exist_ok=True)
        save_jpg(Image.fromarray(res.astype(np.uint8)), os.path.join(d_out, "body.jpg"), (1024, 1024))
        write_person(g0, d, d_out, name)
        hair_dir = os.path.dirname(one(f"base/**/Rigged to Head Bone/glTF*/{hair}.gltf"))
        copy_gltf(os.path.join(hair_dir, f"{hair}.gltf"), out("hair", hair, "x")[:-2])
    print("people:", ", ".join(PEOPLE))


def write_person(g0, src_dir, dst, name):
    """The base glTF with our painted body; the mesh data and the other maps are shared by every
    person (people/shared/), so each one adds only its picture."""
    g = json.loads(json.dumps(g0))
    shared = out("people", "shared", "x")[:-2]
    os.makedirs(shared, exist_ok=True)
    bin_dst = os.path.join(shared, "person.bin")
    if not os.path.exists(bin_dst):
        shutil.copy(os.path.join(src_dir, g["buffers"][0]["uri"]), bin_dst)
    g["buffers"][0]["uri"] = "../shared/person.bin"
    for m in g["meshes"]:
        for p in m["primitives"]:
            for k in [k for k in p["attributes"] if k.startswith("COLOR_") or k in ("TEXCOORD_2", "TEXCOORD_3")]:
                del p["attributes"][k]
    for im in g["images"]:
        uri = im["uri"]
        srcp = os.path.join(src_dir, uri)
        if not os.path.exists(srcp):
            srcp = os.path.join(src_dir, uri.replace("_png.png", ".png"))
        base = os.path.splitext(uri)[0].replace("_png", "")
        if "Superhero" in uri and "Normal" not in uri and "Roughness" not in uri:
            im["uri"] = "body.jpg"
            im["mimeType"] = "image/jpeg"
            continue
        small = 512 if "Normal" in uri else 256
        new = base + ".png"
        if not os.path.exists(os.path.join(shared, new)):
            Image.open(srcp).resize((small, small), Image.LANCZOS).save(os.path.join(shared, new), optimize=True)
        im["uri"] = "../shared/" + new
        im["mimeType"] = "image/png"
    json.dump(g, open(os.path.join(dst, "person.gltf"), "w"))


def copy_gltf(path, dst):
    os.makedirs(dst, exist_ok=True)
    g = json.load(open(path))
    d = os.path.dirname(path)
    for b in g["buffers"]:
        shutil.copy(os.path.join(d, b["uri"]), os.path.join(dst, os.path.basename(b["uri"])))
    for im in g.get("images", []):
        new = os.path.splitext(os.path.basename(im["uri"]))[0] + ".png"
        Image.open(os.path.join(d, im["uri"])).resize((256, 256), Image.LANCZOS).save(os.path.join(dst, new), optimize=True)
        im["uri"] = new
    json.dump(g, open(os.path.join(dst, os.path.basename(path)), "w"))


# ---------------------------------------------------------------------------------------------
# GLB: read, swap an image, write.


def read_glb(path):
    b = open(path, "rb").read()
    jlen = struct.unpack("<I", b[12:16])[0]
    g = json.loads(b[20 : 20 + jlen])
    rest = b[20 + jlen :]
    blen = struct.unpack("<I", rest[0:4])[0]
    return g, bytearray(rest[8 : 8 + blen])


def write_glb(path, g, binary):
    while len(binary) % 4:
        binary.append(0)
    g["buffers"] = [{"byteLength": len(binary)}]
    j = json.dumps(g).encode()
    while len(j) % 4:
        j += b" "
    total = 12 + 8 + len(j) + 8 + len(binary)
    with open(path, "wb") as f:
        f.write(struct.pack("<4sII", b"glTF", 2, total))
        f.write(struct.pack("<I4s", len(j), b"JSON") + j)
        f.write(struct.pack("<I4s", len(binary), b"BIN\0") + bytes(binary))


def replace_image(g, binary, index, data, mime):
    view = len(g["bufferViews"])
    while len(binary) % 4:
        binary.append(0)
    g["bufferViews"].append({"buffer": 0, "byteOffset": len(binary), "byteLength": len(data)})
    binary.extend(data)
    g["images"][index] = {"bufferView": view, "mimeType": mime}


def make_dog():
    """The fox model, recoloured as a Vietnamese yellow dog (chó cỏ)."""
    import io

    g, binary = read_glb(one("Fox.glb"))
    im = g["images"][0]
    v = g["bufferViews"][im["bufferView"]]
    tex = Image.open(io.BytesIO(bytes(binary[v.get("byteOffset", 0) : v.get("byteOffset", 0) + v["byteLength"]]))).convert("RGB")
    a = np.asarray(tex, np.float32)
    lum = a.mean(2, keepdims=True) / 255.0
    tan = np.array([214, 160, 84], np.float32)[None, None, :]
    cream = np.array([238, 222, 190], np.float32)[None, None, :]
    res = np.where(lum > 0.72, cream * lum, tan * (0.55 + 0.6 * lum))
    buf = io.BytesIO()
    Image.fromarray(np.clip(res, 0, 255).astype(np.uint8)).save(buf, "PNG", optimize=True)
    replace_image(g, binary, 0, buf.getvalue(), "image/png")
    write_glb(out("dog.glb"), g, binary)
    print("dog: ok")


def copy_anims():
    shutil.copy(one("anim/**/Unreal-Godot/UAL1_Standard.glb"), out("anim.glb"))
    print("animations: ok")


# ---------------------------------------------------------------------------------------------
# Textures: ground, roofs, and the faces of tube houses.


def ground_textures():
    for name, src, size in [
        ("asphalt", "Asphalt031", 512),
        ("paving", "PavingStones070", 512),
        ("grass", "Grass004", 512),
        ("bark", "Bark012", 256),
        ("roof_tiles", "RoofingTiles006", 512),
        ("concrete", "Concrete034", 512),
    ]:
        save_jpg(Image.open(one(f"acg/{src}/*_Color.jpg")), out("tex", f"{name}.jpg"), (size, size))
    print("ground textures: ok")


WALLS = [(227, 184, 90), (234, 217, 176), (156, 192, 198), (217, 163, 160), (232, 228, 220), (196, 190, 178)]
SHUTTERS = [(47, 107, 79), (92, 64, 44), (40, 80, 110), (70, 70, 70)]


def plaster(color, size):
    base = Image.open(one("acg/Plaster003/*_Color.jpg")).convert("L").resize((size, size))
    a = np.asarray(base, np.float32)[:, :, None] / 255.0
    a = 0.75 + 0.35 * (a - a.mean())
    return np.clip(np.array(color, np.float32)[None, None, :] * a, 0, 255)


def facades():
    """Per wall colour: an upper floor (window with shutters and a balcony) and a ground floor
    (shopfront), each with a glow map so windows and open shops light up at dusk."""
    s = 256
    for i, wall in enumerate(WALLS):
        sh = SHUTTERS[i % len(SHUTTERS)]
        up = Image.fromarray(plaster(wall, s).astype(np.uint8))
        glow = Image.new("RGB", (s, s), (0, 0, 0))
        d, gd = ImageDraw.Draw(up), ImageDraw.Draw(glow)
        # A tall window with wooden shutters.
        d.rectangle([88, 40, 168, 190], fill=(36, 40, 46))
        d.rectangle([92, 44, 164, 186], fill=(58, 64, 70))
        gd.rectangle([92, 44, 164, 186], fill=(255, 190, 110) if i % 2 == 0 else (0, 0, 0))
        for x0 in (58, 168):
            d.rectangle([x0, 40, x0 + 30, 190], fill=sh)
            for y in range(48, 186, 9):
                d.line([x0 + 3, y, x0 + 27, y], fill=tuple(max(0, c - 30) for c in sh), width=2)
        # A balcony: a slab, iron railings and potted plants.
        d.rectangle([30, 196, 226, 206], fill=(150, 150, 150))
        for x in range(34, 224, 12):
            d.line([x, 168, x, 196], fill=(30, 30, 30), width=2)
        d.line([30, 168, 226, 168], fill=(30, 30, 30), width=4)
        for x in (52, 196):
            d.ellipse([x - 16, 150, x + 16, 182], fill=(52, 110, 52))
        d.rectangle([0, 246, s, s], fill=tuple(max(0, c - 40) for c in wall))
        if i % 3 == 1:
            d.rectangle([200, 70, 240, 100], fill=(210, 210, 214))  # an air conditioner
        up.save(out("tex", f"wall{i}_up.jpg"), quality=85)
        glow.save(out("tex", f"wall{i}_up_glow.jpg"), quality=85)

        gr = Image.fromarray(plaster(wall, s).astype(np.uint8))
        glow = Image.new("RGB", (s, s), (0, 0, 0))
        d, gd = ImageDraw.Draw(gr), ImageDraw.Draw(glow)
        d.rectangle([14, 60, 242, s], fill=(30, 30, 32))
        if i % 2 == 0:
            # An open shop: warm light, shelves of goods.
            d.rectangle([20, 70, 236, s], fill=(120, 92, 60))
            gd.rectangle([20, 70, 236, s], fill=(255, 196, 120))
            for y in range(90, s, 34):
                d.rectangle([28, y, 228, y + 6], fill=(70, 50, 34))
                for x in range(34, 222, 22):
                    c = rng.choice([(200, 60, 50), (230, 190, 70), (60, 120, 180), (80, 150, 80), (230, 230, 230)])
                    d.rectangle([x, y - 18, x + 14, y], fill=c)
        else:
            # A rolled-down shutter.
            for y in range(64, s, 8):
                d.line([16, y, 240, y], fill=(150, 152, 156), width=5)
        d.rectangle([14, 52, 242, 62], fill=(60, 60, 64))
        gr.save(out("tex", f"wall{i}_ground.jpg"), quality=85)
        glow.save(out("tex", f"wall{i}_ground_glow.jpg"), quality=85)
    print(f"facades: {len(WALLS)} styles")


def signs():
    """Shop signs with the real names of places from the map, in an atlas of 8 × 32 boards."""
    city = json.load(open(out("hoankiem.json")))
    kinds = ("amenity:restaurant", "amenity:cafe", "tourism:hotel", "amenity:fast_food", "amenity:bar", "shop:", "tourism:hostel", "amenity:pharmacy", "amenity:bank")
    pois = [p for p in city["pois"] if p["kind"].startswith(kinds) and 2 <= len(p["name"]) <= 34]
    # Closest to the old quarter first (around Hàng Đào, north of the lake).
    pois.sort(key=lambda p: (p["at"][0] + 600) ** 2 + (p["at"][1] + 4500) ** 2)
    pois = pois[:256]
    w, h = 256, 64
    atlas = Image.new("RGB", (w * 8, h * 32), (0, 0, 0))
    styles = [((178, 34, 34), (255, 214, 64)), ((16, 70, 140), (255, 255, 255)), ((245, 240, 230), (180, 20, 20)), ((20, 20, 24), (255, 200, 60)), ((0, 110, 80), (255, 255, 255))]
    index = []
    for i, p in enumerate(pois):
        bg, fg = styles[i % len(styles)]
        board = Image.new("RGB", (w, h), bg)
        d = ImageDraw.Draw(board)
        d.rectangle([2, 2, w - 3, h - 3], outline=fg, width=2)
        text = p["name"].upper()
        size = 30
        while size > 10:
            f = ImageFont.truetype(FONT, size)
            box = d.textbbox((0, 0), text, font=f)
            if box[2] - box[0] <= w - 16:
                break
            size -= 2
        f = ImageFont.truetype(FONT, size)
        box = d.textbbox((0, 0), text, font=f)
        d.text(((w - (box[2] - box[0])) / 2 - box[0], (h - (box[3] - box[1])) / 2 - box[1]), text, font=f, fill=fg)
        atlas.paste(board, ((i % 8) * w, (i // 8) * h))
        index.append({"name": p["name"], "at": p["at"]})
    atlas.save(out("tex", "signs.jpg"), quality=88)
    json.dump(index, open(out("signs.json"), "w"), ensure_ascii=False)
    print(f"signs: {len(index)} real names")


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    paint_people()
    make_dog()
    copy_anims()
    ground_textures()
    facades()
    signs()
