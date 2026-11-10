#!/usr/bin/env python3
"""Prepares LykanOS's mark for scripts/gen-logo.py: a wolf's head of maze walls, white lines on black. I made the first picture
with Gemini from my Labyrinth logo and the first LykanOS logo and then drew more maze walls into it by hand (2026-11-07); it is
assets/logo/lykanos-original.jpg, kept as it came.

    python3 scripts/gen-source.py && python3 scripts/gen-logo.py

What it does, so lines drawn with a finger come out straight:
  1. centres the drawing on a square with a margin and scales it to 1024 px;
  2. traces it with potrace as straight segments only (no curves);
  3. straightens every outline: runs of short wobbly segments within TOLERANCE px of one straight line become that line
     (Douglas-Peucker), so a wall's edge is one clean edge and its corners stay sharp;
  4. renders the straightened outline back to a picture and finds the inside of each eye (filled from a point in it).

Writes assets/logo/lykanos-source.svg (the straightened outline, black), assets/logo/lykanos-source.png (white on black,
1024 px) and assets/logo/lykanos-eyes.png. Needs Pillow, numpy, potrace and rsvg-convert (librsvg).
A new picture: replace lykanos-original.jpg (white lines on black) and check EYE_SEEDS still fall inside the eyes.
"""
import math
import os
import re
import subprocess
import sys
import tempfile
from collections import deque

import numpy as np
from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LOGO = os.path.join(ROOT, "assets", "logo")
ORIGINAL = os.path.join(LOGO, "lykanos-original.jpg")
SVG = os.path.join(LOGO, "lykanos-source.svg")
OUT = os.path.join(LOGO, "lykanos-source.png")
EYES = os.path.join(LOGO, "lykanos-eyes.png")

MARGIN = 1.10  # the square is this much larger than the drawing
TOLERANCE = 4.0  # px at 1024: wobbles smaller than this are straightened away
EYE_SEEDS = [(394, 462), (630, 462)]  # a point inside each eye, in the 1024 px picture
LARGEST_EYE = 20000  # pixels: a fill that grows past this is not inside an eye (a seed in the open)


def square(img):
    """The drawing centred on a black square with a margin, 1024 px."""
    g = np.array(img)
    ys, xs = np.nonzero(g > 128)
    cx, cy = (xs.min() + xs.max()) / 2, (ys.min() + ys.max()) / 2
    side = int(max(xs.max() - xs.min(), ys.max() - ys.min()) * MARGIN)
    sq = Image.new("L", (side, side), 0)
    sq.paste(img, (int(side / 2 - cx), int(side / 2 - cy)))
    return sq.resize((1024, 1024), Image.LANCZOS)


def polygons(d):
    """potrace's path data (M, l and z) as lists of absolute points."""
    toks = re.findall(r"[MmLlZz]|-?\d+(?:\.\d+)?", d)
    res, cur, cmd, pos, i = [], None, None, (0.0, 0.0), 0
    while i < len(toks):
        t = toks[i]
        if t in "MmLlZz":
            cmd = t
            i += 1
            if t in "Zz" and cur:
                res.append(cur)
                cur = None
            continue
        x, y = float(toks[i]), float(toks[i + 1])
        i += 2
        if cmd in "Mm":
            pos = (x, y) if cmd == "M" else (pos[0] + x, pos[1] + y)
            if cur:
                res.append(cur)
            cur = [pos]
            cmd = "L" if cmd == "M" else "l"
        else:
            pos = (x, y) if cmd == "L" else (pos[0] + x, pos[1] + y)
            cur.append(pos)
    if cur:
        res.append(cur)
    return res


def simplify(pts, eps):
    """Douglas-Peucker: the fewest points that stay within eps of the line."""
    if len(pts) < 3:
        return pts
    (x1, y1), (x2, y2) = pts[0], pts[-1]
    length = math.hypot(x2 - x1, y2 - y1)
    best, idx = -1.0, 0
    for k in range(1, len(pts) - 1):
        x0, y0 = pts[k]
        d = abs((y2 - y1) * x0 - (x2 - x1) * y0 + x2 * y1 - y2 * x1) / length if length else math.hypot(x0 - x1, y0 - y1)
        if d > best:
            best, idx = d, k
    if best > eps:
        return simplify(pts[: idx + 1], eps)[:-1] + simplify(pts[idx:], eps)
    return [pts[0], pts[-1]]


def simplify_closed(p, eps):
    """A closed outline, split at its two points farthest apart and each half simplified."""
    if len(p) < 4:
        return p
    b = max(range(len(p)), key=lambda k: (p[k][0] - p[0][0]) ** 2 + (p[k][1] - p[0][1]) ** 2)
    a = max(range(len(p)), key=lambda k: (p[k][0] - p[b][0]) ** 2 + (p[k][1] - p[b][1]) ** 2)
    a, b = min(a, b), max(a, b)
    return simplify(p[a : b + 1], eps)[:-1] + simplify(p[b:] + p[: a + 1], eps)[:-1]


def fill(dark, seed):
    """The enclosed dark area around `seed`, as a mask."""
    x0, y0 = seed
    if not dark[y0, x0]:
        sys.exit(f"eye point {seed} is on a line, not inside an eye: move it in EYE_SEEDS")
    seen = np.zeros(dark.shape, bool)
    q = deque([(y0, x0)])
    seen[y0, x0] = True
    n = 0
    h, w = dark.shape
    while q:
        y, x = q.popleft()
        n += 1
        if n > LARGEST_EYE:
            sys.exit(f"eye point {seed} is not inside a closed eye: move it in EYE_SEEDS")
        for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            ny, nx = y + dy, x + dx
            if 0 <= ny < h and 0 <= nx < w and dark[ny, nx] and not seen[ny, nx]:
                seen[ny, nx] = True
                q.append((ny, nx))
    return seen


def main():
    src = square(Image.open(ORIGINAL).convert("L"))
    with tempfile.TemporaryDirectory() as tmp:
        pbm, traced, white, png = (os.path.join(tmp, n) for n in ("in.pbm", "traced.svg", "white.svg", "out.png"))
        # potrace traces black: the white walls become black on white
        src.point(lambda v: 0 if v >= 128 else 255).convert("1").save(pbm)
        subprocess.run(["potrace", "-s", "-o", traced, "--turdsize", "40", "--alphamax", "0", "--opttolerance", "0.2", pbm], check=True)
        text = open(traced).read()
        # potrace's units are tenths of a pixel (its transform scales by 0.1)
        counts = [0, 0]

        def straighten(m):
            out = []
            for p in polygons(m.group(1)):
                q = simplify_closed(p, TOLERANCE * 10)
                counts[0] += len(p)
                counts[1] += len(q)
                if len(q) >= 3:
                    out.append("M" + " L".join(f"{x:.0f} {y:.0f}" for x, y in q) + " Z")
            return 'd="' + " ".join(out) + '"'

        text = re.sub(r'd="([^"]*)"', straighten, text, flags=re.S)
        with open(SVG, "w") as f:
            f.write(text)
        with open(white, "w") as f:
            f.write(text.replace('fill="#000000"', 'fill="#ffffff"'))
        subprocess.run(["rsvg-convert", "-w", "1024", "-h", "1024", "-b", "black", white, "-o", png], check=True)
        gray = Image.open(png).convert("L")
    gray.save(OUT, optimize=True)
    dark = np.array(gray) < 128
    eyes = np.zeros(dark.shape, bool)
    for seed in EYE_SEEDS:
        eyes |= fill(dark, seed)
    # a little wider, so the colour runs under the walls' soft edge instead of leaving a dark rim
    from PIL import ImageFilter

    Image.fromarray((eyes * 255).astype(np.uint8)).filter(ImageFilter.MaxFilter(5)).save(EYES, optimize=True)
    print(f"outline points {counts[0]} -> {counts[1]}; wrote {SVG}, {OUT} and {EYES}")


if __name__ == "__main__":
    main()
