"""Packs delivered art (assets/hq) for the web, the way big games ship their data: what the artist
delivers stays as it is in the repository, and the copy in dist/ is made small.

- Every image embedded in a .glb is taken out into dist/assets/hq/_tex/, named by its content, so
  a texture shared by LOD0, LOD1 and LOD2 (or by two models) is downloaded once.
- Images with no transparency are re-encoded as JPEG (quality 92) when Pillow is installed;
  colour images with transparency stay PNG.
- Previews, reports, editable texture sources and notes are left out of the web copy.

Usage: python3 -I tools/pack_hq.py <assets/hq> <dist/assets/hq>
Standard library only, except the optional JPEG step (Pillow)."""

import hashlib
import io
import json
import os
import shutil
import struct
import sys

try:
    from PIL import Image  # optional
except ImportError:  # pragma: no cover
    Image = None

SKIP_DIRS = {"previews", "reports", "textures"}
SKIP_FILES = {"README.md", "SOURCES.md", "LICENSE.txt", ".gitattributes"}
GLB_MAGIC = 0x46546C67


def read_glb(data):
    magic, version, total = struct.unpack_from("<III", data, 0)
    if magic != GLB_MAGIC or version != 2 or total > len(data):
        raise ValueError("not a glTF 2.0 binary")
    off, js, binary = 12, None, b""
    while off + 8 <= total:
        length, kind = struct.unpack_from("<II", data, off)
        chunk = data[off + 8 : off + 8 + length]
        if kind == 0x4E4F534A:
            js = json.loads(chunk)
        elif kind == 0x004E4942:
            binary = chunk
        off += 8 + length
    if js is None:
        raise ValueError("no JSON chunk")
    return js, binary


def write_glb(js, binary):
    j = json.dumps(js, separators=(",", ":"), ensure_ascii=False).encode()
    j += b" " * ((4 - len(j) % 4) % 4)
    binary += b"\0" * ((4 - len(binary) % 4) % 4)
    total = 12 + 8 + len(j) + (8 + len(binary) if binary else 0)
    out = struct.pack("<III", GLB_MAGIC, 2, total) + struct.pack("<II", len(j), 0x4E4F534A) + j
    if binary:
        out += struct.pack("<II", len(binary), 0x004E4942) + binary
    return out


def has_alpha(png):
    if Image is None:
        return True
    with Image.open(io.BytesIO(png)) as im:
        if im.mode in ("RGBA", "LA") or "transparency" in im.info:
            alpha = im.convert("RGBA").getchannel("A")
            return alpha.getextrema()[0] < 255
        return False


def to_jpeg(png):
    with Image.open(io.BytesIO(png)) as im:
        buf = io.BytesIO()
        im.convert("RGB").save(buf, "JPEG", quality=92, optimize=True)
        return buf.getvalue()


def store_image(raw, mime, tex_dir, cache):
    """Writes an image once by content; returns its file name in _tex/."""
    key = hashlib.sha256(raw).hexdigest()[:20]
    if key in cache:
        return cache[key]
    data, ext = raw, {"image/png": "png", "image/jpeg": "jpg"}.get(mime, "bin")
    if Image is not None and mime == "image/png" and not has_alpha(raw):
        jpg = to_jpeg(raw)
        if len(jpg) < len(raw):
            data, ext = jpg, "jpg"
    name = f"{key}.{ext}"
    with open(os.path.join(tex_dir, name), "wb") as f:
        f.write(data)
    cache[key] = name
    return name


def pack_glb(src, dst, tex_dir, cache):
    js, binary = read_glb(open(src, "rb").read())
    views = js.get("bufferViews", [])
    image_views = set()
    rel = os.path.relpath(tex_dir, os.path.dirname(dst)).replace(os.sep, "/")
    for img in js.get("images", []):
        bv = img.get("bufferView")
        if bv is None or bv >= len(views):
            continue
        v = views[bv]
        raw = binary[v.get("byteOffset", 0) : v.get("byteOffset", 0) + v["byteLength"]]
        name = store_image(raw, img.get("mimeType", "image/png"), tex_dir, cache)
        img.pop("bufferView", None)
        img["uri"] = f"{rel}/{name}"
        img["mimeType"] = "image/jpeg" if name.endswith(".jpg") else "image/png"
        image_views.add(bv)
    if not image_views:
        shutil.copyfile(src, dst)
        return
    # Rebuild the binary chunk without the images, keeping every other view 4-byte aligned.
    keep = [i for i in range(len(views)) if i not in image_views]
    remap, new_bin = {}, bytearray()
    for new_i, old_i in enumerate(keep):
        v = dict(views[old_i])
        start = v.get("byteOffset", 0)
        chunk = binary[start : start + v["byteLength"]]
        new_bin += b"\0" * ((4 - len(new_bin) % 4) % 4)
        v["byteOffset"] = len(new_bin)
        new_bin += chunk
        views[old_i] = v
        remap[old_i] = new_i
    js["bufferViews"] = [views[i] for i in keep]
    for acc in js.get("accessors", []):
        if "bufferView" in acc:
            acc["bufferView"] = remap[acc["bufferView"]]
        sparse = acc.get("sparse")
        if sparse:
            for part in ("indices", "values"):
                if part in sparse:
                    sparse[part]["bufferView"] = remap[sparse[part]["bufferView"]]
    if js.get("buffers"):
        js["buffers"][0]["byteLength"] = len(new_bin)
    with open(dst, "wb") as f:
        f.write(write_glb(js, bytes(new_bin)))


def main(src_root, dst_root):
    if not os.path.isdir(src_root):
        return
    shutil.rmtree(dst_root, ignore_errors=True)
    tex_dir = os.path.join(dst_root, "_tex")
    os.makedirs(tex_dir)
    cache, before, after = {}, 0, 0
    for root, dirs, files in os.walk(src_root):
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        for name in files:
            if name in SKIP_FILES:
                continue
            src = os.path.join(root, name)
            dst = os.path.join(dst_root, os.path.relpath(src, src_root))
            os.makedirs(os.path.dirname(dst), exist_ok=True)
            before += os.path.getsize(src)
            if name.endswith(".glb"):
                try:
                    pack_glb(src, dst, tex_dir, cache)
                except (ValueError, KeyError, struct.error) as e:
                    print(f"pack_hq: {src}: {e}; copied as it is", file=sys.stderr)
                    shutil.copyfile(src, dst)
            else:
                shutil.copyfile(src, dst)
    for root, _, files in os.walk(dst_root):
        after += sum(os.path.getsize(os.path.join(root, f)) for f in files)
    jpeg = "with JPEG" if Image is not None else "without Pillow: no JPEG step"
    print(f"pack_hq: {before / 1e6:.1f} MB delivered -> {after / 1e6:.1f} MB for the web ({len(cache)} textures, {jpeg})")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit("usage: pack_hq.py <assets/hq> <dist/assets/hq>")
    main(sys.argv[1], sys.argv[2])
