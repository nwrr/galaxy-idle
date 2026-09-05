"""Unit test `imgconv.py` (M09.10). `unittest` stdlib (bukan `pytest` — tak terpasang di
sandbox ini, stdlib jamin jalan di mana saja tanpa dependency baru).

Usage: python3 scripts/genassets/test_imgconv.py
"""
import os
import re
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import numpy as np
from PIL import Image

from genassets.imgconv import _compute_mask, _fit_resize, ans_to_rgb, fidelity, png_to_ansi
from genassets.ansi import quantized_rgb_16, quantized_rgb_256, rgb_to_16, rgb_to_256

_ESC = "\x1b"
_RE_TC = re.compile(rf"{_ESC}\[3[48];2;\d+;\d+;\d+m")
_RE_256 = re.compile(rf"{_ESC}\[3[48];5;\d+m")
_RE_16 = re.compile(rf"{_ESC}\[\d+m")


class PngToAnsiTests(unittest.TestCase):
    def test_dimensions_small_solid_image(self):
        img = Image.new("RGBA", (32, 32), (200, 50, 30, 255))
        lines = png_to_ansi(img, 10, 5, "tc")
        self.assertLessEqual(len(lines), 5)  # <= rows (baris kosong di-trim to_halfblock)
        self.assertGreater(len(lines), 0)
        for line in lines:
            self.assertLessEqual(len(re.findall("▀|▄", line)), 10)  # <= cols sel per baris

    def test_escape_format_matches_depth(self):
        img = Image.new("RGBA", (16, 16), (200, 50, 30, 255))
        for depth, pattern in (("tc", _RE_TC), ("256", _RE_256), ("16", _RE_16)):
            lines = png_to_ansi(img, 8, 4, depth)
            joined = "\n".join(lines)
            self.assertTrue(pattern.search(joined), f"depth={depth} tak cocok pola escape")
        # tc TIDAK boleh cocok pola 256 (beda format persis, bukan superset kebetulan)
        tc_lines = "\n".join(png_to_ansi(img, 8, 4, "tc"))
        self.assertFalse(_RE_256.search(tc_lines))

    def test_aspect_lock_no_distortion(self):
        # 200x50 (rasio 4:1) ke box persegi 40x40 -> tinggi konten harus proporsional (~10px),
        # bukan diregangkan penuh 40px.
        wide = Image.new("RGBA", (200, 50), (255, 0, 0, 255))
        canvas = _fit_resize(wide, 40, 40)
        alpha = np.asarray(canvas)[..., 3]
        rows_with_content = int((alpha > 0).any(axis=1).sum())
        self.assertEqual(rows_with_content, 10)  # persis 40*50/200

    def test_mask_excludes_near_black_background(self):
        # separuh kiri near-black (latar), separuh kanan terang (konten) — opaque penuh spt
        # aset ComfyUI nyata (bukan transparansi alpha).
        arr = np.zeros((4, 4, 4), dtype=np.uint8)
        arr[:, :2] = (2, 2, 2, 255)      # near-black, alpha opaque
        arr[:, 2:] = (220, 220, 220, 255)  # terang, alpha opaque
        rgb255 = arr[..., :3].astype(np.float64)
        alpha = arr[..., 3].astype(np.float64)
        mask = _compute_mask(rgb255, alpha)
        self.assertFalse(mask[:, :2].any())  # near-black -> semua kosong walau alpha=255
        self.assertTrue(mask[:, 2:].all())   # terang -> semua tampil

    def test_dither_changes_output_for_256_and_16(self):
        # gradient (bukan solid) -- solid color tak menghasilkan galat kuantisasi utk disebar.
        w = 32
        arr = np.zeros((w, w, 4), dtype=np.uint8)
        for x in range(w):
            arr[:, x, 0] = int(x / (w - 1) * 255)
            arr[:, x, 1] = 40
            arr[:, x, 2] = 40
            arr[:, x, 3] = 255
        img = Image.fromarray(arr, "RGBA")
        for depth in ("256", "16"):
            plain = png_to_ansi(img, 24, 12, depth, dither=False)
            dithered = png_to_ansi(img, 24, 12, depth, dither=True)
            self.assertNotEqual(plain, dithered, f"depth={depth}: dither harus ubah output")
        # tc: dither diabaikan (tak ada kuantisasi presisi-penuh utk didither)
        tc_plain = png_to_ansi(img, 24, 12, "tc", dither=False)
        tc_dithered = png_to_ansi(img, 24, 12, "tc", dither=True)
        self.assertEqual(tc_plain, tc_dithered)

    def test_quantized_rgb_roundtrips_to_same_code(self):
        for r, g, b in ((200, 50, 30), (10, 200, 220), (128, 128, 128), (0, 0, 0), (255, 255, 255)):
            qr, qg, qb = quantized_rgb_256(r, g, b)
            self.assertEqual(rgb_to_256(r, g, b), rgb_to_256(qr, qg, qb))
            qr, qg, qb = quantized_rgb_16(r, g, b)
            self.assertEqual(rgb_to_16(r, g, b), rgb_to_16(qr, qg, qb))

    def test_ans_to_rgb_roundtrips_exactly_for_tc(self):
        # M10.1: encode lalu decode gambar solid (non-persegi, biar aspect-lock jg terpakai)
        # harus balik ke warna PERSIS sama (tc = lossless, cuma pembulatan int di awal).
        img = Image.new("RGBA", (60, 30), (12, 200, 90, 255))
        lines = png_to_ansi(img, 16, 8, "tc")
        decoded_rgb, decoded_mask = ans_to_rgb(lines)
        self.assertTrue(decoded_mask.any())
        visible = decoded_rgb[decoded_mask] * 255.0
        expected = np.array([12, 200, 90], dtype=np.float64)
        self.assertTrue(np.allclose(visible, expected, atol=1.0))

    def test_fidelity_high_for_solid_image_tc(self):
        # gambar solid (tanpa detail) di depth tc harus skor SANGAT tinggi (dekat 100) -- kasus
        # paling mudah utk konverter, gerbang sanity sblm tuning M10.4 thd gambar kompleks.
        img = Image.new("RGBA", (64, 64), (180, 60, 200, 255))
        ans = png_to_ansi(img, 20, 10, "tc")
        score = fidelity(img, ans, 20, 10)
        self.assertGreaterEqual(score, 99.0)

    def test_fidelity_lower_for_worse_depth(self):
        # depth 16 (kuantisasi kasar) pd gradient harus skor LEBIH RENDAH drpd tc (bukti fidelity()
        # benar2 sensitif ke kualitas rekonstruksi, bukan skor tetap/selalu tinggi).
        w = 48
        arr = np.zeros((w, w, 4), dtype=np.uint8)
        for x in range(w):
            arr[:, x] = (int(x / (w - 1) * 255), 40, 200, 255)
        img = Image.fromarray(arr, "RGBA")
        score_tc = fidelity(img, png_to_ansi(img, 24, 12, "tc"), 24, 12)
        score_16 = fidelity(img, png_to_ansi(img, 24, 12, "16"), 24, 12)
        self.assertGreater(score_tc, score_16)

    def test_fidelity_tc_hits_target_even_with_background(self):
        # M10.4: gambar dgn latar near-black LUAS (spt semua aset ComfyUI M08, "black space
        # background") + subjek kecil di tengah -- kasus REALISTIS yg dulu bikin skor tc jatuh
        # jauh di bawah 99 (lg=88.62) sblm fix "nolkan latar sisi sumber sblm SSIM" M10.4.
        # Regresi ini KUNCI angka target, bukan cuma cek "lebih tinggi dari sebelumnya".
        arr = np.full((64, 64, 4), (2, 2, 2, 255), dtype=np.uint8)  # latar near-black opaque
        arr[24:40, 24:40] = (200, 60, 220, 255)  # subjek terang di tengah
        img = Image.fromarray(arr, "RGBA")
        for cols, rows in ((20, 10), (40, 20), (64, 32)):
            ans = png_to_ansi(img, cols, rows, "tc")
            score = fidelity(img, ans, cols, rows)
            self.assertGreaterEqual(score, 99.0, f"cols={cols} rows={rows}: skor={score}")


if __name__ == "__main__":
    unittest.main()
