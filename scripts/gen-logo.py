#!/usr/bin/env python3
"""Builds Niflheim OS's logo set from assets/logo/niflheim-source.png (the snake-and-labyrinth mark: white lines on black, any
size; scaled to 1024 px first). Draft art from 2026-11-07, to be replaced freely: run this again after changing the source.

    python3 scripts/gen-logo.py

Needs Pillow and potrace (for the SVGs). Writes:
    assets/logo/niflheim-{white,mint,green,grey,black}.png    1024 px, transparent background
    assets/logo/niflheim-{white,mint,green,grey,black}.svg    traced vector
    profile/airootfs/usr/share/niflheim/logo/                 the set as shipped, and niflheim-splash.png (Yggdrasil's splash)
    profile/airootfs/usr/share/icons/hicolor/<size>/apps/niflheimos.png   (mint, for dark UIs; os-release's LOGO)
Tones: white and mint for dark backgrounds, green (Niflheim's accent, #437569) and black for light ones, grey in between.
The icon is "niflheimos" with no dash: icon lookup shortens "niflheim-os" to "niflheim", which another theme may hold.
"""
import os
import shutil
import subprocess
import sys
import tempfile

from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "assets", "logo", "niflheim-source.png")
OUT = os.path.join(ROOT, "assets", "logo")
SHIPPED = os.path.join(ROOT, "profile", "airootfs", "usr", "share", "niflheim", "logo")
ICONS = os.path.join(ROOT, "profile", "airootfs", "usr", "share", "icons", "hicolor")
TONES = {"white": (255, 255, 255), "mint": (159, 209, 195), "green": (67, 117, 105), "grey": (142, 142, 147), "black": (0, 0, 0)}


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
        im.save(os.path.join(OUT, f"niflheim-{name}.png"), optimize=True)
    # vector: the shapes traced once, then recoloured
    if shutil.which("potrace"):
        with tempfile.TemporaryDirectory() as tmp:
            pbm = os.path.join(tmp, "trace.pbm")
            # potrace traces black: the white lines become black on white
            gray.point(lambda v: 0 if v >= 128 else 255).convert("1").save(pbm)
            svg = os.path.join(tmp, "trace.svg")
            subprocess.run(["potrace", "-s", "-o", svg, "--turdsize", "20", "--alphamax", "0.9", "--opttolerance", "0.4", pbm], check=True)
            text = open(svg).read()
        for name, rgb in TONES.items():
            colour = "#%02x%02x%02x" % rgb
            with open(os.path.join(OUT, f"niflheim-{name}.svg"), "w") as f:
                f.write(text.replace('fill="#000000"', f'fill="{colour}"'))
    else:
        print("potrace is not installed: SVGs left as they were", file=sys.stderr)
    os.makedirs(SHIPPED, exist_ok=True)
    for name in TONES:
        for ext in ("png", "svg"):
            p = os.path.join(OUT, f"niflheim-{name}.{ext}")
            if os.path.exists(p):
                shutil.copy(p, SHIPPED)
    mint = Image.open(os.path.join(OUT, "niflheim-mint.png"))
    # the splash after signing in, the size Labyrinth's is
    mint.resize((340, 340), Image.LANCZOS).save(os.path.join(SHIPPED, "niflheim-splash.png"), optimize=True)
    for s in (512, 256, 128, 64, 48, 32):
        d = os.path.join(ICONS, f"{s}x{s}", "apps")
        os.makedirs(d, exist_ok=True)
        mint.resize((s, s), Image.LANCZOS).save(os.path.join(d, "niflheimos.png"), optimize=True)
    print(f"logo set written to {OUT} and the profile")


if __name__ == "__main__":
    main()
