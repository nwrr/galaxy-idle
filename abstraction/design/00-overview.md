# 00 — Overview & Visi

> **Spec set referensi `galaxy-idle`.** Dokumen ini adalah tolak ukur pembuatan project.
> Semua keputusan di sini bersifat final kecuali ditandai `TBD`. Istilah teknis sengaja tetap
> bahasa Inggris agar konsisten dengan penamaan kode.

## Visi

`galaxy-idle` adalah **incremental/idle game** berbasis **TUI** (terminal) bertema sci-fi ala
*Stellaris*, **tanpa combat**. Inti gameplay: membangun ekonomi produksi yang scaling,
mengeksplorasi galaksi prosedural, riset teknologi, lalu melakukan **prestige** dengan "warp jump"
ke galaksi level berikutnya. Fokus desain: **scaling**, **efisiensi**, **eksplorasi**, dan
**risk vs reward** (lewat event, bukan pertempuran).

Target pemain: pengguna terminal yang suka idle game dengan estetika ASCII dan automasi.

## 6 Pilar Desain

1. **Resource Economy** — hirarki resource bertingkat (Tier 1–3), produksi & konsumsi real-time,
   net income jelas di dashboard. Lihat [`02-economy.md`](02-economy.md).
2. **Abstraksi 4X** — tidak ada micromanagement per-tile. Planet = "node" dengan slot
   District/Building. Tidak ada combat; "fleet" diganti **Ship** sebagai gatekeeper + buff pasif.
   Lihat [`03-progression.md`](03-progression.md).
3. **Prestige / Reset** — "Warp Jump" pindah galaksi = prestige. Mata uang permanen **Warp Core**.
   Milky Way jadi **anchor** yang tidak pernah reset. Lihat [`04-prestige.md`](04-prestige.md).
4. **Event & Anomali** — pop-up teks risk/reward saat ship traveling: fenomena kosmik, derelict,
   alien encounter, roaming merchant. Lihat [`05-events.md`](05-events.md).
5. **TUI UX** — layout panel responsif, keybinding ala Vim, color feedback ANSI, animasi partikel
   ASCII. Lihat [`06-ui.md`](06-ui.md).
6. **Arsitektur & Save** — game loop tick, offline progress, save JSON, ProcGen seed-based.
   Lihat [`07-architecture.md`](07-architecture.md).

## Core Loop

```
        ┌─────────────────────────────────────────────────────────┐
        │                                                         │
        ▼                                                         │
  [Extract raw]──▶[Refine/craft]──▶[Sell→Credits / pakai upgrade] │
        │                │                     │                  │
        │                │                     ▼                  │
        │                │            [Upgrade Factory & Ship]    │
        │                │                     │                  │
        │                ▼                     ▼                  │
        │         [Research Data]──▶[Unlock tech/recipe/warp tier]│
        │                                      │                  │
        ▼                                      ▼                  │
  [Unlock planet baru]◀───────────[Ship warp tier naik]          │
        │                                                         │
        └────────── kumpul Warp Threshold ──▶ [WARP JUMP] ────────┘
                                                  │
                                                  ▼
                                   [Galaksi Lvl+1, dapat Warp Core,
                                    Milky Way tetap jalan sbg anchor]
```

Loop pendek (detik–menit): extract → refine → upgrade.
Loop menengah (menit–jam): research → unlock planet/tier → eksplorasi galaksi.
Loop panjang (jam–hari): warp jump prestige → multiplier permanen.

## Glossary

| Istilah | Arti |
|---------|------|
| **Tick** | Satu langkah simulasi logical (1 detik game-time). |
| **Node / ResourceNode** | Sumber raw resource di sebuah planet; bisa di-upgrade levelnya. |
| **Extractor** | Factory yang menambang raw resource dari node. |
| **Refinery / Assembler** | Factory yang mengkonsumsi raw resource → crafted resource via recipe. |
| **Planet** | Lokasi berisi slot untuk node, factory, lab. Punya tier & biome. |
| **Galaxy** | Kumpulan planet. Milky Way = fixed (handcrafted); galaksi luar = ProcGen. |
| **Ship** | Entitas tunggal pemain; level = **Warp Tier**. Gatekeeper unlock + buff pasif. Bukan unit combat. |
| **Warp Tier** | Level ship yang menentukan galaksi mana yang bisa diakses. |
| **Travel Time** | Durasi (game-time) ship menempuh jarak; basis trigger event. |
| **Data / Science Point** | Mata uang riset, dihasilkan Research Lab per detik. |
| **Tech Node** | Simpul tech tree; butuh Data + waktu untuk diselesaikan. |
| **Blueprint** | Recipe crafting yang harus dibeli (umumnya dari Void Merchant). |
| **Credits** | Soft currency dari menjual resource. |
| **Warp Jump** | Aksi prestige: reset galaksi current, pindah ke galaksi level berikutnya. |
| **Warp Core** | Mata uang prestige permanen; tidak ikut reset. |
| **Anchor** | Milky Way (Lvl 0); tidak pernah reset, memberi passive income lintas prestige. |
| **Void Merchant** | Pedagang roaming (event) yang menjual blueprint & rare resource. |
| **Offline Progress** | Resource yang diperhitungkan saat game ditutup, dihitung dari time-delta. |

## Urutan Baca Spec

1. [`00-overview.md`](00-overview.md) — dokumen ini.
2. [`01-data-model.md`](01-data-model.md) — struct & enum (kontrak data).
3. [`02-economy.md`](02-economy.md) — produksi, crafting, market.
4. [`03-progression.md`](03-progression.md) — ProcGen, ship, research.
5. [`04-prestige.md`](04-prestige.md) — warp jump, anchor, merchant.
6. [`05-events.md`](05-events.md) — sistem event.
7. [`06-ui.md`](06-ui.md) — layout TUI.
8. [`07-architecture.md`](07-architecture.md) — loop, save, modul, content loader.
9. [`08-balancing.md`](08-balancing.md) — konstanta global tunable.

### Katalog Konten (data-driven, `data/*.ron`)
10. [`09-milky-way.md`](../content/09-milky-way.md) — Bimasakti = Sol system (galaxy awal/anchor).
11. [`10-resources.md`](../content/10-resources.md) — katalog ~165 resource (sumber semua `ResourceId`).
12. [`11-items.md`](../content/11-items.md) — katalog ~80 item (ship part, artifact, consumable, blueprint).
13. [`12-crafting.md`](../content/12-crafting.md) — katalog ~60 recipe.
14. [`13-buildings.md`](../content/13-buildings.md) — katalog ~40 building.

### Sistem Visual & Generatif (math)
15. [`14-galaxy-animation.md`](../content/14-galaxy-animation.md) — animasi galaksi spiral menu utama.
16. [`15-particle-effects.md`](../content/15-particle-effects.md) — sistem partikel terminal.
17. [`16-procgen-logic.md`](../content/16-procgen-logic.md) — logika & math procedural generation.

## Status & Scope MVP

MVP harus mencakup: Milky Way fixed (Earth + 2 planet), economy loop (extractor + refinery +
market/auto-sell), research dasar, satu warp jump, save/load + offline progress, UI full layout.
ProcGen galaksi luar, event lengkap, dan Void Merchant boleh menyusul pasca-MVP — tapi data model
& arsitektur di spec ini sudah harus mengakomodasinya sejak awal.
