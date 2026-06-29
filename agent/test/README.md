# Visual Verification — "Agent Melihat Terminal"

Cara agent memverifikasi tampilan TUI **secara deterministik & langsung terbaca**, tanpa PTY/real
terminal. Inti: render ke `ratatui::backend::TestBackend` → buffer teks → bandingkan golden snapshot.

## Kenapa TestBackend (bukan capture terminal asli)

- **Deterministik:** output sama tiap run → bisa di-assert otomatis.
- **Terbaca agent:** hasil = teks biasa, muncul langsung di tool-result; agent "melihat" frame.
- **Tanpa dependency:** tak perlu tmux/asciinema/PTY; jalan di CI.

## Komponen (dibuat saat loop, M3)

### 1. Helper `buffer_to_text` (`src/ui/mod.rs` atau `src/test_support.rs`)

```rust
use ratatui::buffer::Buffer;

/// Ubah Buffer ratatui jadi Vec<String> (satu string per baris), tanpa warna.
pub fn buffer_to_text(buf: &Buffer) -> Vec<String> {
    let area = buf.area();
    (0..area.height).map(|y| {
        (0..area.width).map(|x| buf[(x, y)].symbol().chars().next().unwrap_or(' '))
            .collect::<String>()
            .trim_end().to_string()
    }).collect()
}
```

### 2. Binari `src/bin/snapshot.rs` (dump headless ke stdout)

```rust
// Usage: cargo run --bin snapshot -- <view> <w> <h>
//   view: main_menu | planet_view | galaxy_map | research | warp
use ratatui::{Terminal, backend::TestBackend};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let view = args.get(1).map(String::as_str).unwrap_or("main_menu");
    let w: u16 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(120);
    let h: u16 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(40);

    let backend = TestBackend::new(w, h);
    let mut term = Terminal::new(backend).unwrap();
    let state = galaxy_idle::demo_state();          // state deterministik utk snapshot
    term.draw(|f| galaxy_idle::ui::draw_view(f, &state, view)).unwrap();

    for line in galaxy_idle::ui::buffer_to_text(term.backend().buffer()) {
        println!("{line}");
    }
}
```

> `demo_state()` = state contoh tetap (resource/ planet terisi) agar snapshot stabil — bukan state
> acak. Definisikan di lib crate.

### 3. Snapshot test (`tests/snapshots.rs`)

```rust
// Bandingkan render dengan golden file di agent/test/snapshots/.
// Set UPDATE_SNAPSHOTS=1 untuk menulis ulang golden (keputusan sadar agent/manusia).
fn check(view: &str, w: u16, h: u16, golden: &str) {
    let got = render_to_string(view, w, h);        // pakai helper sama dgn bin
    let path = format!("agent/test/snapshots/{golden}");
    if std::env::var("UPDATE_SNAPSHOTS").is_ok() {
        std::fs::write(&path, &got).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&path).unwrap();
    assert_eq!(got.trim_end(), want.trim_end(), "snapshot {golden} mismatch");
}

#[test] fn main_menu()  { check("main_menu",   120, 40, "main_menu_120x40.txt"); }
#[test] fn planet_view(){ check("planet_view", 80,  30, "planet_view_80x30.txt"); }
```

## Alur Verifikasi Agent (tiap iterasi yang menyentuh UI)

```
1. scripts/snapshot.sh <view> <w> <h>     # cetak frame teks → agent BACA langsung
2. cocokkan dengan agent/test/visual_checks.md   # assertion semantik (apa yang HARUS ada)
3. temukan yang kurang (panel hilang, overflow, teks salah, misalignment) → perbaiki
4. scripts/verify.sh                       # termasuk snapshot test (golden diff)
5. bila perubahan layout memang disengaja → UPDATE_SNAPSHOTS=1 untuk regenerasi golden
```

## File

- [`visual_checks.md`](visual_checks.md) — daftar assertion semantik per view.
- [`snapshots/`](snapshots/) — golden frame `.txt` (target render). Contoh `main_menu_120x40.txt`,
  `planet_view_80x30.txt` sudah disediakan sebagai **target awal**; sesuaikan saat UI nyata jadi
  (lewat `UPDATE_SNAPSHOTS=1`), tapi pastikan tetap lulus `visual_checks.md`.
