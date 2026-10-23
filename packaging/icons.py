#!/usr/bin/env python3
"""Render CloudNode's app icon from TradeCloud's master drawing.

Usage: python3 packaging/icons.py <master.png>

The master is the square, full-bleed drawing of the running cheetah (2000 px, kept by TradeCloud,
not in this repository). Every file under assets/app-icon/ except README.md and LICENSE.txt is
written by this script: never edit the renders by hand, run it again instead. Needs Pillow.

  cloudnode-1024.png           full tile, 1024 px (README, store listings)
  cloudnode-macos-1024.png     the tile on Apple's icon grid (824 of 1024 px, centred)
  cloudnode.icns               macOS bundle icon, from the grid render
  cloudnode.ico                Windows icon, 16-256 px
  hicolor/<n>x<n>/apps/com.tradecloud.cloudnode.png   Linux icon theme, 16-512 px
"""

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "app-icon"
APP_ID = "com.tradecloud.cloudnode"
HICOLOR_SIZES = [16, 24, 32, 48, 64, 128, 256, 512]
ICO_SIZES = [16, 24, 32, 48, 64, 128, 256]
# Apple's grid: the tile fills 824 of 1024 px, so the Dock shows it at the size of other apps.
MAC_CANVAS = 1024
MAC_TILE = 824


def resized(image: Image.Image, size: int) -> Image.Image:
    return image.resize((size, size), Image.Resampling.LANCZOS)


def write_icns(grid: Image.Image, path: Path) -> None:
    """Every size Apple's iconset names (16-512 px at 1x and 2x) through `iconutil` on macOS.
    Elsewhere Pillow writes the file, without the 16 and 32 px 1x sizes."""
    if shutil.which("iconutil") is None:
        grid.save(path)
        return
    with tempfile.TemporaryDirectory() as tmp:
        iconset = Path(tmp) / "cloudnode.iconset"
        iconset.mkdir()
        for size in [16, 32, 128, 256, 512]:
            resized(grid, size).save(iconset / f"icon_{size}x{size}.png")
            resized(grid, size * 2).save(iconset / f"icon_{size}x{size}@2x.png")
        subprocess.run(["iconutil", "-c", "icns", "-o", str(path), str(iconset)], check=True)


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    master = Image.open(sys.argv[1]).convert("RGBA")
    if master.width != master.height or master.width < MAC_CANVAS:
        print(f"the master must be square and at least {MAC_CANVAS} px, not {master.width}x{master.height}", file=sys.stderr)
        return 1

    OUT.mkdir(parents=True, exist_ok=True)
    resized(master, 1024).save(OUT / "cloudnode-1024.png", optimize=True)

    for size in HICOLOR_SIZES:
        folder = OUT / "hicolor" / f"{size}x{size}" / "apps"
        folder.mkdir(parents=True, exist_ok=True)
        resized(master, size).save(folder / f"{APP_ID}.png", optimize=True)

    mac = Image.new("RGBA", (MAC_CANVAS, MAC_CANVAS), (0, 0, 0, 0))
    offset = (MAC_CANVAS - MAC_TILE) // 2
    mac.paste(resized(master, MAC_TILE), (offset, offset))
    mac.save(OUT / "cloudnode-macos-1024.png", optimize=True)
    write_icns(mac, OUT / "cloudnode.icns")

    resized(master, max(ICO_SIZES)).save(OUT / "cloudnode.ico", sizes=[(s, s) for s in ICO_SIZES])

    print(f"wrote {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
