#!/usr/bin/env python3
"""Generates LykanOS's default wallpaper: deep teal, darker at the edges, a soft lighter field in the middle and a glow low
on the right, in the colours of the reference picture chosen on 2026-11-07 (#101c1c edges, #24413c middle, #437569 glow),
and Rosewood (#65000b) glowing up from the lower left.
Placeholder art for the look; replace it with real artwork freely.

    python3 scripts/gen-wallpaper.py [OUT.png] [WIDTH HEIGHT]

Needs Pillow. A little noise is added so the gradient does not band on 8-bit screens.
"""
import os
import random
import sys

from PIL import Image, ImageChops, ImageDraw, ImageFilter

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
out = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, "profile", "airootfs", "usr", "share", "wallpapers", "lykanos", "lykanos.png")
w = int(sys.argv[2]) if len(sys.argv) > 2 else 2560
h = int(sys.argv[3]) if len(sys.argv) > 3 else 1440

EDGE, MIDDLE, GLOW = (16, 28, 28), (36, 65, 60), (67, 117, 105)
ROSEWOOD = (101, 0, 11)  # #65000b, added 2026-11-07: BlackArch's kind of red beside the teal


def layer(colour, shape, blur):
    """`colour` where `shape` (a drawing function on a mask) is, blurred to a soft edge."""
    mask = Image.new("L", (w, h), 0)
    shape(ImageDraw.Draw(mask))
    mask = mask.filter(ImageFilter.GaussianBlur(blur))
    return Image.new("RGB", (w, h), colour), mask


def main():
    img = Image.new("RGB", (w, h), EDGE)
    # the lighter field: a wide rounded shape, very soft
    field, mask = layer(MIDDLE, lambda d: d.rounded_rectangle([w * 0.16, h * 0.08, w * 0.84, h * 0.94], radius=int(h * 0.3), fill=230), h * 0.12)
    img = Image.composite(field, img, mask)
    # the glow low on the right
    glow, mask = layer(GLOW, lambda d: d.ellipse([w * 0.70, h * 0.55, w * 1.08, h * 1.15], fill=170), h * 0.14)
    img = Image.composite(glow, img, mask)
    # Rosewood glowing up from the lower left, the teal's counterpart
    rose, mask = layer(ROSEWOOD, lambda d: d.ellipse([-w * 0.12, h * 0.50, w * 0.34, h * 1.18], fill=200), h * 0.16)
    img = Image.composite(rose, img, mask)
    # a darker band down each side, as in the reference
    shade, mask = layer((10, 18, 18), lambda d: (d.rectangle([0, 0, w * 0.05, h], fill=200), d.rectangle([w * 0.95, 0, w, h], fill=200)), w * 0.04)
    img = Image.composite(shade, img, mask)
    # noise against banding: +-1 per channel
    rnd = random.Random(7)
    noise = Image.frombytes("L", (w, h), bytes(rnd.randint(126, 129) for _ in range(w * h)))
    img = ImageChops.add(img, Image.merge("RGB", (noise, noise, noise)), 1.0, -127)
    os.makedirs(os.path.dirname(out), exist_ok=True)
    img.save(out, optimize=True)
    print(f"{out}: {w}x{h}")


if __name__ == "__main__":
    main()
