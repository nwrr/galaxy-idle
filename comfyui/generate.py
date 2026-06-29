#!/usr/bin/env python3
"""Auto-generate 21 portrait karakter via API ComfyUI (Z-Image Turbo) → simpan PNG sumber.

ComfyUI HARUS sudah berjalan (default API http://127.0.0.1:8188). Script ini tidak menjalankan
ComfyUI; ia memposting workflow_zimage_turbo_api.json untuk tiap prompt di prompts/characters.json,
lalu mengunduh hasil ke assets/source/characters/<group>/<filename>.

Usage:
  python3 comfyui/generate.py                     # generate semua 21
  python3 comfyui/generate.py --only char_male_01
  python3 comfyui/generate.py --group alien
  python3 comfyui/generate.py --size 832x1216 --steps 6
  python3 comfyui/generate.py --dry-run           # tampilkan rencana, tanpa memanggil server

Lalu konversi ke ASCII:
  python3 scripts/gen_assets.py all && python3 scripts/check_assets.py
"""
import argparse
import json
import os
import time
import urllib.request
import urllib.parse
import uuid

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
OUT_BASE = os.path.join(ROOT, "assets", "source", "characters")
WORKFLOW = os.path.join(HERE, "workflow_zimage_turbo_api.json")
PROMPTS = os.path.join(HERE, "prompts", "characters.json")


def http_json(url, payload):
    data = json.dumps(payload).encode()
    req = urllib.request.Request(url, data=data, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=30) as r:
        return json.loads(r.read())


def http_get(url):
    with urllib.request.urlopen(url, timeout=30) as r:
        return r.read()


def queue_prompt(server, graph, client_id):
    return http_json(f"{server}/prompt", {"prompt": graph, "client_id": client_id})["prompt_id"]


def wait_images(server, prompt_id, timeout=600):
    t0 = time.time()
    while time.time() - t0 < timeout:
        try:
            hist = json.loads(http_get(f"{server}/history/{prompt_id}"))
        except Exception:
            hist = {}
        if prompt_id in hist:
            outs = hist[prompt_id].get("outputs", {})
            imgs = []
            for node in outs.values():
                imgs += node.get("images", [])
            if imgs:
                return imgs
        time.sleep(1.0)
    raise TimeoutError(f"timeout menunggu {prompt_id}")


def fetch_image(server, img):
    q = urllib.parse.urlencode({
        "filename": img["filename"],
        "subfolder": img.get("subfolder", ""),
        "type": img.get("type", "output"),
    })
    return http_get(f"{server}/view?{q}")


def build_graph(tmpl, prompt, seed, w, h, steps, prefix):
    g = json.loads(json.dumps(tmpl))  # deep copy
    g["4"]["inputs"]["text"] = prompt
    g["7"]["inputs"]["width"] = w
    g["7"]["inputs"]["height"] = h
    g["8"]["inputs"]["seed"] = seed
    g["8"]["inputs"]["steps"] = steps
    g["10"]["inputs"]["filename_prefix"] = prefix
    return g


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--server", default="http://127.0.0.1:8188")
    ap.add_argument("--only", help="id karakter tunggal, mis. char_alien_03")
    ap.add_argument("--group", choices=["male", "female", "alien"])
    ap.add_argument("--size", help="WxH, mis. 1024x1024 (override characters.json)")
    ap.add_argument("--steps", type=int, default=4)
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    tmpl = json.load(open(WORKFLOW))
    cfg = json.load(open(PROMPTS))
    style = cfg["style"]
    size = args.size or cfg.get("size", "1024x1024")
    w, h = (int(x) for x in size.lower().split("x"))

    chars = cfg["characters"]
    if args.only:
        chars = [c for c in chars if c["id"] == args.only]
    if args.group:
        chars = [c for c in chars if c["group"] == args.group]
    if not chars:
        print("Tidak ada karakter cocok filter."); return

    client_id = uuid.uuid4().hex
    print(f"Server {args.server} | {len(chars)} karakter | {w}x{h} | steps {args.steps}\n")

    for c in chars:
        positive = f'{c["subject"]}, {style}'
        out_dir = os.path.join(OUT_BASE, c["group"])
        out_path = os.path.join(out_dir, c["filename"])
        if args.dry_run:
            print(f"[dry] {c['id']:16} seed={c['seed']:>8} -> {os.path.relpath(out_path, ROOT)}")
            print(f"      {positive[:100]}...")
            continue
        graph = build_graph(tmpl, positive, c["seed"], w, h, args.steps,
                            f"galaxy_idle/{c['id']}")
        try:
            pid = queue_prompt(args.server, graph, client_id)
            imgs = wait_images(args.server, pid)
            os.makedirs(out_dir, exist_ok=True)
            with open(out_path, "wb") as f:
                f.write(fetch_image(args.server, imgs[0]))
            print(f"OK  {c['id']:16} -> {os.path.relpath(out_path, ROOT)}")
        except Exception as e:
            print(f"ERR {c['id']:16} {e}")

    if not args.dry_run:
        print("\nSelesai. Konversi ke ASCII:")
        print("  python3 scripts/gen_assets.py all && python3 scripts/check_assets.py")


if __name__ == "__main__":
    main()
