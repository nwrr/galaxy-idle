"""Orkestrasi `do_all`: regen semua .ans celestial + publish portrait."""
import os
from . import catalog
from .celestial import render_celestial_rgb
from .ansi import to_halfblock


def do_all():
    for base, kind, biome, tag, cols, rows in catalog.celestial_jobs():
        # Grid piksel persegi: lebar=cols, tinggi=2*rows (tiap sel 2 piksel vertikal).
        rgb, mask = render_celestial_rgb(kind, biome, cols, rows * 2)
        for depth in catalog.DEPTHS:
            rel = catalog.ans_path(base, tag, depth)
            catalog.write_txt(rel, to_halfblock(rgb, mask, depth))
            print(f"gen {rel}")
    # Banner = logo ANSI-shadow DI-AUTHOR TANGAN (gaya LazyVim). Sengaja TIDAK di-generate agar
    # `all` tak menimpa logo. Edit langsung: assets/banner/title.txt & title_small.txt.
    print("skip banner (logo ANSI-shadow di-author tangan; tidak di-generate)")
    miss = 0
    for out, src, W, H in catalog.portrait_jobs():
        if not os.path.exists(src):
            miss += 1
            print(f"skip {out} (gambar sumber belum ada: {os.path.relpath(src, catalog.ROOT)})")
            continue
        catalog.publish_portrait(src, out, W)
        print(f"gen {out} (image, dirender via ratatui-image)")
    if miss:
        print(f"\n{miss} portrait dilewati — taruh PNG di "
              "assets/source/characters/.. lalu jalankan lagi.")
