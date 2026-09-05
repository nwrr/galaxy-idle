"""Encode RGB → ANSI half-block. Depth: tc (24-bit) / 256 / 16 (fallback color-depth)."""

_PAL16 = [  # (fg_code, r, g, b)
    (30, 0, 0, 0), (31, 205, 0, 0), (32, 0, 205, 0), (33, 205, 205, 0),
    (34, 0, 0, 238), (35, 205, 0, 205), (36, 0, 205, 205), (37, 229, 229, 229),
    (90, 127, 127, 127), (91, 255, 85, 85), (92, 85, 255, 85), (93, 255, 255, 85),
    (94, 92, 92, 255), (95, 255, 85, 255), (96, 85, 255, 255), (97, 255, 255, 255),
]


def rgb_to_256(r, g, b):
    if abs(r - g) < 8 and abs(g - b) < 8:  # abu-abu → grayscale ramp 232..255
        gray = round((r + g + b) / 3)
        if gray < 8:
            return 16
        if gray > 248:
            return 231
        return 232 + round((gray - 8) / 247 * 23)
    q = lambda x: round(x / 255 * 5)
    return 16 + 36 * q(r) + 6 * q(g) + q(b)


def rgb_to_16(r, g, b):
    best, bd = _PAL16[0][0], 1e18
    for code, pr, pg, pb in _PAL16:
        d = (r - pr) ** 2 + (g - pg) ** 2 + (b - pb) ** 2
        if d < bd:
            bd, best = d, code
    return best


_LEVELS6 = [0, 51, 102, 153, 204, 255]  # 6 nilai seragam kubus 256-warna (`rgb_to_256`)


def quantized_rgb_256(r, g, b):
    """RGB (0..255 int) yg SEBENARNYA direpresentasikan kode 256-warna dari `rgb_to_256(r,g,b)`
    — formula sama persis (bukan tabel terpisah yg bisa nyimpang), dipakai dithering error-
    diffusion (`imgconv.py` M09.7) utk hitung galat kuantisasi antara warna asli & terkuantisasi."""
    if abs(r - g) < 8 and abs(g - b) < 8:
        gray = round((r + g + b) / 3)
        if gray < 8:
            return (0, 0, 0)
        if gray > 248:
            return (255, 255, 255)
        step = round((gray - 8) / 247 * 23)
        val = round(8 + step / 23 * 247)
        return (val, val, val)
    q = lambda x: round(x / 255 * 5)
    return (_LEVELS6[q(r)], _LEVELS6[q(g)], _LEVELS6[q(b)])


def quantized_rgb_16(r, g, b):
    """RGB (0..255 int) palet 16-warna terdekat (entri `_PAL16` yg dipilih `rgb_to_16`) — dipakai
    dithering (M09.7), sama alasan dgn `quantized_rgb_256`."""
    best, bd = _PAL16[0][1:], 1e18
    for _, pr, pg, pb in _PAL16:
        d = (r - pr) ** 2 + (g - pg) ** 2 + (b - pb) ** 2
        if d < bd:
            bd, best = d, (pr, pg, pb)
    return best


def code_to_rgb_16(code):
    """Kebalikan `rgb_to_16`: kode terminal (30-37/90-97) → RGB `_PAL16` (M10.1, dipakai decoder
    `imgconv.ans_to_rgb` utk scoring fidelitas — re-render `.ans` balik jadi raster)."""
    for c, r, g, b in _PAL16:
        if c == code:
            return (r, g, b)
    return (0, 0, 0)  # kode tak dikenal (harusnya tak terjadi dari output `to_halfblock` sendiri)


def code_to_rgb_256(code):
    """Kebalikan `rgb_to_256`: kode 256-warna (16-255) → RGB (M10.1, sama alasan `code_to_rgb_16`).
    Formula sama persis dgn `rgb_to_256`/`quantized_rgb_256` (bukan tabel terpisah)."""
    if 232 <= code <= 255:
        val = round(8 + (code - 232) / 23 * 247)
        return (val, val, val)
    idx = code - 16
    qr, qg, qb = idx // 36, (idx % 36) // 6, idx % 6
    return (_LEVELS6[qr], _LEVELS6[qg], _LEVELS6[qb])


def _fg(rgb, depth):
    r, g, b = int(rgb[0] * 255), int(rgb[1] * 255), int(rgb[2] * 255)
    if depth == "tc":
        return f"\x1b[38;2;{r};{g};{b}m"
    if depth == "256":
        return f"\x1b[38;5;{rgb_to_256(r, g, b)}m"
    return f"\x1b[{rgb_to_16(r, g, b)}m"


def _bg(rgb, depth):
    r, g, b = int(rgb[0] * 255), int(rgb[1] * 255), int(rgb[2] * 255)
    if depth == "tc":
        return f"\x1b[48;2;{r};{g};{b}m"
    if depth == "256":
        return f"\x1b[48;5;{rgb_to_256(r, g, b)}m"
    return f"\x1b[{rgb_to_16(r, g, b) + 10}m"


def to_halfblock(rgb, mask, depth):
    """rgb: (2*rows, cols, 3) float; mask: (2*rows, cols) bool. → list[str] baris ANSI."""
    H, W, _ = rgb.shape
    rows = H // 2
    out = []
    for ry in range(rows):
        line = []
        for c in range(W):
            ton = mask[2 * ry, c]; bon = mask[2 * ry + 1, c]
            tp = rgb[2 * ry, c]; bp = rgb[2 * ry + 1, c]
            if ton and bon:
                line.append(_fg(tp, depth) + _bg(bp, depth) + "▀" + "\x1b[0m")
            elif ton:
                line.append(_fg(tp, depth) + "▀" + "\x1b[0m")
            elif bon:
                line.append(_fg(bp, depth) + "▄" + "\x1b[0m")
            else:
                line.append(" ")
        out.append("".join(line).rstrip())
    while out and out[0].strip() == "":
        out.pop(0)
    while out and out[-1].strip() == "":
        out.pop()
    return out
