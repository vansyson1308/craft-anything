"""Pack long rod surfaces as connected strips instead of tiny smart-UV islands.

The imported cage is made of straight, axis-aligned polygonal rods and a box
padlock. All geometry is triangulated. Return an approximate packing report.
"""
import math

from rectpack import (newPacker, MaxRectsBssf, MaxRectsBaf, MaxRectsBlsf,
                      MaxRectsBl, SkylineBl, SkylineMwf, GuillotineBssfSas,
                      GuillotineBafLas, GuillotineBlsfSas, SORT_AREA, SORT_LSIDE, SORT_PERI)


def unwrap(meshes, size=512, padding=1, axis_override=None):
    islands = []

    def add(obj, loops, width, height):
        if loops:
            islands.append((obj, loops, max(width, 1e-6), max(height, 1e-6)))

    for obj in meshes:
        data = obj.data
        if not data.uv_layers:
            data.uv_layers.new(name="UVMap")
        positions = [v.co for v in data.vertices]
        low = [min(p[i] for p in positions) for i in range(3)]
        high = [max(p[i] for p in positions) for i in range(3)]
        spans = [high[i] - low[i] for i in range(3)]
        if obj.name.startswith("padlock_body"):
            faces = {}
            for poly in data.polygons:
                axis = max(range(3), key=lambda i: abs(poly.normal[i]))
                sign = 1 if poly.normal[axis] >= 0 else -1
                faces.setdefault((axis, sign), []).extend(poly.loop_indices)
            for (axis, _), indices in faces.items():
                a, b = [i for i in range(3) if i != axis]
                points = [(idx, (positions[data.loops[idx].vertex_index][a] - low[a]) / spans[a],
                           (positions[data.loops[idx].vertex_index][b] - low[b]) / spans[b]) for idx in indices]
                add(obj, points, spans[a], spans[b])
            continue
        axis = axis_override if axis_override is not None else max(range(3), key=lambda i: spans[i])
        a, b = [i for i in range(3) if i != axis]
        center_a, center_b = (low[a] + high[a]) / 2, (low[b] + high[b]) / 2
        radius = (spans[a] + spans[b]) / 4
        angles = [math.atan2(p[b] - center_b, p[a] - center_a) % (2 * math.pi) for p in positions]
        seam = min(angles)
        sides, caps = [], {0: [], 1: []}
        for poly in data.polygons:
            if abs(poly.normal[axis]) > .8:
                cap = 0 if poly.center[axis] < (low[axis] + high[axis]) / 2 else 1
                for idx in poly.loop_indices:
                    p = positions[data.loops[idx].vertex_index]
                    caps[cap].append((idx, (p[a] - low[a]) / spans[a], (p[b] - low[b]) / spans[b]))
            else:
                vals = []
                for idx in poly.loop_indices:
                    vi = data.loops[idx].vertex_index
                    p = positions[vi]
                    angle = ((angles[vi] - seam) % (2 * math.pi)) / (2 * math.pi)
                    vals.append([idx, (p[axis] - low[axis]) / spans[axis], angle])
                if max(v[2] for v in vals) - min(v[2] for v in vals) > .5:
                    for v in vals:
                        if v[2] < .5:
                            v[2] += 1
                sides.extend(tuple(v) for v in vals)
        add(obj, sides, spans[axis], 2 * math.pi * radius)
        for points in caps.values():
            add(obj, points, spans[a], spans[b])

    def try_pack(scale, algorithm, sort):
        pack = newPacker(pack_algo=algorithm, sort_algo=sort, rotation=True)
        for i, (_, _, w, h) in enumerate(islands):
            pack.add_rect(max(1, math.ceil(w * scale)) + 2 * padding,
                          max(1, math.ceil(h * scale)) + 2 * padding, rid=i)
        pack.add_bin(size, size)
        pack.pack()
        rects = pack.rect_list()
        return rects if len(rects) == len(islands) else None

    left, chosen, best_algorithm = 0.0, None, None
    for algorithm in (MaxRectsBssf, MaxRectsBaf, MaxRectsBlsf, MaxRectsBl,
                      SkylineBl, SkylineMwf, GuillotineBssfSas, GuillotineBafLas,
                      GuillotineBlsfSas):
        for sort in (SORT_AREA, SORT_LSIDE, SORT_PERI):
            lower, upper, candidate = 0.0, 3000.0, None
            for _ in range(16):
                mid = (lower + upper) / 2
                packed = try_pack(mid, algorithm, sort)
                if packed is None:
                    upper = mid
                else:
                    lower, candidate = mid, packed
            if candidate is not None and lower > left:
                left, chosen, best_algorithm = lower, candidate, algorithm.__name__
    if chosen is None:
        raise RuntimeError("Could not pack UV islands")
    for _, x, y, w, h, index in chosen:
        obj, loops, source_w, source_h = islands[index]
        uv = obj.data.uv_layers.active.data
        inner_w, inner_h = w - 2 * padding, h - 2 * padding
        natural_w = max(1, math.ceil(source_w * left))
        natural_h = max(1, math.ceil(source_h * left))
        rotated = natural_w != natural_h and inner_w == natural_h and inner_h == natural_w
        for idx, u, v in loops:
            if rotated:
                u, v = v, 1 - u
            uv[idx].uv = ((x + padding + u * inner_w) / size, (y + padding + v * inner_h) / size)
    return {"islands": len(islands), "scale_px_per_m": left,
            "padding_px": padding, "algorithm": best_algorithm}
