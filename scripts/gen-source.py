#!/usr/bin/env python3
"""Prepares LykanOS's mark for scripts/gen-logo.py: a wolf's head of maze walls, white lines on black. I made the picture
with Gemini from my Labyrinth and Niflheim logos (2026-11-07); it is assets/logo/lykanos-original.jpg, kept as it came.

    python3 scripts/gen-source.py && python3 scripts/gen-logo.py

Writes assets/logo/lykanos-source.png (the picture in greys, 1024 px) and assets/logo/lykanos-eyes.png (the inside of each
eye, found by filling the black from a point inside it, so gen-logo.py can colour the eyes). Needs Pillow and numpy.
A new picture: replace lykanos-original.jpg (white lines on black, square) and check EYE_SEEDS still fall inside the eyes.
"""
import os
from collections import deque

import numpy as np
from PIL import Image, ImageFilter

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ORIGINAL = os.path.join(ROOT, "assets", "logo", "lykanos-original.jpg")
OUT = os.path.join(ROOT, "assets", "logo", "lykanos-source.png")
EYES = os.path.join(ROOT, "assets", "logo", "lykanos-eyes.png")
EYE_SEEDS = [(390, 455), (634, 455)]  # a point inside each eye, in the 1024 px picture
LARGEST_EYE = 20000  # pixels: a fill that grows past this is not inside an eye (a seed in the open)


def fill(dark, seed):
    """The enclosed dark area around `seed`, as a mask."""
    x0, y0 = seed
    if not dark[y0, x0]:
        raise SystemExit(f"eye point {seed} is on a line, not inside an eye: move it in EYE_SEEDS")
    seen = np.zeros(dark.shape, bool)
    q = deque([(y0, x0)])
    seen[y0, x0] = True
    n = 0
    h, w = dark.shape
    while q:
        y, x = q.popleft()
        n += 1
        if n > LARGEST_EYE:
            raise SystemExit(f"eye point {seed} is not inside a closed eye: move it in EYE_SEEDS")
        for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            ny, nx = y + dy, x + dx
            if 0 <= ny < h and 0 <= nx < w and dark[ny, nx] and not seen[ny, nx]:
                seen[ny, nx] = True
                q.append((ny, nx))
    return seen


def main():
    gray = Image.open(ORIGINAL).convert("L").resize((1024, 1024), Image.LANCZOS)
    gray.save(OUT, optimize=True)
    dark = np.array(gray) < 128
    eyes = np.zeros(dark.shape, bool)
    for seed in EYE_SEEDS:
        eyes |= fill(dark, seed)
    # a pixel wider, so the colour runs under the lines' soft edge instead of leaving a dark rim
    mask = Image.fromarray((eyes * 255).astype(np.uint8)).filter(ImageFilter.MaxFilter(5))
    mask.save(EYES, optimize=True)
    print(f"wrote {OUT} and {EYES}")


if __name__ == "__main__":
    main()
