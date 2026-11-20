"""Builds the macOS app icon: the transparent logo on a dark rounded square
following Apple's icon grid (1024 canvas, 824 square, ~185 corner radius).

Usage: python3 assets/make_macos_icon.py  ->  assets/icon-macos.png
"""
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

HERE = Path(__file__).parent
SIZE, BOX, RADIUS = 1024, 824, 185
TOP, BOTTOM = (46, 46, 52), (18, 18, 21)  # dark vertical gradient

logo = Image.open(HERE / "icon.png").convert("RGBA")
logo = logo.crop(logo.getbbox())

canvas = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
offset = (SIZE - BOX) // 2

# Soft shadow under the square.
shadow = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
ImageDraw.Draw(shadow).rounded_rectangle(
    (offset, offset + 12, offset + BOX, offset + BOX + 12), RADIUS, fill=(0, 0, 0, 110)
)
canvas.alpha_composite(shadow.filter(ImageFilter.GaussianBlur(14)))

# Gradient square.
gradient = Image.new("RGBA", (BOX, BOX))
for y in range(BOX):
    t = y / (BOX - 1)
    color = tuple(round(a + (b - a) * t) for a, b in zip(TOP, BOTTOM)) + (255,)
    ImageDraw.Draw(gradient).line([(0, y), (BOX, y)], fill=color)
mask = Image.new("L", (BOX, BOX), 0)
ImageDraw.Draw(mask).rounded_rectangle((0, 0, BOX - 1, BOX - 1), RADIUS, fill=255)
canvas.paste(gradient, (offset, offset), mask)

# Logo centered, about 64% of the square.
target = round(BOX * 0.64)
scale = target / max(logo.size)
logo = logo.resize((round(logo.width * scale), round(logo.height * scale)), Image.LANCZOS)
canvas.alpha_composite(logo, ((SIZE - logo.width) // 2, (SIZE - logo.height) // 2))

canvas.save(HERE / "icon-macos.png")
print("written", HERE / "icon-macos.png")
