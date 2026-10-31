"""Rend la première page d'un PDF en PNG (inspection visuelle). Usage : to_png.py entrée.pdf sortie.png [dpi]"""
import sys

import pymupdf

doc = pymupdf.open(sys.argv[1])
dpi = int(sys.argv[3]) if len(sys.argv) > 3 else 80
doc[0].get_pixmap(dpi=dpi).save(sys.argv[2])
print(f"{len(doc)} page(s), {doc[0].rect.width:.0f} × {doc[0].rect.height:.0f} pt")
