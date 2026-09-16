#!/usr/bin/env python3
"""Build hicolor PNGs from marius-agenda-magic-source.png (round, transparent corners)."""
from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parent
SOURCE = ROOT / "marius-agenda-magic-source.png"
SIZES = (48, 128, 256, 512)


def _lerp(c1: str, c2: str, t: float) -> tuple[int, int, int]:
    a = int(c1[1:3], 16), int(c1[3:5], 16), int(c1[5:7], 16)
    b = int(c2[1:3], 16), int(c2[3:5], 16), int(c2[5:7], 16)
    return (
        int(a[0] + (b[0] - a[0]) * t),
        int(a[1] + (b[1] - a[1]) * t),
        int(a[2] + (b[2] - a[2]) * t),
    )


def _clip_circular_feather(img: Image.Image, feather: int) -> Image.Image:
    size = img.size[0]
    cx = size // 2
    r = size // 2 - max(1, size // 64)
    hard = Image.new("L", (size, size), 0)
    ImageDraw.Draw(hard).ellipse([cx - r, cx - r, cx + r, cx + r], fill=255)
    soft = hard.filter(ImageFilter.GaussianBlur(radius=max(1, feather)))
    out = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    out.paste(img, (0, 0), soft)
    return out


def _is_book_pixel(r: int, g: int, b: int, lum: float, chroma: int) -> bool:
    if chroma > 38:
        return True
    if lum > 155:
        return True
    if r > 70 and b > 90 and g < r * 0.85:
        return True
    if r > 160 and g > 120 and b < 130:
        return True
    return False


def prepare_magic_source(im: Image.Image) -> Image.Image:
    """Transparent corners + sober slate disk behind the book."""
    im = im.convert("RGBA")
    w, h = im.size
    cx, cy = w / 2, h / 2
    max_r = min(cx, cy)
    pixels: list[tuple[int, int, int, int]] = []
    for y in range(h):
        for x in range(w):
            r, g, b, a = im.getpixel((x, y))
            dist = ((x - cx) ** 2 + (y - cy) ** 2) ** 0.5 / max_r
            chroma = max(r, g, b) - min(r, g, b)
            lum = (r + g + b) / 3

            if dist > 1.02 or (chroma < 22 and lum > 185 and dist > 0.42):
                pixels.append((0, 0, 0, 0))
                continue

            if _is_book_pixel(r, g, b, lum, chroma):
                pixels.append((r, g, b, a))
                continue

            t = min(1.0, dist * 0.95)
            bg = _lerp("#2c323a", "#12151a", t)
            blend = 0.88 if dist > 0.35 else 0.55
            nr = int(r * (1 - blend) + bg[0] * blend)
            ng = int(g * (1 - blend) + bg[1] * blend)
            nb = int(b * (1 - blend) + bg[2] * blend)
            pixels.append((nr, ng, nb, 255))

    out = Image.new("RGBA", (w, h))
    out.putdata(pixels)
    return _clip_circular_feather(out, feather=max(3, w // 32))


def bake(size: int, master: Image.Image) -> Image.Image:
    img = master.resize((size, size), Image.Resampling.LANCZOS)
    return _clip_circular_feather(img, feather=max(2, size // 26))


def main() -> None:
    if not SOURCE.is_file():
        raise SystemExit(f"Missing source: {SOURCE}")
    master = prepare_magic_source(Image.open(SOURCE))
    for s in SIZES:
        out = ROOT / f"marius-agenda-{s}.png"
        bake(s, master).save(out, optimize=True)
        print(f"wrote {out}")


if __name__ == "__main__":
    main()
