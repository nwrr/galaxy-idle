# 04 — Prestige: Warp Jump, Anchor, Void Merchant

> Loop tak-hingga incremental. Struct: `PrestigeState`, `AnchorState`, `Merchant`, `MerchantOffer`
> di [`01-data-model.md`](01-data-model.md). Konstanta di [`08-balancing.md`](08-balancing.md).

## 1. Warp Jump = Prestige Reset

Pindah galaksi (naik level) = prestige. Memberi konteks naratif kuat untuk reset.

### Trigger (Warp Threshold)
Player harus mengumpulkan ambang resource untuk warp ke galaksi level berikutnya.

```
threshold(level_target) = scaling per level (lihat 08), mis:
  Lvl 1: 1,000,000 Energy + 500 Titanium
  Lvl 2: + Antimatter, dst (makin tinggi makin berat)
```

### Yang **di-reset** saat warp
- Resource dasar di galaksi aktif (`active_galaxy`).
- Factory & node di galaksi aktif.
- `credits`.
- `ResearchState.data` (Data terkumpul) — tech yang sudah `completed` **tidak** hilang.

> Catatan desain: tech `completed` dipertahankan agar progress riset terasa permanen; hanya Data
> "cash" yang reset. (Bisa di-tune; alternatif "soft reset tech" ditandai TBD di `08`.)

### Yang **TIDAK** di-reset (persistent)
- `PrestigeState`: `warp_cores`, `permanent_upgrades`, `blueprints`, `galaxy_level_reached`.
- `AnchorState` (Milky Way) dan seluruh isinya.
- `Ship` (warp tier & part) — progress eksplorasi tetap.

### Warp Core Gain
Mata uang prestige diperoleh berdasarkan seberapa jauh/banyak yang dicapai:

```
warp_cores_gain = floor( WARP_K * sqrt(total_value / WARP_THRESHOLD_REF) )
```

- `total_value` = nilai agregat resource+credits saat warp (dikonversi via price table).
- `sqrt` → diminishing returns: mendorong push lebih jauh sebelum reset, bukan reset cepat.
- `WARP_K`, `WARP_THRESHOLD_REF` di `08`.

### Efek Pindah
Sampai di galaksi Lvl+1: mulai dari nol di galaksi baru, **tetapi** dapat **production multiplier**
berbasis level yang dicapai:

```
production_mult = 1.0 + PRESTIGE_MULT_PER_LEVEL * galaxy_level_reached
```

## 2. Milky Way sebagai Anchor (Lvl 0)

Fitur pembeda: Milky Way **tidak pernah reset**.

- **Fungsi:** base of operations / "mesin pasif". Saat player bermain di galaksi Lvl≥1, factory di
  Milky Way tetap jalan dan menghasilkan resource dasar yang **otomatis di-warp** ke galaksi aktif
  untuk membantu early-game pasca-reset.
- **Passive income ke galaksi aktif per detik:**

```
anchor_feed = ANCHOR_BASE * (1 + ANCHOR_PER_LVL * anchor.passive_upgrade_level)
```

- **Anchor upgrade** dibeli pakai **Warp Core** (bukan Credits) → `AnchorState.passive_upgrade_level`.
  Makin tinggi, makin besar boost ke setiap galaksi berikutnya. Ini sink utama Warp Core jangka panjang.

## 3. Permanent Upgrades (Warp Core sink)

`PrestigeState.permanent_upgrades: HashMap<&str, u32>` — dibeli dengan Warp Core, berlaku lintas
semua galaksi. Contoh (final di `08`):

| Upgrade id | Efek per level | Biaya |
|------------|----------------|-------|
| `prod_speed` | +10% kecepatan produksi global | geometrik (Warp Core) |
| `extra_slot` | +1 factory slot per planet | naik tajam |
| `auto_collect` | Auto-collect/auto-sell lebih agresif | flat tier |
| `offline_eff` | ↑ efisiensi offline progress | geometrik |

## 4. The Void Merchant

Pusat ekonomi mid–late game. **Roaming event**, bukan bangunan statis (lihat juga
[`05-events.md`](05-events.md) §4).

- **Mata uang:** menerima **Warp Core** atau **Rare Resource** (mis. Antimatter) — *bukan* Credits
  biasa untuk blueprint.
- **Blueprint/Formula:** recipe crafting tertentu hanya bisa didapat dengan **membeli blueprint**
  di merchant → masuk `PrestigeState.blueprints`. Recipe dengan `requires_blueprint=true` baru bisa
  dipakai setelah blueprint dimiliki (lihat [`02-economy.md`](02-economy.md) §3).
- **Rotasi stock:** `Merchant.stock` di-restock tiap interval (`restock_at_tick`) agar player rutin
  cek. Saat roaming, merchant `active` hanya sampai `expires_at_tick`.

`MerchantOffer` mencakup `Blueprint`, `BuyResource` (Cores→resource langka), `SellResource`
(resource→Credits dengan rate lebih baik dari market biasa).

## 5. Layout TUI (ringkas — detail di 06)

```
[ WARP NAVIGATION ] - Current: Milky Way (Lvl 0)
> Target: Andromeda (Lvl 1)
> Requirement: 1,000,000 Energy | 500 Titanium
> Current:     1,240,500 Energy | 620 Titanium  [ OK ]
> Estimated Warp Cores gained: 15
> Warning: Factories in current sector will be reset.
> Milky Way (Lvl 0) will remain active as Anchor.
[1] INITIATE WARP JUMP  [2] Back
```

```
[ VOID MERCHANT ] - Warp Cores: 15 | Antimatter: 40
[ BLUEPRINTS ]
> [B-01] Plasma Refinery   | Cost: 5 Cores  | [OWNED]
> [B-02] Quantum Assembler | Cost: 12 Cores | [LOCKED]
[ STOCK (Resets in 2h 14m) ]
> [S-01] Rare Blueprint: Void Drill | Cost: 30 Cores
```

## 6. Loop Prestige

```
push ekonomi/research ──▶ capai Warp Threshold ──▶ WARP JUMP
   │                                                  │
   │                                  ┌───────────────┤
   ▼                                  ▼               ▼
reset galaksi aktif        +Warp Core (sqrt)   production_mult ↑
   │                                  │
   └── Milky Way (anchor) feed ──▶ early-game galaksi baru lebih cepat
                                      │
                       Warp Core ──▶ anchor upgrade / permanent upgrade / blueprint
```
