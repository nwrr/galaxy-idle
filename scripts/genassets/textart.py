"""Luminance ramp (debug) + render portrait & banner (Pillow). Mono ASCII — bukan runtime."""
import os
import numpy as np

RAMP_LONG = (" .'`^\",:;Il!i~+_-?][}{1)(|/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$")
RAMP_SHORT = " .:-=+*#%@"
RAMP = RAMP_LONG


def lum_to_chars(lum, mask, fill_subject=True):
    """lum: HxW float 0..1; mask: HxW bool (subjek). Return list[str]."""
    n = len(RAMP)
    out = []
    H, W = lum.shape
    for r in range(H):
        row = []
        for c in range(W):
            if not mask[r, c]:
                row.append(" ")
                continue
            v = float(np.clip(lum[r, c], 0.0, 1.0))
            if fill_subject:
                idx = 1 + int(round(v * (n - 2)))  # jangan pernah spasi di subjek
            else:
                idx = int(round(v * (n - 1)))
            row.append(RAMP[min(idx, n - 1)])
        out.append("".join(row).rstrip())
    while out and out[0].strip() == "":
        out.pop(0)
    while out and out[-1].strip() == "":
        out.pop()
    return out


def render_portrait(src, W, H, invert=False):
    from PIL import Image, ImageOps
    im = Image.open(src).convert("L")
    im = ImageOps.autocontrast(im, cutoff=2)
    iw, ih = im.size
    # koreksi aspek sel 2:1: tinggi efektif gambar dibagi 2
    target_w = W
    target_h = max(1, int(round(target_w * (ih / iw) * 0.5)))
    if target_h > H:
        target_h = H
        target_w = max(1, int(round(target_h * (iw / ih) * 2)))
    im = im.resize((target_w, target_h))
    a = np.asarray(im, float) / 255.0
    if invert:
        a = 1 - a
    pad = np.zeros((H, W))
    y0 = (H - target_h) // 2; x0 = (W - target_w) // 2
    pad[y0:y0 + target_h, x0:x0 + target_w] = a
    mask = pad > 0.12  # ambang background → spasi; subjek terang = padat
    return pad, mask


def find_font():
    cands = [
        "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ]
    for p in cands:
        if os.path.exists(p):
            return p
    return None


def render_banner(text, W, H):
    from PIL import Image, ImageDraw, ImageFont
    ph = H * 2  # render 2x tinggi lalu kompres ke sel 2:1
    img = Image.new("L", (W * 2, ph), 0)
    d = ImageDraw.Draw(img)
    fp = find_font()
    size = ph
    while size > 6:
        font = ImageFont.truetype(fp, size) if fp else ImageFont.load_default()
        l, t, r, b = d.textbbox((0, 0), text, font=font)
        if (r - l) <= W * 2 - 2 and (b - t) <= ph:
            break
        size -= 2
        if not fp:
            break
    tw, th = r - l, b - t
    d.text(((W * 2 - tw) / 2 - l, (ph - th) / 2 - t), text, fill=255, font=font)
    img = img.resize((W, H))
    a = np.asarray(img, float) / 255.0
    mask = a > 0.20
    return a, mask
