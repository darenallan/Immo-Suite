"""Planche de contrôle visuel : montage.py <moteur> <a4|a5> <cas1,cas2,...> [colonnes] -> build/montage-<moteur>-<fmt>.png"""
import sys
from pathlib import Path

import pymupdf

HERE = Path(__file__).parent
engine, fmt, ids = sys.argv[1], sys.argv[2], sys.argv[3].split(",")
cols = int(sys.argv[4]) if len(sys.argv) > 4 else 4
files = [next((HERE / "build" / engine).glob(f"{i}*-{fmt}.pdf")) for i in ids]
src = pymupdf.open(files[0])
pw, ph = src[0].rect.width, src[0].rect.height
scale, gap = 0.78, 10
rows = (len(files) + cols - 1) // cols
out = pymupdf.open()
page = out.new_page(width=cols * (pw * scale + gap) + gap, height=rows * (ph * scale + gap) + gap)
for k, f in enumerate(files):
    r, c = divmod(k, cols)
    x, y = gap + c * (pw * scale + gap), gap + r * (ph * scale + gap)
    rect = pymupdf.Rect(x, y, x + pw * scale, y + ph * scale)
    page.show_pdf_page(rect, pymupdf.open(f), 0)
    page.draw_rect(rect, color=(0.6, 0.6, 0.6), width=0.5)
dst = HERE / "build" / f"montage-{engine}-{fmt}.png"
page.get_pixmap(dpi=100).save(str(dst))
print(dst)
