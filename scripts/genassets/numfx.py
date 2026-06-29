"""Util numerik: resize bilinear, fractal noise, geometri bola, lighting/specular."""
import numpy as np


def resize_bilinear(a, H, W):
    h, w = a.shape
    yi = np.linspace(0, h - 1, H)
    xi = np.linspace(0, w - 1, W)
    y0 = np.floor(yi).astype(int); x0 = np.floor(xi).astype(int)
    y1 = np.clip(y0 + 1, 0, h - 1); x1 = np.clip(x0 + 1, 0, w - 1)
    wy = (yi - y0)[:, None]; wx = (xi - x0)[None, :]
    a00 = a[np.ix_(y0, x0)]; a01 = a[np.ix_(y0, x1)]
    a10 = a[np.ix_(y1, x0)]; a11 = a[np.ix_(y1, x1)]
    top = a00 * (1 - wx) + a01 * wx
    bot = a10 * (1 - wx) + a11 * wx
    return top * (1 - wy) + bot * wy


def fractal_noise(H, W, base=4, octaves=4, seed=0):
    rng = np.random.default_rng(seed)
    total = np.zeros((H, W)); amp = 1.0; freq = base; norm = 0.0
    for _ in range(octaves):
        g = rng.random((freq + 2, freq + 2))
        total += amp * resize_bilinear(g, H, W)
        norm += amp; amp *= 0.5; freq *= 2
    total /= norm
    return (total - total.min()) / (np.ptp(total) + 1e-9)


def sphere(H, W):
    xs = np.linspace(-1, 1, W); ys = np.linspace(-1, 1, H)
    X, Y = np.meshgrid(xs, ys)
    R2 = X * X + Y * Y
    mask = R2 <= 1.0
    Z = np.sqrt(np.clip(1 - R2, 0, 1))
    return X, Y, Z, R2, mask


def lighting(X, Y, Z, L=(-0.5, -0.6, 0.62), ambient=0.18):
    L = np.array(L, float); L /= np.linalg.norm(L)
    diff = np.clip(X * L[0] + Y * L[1] + Z * L[2], 0, 1)
    return ambient + (1 - ambient) * diff, L


def specular(X, Y, Z, L, power=24, ks=0.7):
    V = np.array([0, 0, 1.0])
    H = L + V; H /= np.linalg.norm(H)
    s = np.clip(X * H[0] + Y * H[1] + Z * H[2], 0, 1) ** power
    return ks * s


def C(*v):
    """RGB 0..255 → array float 0..1."""
    return np.array(v, float) / 255.0


def lerp(a, b, t):
    t = np.clip(t, 0, 1)[..., None]
    return a[None, None, :] * (1 - t) + b[None, None, :] * t
