# 02 — Economy: Produksi, Crafting, Market

> Jantung game. Semua angka default ada di [`08-balancing.md`](08-balancing.md); di sini dijelaskan
> mekanik + rumusnya. Struct terkait: `Factory`, `ResourceNode`, `Recipe`, `Resource`, `AutoSellRule`
> di [`01-data-model.md`](01-data-model.md).

## 1. Hirarki Resource

| Tier | Resource | Sumber | Kegunaan |
|------|----------|--------|----------|
| **T1 Basic** | Iron, Carbon, Water, Energy | Extractor di node planet T1 | Bahan dasar, upkeep |
| **T2 Advanced** | Silicon, Titanium, Steel, Alloys, ConsumerGoods | Node planet T2 / hasil refinery | Upgrade factory & ship, riset |
| **T3 Rare** | ExoticGas, RareCrystals, DarkMatter, Antimatter, Plasma | Node planet T3 / refinery blueprint | Endgame, syarat warp, beli di merchant |
| **Special** | SingularityMatter | Event (black hole) | Upgrade prestige |

`Energy` istimewa: dipakai sebagai **upkeep** beberapa factory (lihat §3) dan sebagai biaya
sebagian aksi (mis. scan event). Defisit Energy memberi penalti, bukan game-over (lihat §6).

## 2. Node & Extractor

`ResourceNode` = titik tambang di planet. `Extractor` factory menambang dari satu node.

**Output extractor per detik:**

```
output_per_sec(factory, node) =
    BASE_EXTRACTOR_RATE
    * factory.level
    * node.richness
    * tech_multiplier(node.resource)   // dari research, default 1.0
    * (if factory.enabled { 1.0 } else { 0.0 })
```

- `BASE_EXTRACTOR_RATE` & `node.richness` dari [`08-balancing.md`](08-balancing.md).
- `node.level` menaikkan *plafon* (richness efektif) — lihat biaya upgrade node di §5.
- Output diakumulasi ke `Planet.stockpile` (dibatasi `stockpile_cap`).

## 3. Refinery / Assembler & Crafting

Refinery mengkonsumsi raw → menghasilkan crafted via `Recipe`.

**Per tick, untuk tiap Refinery `enabled`:**

```
craft_runs = factory.level                      // throughput linear thd level
for each run (sampai input habis atau cap output):
    if stockpile punya semua recipe.inputs:
        kurangi inputs
        tambah outputs (hormati stockpile_cap)
    else:
        catat "deficit <resource>" ke event log, stop
```

- Recipe dengan `craft_time_secs > 0` butuh akumulasi waktu (progress bar di UI); `0.0` = instan.
- Recipe dengan `requires_blueprint = true` hanya jalan kalau `recipe_id ∈ prestige.blueprints`.

**Upkeep (opsional, beberapa factory):** beberapa factory mengkonsumsi `Energy` atau `Credits`
per detik. Net income = produksi − upkeep, ditampilkan real-time di dashboard.

Contoh recipe (katalog lengkap di [`12-crafting.md`](../content/12-crafting.md)):

| Recipe id | Inputs | Output | Blueprint? |
|-----------|--------|--------|-----------|
| `steel_mill` | 2 Iron + 1 Carbon | 1 Steel | no |
| `alloy_forge` | 2 Steel + 1 Titanium | 1 Alloys | no |
| `plasma_refinery` | 5 ExoticGas + 2 Energy | 1 Plasma | **yes** |
| `quantum_assembler` | 1 Silicon + 1 Plasma | 1 QuantumChip* | **yes** |

\* QuantumChip bisa ditambahkan ke enum `Resource` saat dibutuhkan (lihat MVP scope di `00`).

## 4. Market, Credits & Auto-Sell

**Credits** = soft currency dari menjual resource.

```
credits_gain = amount_sold * price(resource)
```

`price(resource)` = `base_price` per resource di katalog [`10-resources.md`](../content/10-resources.md)
(Rare > Advanced > Basic).

**Auto-Sell (fitur wajib idle):** `Settings.auto_sell: Vec<AutoSellRule>`. Tiap tick, untuk rule
`enabled`:

```
surplus = stockpile[resource] - rule.keep_above
if surplus > 0:
    jual surplus, tambah credits
```

Auto-sell mencegah `stockpile` mentok di cap dan jadi sumber Credits pasif. Player set ambang
per-resource lewat panel `[ AUTO-SELL CONFIG ]` (lihat [`06-ui.md`](06-ui.md)).

## 5. Scaling: Biaya Upgrade

Rumus tunggal untuk semua upgrade (factory level, node level, ship part):

```
cost(level_sekarang) = base_cost * GROWTH ^ level_sekarang
```

- `GROWTH = 1.15` default (geometrik — standar incremental).
- `base_cost` berbeda per jenis (factory vs ship part vs node) → di `08`.
- Biaya bisa dibayar pakai Credits dan/atau resource crafted, tergantung jenis upgrade
  (factory: Credits + crafted; ship: crafted T2/T3).

**Output naik linear, biaya naik geometrik** → menciptakan "wall" alami yang mendorong player
membuka resource/tier baru dan akhirnya warp jump. Lihat kurva di `08`.

## 6. Deficit & Feedback

- **Deficit input refinery:** refinery idle (tidak craft), log kuning `Deficit <resource>`.
- **Deficit Energy upkeep:** factory ber-upkeep Energy berproduksi pada efisiensi tereduksi
  (mis. ×0.5) sampai Energy positif lagi. Tidak ada kerusakan permanen.
- **Stockpile penuh:** kelebihan produksi terbuang (atau auto-sold bila ada rule). UI menandai
  resource yang `>= cap` dengan warna.

Semua status ini dirender real-time sebagai **net income** (hijau surplus / merah defisit) per
resource di panel `[ RESOURCES ]`.

## 7. Ringkasan Loop Ekonomi

```
node ──Extractor──▶ raw ──Refinery──▶ crafted ──┬─▶ Market (Credits)
                                                 ├─▶ Upgrade factory/node/ship
                                                 └─▶ Research / Warp threshold
```

Auto-sell + akumulasi offline membuat loop ini berjalan tanpa input terus-menerus — sesuai prinsip
idle.
