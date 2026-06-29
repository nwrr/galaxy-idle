# 05 — Events & Encounters

> Tanpa combat → semua event berfokus **risk vs reward**, eksplorasi, dan ekonomi/diplomasi.
> Pemain *investigate*, *salvage*, *trade* — bukan *fight*. Struct: `EventQueue`, `GameEvent`,
> `Merchant` di [`01-data-model.md`](01-data-model.md). Konstanta di [`08-balancing.md`](08-balancing.md).

## Tipe `GameEvent`

```rust
pub struct GameEvent {
    pub id: u64,
    pub kind: EventKind,
    pub category: EventCategory,     // utk warna log
    pub created_tick: u64,
    pub expires_tick: Option<u64>,   // None = menunggu sampai direspon
    pub options: Vec<EventOption>,   // aksi yang bisa dipilih player
}

pub enum EventCategory { Hazard, Loot, Trade, Neutral } // → warna: merah/hijau/kuning/abu

pub struct EventOption {
    pub label: String,               // "Salvage", "Ignore", "Scan"
    pub requires: Option<EventReq>,  // mis. ShipPart minimal, Energy cost
    pub duration_secs: f64,          // 0 = instan; >0 = pause travel / butuh waktu
    pub effect: EventEffect,         // resolusi: tambah/kurang resource, buff, dll
}

pub enum EventReq {
    EnergyCost(f64),
    ScannerLevel(u32),
    ShipPart { /* mis. cloaking, shield */ tag: &'static str, level: u32 },
}

pub enum EventEffect {
    GainResources(Vec<(ResourceId, f64)>),
    LoseResources(Vec<(ResourceId, f64)>),
    GainBlueprint(RecipeId),
    TempProductionBuff { mult: f64, duration_secs: f64 },
    AdjustTravelTime { delta_secs: f64 },
    Reputation(i32),
    Nothing,
}

pub enum EventKind {
    BlackHole, AsteroidField, Nebula,         // §1 Cosmic
    ShipWreck, AlienOutpost,                  // §2 Derelict
    AlienCaravan, DistressSignal,             // §3 Alien
    VoidMerchant,                             // §4 (lihat 04-prestige.md)
}
```

## 1. Cosmic Phenomena (Hazard / Peluang)

| Event | Mekanik | Reward |
|-------|---------|--------|
| **Black Hole** | Butuh Shield/Gravity Plating cukup; kalau tidak, ship kehilangan durabilitas/cargo | `SingularityMatter` (ultra-rare prestige) atau Wormhole (shortcut ke galaksi berikut) |
| **Asteroid Field / Nebula** | `AdjustTravelTime +20%` | Jika `ScannerLevel` cukup → menambang Rare Asteroid pasif selama nyangkut |

## 2. Derelicts & Wrecks (Salvage)

| Event | Mekanik | Reward |
|-------|---------|--------|
| **Ship Wreck** | Pilih `Ignore` atau `Salvage` (duration ~10m, pause travel) | Scrap Metal, blueprint acak, atau ship part mentah |
| **Alien Outpost** | Butuh Energy untuk "nyalakan reaktor" (`EnergyCost`) | `TempProductionBuff` (mis. +50% 1 jam) atau Artifact (dijual mahal ke Merchant) |

## 3. Alien Encounters (Trade / Quest)

| Event | Mekanik | Pilihan & Efek |
|-------|---------|----------------|
| **Alien Caravan** | Tawaran trade unik (tidak ada di Merchant biasa) | mis. 500 Titanium → 10 Exotic Spices |
| **Distress Signal** | Sinyal SOS | [1] Bantu (kasih resource) → Reputation/Tech Data · [2] Abaikan → kadang hidden penalty · [3] Jarah (butuh Cloaking) → resource curian, Reputation turun |

`Reputation` (disimpan sebagai field sederhana, dapat ditambahkan ke `GameState` saat alien faction
diimplementasi — TBD di `08`) memengaruhi tawaran trade di masa depan.

## 4. Void Merchant (Roaming)

Muncul acak antar-galaksi, stay terbatas (`expires_at_tick`). Detail penuh di
[`04-prestige.md`](04-prestige.md) §4. Di sistem event, kemunculannya dipicu oleh travel + scanner
seperti event lain.

## 5. Trigger & Queue

**Trigger chance per "segmen" travel:**

```
event_chance = clamp01( BASE_EVENT_CHANCE
                        * (1 + DIST_FACTOR * travel_distance)
                        * (1 + SCANNER_FACTOR * ship.scanner) )
```

- Makin jauh travel + makin tinggi scanner → makin sering event (dan scanner juga ↑ kualitas/loot).
- Saat ship `Traveling`, sistem roll event tiap `EVENT_ROLL_INTERVAL` tick.

**Queue & resolusi:**
- Event yang belum direspon masuk `EventQueue.pending` dan tampil di panel `[ LOG / EVENTS ]`
  dengan warna per `EventCategory`.
- Player memilih `EventOption`; jika `duration_secs > 0`, travel pause / drone dikerahkan selama itu,
  lalu `EventEffect` diterapkan.
- Event dengan `expires_tick` hilang otomatis bila diabaikan terlalu lama.

**Auto-Resolve (opsional, idle-friendly):** rule sederhana, mis. "Auto-salvage wrecks if Cargo < 50%",
disimpan di `Settings` (sejajar dengan auto-sell). Mencegah player harus klik manual terus.

## 6. Layout TUI (ringkas — detail di 06)

```
[ ENCOUNTER ALERT! ] - Scanner detected anomaly at Sector 4G
> Type: Derelict Cruiser (Wreck)
> Distance: 0.5 LY | Est. Time to Salvage: 15m
> Risk: Low | Potential Loot: [Ship Parts], [Data Cores]
[1] Deploy Salvage Drone (Pause travel 15m)
[2] Scan from afar (Cost: 50 Energy, reveals exact loot)
[3] Ignore & Continue Warp

[ ACTIVE EVENTS QUEUE ]
> [!] Alien Caravan approaching (Arrives in 2m)
> [i] Nebula detected (Will slow travel by 20%)
```
