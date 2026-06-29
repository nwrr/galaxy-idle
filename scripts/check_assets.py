#!/usr/bin/env python3
"""Validasi dimensi asset terhadap assets/manifest.ron.

Usage:
  scripts/check_assets.py            # cek semua entri
  scripts/check_assets.py <id> ...   # cek id tertentu

Mengukur display width (East Asian Wide/Fullwidth = 2 sel, combining = 0, lain = 1).
Aturan:
  - Frame (w==0 atau h==0): PASS bila file ada (ukuran dinamis).
  - Fixed sprite/banner/portrait: FAIL bila tinggi > h atau lebar tampil > w.
    WARN bila underfill (<60% box) — bentuk mungkin terlalu kecil/"kurang mirip".
Exit code != 0 bila ada FAIL.
"""
import os
import re
import sys
import unicodedata

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MANIFEST = os.path.join(ROOT, "assets", "manifest.ron")

ENTRY_RE = re.compile(
    r'id:\s*"([^"]+)"\s*,\s*path:\s*"([^"]+)"\s*,\s*kind:\s*(\w+)\s*,\s*w:\s*(\d+)\s*,\s*h:\s*(\d+)'
)
ANSI_RE = re.compile(r"\x1b\[[0-9;]*m")

# Sprite celestial = .ans half-block: tiap base wajib punya 9 varian (3 ukuran × 3 depth).
SIZES = ["sm", "md", "lg"]
DEPTHS = ["tc", "256", "16"]


def disp_width(s: str) -> int:
    w = 0
    for ch in s:
        if unicodedata.combining(ch):
            continue
        w += 2 if unicodedata.east_asian_width(ch) in ("W", "F") else 1
    return w


def load_entries():
    with open(MANIFEST, encoding="utf-8") as f:
        text = f.read()
    return [
        (m.group(1), m.group(2), m.group(3), int(m.group(4)), int(m.group(5)))
        for m in ENTRY_RE.finditer(text)
    ]


def measure(path):
    with open(path, encoding="utf-8") as f:
        lines = f.read().split("\n")
    while lines and lines[-1] == "":
        lines.pop()
    lines = [ANSI_RE.sub("", ln) for ln in lines]   # buang escape warna sebelum ukur
    h = len(lines)
    w = max((disp_width(ln) for ln in lines), default=0)
    return w, h, lines


def check_variants(rel):
    """rel = path canonical celestial (mis sprites/planet_ocean/lg.tc.ans).
    → list varian yang hilang dari 9 (sm/md/lg × tc/256/16)."""
    m = re.match(r"(.*)/(sm|md|lg)\.(tc|256|16)\.ans$", rel)
    if not m:
        return None
    base_dir = m.group(1)
    missing = []
    for s in SIZES:
        for d in DEPTHS:
            p = os.path.join(ROOT, "assets", base_dir, f"{s}.{d}.ans")
            if not os.path.exists(p):
                missing.append(f"{s}.{d}")
    return missing


def main():
    wanted = set(sys.argv[1:])
    entries = load_entries()
    if wanted:
        entries = [e for e in entries if e[0] in wanted]
        if not entries:
            print(f"!! id tidak ditemukan di manifest: {', '.join(wanted)}")
            return 2
    fails = 0
    for aid, rel, kind, w, h in entries:
        path = os.path.join(ROOT, "assets", rel)
        if not os.path.exists(path):
            print(f"FAIL {aid:22} {rel:34} (file tidak ada)")
            fails += 1
            continue
        # Portrait = gambar (dirender via ratatui-image), bukan ASCII → validasi sbg image.
        if kind == "Portrait" or rel.lower().endswith(".png"):
            try:
                from PIL import Image
                with Image.open(path) as im:
                    iw, ih = im.size
                print(f"PASS {aid:22} {rel:34} img {iw}x{ih}")
            except Exception as e:
                print(f"FAIL {aid:22} {rel:34} (image rusak: {e})")
                fails += 1
            continue
        mw, mh, _ = measure(path)
        dynamic = (w == 0 or h == 0)
        if dynamic:
            print(f"PASS {aid:22} {rel:34} dyn  {mw}x{mh}")
            continue
        status = "PASS"
        notes = []
        if mw > w or mh > h:
            status = "FAIL"
            notes.append(f"melebihi box {w}x{h}")
            fails += 1
        elif mw < w * 0.6 or mh < h * 0.6:
            status = "WARN"
            notes.append("underfill <60%")
        missing = check_variants(rel)
        if missing:
            status = "FAIL"
            notes.append(f"varian hilang: {', '.join(missing)}")
            fails += 1
        note = ("  " + "; ".join(notes)) if notes else ""
        print(f"{status} {aid:22} {rel:34} {mw}x{mh} (box {w}x{h}){note}")
    print(f"\n{'OK' if fails == 0 else 'GAGAL'}: {len(entries)} dicek, {fails} FAIL")
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main())
