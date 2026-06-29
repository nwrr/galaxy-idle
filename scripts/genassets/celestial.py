"""Render celestial: luminance (mono, debug) & RGB per-biome (untuk half-block ANSI).

Grid piksel persegi (W=cols, H=2*rows) → tiap sel half-block = 2 piksel vertikal.
Warna per-biome biar biome jelas beda (root masalah ASCII mono).
"""
import numpy as np
from .numfx import C, lerp, sphere, lighting, specular, fractal_noise

# ---------- luminance (mono) ----------

def render_planet(biome, W, H, seed=None):
    if seed is None:
        seed = abs(hash(biome)) % 100000
    X, Y, Z, R2, mask = sphere(H, W)
    base, L = lighting(X, Y, Z)
    n = fractal_noise(H, W, base=5, octaves=5, seed=seed)
    n2 = fractal_noise(H, W, base=11, octaves=3, seed=seed + 7)
    alb = np.full((H, W), 0.5)

    if biome == "terran":
        land = n > 0.52
        alb = np.where(land, 0.55 + 0.45 * (n - 0.52), 0.30 + 0.08 * n2)
        ice = np.abs(Y) > 0.82
        alb = np.where(ice, 0.95, alb)
        lum = base * alb + np.where(~land & mask, specular(X, Y, Z, L, 32, 0.5), 0)
    elif biome == "ocean":
        alb = 0.34 + 0.10 * n2
        lum = base * alb + specular(X, Y, Z, L, 48, 0.8)
    elif biome == "gasgiant":
        turb = 0.25 * (n - 0.5)
        bands = 0.5 + 0.5 * np.sin((Y * 7.0) + turb * 8.0)
        alb = 0.35 + 0.5 * bands
        spot = np.exp(-(((X - 0.25) ** 2) / 0.05 + ((Y - 0.2) ** 2) / 0.02))
        alb = np.clip(alb - 0.5 * spot, 0.1, 1)
        lum = base * alb
    elif biome == "deadworld":
        alb = 0.42 + 0.18 * (n - 0.5)
        rng = np.random.default_rng(seed + 3)
        for _ in range(14):
            cx, cy = rng.uniform(-0.7, 0.7), rng.uniform(-0.7, 0.7)
            cr = rng.uniform(0.06, 0.18)
            d = np.sqrt((X - cx) ** 2 + (Y - cy) ** 2)
            bowl = np.clip(1 - d / cr, 0, 1)
            alb = alb - 0.25 * bowl + 0.20 * np.clip(1 - np.abs(d - cr) / (0.25 * cr), 0, 1)
        lum = base * np.clip(alb, 0.08, 1)
    elif biome == "crystalworld":
        facet = np.abs(n2 - 0.5) * 2.0
        alb = 0.3 + 0.7 * facet
        lum = base * alb + specular(X, Y, Z, L, 60, 0.9) * facet
    elif biome == "iceworld":
        cracks = 1 - np.abs(2 * fractal_noise(H, W, 7, 4, seed + 5) - 1)
        crackline = (cracks > 0.85).astype(float)
        alb = np.clip(0.85 - 0.5 * crackline + 0.08 * n2, 0.2, 1)
        lum = base * alb + specular(X, Y, Z, L, 40, 0.6)
    elif biome == "lavaworld":
        ridg = 1 - np.abs(2 * n - 1)
        veins = np.clip((ridg - 0.65) / 0.35, 0, 1)
        crust = 0.18 + 0.12 * n2
        lum = base * crust + veins
        lum = np.clip(lum, 0, 1)
    elif biome == "ironworld":
        bands = 0.5 + 0.5 * np.sin(Y * 9 + 0.5 * np.sin(X * 6))
        rivets = ((np.sin(X * 22) > 0.9) & (np.sin(Y * 22) > 0.9)).astype(float)
        alb = 0.4 + 0.25 * bands + 0.2 * rivets
        lum = base * alb + specular(X, Y, Z, L, 50, 0.8)
    else:
        lum = base
    lum = np.where(mask, lum, 0.0)
    return lum, mask


def render_asteroidbelt(W, H, seed=42):
    lum = np.zeros((H, W)); mask = np.zeros((H, W), bool)
    rng = np.random.default_rng(seed)
    xs = np.linspace(-1, 1, W); ys = np.linspace(-1, 1, H)
    X, Y = np.meshgrid(xs, ys)
    for _ in range(22):
        cx, cy = rng.uniform(-0.9, 0.9), rng.uniform(-0.9, 0.9)
        rad = rng.uniform(0.06, 0.20)
        d2 = ((X - cx) / rad) ** 2 + ((Y - cy) / rad) ** 2
        m = d2 <= 1
        z = np.sqrt(np.clip(1 - d2, 0, 1))
        L = np.array([-0.5, -0.6, 0.6]); L /= np.linalg.norm(L)
        proj = ((X - cx) / rad) * L[0] + ((Y - cy) / rad) * L[1] + z * L[2]
        shade = 0.2 + 0.8 * np.clip(proj, 0, 1)
        lum = np.where(m & (z > 0), shade, lum)
        mask = mask | (m & (z > 0))
    return lum, mask


def render_star(W, H, seed=99):
    xs = np.linspace(-1, 1, W); ys = np.linspace(-1, 1, H)
    X, Y = np.meshgrid(xs, ys)
    R = np.sqrt(X * X + Y * Y)
    theta = np.arctan2(Y, X)
    gran = 0.15 * (fractal_noise(H, W, 9, 4, seed) - 0.5)
    core = np.clip(1.1 - R * 1.2, 0, 1) + gran
    disk = R <= 0.78
    rays = (0.5 + 0.5 * np.sin(theta * 12)) * np.clip(1 - (R - 0.6) / 0.5, 0, 1)
    corona = (R > 0.6) & (R < 1.05) & (rays > 0.45)
    lum = np.where(disk, np.clip(core, 0.3, 1), 0.0)
    lum = np.where(corona, np.clip(0.25 + 0.4 * rays, 0, 0.7), lum)
    mask = disk | corona
    return lum, mask


def render_ship(W, H, seed=7):
    lum = np.zeros((H, W)); mask = np.zeros((H, W), bool)
    xs = np.linspace(-1, 1, W); ys = np.linspace(0, 1, H)
    X, Y = np.meshgrid(xs, ys)
    flame = fractal_noise(H, W, 8, 3, seed)

    def bw(y):  # half-width body sbg fungsi y(0 atas..1 bawah)
        w = np.where(y < 0.18, (y / 0.18) * 0.34,
            np.where(y < 0.74, 0.34,
            np.where(y < 0.86, 0.34 - (y - 0.74) / 0.12 * 0.12, 0.0)))
        return w
    halfw = bw(Y)
    body = (np.abs(X) <= halfw) & (Y < 0.86)
    cyl = np.sqrt(np.clip(1 - (X / (halfw + 1e-6)) ** 2, 0, 1))
    Ldir = np.array([-0.5, -0.4, 0.75]); Ldir /= np.linalg.norm(Ldir)
    shade = 0.25 + 0.75 * np.clip(X * Ldir[0] + cyl * Ldir[2] - 0.2 * Ldir[1], 0, 1)
    nose_dark = np.clip(Y / 0.18, 0.4, 1)
    lum = np.where(body, shade * (0.7 + 0.3 * nose_dark), lum)
    mask = mask | body
    win = ((X) ** 2 / 0.02 + (Y - 0.34) ** 2 / 0.006) <= 1
    lum = np.where(win, 0.05, lum)
    fin = ((Y > 0.6) & (Y < 0.86) & (np.abs(X) > halfw)
           & (np.abs(X) < halfw + 0.28 * (Y - 0.6) / 0.26))
    lum = np.where(fin, 0.5, lum); mask = mask | fin
    ex = (Y >= 0.86) & (np.abs(X) < 0.22 * (1 - (Y - 0.86) / 0.14))
    lum = np.where(ex, 0.4 + 0.6 * flame, lum); mask = mask | ex
    lum = np.where(mask, np.clip(lum, 0, 1), 0)
    return lum, mask


def render_one(kind, biome, W, H):
    if kind == "planet":
        return render_planet(biome, W, H)
    if kind == "asteroid":
        return render_asteroidbelt(W, H)
    if kind == "star":
        return render_star(W, H)
    if kind == "ship":
        return render_ship(W, H)
    raise ValueError(kind)


# ---------- RGB per-biome (untuk half-block berwarna) ----------

def render_planet_rgb(biome, W, H, seed=None):
    if seed is None:
        seed = abs(hash(biome)) % 100000
    X, Y, Z, R2, mask = sphere(H, W)
    shade, L = lighting(X, Y, Z)          # 0..1 diffuse+ambient
    n = fractal_noise(H, W, base=5, octaves=5, seed=seed)
    n2 = fractal_noise(H, W, base=11, octaves=3, seed=seed + 7)
    rgb = np.zeros((H, W, 3))

    if biome == "terran":
        land = n > 0.52
        e = np.clip((n - 0.52) / 0.46, 0, 1)
        landcol = lerp(C(46, 110, 52), C(120, 100, 70), e)        # hijau→coklat gunung
        seacol = lerp(C(18, 56, 130), C(40, 120, 185), n2)         # laut dalam→dangkal
        col = np.where(land[..., None], landcol, seacol)
        ice = (np.abs(Y) > 0.82)[..., None]
        col = np.where(ice, C(232, 240, 250)[None, None, :], col)
        spec = np.where(~land & mask, specular(X, Y, Z, L, 32, 0.5), 0)
        rgb = col * shade[..., None] + spec[..., None] * C(255, 255, 255)[None, None, :]
    elif biome == "ocean":
        col = lerp(C(16, 52, 120), C(44, 130, 200), 0.5 * n2 + 0.5 * (1 - np.abs(Y)))
        spec = specular(X, Y, Z, L, 48, 0.8)
        rgb = col * shade[..., None] + spec[..., None] * C(200, 240, 255)[None, None, :]
    elif biome == "gasgiant":
        turb = 0.25 * (n - 0.5)
        bands = 0.5 + 0.5 * np.sin((Y * 7.0) + turb * 8.0)
        col = lerp(C(225, 200, 150), C(200, 120, 55), bands)        # krem↔oranye band
        spot = np.exp(-(((X - 0.25) ** 2) / 0.05 + ((Y - 0.2) ** 2) / 0.02))
        col = col * (1 - 0.85 * spot[..., None]) + spot[..., None] * C(175, 50, 38)[None, None, :]
        rgb = col * shade[..., None]
    elif biome == "deadworld":
        alb = np.full((H, W), 0.55)
        rng = np.random.default_rng(seed + 3)
        for _ in range(16):
            cx, cy = rng.uniform(-0.7, 0.7), rng.uniform(-0.7, 0.7)
            cr = rng.uniform(0.06, 0.18)
            d = np.sqrt((X - cx) ** 2 + (Y - cy) ** 2)
            rim = 0.18 * np.clip(1 - np.abs(d - cr) / (0.25 * cr), 0, 1)
            alb = alb - 0.30 * np.clip(1 - d / cr, 0, 1) + rim
        col = C(130, 124, 116)[None, None, :] * np.clip(alb, 0.25, 1.2)[..., None]
        rgb = col * shade[..., None]
    elif biome == "crystalworld":
        facet = np.abs(n2 - 0.5) * 2.0
        col = lerp(C(60, 200, 210), C(150, 80, 205), facet)        # cyan↔ungu faset
        spec = specular(X, Y, Z, L, 60, 0.9) * facet
        rgb = col * shade[..., None] + spec[..., None] * C(235, 245, 255)[None, None, :]
    elif biome == "iceworld":
        cracks = 1 - np.abs(2 * fractal_noise(H, W, 7, 4, seed + 5) - 1)
        crackline = (cracks > 0.85).astype(float)
        col = lerp(C(205, 222, 240), C(120, 160, 205), crackline)   # putih↔retak biru
        spec = specular(X, Y, Z, L, 40, 0.6)
        rgb = col * shade[..., None] + spec[..., None] * C(255, 255, 255)[None, None, :]
    elif biome == "lavaworld":
        ridg = 1 - np.abs(2 * n - 1)
        veins = np.clip((ridg - 0.62) / 0.38, 0, 1)                  # vena lava
        crust = C(28, 22, 20)[None, None, :] * (0.6 + 0.8 * n2)[..., None]
        emis = lerp(C(190, 30, 10), C(255, 225, 90), veins)        # merah→kuning emissive
        rgb = crust * shade[..., None] * (1 - veins[..., None]) + emis * veins[..., None]
    elif biome == "ironworld":
        bands = 0.5 + 0.5 * np.sin(Y * 9 + 0.5 * np.sin(X * 6))
        rust = (fractal_noise(H, W, 6, 3, seed + 9) > 0.62).astype(float)
        col = lerp(C(140, 142, 150), C(168, 92, 42), 0.4 * bands + 0.6 * rust)  # metalik↔rust
        spec = specular(X, Y, Z, L, 50, 0.8)
        rgb = col * shade[..., None] + spec[..., None] * C(220, 220, 230)[None, None, :]
    else:
        rgb = C(150, 150, 150)[None, None, :] * shade[..., None]
    rgb = np.where(mask[..., None], np.clip(rgb, 0, 1), 0.0)
    return rgb, mask


def render_asteroidbelt_rgb(W, H, seed=42):
    lum, mask = render_asteroidbelt(W, H, seed)
    tint = C(122, 116, 108)[None, None, :]
    var = 0.85 + 0.3 * fractal_noise(H, W, 8, 3, seed + 2)[..., None]   # variasi batu
    rgb = tint * var * lum[..., None]
    rgb = np.where(mask[..., None], np.clip(rgb, 0, 1), 0.0)
    return rgb, mask


def render_star_rgb(W, H, seed=99):
    lum, mask = render_star(W, H, seed)
    xs = np.linspace(-1, 1, W); ys = np.linspace(-1, 1, H)
    X, Y = np.meshgrid(xs, ys)
    R = np.sqrt(X * X + Y * Y)
    col = lerp(C(255, 250, 215), C(225, 70, 25), np.clip(R / 0.95, 0, 1))  # inti putih→korona merah
    rgb = col * np.clip(lum * 1.4, 0, 1)[..., None]
    rgb = np.where(mask[..., None], np.clip(rgb, 0, 1), 0.0)
    return rgb, mask


def render_ship_rgb(W, H, seed=7):
    lum, mask = render_ship(W, H, seed)
    xs = np.linspace(-1, 1, W); ys = np.linspace(0, 1, H)
    X, Y = np.meshgrid(xs, ys)
    flame = fractal_noise(H, W, 8, 3, seed)
    win = ((X) ** 2 / 0.02 + (Y - 0.34) ** 2 / 0.006) <= 1
    ex = (Y >= 0.86) & (np.abs(X) < 0.22 * (1 - (Y - 0.86) / 0.14))
    rgb = C(150, 155, 165)[None, None, :] * lum[..., None]                  # lambung metalik
    rgb = np.where(win[..., None], C(90, 200, 240)[None, None, :], rgb)     # jendela cyan
    excol = lerp(C(255, 140, 30), C(255, 235, 120), flame)                 # exhaust oranye→kuning
    rgb = np.where(ex[..., None], excol, rgb)
    rgb = np.where(mask[..., None], np.clip(rgb, 0, 1), 0.0)
    return rgb, mask


def render_celestial_rgb(kind, biome, W, H):
    if kind == "planet":
        return render_planet_rgb(biome, W, H)
    if kind == "asteroid":
        return render_asteroidbelt_rgb(W, H)
    if kind == "star":
        return render_star_rgb(W, H)
    if kind == "ship":
        return render_ship_rgb(W, H)
    raise ValueError(kind)
