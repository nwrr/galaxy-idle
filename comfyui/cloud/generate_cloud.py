#!/usr/bin/env python3
"""Opsi kedua: generate portrait via Flux (fal.ai / Replicate) → simpan PNG sumber.

Baca prompt yang sama dgn pipeline lokal (comfyui/prompts/characters.json) agar konsisten.

Prasyarat:
  fal:       pip install fal-client ; export FAL_KEY=...
  replicate: pip install replicate  ; export REPLICATE_API_TOKEN=...

Usage:
  python3 comfyui/cloud/generate_cloud.py --provider fal --model fal-ai/flux/dev
  python3 comfyui/cloud/generate_cloud.py --provider replicate --model black-forest-labs/flux-dev
  python3 comfyui/cloud/generate_cloud.py --provider fal --only char_alien_03
  python3 comfyui/cloud/generate_cloud.py --provider fal --group female --aspect 4:5

Lalu: python3 scripts/gen_assets.py all && python3 scripts/check_assets.py
"""
import argparse
import json
import os
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
COMFY = os.path.dirname(HERE)
ROOT = os.path.dirname(COMFY)
OUT_BASE = os.path.join(ROOT, "assets", "source", "characters")
PROMPTS = os.path.join(COMFY, "prompts", "characters.json")

ASPECT_TO_SIZE = {"1:1": (1024, 1024), "4:5": (1024, 1280), "3:4": (1024, 1365)}


def save_bytes(path, data):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb") as f:
        f.write(data)


def download(url):
    with urllib.request.urlopen(url, timeout=120) as r:
        return r.read()


def run_fal(model, prompt, w, h, steps):
    import fal_client
    res = fal_client.run(model, arguments={
        "prompt": prompt,
        "image_size": {"width": w, "height": h},
        "num_inference_steps": steps,
        "guidance_scale": 3.5,
        "num_images": 1,
        "output_format": "png",
    })
    return download(res["images"][0]["url"])


def run_replicate(model, prompt, aspect, steps):
    import replicate
    out = replicate.run(model, input={
        "prompt": prompt,
        "aspect_ratio": aspect,
        "num_inference_steps": steps,
        "guidance": 3.5,
        "output_format": "png",
    })
    item = out[0] if isinstance(out, list) else out
    if hasattr(item, "read"):
        return item.read()
    return download(str(item))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--provider", required=True, choices=["fal", "replicate"])
    ap.add_argument("--model", required=True,
                    help="mis. fal-ai/flux/dev atau black-forest-labs/flux-dev")
    ap.add_argument("--only")
    ap.add_argument("--group", choices=["male", "female", "alien"])
    ap.add_argument("--aspect", default="1:1", choices=list(ASPECT_TO_SIZE))
    ap.add_argument("--steps", type=int, default=28)
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    cfg = json.load(open(PROMPTS))
    style = cfg["style"]
    w, h = ASPECT_TO_SIZE[args.aspect]
    chars = cfg["characters"]
    if args.only:
        chars = [c for c in chars if c["id"] == args.only]
    if args.group:
        chars = [c for c in chars if c["group"] == args.group]

    print(f"{args.provider}:{args.model} | {len(chars)} gambar | {args.aspect} ({w}x{h})\n")
    for c in chars:
        prompt = f'{c["subject"]}, {style}'
        out_path = os.path.join(OUT_BASE, c["group"], c["filename"])
        if args.dry_run:
            print(f"[dry] {c['id']:16} -> {os.path.relpath(out_path, ROOT)}")
            continue
        try:
            if args.provider == "fal":
                data = run_fal(args.model, prompt, w, h, args.steps)
            else:
                data = run_replicate(args.model, prompt, args.aspect, args.steps)
            save_bytes(out_path, data)
            print(f"OK  {c['id']:16} -> {os.path.relpath(out_path, ROOT)}")
        except Exception as e:
            print(f"ERR {c['id']:16} {e}")

    if not args.dry_run:
        print("\nLalu: python3 scripts/gen_assets.py all && python3 scripts/check_assets.py")


if __name__ == "__main__":
    main()
