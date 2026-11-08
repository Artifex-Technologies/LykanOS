#!/usr/bin/env python3
"""Builds LykanOS's logo set from assets/logo/lykanos-source.png (the maze wolf's head, white lines on black, prepared by
scripts/gen-source.py from my picture), assets/logo/lykanos-source.svg (its straightened outline) and assets/logo/lykanos-eyes.png (the inside of its eyes). Run this again after
changing the source.

    python3 scripts/gen-source.py && python3 scripts/gen-logo.py

Needs Pillow and potrace (for the SVGs). Writes:
    assets/logo/lykanos-{white,mint,green,rosewood,grey,black}.png    1024 px, transparent background, one colour
    assets/logo/lykanos-{white,mint,green,rosewood,grey,black}.svg    traced vector
    assets/logo/lykanos-duo.png        mint walls with Rosewood eyes (lifted for dark backgrounds), filled: the icon and the splash
    profile/airootfs/usr/share/lykanos/logo/                 the set as shipped, and lykanos-splash.png (Yggdrasil's splash)
    profile/airootfs/usr/share/icons/hicolor/<size>/apps/lykanos.png   (the duo, for dark UIs; os-release's LOGO)
Colours (decided 2026-11-07): the deep teal kept from Niflheim (#437569, the accent; mint #9fd1c3 on dark backgrounds) and
Rosewood #65000b added, BlackArch's kind of red. Rosewood itself is too dark to read on a dark background, so the eyes on dark
use it lifted (#b0122a); the rosewood logo tone is for light backgrounds.
"""
import os
import shutil
import subprocess
import sys
import tempfile

from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "assets", "logo", "lykanos-source.png")
EYES = os.path.join(ROOT, "assets", "logo", "lykanos-eyes.png")
SRC_SVG = os.path.join(ROOT, "assets", "logo", "lykanos-source.svg")
OUT = os.path.join(ROOT, "assets", "logo")
SHIPPED = os.path.join(ROOT, "profile", "airootfs", "usr", "share", "lykanos", "logo")
ICONS = os.path.join(ROOT, "profile", "airootfs", "usr", "share", "icons", "hicolor")
TONES = {"white": (255, 255, 255), "mint": (159, 209, 195), "green": (67, 117, 105), "rosewood": (101, 0, 11), "grey": (142, 142, 147), "black": (0, 0, 0)}
EYE_ON_DARK = (176, 18, 42)  # Rosewood #65000b lifted so it reads on a dark background


def alpha_of(gray):
    """Brightness becomes opacity: white lines on black turn into opaque shapes on transparency (dark noise and the soft edge's
    last bit of grey cut off, as the levels 14%..86% do)."""
    lo, hi = 0.14 * 255, 0.86 * 255
    return gray.point(lambda v: 0 if v <= lo else 255 if v >= hi else int((v - lo) * 255 / (hi - lo)))


def main():
    gray = Image.open(SRC).convert("L").resize((1024, 1024), Image.LANCZOS)
    alpha = alpha_of(gray)
    os.makedirs(OUT, exist_ok=True)
    for name, rgb in TONES.items():
        im = Image.new("RGBA", (1024, 1024), rgb + (0,))
        im.putalpha(alpha)
        im.save(os.path.join(OUT, f"lykanos-{name}.png"), optimize=True)
    # vector: the straightened outline gen-source.py made (sharp corners), recoloured; without it, traced here
    text = None
    if os.path.exists(SRC_SVG):
        text = open(SRC_SVG).read()
    elif shutil.which("potrace"):
        with tempfile.TemporaryDirectory() as tmp:
            pbm = os.path.join(tmp, "trace.pbm")
            # potrace traces black: the white lines become black on white
            gray.point(lambda v: 0 if v >= 128 else 255).convert("1").save(pbm)
            svg = os.path.join(tmp, "trace.svg")
            subprocess.run(["potrace", "-s", "-o", svg, "--turdsize", "20", "--alphamax", "0", "--opttolerance", "0.4", pbm], check=True)
            text = open(svg).read()
    else:
        print("potrace is not installed: SVGs left as they were", file=sys.stderr)
    if text:
        for name, rgb in TONES.items():
            colour = "#%02x%02x%02x" % rgb
            with open(os.path.join(OUT, f"lykanos-{name}.svg"), "w") as f:
                f.write(text.replace('fill="#000000"', f'fill="{colour}"'))
    os.makedirs(SHIPPED, exist_ok=True)
    for name in TONES:
        for ext in ("png", "svg"):
            p = os.path.join(OUT, f"lykanos-{name}.{ext}")
            if os.path.exists(p):
                shutil.copy(p, SHIPPED)
    # the duo: mint walls, red eyes
    eyes = Image.open(EYES).convert("L").resize((1024, 1024), Image.LANCZOS)
    duo = Image.new("RGBA", (1024, 1024), TONES["mint"] + (0,))
    duo.putalpha(alpha)
    red = Image.new("RGBA", (1024, 1024), EYE_ON_DARK + (255,))
    red.putalpha(eyes)  # the inside of the eyes, under the walls
    duo = Image.alpha_composite(red, duo)
    duo.save(os.path.join(OUT, "lykanos-duo.png"), optimize=True)
    shutil.copy(os.path.join(OUT, "lykanos-duo.png"), SHIPPED)
    mint = duo
    # the splash after signing in, the size Labyrinth's is
    mint.resize((340, 340), Image.LANCZOS).save(os.path.join(SHIPPED, "lykanos-splash.png"), optimize=True)
    for s in (512, 256, 128, 64, 48, 32):
        d = os.path.join(ICONS, f"{s}x{s}", "apps")
        os.makedirs(d, exist_ok=True)
        mint.resize((s, s), Image.LANCZOS).save(os.path.join(d, "lykanos.png"), optimize=True)
    print(f"logo set written to {OUT} and the profile")


if __name__ == "__main__":
    main()
