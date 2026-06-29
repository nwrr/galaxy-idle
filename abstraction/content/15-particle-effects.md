# 15 — Particle Effects (Terminal)

> Sistem partikel **berwarna** (glyph + warna ANSI per kind, truecolor→256→16 sesuai terminal) untuk
> efek visual (engine exhaust, warp trail, nebula, sparkle). **Kosmetik**
> — tidak memutasi `GameState`; di-update di loop render ([`07-architecture.md`](../design/07-architecture.md))
> dengan `dt` waktu nyata. Melengkapi [`14-galaxy-animation.md`](14-galaxy-animation.md) &
> [`06-ui.md`](../design/06-ui.md).

## Struct Partikel

```rust
struct Particle {
    pos: (f32, f32),     // koordinat sel (kolom, baris) — pakai f32 utk gerak halus sub-sel
    vel: (f32, f32),     // sel/detik
    accel: (f32, f32),   // mis. gravitasi/drift; (0,0) bila lurus
    life: f32,           // sisa umur (detik)
    max_life: f32,       // untuk normalisasi fade
    kind: ParticleKind,  // menentukan ramp & warna
}

struct ParticleSystem {
    particles: Vec<Particle>,   // pre-alokasi kapasitas = budget
    emitters: Vec<Emitter>,
}

struct Emitter {
    origin: (f32, f32),
    rate: f32,           // partikel/detik
    accum: f32,          // akumulator spawn fractional
    spread: f32,         // sudut sebar (rad)
    speed: (f32, f32),   // rentang kecepatan (min,max)
    life: (f32, f32),    // rentang umur
    kind: ParticleKind,
    enabled: bool,
}
```

## Update (integrasi Euler, per frame)

```rust
fn update(&mut self, dt: f32) {
    // 1. Spawn dari emitter (fractional-accurate)
    for e in &mut self.emitters {
        if !e.enabled { continue; }
        e.accum += e.rate * dt;
        while e.accum >= 1.0 && self.particles.len() < BUDGET {
            e.accum -= 1.0;
            self.particles.push(e.spawn());   // sample sudut/speed/life acak
        }
    }
    // 2. Integrasi + decay
    for p in &mut self.particles {
        p.vel.0 += p.accel.0 * dt;  p.vel.1 += p.accel.1 * dt;
        p.pos.0 += p.vel.0 * dt;    p.pos.1 += p.vel.1 * dt;
        p.life  -= dt;
    }
    // 3. Cull: mati / keluar layar (swap_remove → O(1))
    self.particles.retain(|p| p.life > 0.0 && in_bounds(p.pos));
}
```

`spawn()`:
```
angle = base_dir + uniform(-spread/2, spread/2)
speed = lerp(speed.min, speed.max, rng.f32())
vel   = (cos(angle)·speed, sin(angle)·speed / ASPECT)   // koreksi aspek 2:1
life  = lerp(life.min, life.max, rng.f32())
```

## Lifetime → Karakter (fade ramp)

```
age_frac = p.life / p.max_life            // 1.0 baru → 0.0 mati
RAMP_FADE = ['@','#','*',':','.','·',' '] // padat → hilang (urut dari muda ke tua)
idx   = floor( (1.0 - age_frac) · (RAMP_FADE.len()-1) )
glyph = RAMP_FADE[idx]
```

Untuk efek "menyala lalu pudar", inversikan: muda = terang (`*`/`✦`), tua = redup (`·`).

## Densitas → Glyph (banyak partikel per sel)

Saat beberapa partikel jatuh ke sel yang sama, akumulasi "energi" lalu petakan:

```
RAMP_DENSITY = [' ', '.', ':', '-', '=', '+', '*', '#', '@']
e_cell = Σ age_frac_p   untuk semua p di sel
glyph  = RAMP_DENSITY[ min(floor(e_cell), last) ]
```

Memberi gradien halus pada awan padat (nebula/exhaust) tanpa terlihat sebagai titik terpisah.

## Warna (fade ANSI per kind)

```
ParticleKind::Exhaust  → Putih → Kuning → Merah → redup (panas mendingin)
ParticleKind::WarpTrail→ Cyan → Biru → Magenta (sesuai t)
ParticleKind::Nebula   → Magenta/Ungu redup, drift lambat
ParticleKind::Sparkle  → Hijau terang (resource pickup), umur pendek
ParticleKind::Star     → twinkle (lihat 14)
```

Interpolasi warna pakai `age_frac`: warna muda → warna tua. Konsisten dgn [`06-ui.md`](../design/06-ui.md).

## Contoh Efek (parameter Emitter)

| Efek | rate | spread | speed | life | accel | kind | dipicu |
|------|------|--------|-------|------|-------|------|--------|
| Engine Exhaust | 40/s | 0.3 | 8–15 | 0.4–0.8 | drift mundur | Exhaust | ShipStatus::Traveling |
| Warp Trail | 60/s | 0.1 | 20–30 | 0.3–0.6 | 0 | WarpTrail | saat Warp Jump (sekali, burst) |
| Nebula Drift | 5/s | 6.28 | 0.5–1.5 | 4–8 | angin lemah | Nebula | latar Galaxy Map |
| Resource Sparkle | burst 20 | 6.28 | 3–6 | 0.3–0.5 | gravitasi naik | Sparkle | resource ter-collect/upgrade sukses |
| Explosion-free Pulse | burst 30 | 6.28 | 5–10 | 0.5 | 0 | Sparkle | event selesai (loot) |

> Tidak ada efek combat (tema tanpa combat). "Burst" = naikkan `rate` sesaat / spawn N sekaligus
> lalu `enabled=false`.

## Budget & Perf

| Const | Nilai | Catatan |
|-------|-------|---------|
| `PARTICLE_BUDGET` | 500 | total partikel maksimum (hard cap) |
| `EMITTER_MAX` | 16 | emitter aktif bersamaan |

- `Vec` di-pre-alokasi `PARTICLE_BUDGET`; spawn berhenti saat penuh (drop tertua opsional).
- `retain`/`swap_remove` → cull O(n); update keseluruhan O(PARTICLE_BUDGET) per frame.
- Render: tulis ke buffer densitas (reuse antar-frame), satu pass ke `ratatui` Buffer.
- Pada `LayoutMode::Minimal`, matikan efek non-esensial (nebula) demi terminal kecil/lambat.

## Integrasi Loop

```
loop render (07):
    dt = now - last_render
    galaxy_anim.render(...)           // 14
    particles.update(dt)              // di sini
    particles.render(buf, area)
    last_render = now
```

Partikel & animasi galaxy hidup di layer render yang sama; keduanya membaca `GameState` read-only
(mis. posisi ship) untuk memicu emitter, tapi tidak pernah menulisnya.
