#!/usr/bin/env python3
"""Generator asset (entrypoint CLI). Logika di paket `genassets/` (lihat CONVENTIONS.md §3).

Celestial (planet/star/ship) = render prosedural numpy → ANSI half-block BERWARNA (.ans): tiap sel
▀ = 2 piksel vertikal (fg atas / bg bawah) → resolusi 2× + warna penuh; warna per-biome. Tiap base
ditulis 3 ukuran (sm/md/lg) × 3 depth (tc/256/16) = 9 `.ans` di `sprites/<base>/`.
Karakter = konversi gambar (PNG via ratatui-image). Banner = logo ANSI-shadow tangan.

Usage:
  scripts/gen_assets.py all                       # regen .ans celestial + publish portrait
  scripts/gen_assets.py planet terran 40 20        # cetak ANSI ke stdout (debug), depth tc
  scripts/gen_assets.py planet lavaworld 64 32 256 # depth opsional: tc|256|16
  scripts/gen_assets.py star 40 20
  scripts/gen_assets.py ship 20 10 16
  scripts/gen_assets.py portrait <src.png> 44 32
  scripts/gen_assets.py banner "STELLAR IDLE" 80 12
Catatan: argumen ukuran = sel cols×rows (grid piksel = cols × 2*rows, persegi).
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from genassets import textart, celestial, ansi
from genassets.build import do_all
from genassets.catalog import DEPTHS


def main():
    args = [a for a in sys.argv[1:] if a != "--ramp"]
    if "--ramp" in sys.argv:
        i = sys.argv.index("--ramp")
        if sys.argv[i + 1] == "short":
            textart.RAMP = textart.RAMP_SHORT
        args = [a for a in args if a not in ("short", "long")]
    if not args or args[0] == "all":
        do_all(); return
    cmd = args[0]
    if cmd in ("planet", "star", "ship", "asteroid"):
        biome = args[1] if cmd == "planet" else None
        cols = int(args[-2]); rows = int(args[-1])
        depth = "tc"
        for d in DEPTHS:
            if d in args:
                depth = d
        kind = "asteroid" if cmd == "asteroid" else cmd
        rgb, mask = celestial.render_celestial_rgb(kind, biome, cols, rows * 2)
        print("\n".join(ansi.to_halfblock(rgb, mask, depth)))
    elif cmd == "portrait":
        lum, mask = textart.render_portrait(args[1], int(args[2]), int(args[3]))
        print("\n".join(textart.lum_to_chars(lum, mask, fill_subject=False)))
    elif cmd == "banner":
        lum, mask = textart.render_banner(args[1], int(args[2]), int(args[3]))
        print("\n".join(textart.lum_to_chars(lum, mask, fill_subject=False)))
    else:
        print(__doc__); sys.exit(1)


if __name__ == "__main__":
    main()
