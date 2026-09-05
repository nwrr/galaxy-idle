#!/usr/bin/env python3
"""Auto-generate asset via API ComfyUI → simpan PNG sumber. Multi-set (M05.4): tiap set =
`prompts/<set>.json` (skema `comfyui/README.md` §Skema prompt-set), workflow diresolusi dari
field `workflow` di set itu sendiri (`workflows/<workflow>.json`).

ComfyUI HARUS sudah berjalan (default API http://127.0.0.1:8188). Script ini tidak menjalankan
ComfyUI; ia memposting workflow tiap item ke server, lalu mengunduh hasil ke
assets/source/<set>/[<group>/]<filename>.

Usage:
  python3 comfyui/generate.py                        # --set characters (default), semua item
  python3 comfyui/generate.py --set characters --only char_male_01
  python3 comfyui/generate.py --set characters --group alien
  python3 comfyui/generate.py --size 832x1216 --steps 6
  python3 comfyui/generate.py --dry-run               # tampilkan rencana, tanpa memanggil server

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


def load_prompts(name):
    """Baca `prompts/<name>.json` saja (tanpa workflow) — dipakai `verify.py` (M07) juga."""
    prompts_path = os.path.join(HERE, "prompts", f"{name}.json")
    if not os.path.isfile(prompts_path):
        raise SystemExit(f"Set '{name}' tak ditemukan: {prompts_path}")
    return json.load(open(prompts_path))


def load_set(name):
    """Baca `prompts/<name>.json` + resolusi workflow-nya (`workflows/<workflow>.json`)."""
    cfg = load_prompts(name)
    workflow_path = os.path.join(HERE, "workflows", f"{cfg['workflow']}.json")
    if not os.path.isfile(workflow_path):
        raise SystemExit(f"Workflow '{cfg['workflow']}' tak ditemukan: {workflow_path}")
    tmpl = json.load(open(workflow_path))
    return cfg, tmpl


def item_output_path(cfg, item):
    """Path output PNG item (`assets/source/<set>/[<group>/]<filename>`) — satu sumber kebenaran
    dipakai `generate.py` & `verify.py` (M07), jangan duplikasi logika ini di tempat lain."""
    out_base = os.path.join(ROOT, "assets", "source", cfg["set"])
    out_dir = os.path.join(out_base, item["group"]) if "group" in item else out_base
    filename = item.get("filename", f"{item['id']}.png")
    return os.path.join(out_dir, filename)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--server", default="http://127.0.0.1:8188")
    ap.add_argument("--set", default="characters", help="nama prompt-set di prompts/<set>.json")
    ap.add_argument("--only", help="id item tunggal, mis. char_alien_03")
    ap.add_argument("--group", help="filter field 'group' item (mis. male/female/alien)")
    ap.add_argument("--size", help="WxH, mis. 1024x1024 (override prompts/<set>.json)")
    ap.add_argument("--steps", type=int, default=4)
    ap.add_argument("--seed", type=int,
                     help="override seed semua item terpilih (regen: pakai bareng --only)")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    cfg, tmpl = load_set(args.set)
    style = cfg["style"]
    size = args.size or cfg.get("size", "1024x1024")
    w, h = (int(x) for x in size.lower().split("x"))

    items = cfg["items"]
    if args.only:
        items = [c for c in items if c["id"] == args.only]
    if args.group:
        items = [c for c in items if c.get("group") == args.group]
    if not items:
        print("Tidak ada item cocok filter."); return

    client_id = uuid.uuid4().hex
    print(f"Set '{cfg['set']}' | Server {args.server} | {len(items)} item | "
          f"{w}x{h} | steps {args.steps}\n")

    for c in items:
        positive = f'{c["subject"]}, {style}'
        seed = args.seed if args.seed is not None else c["seed"]
        out_path = item_output_path(cfg, c)
        out_dir = os.path.dirname(out_path)
        if args.dry_run:
            print(f"[dry] {c['id']:16} seed={seed:>8} -> {os.path.relpath(out_path, ROOT)}")
            print(f"      {positive[:100]}...")
            continue
        graph = build_graph(tmpl, positive, seed, w, h, args.steps,
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
