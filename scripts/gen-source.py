#!/usr/bin/env python3
"""Draws LykanOS's mark: Labyrinth OS's maze-arrow turned upside down into a wolf's head (the arrow's tip is the muzzle, its
wings the cheeks, the inner diamond and middle wall the bridge of the nose), with ears, eyes and a nose drawn as maze walls of
the same width. First version 2026-11-07, meant to be refined: the shapes are the numbers below.

    python3 scripts/gen-source.py && python3 scripts/gen-logo.py

Reads assets/logo/labyrinth-base.png (Labyrinth's white mark, copied from the Labyrinth_OS repository). Writes
assets/logo/lykanos-source.png (white lines on black, 1024 px) and assets/logo/lykanos-eyes.png (the eyes alone, so
gen-logo.py can colour them). Needs Pillow and numpy.
"""
import math
import os
from collections import deque

import numpy as np
from PIL import Image, ImageDraw

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BASE = os.path.join(ROOT, "assets", "logo", "labyrinth-base.png")
OUT = os.path.join(ROOT, "assets", "logo", "lykanos-source.png")
EYES = os.path.join(ROOT, "assets", "logo", "lykanos-eyes.png")

S = 4  # drawn at 4x, then scaled down (smooth edges)
K, OY, LW = 0.80, 190, 24  # the flipped maze's scale, its top, and the wall width at 1024 px
EAR_TIP = (215, 60)  # the left ear's tip (the right ear is its mirror)
EAR_BASE_OUT = (190, 238)  # where the ear meets the cheek, in the flipped maze's own coordinates
EAR_BASE_IN = (430, 106)
EYE = (400, 478)  # the left eye's centre
NOSE_Y = 838


def miter(pts, h):
    """Outline of a thick open polyline with mitred corners and square ends, as the maze's walls are drawn."""

    def norm(a, b):
        dx, dy = b[0] - a[0], b[1] - a[1]
        l = math.hypot(dx, dy)
        return (-dy / l, dx / l), (dx / l, dy / l)

    left, right = [], []
    n = len(pts)
    for i, p in enumerate(pts):
        if i in (0, n - 1):
            nv, tv = norm(pts[0], pts[1]) if i == 0 else norm(pts[-2], pts[-1])
            s = -1 if i == 0 else 1
            p = (p[0] + s * tv[0] * h, p[1] + s * tv[1] * h)
            left.append((p[0] + nv[0] * h, p[1] + nv[1] * h))
            right.append((p[0] - nv[0] * h, p[1] - nv[1] * h))
        else:
            n1, _ = norm(pts[i - 1], p)
            n2, _ = norm(p, pts[i + 1])
            mx, my = n1[0] + n2[0], n1[1] + n2[1]
            ml = math.hypot(mx, my)
            mx, my = mx / ml, my / ml
            k = h / max(0.45, mx * n1[0] + my * n1[1])
            left.append((p[0] + mx * k, p[1] + my * k))
            right.append((p[0] - mx * k, p[1] - my * k))
    return left + right[::-1]


def remove_crumbs(a, smallest=900):
    """Bits of wall the cut-outs left behind, too small to read as a wall, are cleared."""
    on = a > 100
    seen = np.zeros_like(on)
    h, w = a.shape
    for y0, x0 in zip(*np.nonzero(on)):
        if seen[y0, x0]:
            continue
        comp, q = [], deque([(y0, x0)])
        seen[y0, x0] = True
        while q:
            y, x = q.popleft()
            comp.append((y, x))
            for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                ny, nx = y + dy, x + dx
                if 0 <= ny < h and 0 <= nx < w and on[ny, nx] and not seen[ny, nx]:
                    seen[ny, nx] = True
                    q.append((ny, nx))
        if len(comp) < smallest:
            for y, x in comp:
                a[max(0, y - 1):y + 2, max(0, x - 1):x + 2] = 0
    return a


def main():
    src = Image.open(BASE).split()[-1].rotate(180)
    face = src.resize((int(1024 * K * S), int(1024 * K * S)), Image.LANCZOS)
    canvas = Image.new("L", (1024 * S, 1024 * S), 0)
    canvas.paste(face, (int((512 - 512 * K) * S), int(OY * S)))
    eyes = Image.new("L", (1024 * S, 1024 * S), 0)
    d, de = ImageDraw.Draw(canvas), ImageDraw.Draw(eyes)

    def T(x, y):  # a point of the flipped maze, in the 1024 px picture
        return (512 + (x - 512) * K, OY + y * K)

    def up(pts):
        return [(x * S, y * S) for x, y in pts]

    def lerp(a, b, t):
        return (a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t)

    for side in (-1, 1):

        def m(p):  # the left half as drawn, the right half mirrored
            return (512 + side * (p[0] - 512), p[1])

        base_out, base_in, tip = T(*EAR_BASE_OUT), T(*EAR_BASE_IN), EAR_TIP
        # the ear's outer wall, closing onto the head
        d.polygon(up(miter([m(base_out), m(tip), m(base_in)], LW / 2)), fill=255)
        # an inner wall along the ear's outer edge, open at the bottom like a maze corridor
        ox, oy = base_in[0] - base_out[0], base_in[1] - base_out[1]
        ol = math.hypot(ox, oy)
        ix, iy = ox / ol * 2.3 * LW, oy / ol * 2.3 * LW
        a, b, c = lerp(base_out, tip, 0.30), lerp(base_out, tip, 0.72), lerp(tip, base_in, 0.36)
        inner = [m((a[0] + ix, a[1] + iy)), m((b[0] + ix * 0.55, b[1] + iy * 0.55)), m((c[0] + ix * 0.2, c[1] + 18))]
        d.polygon(up(miter(inner, LW / 2)), fill=255)
        # an eye: angular and slanted, in a cut-out so the maze does not swallow it
        ex, ey = EYE
        pad = [(ex - 92, ey - 46), (ex + 26, ey - 28), (ex + 90, ey + 38), (ex - 30, ey + 32)]
        eye = [(ex - 66, ey - 30), (ex + 18, ey - 10), (ex + 62, ey + 28), (ex - 22, ey + 14)]
        d.polygon(up([m(p) for p in pad]), fill=0)
        d.polygon(up([m(p) for p in eye]), fill=255)
        de.polygon(up([m(p) for p in eye]), fill=255)
    # the nose: a wedge across the bridge line near the muzzle's tip
    d.polygon(up([(468, NOSE_Y - 16), (556, NOSE_Y - 16), (512, NOSE_Y + 66)]), fill=0)
    d.polygon(up([(478, NOSE_Y - 6), (546, NOSE_Y - 6), (512, NOSE_Y + 40)]), fill=255)
    a = remove_crumbs(np.array(canvas.resize((1024, 1024), Image.LANCZOS)))
    Image.fromarray(a).save(OUT, optimize=True)
    eyes.resize((1024, 1024), Image.LANCZOS).save(EYES, optimize=True)
    print(f"wrote {OUT} and {EYES}")


if __name__ == "__main__":
    main()
