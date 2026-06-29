"""Registry job celestial & portrait + path + publish/tulis. Lihat CONVENTIONS.md (≤10 file/folder).

Sprite celestial dikelompokkan per base: `assets/sprites/<base>/<size>.<depth>.ans`
(11 sub-folder × 9 file) — bukan 99 file datar.
"""
import os

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ASSETS = os.path.join(ROOT, "assets")

BIOMES = ["terran", "ocean", "gasgiant", "deadworld", "crystalworld",
          "iceworld", "lavaworld", "ironworld"]

# (nama_base, kind, biome). 11 base celestial.
CELESTIAL_BASES = (
    [(f"planet_{b}", "planet", b) for b in BIOMES]
    + [("planet_asteroidbelt", "asteroid", None),
       ("star_sun", "star", None),
       ("ship", "ship", None)]
)

# Varian responsif: ukuran sel (cols×rows) → piksel half-block persegi (W=cols, H=2*rows).
SIZES = [("sm", 20, 10), ("md", 40, 20), ("lg", 64, 32)]
DEPTHS = ["tc", "256", "16"]   # truecolor / 256 / 16 — fallback color-depth

PORTRAIT_RUNTIME = 512


def celestial_jobs():
    """→ list (base_name, kind, biome, size_tag, cols, rows). Tiap base × 3 ukuran.
    Tiap entri di-render sekali (RGB) lalu di-encode ke 3 depth `.ans`."""
    return [(name, kind, biome, tag, cols, rows)
            for (name, kind, biome) in CELESTIAL_BASES
            for (tag, cols, rows) in SIZES]


def ans_path(base, size_tag, depth):
    # Per CONVENTIONS.md §1: sub-folder per base (≤10 file/folder).
    return f"sprites/{base}/{size_tag}.{depth}.ans"


def portrait_jobs():
    jobs = []
    for grp in ("male", "female", "alien"):
        for i in range(1, 8):
            src = os.path.join(ASSETS, "source", "characters", grp, f"char_{grp}_{i:02d}.png")
            out = f"characters/{grp}/char_{grp}_{i:02d}.png"
            jobs.append((out, src, PORTRAIT_RUNTIME, PORTRAIT_RUNTIME))
    return jobs


def publish_portrait(src, out_rel, size):
    """Salin PNG sumber → runtime PNG (downscale agar repo ramping, kualitas tetap untuk TUI)."""
    from PIL import Image
    im = Image.open(src).convert("RGB")
    w, h = im.size
    scale = min(size / w, size / h, 1.0)
    if scale < 1.0:
        im = im.resize((max(1, int(w * scale)), max(1, int(h * scale))))
    out_path = os.path.join(ASSETS, out_rel)
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    im.save(out_path)


def write_txt(rel, lines):
    path = os.path.join(ASSETS, rel)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
