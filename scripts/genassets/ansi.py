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
