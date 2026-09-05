"""PNG arbitrer → ANSI half-block (konverter aset ComfyUI, M09).

Beda dari `celestial.py` (render prosedural noise-based): modul ini konversi GAMBAR SUMBER nyata
(hasil ComfyUI, `assets/source/*/*.png` — M08) ke ANSI. Dipakai M10/M11 utk menggantikan render
prosedural planet/star/ship/galaxy/banner dgn aset ComfyUI sungguhan. Reuse encoder
`ansi.to_halfblock` (sama persis dipakai `celestial.py`/`build.py` — satu jalur encode, dua
sumber RGB berbeda).
"""
import re

import numpy as np
from numpy.lib.stride_tricks import sliding_window_view
from PIL import Image

from .ansi import (
    code_to_rgb_16,
    code_to_rgb_256,
    quantized_rgb_16,
    quantized_rgb_256,
    to_halfblock,
)


def _fit_resize(src, w, h):
    """Resize `src` (PIL RGBA) ke kanvas persis `w`×`h` **tanpa distorsi**: kunci rasio aspek
    sumber (skala seragam = `min(w/sw, h/sh)`, agar muat dlm box), tempel di tengah kanvas
    transparan `w`×`h`. Sisa area (letterbox/pillarbox saat rasio sumber ≠ rasio target) tetap
    alpha=0 → otomatis jadi sel kosong lewat `mask` di `png_to_ansi` (bukan warna dipaksa/di-
    stretch spt resize langsung M09.1/M09.2 dulu)."""
    sw, sh = src.size
    scale = min(w / sw, h / sh)
    nw, nh = max(1, round(sw * scale)), max(1, round(sh * scale))
    downscaling = nw <= sw and nh <= sh
    resample = Image.Resampling.BOX if downscaling else Image.Resampling.LANCZOS
    resized = src.resize((nw, nh), resample)
    canvas = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    canvas.paste(resized, ((w - nw) // 2, (h - nh) // 2))
    return canvas


def _dither_fs(rgb255, mask, depth):
    """Floyd–Steinberg error-diffusion (in-place di kopi `rgb255`, piksel 0..255) — cuma
    dipanggil utk `depth` 256/16 (M09.7, **opsional** — dipaksa off utk tc, kuantisasi tc
    presisi penuh, tak butuh dither). Galat kuantisasi tiap piksel (`asli - terkuantisasi`)
    disebar ke tetangga blm-diproses (kanan/kiri-bawah/bawah/kanan-bawah, bobot standar
    7/16·3/16·5/16·1/16) — hasilnya gradient halus meski depth rendah, drpd tiap piksel
    dikuantisasi independen (banding/pita warna kasar). Galat **cuma** menyebar antar piksel
    `mask=True` (biar warna tak 'bocor' ke area transparan/latar kosong hasil `_fit_resize`)."""
    quantize = quantized_rgb_256 if depth == "256" else quantized_rgb_16
    H, W, _ = rgb255.shape
    buf = rgb255.copy()
    for y in range(H):
        for x in range(W):
            if not mask[y, x]:
                continue
            old = buf[y, x]
            qr, qg, qb = quantize(*(int(np.clip(v, 0, 255)) for v in old))
            new = np.array([qr, qg, qb], dtype=np.float64)
            err = old - new
            buf[y, x] = new
            for dx, dy, wgt in ((1, 0, 7 / 16), (-1, 1, 3 / 16), (0, 1, 5 / 16), (1, 1, 1 / 16)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < W and 0 <= ny < H and mask[ny, nx]:
                    buf[ny, nx] += err * wgt
    return buf


NEAR_BLACK = 10  # ambang brightness (kanal maks, 0-255) dianggap "latar hitam" -> sel kosong


def _compute_mask(rgb255, alpha):
    """Mask biner (True = tampil, False = sel kosong/transparan). Gabung 2 sumber transparansi
    (M09.8): (1) alpha channel asli — relevan utk padding `_fit_resize` (selalu alpha=0) & PNG
    yg memang punya transparansi nyata; (2) threshold near-black — **wajib**, krn SEMUA 34 aset
    ComfyUI M08 di-render OPAQUE (alpha=255 seragam, tak ada transparansi nyata sama sekali)
    dgn prompt eksplisit "black space background" (M06) — tanpa ini, background gelap tsb
    dirender sbg ribuan sel hitam SOLID, bukan kosong (dikonfirmasi nyata: `ship.png` 80% piksel
    ≤brightness 10, semua di area background/celah, jelas terpisah dari konten kapal yg langsung
    lompat ke brightness 100+ — lihat log iterasi utk data lengkap)."""
    alpha_ok = alpha > 127.5
    brightness = rgb255.max(axis=-1)
    not_near_black = brightness > NEAR_BLACK
    return alpha_ok & not_near_black


def png_to_ansi(img, cols, rows, depth, dither=False):
    """`img`: PIL Image (RGBA/RGB) sumber. `cols`×`rows` = ukuran sel target (piksel target =
    `cols`×`2*rows`, tiap sel half-block = 2 piksel vertikal). `depth` ∈ {"tc","256","16"}.
    `dither`: aktifkan Floyd–Steinberg (M09.7) — **opsional**, cuma berlaku depth 256/16 (tc
    diabaikan, tak perlu). Return `list[str]` baris ANSI (kompatibel `catalog.write_txt`).

    **M09.2**: downscale via `BOX` (area-average) saat mengecilkan, `LANCZOS` saat memperbesar
    (lihat `_fit_resize`). **M09.3**: rasio aspek sumber dikunci (`_fit_resize`) — sumber non-
    persegi TAK di-stretch paksa memenuhi box; muat proporsional + padding transparan di sisi
    pendek. **M09.8**: mask gabung alpha + near-black (`_compute_mask`) — lihat alasan di sana."""
    w, h = cols, rows * 2
    src = img.convert("RGBA")
    im = _fit_resize(src, w, h)
    arr = np.asarray(im, dtype=np.float64)
    rgb255 = arr[..., :3]
    alpha = arr[..., 3]
    mask = _compute_mask(rgb255, alpha)
    if dither and depth != "tc":
        rgb255 = _dither_fs(rgb255, mask, depth)
    rgb = rgb255 / 255.0
    return to_halfblock(rgb, mask, depth)


_TOKEN_RE = re.compile(r"\x1b\[([0-9;]*)m|(.)", re.DOTALL)


def _parse_sgr(nums):
    """1 kelompok parameter SGR (mis. `[38,2,10,20,30]`) → `("fg"|"bg", (r,g,b))`, `("reset",
    None)`, atau `None` (kode tak dikenal/diabaikan). Cermin persis format yg ditulis `_fg`/`_bg`
    (`ansi.py`) — `38;2;R;G;B`=fg tc, `48;2;...`=bg tc, `38;5;N`=fg 256, `48;5;N`=bg 256,
    30-37/90-97=fg 16, 40-47/100-107=bg 16 (kode fg+10, sesuai `_bg` utk depth 16)."""
    if not nums:
        return ("reset", None)
    if nums[0] == 0:
        return ("reset", None)
    if nums[0] in (38, 48) and len(nums) >= 5 and nums[1] == 2:
        kind = "fg" if nums[0] == 38 else "bg"
        return (kind, (nums[2], nums[3], nums[4]))
    if nums[0] in (38, 48) and len(nums) >= 3 and nums[1] == 5:
        kind = "fg" if nums[0] == 38 else "bg"
        return (kind, code_to_rgb_256(nums[2]))
    if 30 <= nums[0] <= 37 or 90 <= nums[0] <= 97:
        return ("fg", code_to_rgb_16(nums[0]))
    if 40 <= nums[0] <= 47 or 100 <= nums[0] <= 107:
        return ("bg", code_to_rgb_16(nums[0] - 10))
    return None


def ans_to_rgb(lines):
    """Kebalikan `ansi.to_halfblock()` (M10.1) — parse baris ANSI (`list[str]`, format persis
    keluaran `to_halfblock`/`png_to_ansi`) balik jadi raster `(rgb, mask)`: `rgb` shape
    `(2*rows, cols, 3)` float 0..1, `mask` shape `(2*rows, cols)` bool (piksel yg benar2 di-set
    warna vs sel kosong/spasi). Dipakai scoring fidelitas (M10.2): re-render `.ans` → raster →
    bandingkan (mis. SSIM) vs sumber PNG asli.

    Tiap sel dikodekan `to_halfblock` sbg 0-2 SGR (fg/bg) diikuti 1 glyph (`▀`/`▄`/spasi) lalu
    reset (`\\x1b[0m`) — decoder ini pakai aturan sama presis: `▀` + fg+bg → top=fg, bottom=bg
    (keduanya `mask=True`); `▀` + fg saja → top=fg, bottom `mask=False`; `▄` + fg → bottom=fg
    (glyph `▄` = tinta warna FOREGROUND, lihat `to_halfblock` `elif bon` — bukan bug, memang
    begitu desainnya), top `mask=False`; spasi (tanpa SGR) → keduanya `mask=False`."""
    rows = len(lines)
    decoded_rows = []
    max_cols = 0
    for line in lines:
        pending = []
        cells = []
        for m in _TOKEN_RE.finditer(line):
            esc_params, ch = m.group(1), m.group(2)
            if ch is None:  # kelompok SGR
                nums = [int(x) for x in esc_params.split(";") if x] if esc_params else []
                parsed = _parse_sgr(nums)
                if parsed is None:
                    continue
                if parsed[0] == "reset":
                    pending = []
                else:
                    pending.append(parsed)
                continue
            if ch == "▀":
                fg = next((c for k, c in pending if k == "fg"), None)
                bg = next((c for k, c in pending if k == "bg"), None)
                cells.append((fg, bg))
            elif ch == "▄":
                fg = next((c for k, c in pending if k == "fg"), None)
                cells.append((None, fg))
            else:  # spasi (atau char tak dikenal — harusnya tak terjadi dari output kita)
                cells.append((None, None))
            pending = []
        decoded_rows.append(cells)
        max_cols = max(max_cols, len(cells))

    H, W = rows * 2, max_cols
    rgb = np.zeros((H, W, 3), dtype=np.float64)
    mask = np.zeros((H, W), dtype=bool)
    for ry, cells in enumerate(decoded_rows):
        for cx, (top, bot) in enumerate(cells):
            if top is not None:
                rgb[2 * ry, cx] = np.array(top, dtype=np.float64) / 255.0
                mask[2 * ry, cx] = True
            if bot is not None:
                rgb[2 * ry + 1, cx] = np.array(bot, dtype=np.float64) / 255.0
                mask[2 * ry + 1, cx] = True
    return rgb, mask


def _ssim_channel(a, b):
    """SSIM windowed 1 kanal (2D float array 0..255, ukuran sama). Formula standar Wang et al.
    2004 — SAMA PERSIS dgn `comfyui/verify.py` (M07.2), duplikasi kecil disengaja: 2 modul beda
    scope (`comfyui/`=gerbang generate aset, `genassets/`=fidelitas konversi PNG→ANSI), lintas-
    import antar keduanya bikin coupling aneh drpd 15 baris fungsi matematika stabil."""
    win = min(8, a.shape[0], a.shape[1])
    if win < 2:
        return 1.0 if np.allclose(a, b, atol=1.0) else 0.0
    c1 = (0.01 * 255) ** 2
    c2 = (0.03 * 255) ** 2
    aw = sliding_window_view(a, (win, win))
    bw = sliding_window_view(b, (win, win))
    mu_a = aw.mean(axis=(-1, -2))
    mu_b = bw.mean(axis=(-1, -2))
    var_a = aw.var(axis=(-1, -2))
    var_b = bw.var(axis=(-1, -2))
    cov = (aw * bw).mean(axis=(-1, -2)) - mu_a * mu_b
    num = (2 * mu_a * mu_b + c1) * (2 * cov + c2)
    den = (mu_a**2 + mu_b**2 + c1) * (var_a + var_b + c2)
    return float((num / den).mean())


def fidelity(src_img, ans_lines, cols, rows):
    """Skor fidelitas 0-100 (M10.2) = SSIM antara `.ans` (`ans_lines`) di-re-render (`ans_to_rgb`)
    vs `src_img` di-downscale ke box target yg SAMA (`_fit_resize`, cara identik `png_to_ansi`
    memprosesnya) — mengukur seberapa dekat rekonstruksi ANSI ke sumber aslinya.

    `to_halfblock` trim baris/kolom kosong di tepi, jadi `ans_to_rgb` bisa balikin raster LEBIH
    KECIL dari box `cols`×`2*rows` asli — sumber di-crop ke jendela yg SAMA (offset dihitung dari
    pasangan baris piksel pertama berisi konten, cermin logika trim `to_halfblock`) sebelum
    dibandingkan, biar align piksel-demi-piksel benar (bukan asumsi kebetulan sama besar)."""
    w, h = cols, rows * 2
    src_fit = _fit_resize(src_img.convert("RGBA"), w, h)
    src_arr = np.asarray(src_fit, dtype=np.float64)
    src_rgb255 = src_arr[..., :3]

    decoded_rgb, _ = ans_to_rgb(ans_lines)
    Hd, Wd, _ = decoded_rgb.shape
    if Hd == 0 or Wd == 0:
        return 0.0

    full_mask = _compute_mask(src_rgb255, src_arr[..., 3])
    pairs = h // 2
    pair_has_content = full_mask.reshape(pairs, 2, w).any(axis=(1, 2))
    first_pair = int(np.argmax(pair_has_content)) if pair_has_content.any() else 0
    r0 = min(first_pair * 2, max(0, h - Hd))

    # M10.4: nolkan latar (near-black, di-mask M09.8) di SISI SUMBER jg sblm SSIM — konsisten
    # dgn sisi decoded yg SELALU nol persis di situ (sel kosong = tak ada SGR sama sekali).
    # Tanpa ini, sumber tetap simpan nilai near-black asli (0-10) sedangkan decoded 0 tepat →
    # selisih kecil tp konsisten yg dihukum SSIM lokal (makin parah di gambar besar, lebih banyak
    # jendela mencakup batas konten/latar) — PADAHAL bukan cacat konversi, cuma konsekuensi
    # SENGAJA masking M09.8 (fitur, bukan bug). Divalidasi nyata M10.3: `lg tc ship.png`
    # 88.62→100.0 persis dgn fix ini.
    region_mask = full_mask[r0:r0 + Hd, :Wd]
    src_region = np.where(region_mask[..., None], src_rgb255[r0:r0 + Hd, :Wd], 0.0)
    decoded_region255 = decoded_rgb * 255.0

    ssim_vals = [_ssim_channel(src_region[..., c], decoded_region255[..., c]) for c in range(3)]
    score = float(np.mean(ssim_vals))
    return max(0.0, min(100.0, score * 100.0))
