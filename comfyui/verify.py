#!/usr/bin/env python3
"""Gerbang kemiripan hasil ComfyUI ke referensi (M07). Skor SSIM+phash (M07.2/M07.3) atau jarak
warna dominan (`ref` bentuk `"colors:..."`) vs referensi; skor gabungan <90 → regen `seed++`
(M07.7, `--regen`). Agent tetap Read PNG hasil utk konfirmasi visual final (bukan pengganti mata).

M07.1: load tiap item dari prompt-set + resolusi PNG hasil (`generate.py`) & `ref`-nya (3 bentuk:
null / path gambar / `colors:...`). M07.2: utk `ref` bentuk path, **SSIM** grayscale+warna via
Pillow+numpy murni. M07.3 (skor ini): **perceptual hash (pHash)** — tahan resize/kompresi/geser
ringan, melengkapi SSIM yang terlalu sensitif ke posisi piksel persis (lihat catatan M07.2 di
`ITERATION_LOG.md`: 2 gambar galaksi "mirip" tapi SSIM cuma ~0.03). DCT diimplementasi manual via
matriks numpy (tanpa `scipy.fft`) — konsisten dependency ringan M07.2. Skor gabungan (M07.4) &
jarak warna dominan utk `ref` bentuk `colors:` menyusul.

Usage:
  python3 comfyui/verify.py --set celestial
  python3 comfyui/verify.py --set characters --gate 85
"""
import argparse
import datetime
import json
import os
import shutil
import subprocess
import sys

import numpy as np
from numpy.lib.stride_tricks import sliding_window_view
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
REF_ROOT = os.path.join(ROOT, "assets", "source", "references")

sys.path.insert(0, HERE)
from generate import load_prompts, item_output_path  # noqa: E402  (reuse, hindari duplikasi)

SSIM_SIZE = (256, 256)  # ukuran normalisasi sebelum bandingkan (beda resolusi asli tak masalah)
SSIM_WIN = 8            # jendela lokal SSIM (piksel)


def _ssim_channel(a, b, win=SSIM_WIN):
    """SSIM windowed 1 kanal (2D float array, ukuran sama). Formula standar Wang et al. 2004,
    jendela lokal via `sliding_window_view` (murni numpy, tanpa scipy)."""
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


def ssim_scores(png_path, ref_path):
    """`(gray_ssim, color_ssim)` 0..1 antara `png_path` & `ref_path`. Gambar dinormalisasi ke
    `SSIM_SIZE` dulu (resolusi asli beda-beda, tak masalah utk skor structural)."""
    a = Image.open(png_path).convert("RGB").resize(SSIM_SIZE, Image.LANCZOS)
    b = Image.open(ref_path).convert("RGB").resize(SSIM_SIZE, Image.LANCZOS)
    a_arr = np.asarray(a, dtype=np.float64)
    b_arr = np.asarray(b, dtype=np.float64)
    color_ssim = float(np.mean([_ssim_channel(a_arr[..., c], b_arr[..., c]) for c in range(3)]))
    gray_a = np.asarray(a.convert("L"), dtype=np.float64)
    gray_b = np.asarray(b.convert("L"), dtype=np.float64)
    gray_ssim = _ssim_channel(gray_a, gray_b)
    return gray_ssim, color_ssim


PHASH_IMG_SIZE = 32  # gambar diresize ke NxN sebelum DCT (ukuran klasik algoritma pHash)
PHASH_HASH_SIZE = 8  # ambil blok frekuensi rendah NxN dari hasil DCT -> hash 64 bit


def _dct_matrix(n):
    """Matriks basis DCT-II ortonormal `n×n` (dipakai `basis @ a @ basis.T` utk DCT 2D
    separable) — implementasi manual numpy, tanpa `scipy.fft`."""
    x = np.arange(n)
    k = x.reshape(-1, 1)
    m = np.cos(np.pi / n * (x + 0.5) * k)
    m[0, :] *= 1.0 / np.sqrt(n)
    m[1:, :] *= np.sqrt(2.0 / n)
    return m


def phash_bits(path, img_size=PHASH_IMG_SIZE, hash_size=PHASH_HASH_SIZE):
    """Perceptual hash `path` → array boolean panjang `hash_size**2` (64 default). Algoritma
    klasik (Zauner): grayscale → resize kecil → DCT 2D → ambil blok frekuensi rendah kiri-atas
    → bit = koefisien > median blok (DC/indeks 0 dikecualikan dari median, dominan skala terang)."""
    img = Image.open(path).convert("L").resize((img_size, img_size), Image.LANCZOS)
    arr = np.asarray(img, dtype=np.float64)
    basis = _dct_matrix(img_size)
    dct = (basis @ arr @ basis.T)[:hash_size, :hash_size]
    flat = dct.flatten()
    med = np.median(flat[1:])  # exclude DC (indeks 0) — dominan level terang keseluruhan
    bits = flat > med
    bits[0] = False
    return bits


def phash_similarity(png_path, ref_path):
    """Kemiripan pHash 0..1 = `1 - hamming_distance/64` (Hamming dinormalisasi)."""
    a = phash_bits(png_path)
    b = phash_bits(ref_path)
    hamming = int(np.count_nonzero(a != b))
    return 1.0 - hamming / a.size


_CLIP_MODEL = None
_CLIP_PREPROCESS = None


def clip_available():
    """CLIP **opsional**: butuh `torch`+`open_clip` (berat, model diunduh) — SSIM+phash sudah
    cukup sbg gerbang utama (M07.2-M07.4), CLIP cuma sinyal semantik tambahan bila lib ada."""
    try:
        import torch  # noqa: F401
        import open_clip  # noqa: F401
        return True
    except ImportError:
        return False


def _load_clip():
    global _CLIP_MODEL, _CLIP_PREPROCESS
    if _CLIP_MODEL is None:
        import open_clip
        model, _, preprocess = open_clip.create_model_and_transforms(
            "ViT-B-32", pretrained="openai"
        )
        model.eval()
        _CLIP_MODEL, _CLIP_PREPROCESS = model, preprocess
    return _CLIP_MODEL, _CLIP_PREPROCESS


def clip_cosine_similarity(png_path, ref_path):
    """Cosine similarity embedding CLIP, dinormalisasi 0..1. `None` bila lib/model tak tersedia
    — caller **wajib** skip rapi (jangan gagal keras), sesuai M07.5."""
    if not clip_available():
        return None
    import torch
    model, preprocess = _load_clip()
    a = preprocess(Image.open(png_path).convert("RGB")).unsqueeze(0)
    b = preprocess(Image.open(ref_path).convert("RGB")).unsqueeze(0)
    with torch.no_grad():
        fa = model.encode_image(a)
        fb = model.encode_image(b)
        fa = fa / fa.norm(dim=-1, keepdim=True)
        fb = fb / fb.norm(dim=-1, keepdim=True)
        cos = (fa @ fb.T).item()
    return (cos + 1.0) / 2.0  # cosine [-1,1] -> [0,1]


# Bobot skor gabungan. pHash lebih berat (0.65) drpd SSIM (0.35) krn hasil generatif ComfyUI tak
# akan pernah pixel-aligned ke referensi (beda komposisi persis) — SSIM murni cenderung sangat
# rendah bahkan utk gambar yang secara konsep mirip (dibuktikan M07.2/M07.3: pasangan galaksi
# "mirip" cuma SSIM~0.03 tapi pHash~0.69). pHash tahan pergeseran/variasi ringan, jadi sinyal
# lebih relevan utk kasus ini. Bobot ini bisa disetel ulang berdasar data nyata begitu ComfyUI
# tersedia & ada contoh hasil generate sungguhan utk dibandingkan (M08).
SSIM_WEIGHT = 0.35
PHASH_WEIGHT = 0.65


def combined_score(gray_ssim, color_ssim, phash_sim):
    """Skor gabungan 0-100 dari SSIM (rata-rata grayscale+warna) + pHash, sesuai bobot di atas."""
    ssim_avg = (gray_ssim + color_ssim) / 2.0
    raw = SSIM_WEIGHT * ssim_avg + PHASH_WEIGHT * phash_sim
    return max(0.0, min(100.0, raw * 100.0))


def resolve_ref(ref):
    """`ref` item → `(kind, value)`. `kind` ∈ {"none","colors","path"} (lihat `README.md`
    §Skema prompt-set utk 3 bentuk `ref`)."""
    if ref is None:
        return ("none", None)
    if isinstance(ref, str) and ref.startswith("colors:"):
        colors = [c.strip() for c in ref[len("colors:"):].split(",") if c.strip()]
        return ("colors", colors)
    return ("path", os.path.join(REF_ROOT, ref))


GENERATE_PY = os.path.join(HERE, "generate.py")


def regen_item(cfg, item, ref_path, gate, max_attempts=3, tried_seeds=None):
    """Regenerasi `item` via `generate.py --only <id> --seed <next>`, seed naik tiap percobaan,
    sampai skor ≥ `gate` atau `max_attempts` habis. **Simpan skor terbaik** dari semua percobaan
    (bukan cuma yang terakhir) — sesuai "Selesai bila" M07 ("...atau menyerah dengan catatan").
    File di `out_path` di akhir = PNG dari percobaan **terbaik**; PNG lama dikembalikan bila tak
    ada percobaan yang berhasil sama sekali (jangan sampai hilang krn regen gagal total — mis.
    server ComfyUI tak tersedia). File sebelumnya **dihapus tiap percobaan** sebelum panggil
    `generate.py` supaya file basi tak salah dianggap hasil regen baru (bug nyata ditemukan &
    diperbaiki di sini, bukan diasumsikan dari awal — lihat `ITERATION_LOG.md`).
    `tried_seeds`: seed yang sudah dicoba percobaan `--regen` sebelumnya (dari `_verify.json`
    lama) — **dilewati**, supaya panggilan `--regen` berulang menjelajah seed baru, bukan
    mengulang persis yang sama (gap nyata ditemukan: seed selalu `item['seed']+1..+N` tiap
    panggilan, jadi 2x `--regen` berturut cuma reproduksi hasil identik, bukan eksplorasi baru).
    Return dict `{seed, score, attempts, gave_up, tried_seeds}`."""
    out_path = item_output_path(cfg, item)
    orig_backup = out_path + ".orig.bak"
    best_backup = out_path + ".best.bak"
    had_original = os.path.isfile(out_path)
    if had_original:
        shutil.copy2(out_path, orig_backup)

    tried = set(tried_seeds or ())
    best = None
    base_seed = item["seed"]
    offset = 1
    attempt = 0
    while attempt < max_attempts:
        seed = base_seed + offset
        offset += 1
        if seed in tried:
            continue
        tried.add(seed)
        attempt += 1
        if os.path.isfile(out_path):
            os.remove(out_path)  # buang sisa percobaan lalu (incl. file asli) — cek "berhasil"
            # harus berarti generate.py BENAR menulis file baru, bukan file lama kebetulan ada.
        proc = subprocess.run(
            [sys.executable, GENERATE_PY, "--set", cfg["set"], "--only", item["id"],
             "--seed", str(seed)],
            capture_output=True, text=True,
        )
        if proc.returncode != 0 or not os.path.isfile(out_path):
            # cari baris "ERR <id> ..." spesifik dari generate.py dulu (lebih informatif drpd
            # baris terakhir stdout, yg biasanya cuma tip "konversi ke ASCII" tak relevan).
            lines = (proc.stdout or "").strip().splitlines()
            err_lines = [ln for ln in lines if ln.startswith("ERR ")]
            if err_lines:
                reason = err_lines[-1]
            elif (proc.stderr or "").strip():
                reason = proc.stderr.strip().splitlines()[-1]
            elif lines:
                reason = lines[-1]
            else:
                reason = "gagal tanpa pesan"
            print(f"  [regen {attempt}/{max_attempts}] seed={seed} GAGAL: {reason[:120]}")
            continue
        gray_ssim, color_ssim = ssim_scores(out_path, ref_path)
        phash_sim = phash_similarity(out_path, ref_path)
        score = combined_score(gray_ssim, color_ssim, phash_sim)
        print(f"  [regen {attempt}/{max_attempts}] seed={seed} skor={score:.1f}/100")
        if best is None or score > best["score"]:
            best = {"seed": seed, "score": score, "attempts": attempt}
            shutil.copy2(out_path, best_backup)
        if score >= gate:
            break

    # Pasang hasil akhir: percobaan terbaik bila ada; kembalikan PNG lama bila regen gagal total.
    if best is not None:
        shutil.copy2(best_backup, out_path)
        os.remove(best_backup)
    elif had_original:
        shutil.copy2(orig_backup, out_path)
    if had_original and os.path.isfile(orig_backup):
        os.remove(orig_backup)

    if best is None:
        return {"seed": None, "score": None, "attempts": max_attempts, "gave_up": True,
                "tried_seeds": sorted(tried)}
    best["gave_up"] = best["score"] < gate
    best["tried_seeds"] = sorted(tried)
    return best


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--set", default="characters", help="nama prompt-set di prompts/<set>.json")
    ap.add_argument("--gate", type=float, default=90.0, help="ambang skor lulus (0-100)")
    ap.add_argument("--regen", action="store_true",
                     help="regenerasi item REGEN (seed++) sampai lulus atau --max-regen habis")
    ap.add_argument("--max-regen", type=int, default=3, help="maks percobaan regen per item")
    args = ap.parse_args()

    cfg = load_prompts(args.set)
    items = cfg["items"]
    clip_ok = clip_available()

    # Muat _verify.json lama (bila ada) supaya --regen berulang tak mengulang seed yg sudah
    # dicoba (lihat catatan `tried_seeds` di `regen_item`) — per item id -> list seed dicoba.
    prior_tried = {}
    prior_report_path = os.path.join(ROOT, "assets", "source", "_verify", f"{cfg['set']}.json")
    if os.path.isfile(prior_report_path):
        try:
            prior = json.load(open(prior_report_path))
            for r in prior.get("items", []):
                seeds = (r.get("regen") or {}).get("tried_seeds") or []
                if seeds:
                    prior_tried[r["id"]] = seeds
        except (json.JSONDecodeError, OSError):
            pass

    print(f"Set '{cfg['set']}' | {len(items)} item | gate {args.gate}")
    print(f"CLIP: {'aktif' if clip_ok else 'dilewati (torch/open_clip tak terpasang, opsional)'}\n")

    n_png_ok = 0
    n_ref_missing = 0
    records = []
    for it in items:
        out_path = item_output_path(cfg, it)
        has_png = os.path.isfile(out_path)
        n_png_ok += has_png
        kind, val = resolve_ref(it.get("ref"))
        record = {
            "id": it["id"],
            "png": os.path.relpath(out_path, ROOT) if has_png else None,
            "ref_kind": kind,
            "scored": False,
            "score": None,
            "verdict": "SKIPPED",
        }
        if kind == "none":
            ref_desc = "(tak ada target kemiripan)"
            record["ref"] = None
        elif kind == "colors":
            ref_desc = f"warna target {','.join(val)}"
            record["ref"] = val
        else:
            ref_desc = os.path.relpath(val, ROOT)
            record["ref"] = ref_desc
            if not os.path.isfile(val):
                ref_desc += " [FILE TAK ADA]"
                n_ref_missing += 1

        png_status = "PNG ada    " if has_png else "PNG BELUM ada"
        print(f"{it['id']:20} {png_status} ref({kind}): {ref_desc}")

        if kind == "path" and has_png and os.path.isfile(val):
            gray_ssim, color_ssim = ssim_scores(out_path, val)
            phash_sim = phash_similarity(out_path, val)
            score = combined_score(gray_ssim, color_ssim, phash_sim)
            verdict = "PASS" if score >= args.gate else "REGEN"
            print(f"{'':20}   SSIM grayscale={gray_ssim:.3f}  warna={color_ssim:.3f}"
                  f"  pHash={phash_sim:.3f}  -> skor={score:.1f}/100 [{verdict:5}]")
            record.update({
                "scored": True,
                "gray_ssim": round(gray_ssim, 4),
                "color_ssim": round(color_ssim, 4),
                "phash": round(phash_sim, 4),
                "score": round(score, 2),
                "verdict": verdict,
            })
            if clip_ok:
                clip_sim = clip_cosine_similarity(out_path, val)
                record["clip"] = round(clip_sim, 4)
                print(f"{'':20}   CLIP cosine={clip_sim:.3f} (sinyal semantik tambahan, "
                      f"belum masuk skor gabungan)")

            if verdict == "REGEN" and args.regen:
                print(f"{'':20}   --regen aktif, coba ulang ≤{args.max_regen}x:")
                best = regen_item(cfg, it, val, args.gate, args.max_regen,
                                   tried_seeds=prior_tried.get(it["id"]))
                record["regen"] = best
                if not best["gave_up"]:
                    record["score"] = round(best["score"], 2)
                    record["verdict"] = "PASS"
                    verdict = "PASS"
                    print(f"{'':20}   -> lulus stlh regen, seed={best['seed']} "
                          f"skor={best['score']:.1f}/100")
                else:
                    msg = ("tak ada percobaan berhasil (server ComfyUI tak tersedia?)"
                           if best["score"] is None else
                           f"skor terbaik {best['score']:.1f}/100 (seed={best['seed']})")
                    print(f"{'':20}   -> MENYERAH stlh {best['attempts']}x, {msg}")
        records.append(record)

    n_pass = sum(1 for r in records if r["verdict"] == "PASS")
    n_regen = sum(1 for r in records if r["verdict"] == "REGEN")
    n_skipped = sum(1 for r in records if r["verdict"] == "SKIPPED")

    print(f"\n{n_png_ok}/{len(items)} item punya PNG hasil generate.")
    if n_ref_missing:
        print(f"PERINGATAN: {n_ref_missing} ref path tak ditemukan di disk.")

    # Laporan ditulis ke folder terpisah `assets/source/_verify/` (BUKAN di dalam folder set
    # itu sendiri) — set 10-item (mis. celestial) + 1 file laporan akan melanggar CONVENTIONS
    # §1 (≤10 file/folder) bila digabung; ditemukan nyata iterasi ini, bukan diantisipasi awal.
    verify_dir = os.path.join(ROOT, "assets", "source", "_verify")
    os.makedirs(verify_dir, exist_ok=True)
    report = {
        "set": cfg["set"],
        "gate": args.gate,
        "generated_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "summary": {
            "total": len(items), "pass": n_pass, "regen": n_regen, "skipped": n_skipped,
        },
        "items": records,
    }
    report_path = os.path.join(verify_dir, f"{cfg['set']}.json")
    with open(report_path, "w") as f:
        json.dump(report, f, indent=2)
    print(f"Ditulis: {os.path.relpath(report_path, ROOT)}")

    print(f"\n=== Ringkasan '{cfg['set']}': {n_pass}/{n_pass + n_regen} lulus gate {args.gate}"
          f" (skor≥{args.gate}), {n_skipped} dilewati (tak ada target kemiripan) ===")
    if n_regen:
        need_regen = [r["id"] for r in records if r["verdict"] == "REGEN"]
        print(f"Perlu regen ({n_regen}): {', '.join(need_regen)}")
        if not args.regen:
            print("(jalankan ulang dgn --regen utk coba otomatis, atau generate manual + "
                  "verify ulang)")


if __name__ == "__main__":
    main()
