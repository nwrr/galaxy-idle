//! Galaxy pixel via terminal grafis (Sixel/Kitty/iTerm2), fallback ke backdrop procedural
//! (`galaxy_anim.rs`) bila tak didukung. Spec: `07-architecture.md` §Sprite, M17 (`D-galaxy`).
//!
//! Pola cache+detect SAMA `portrait.rs` (dulu tak ada modul galaxy pixel sama sekali — M17.1
//! menutup gap ini). `Picker::protocol_type()` dipakai membedakan protokol PIKSEL ASLI
//! (Sixel/Kitty/iTerm2) vs `Halfblocks` (fallback teks, bukan grafis genuine) — dasar keputusan
//! "tampilkan galaxy PNG" vs "pakai backdrop procedural" (M17.3).
#![allow(dead_code)]

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui_image::StatefulImage;
use ratatui_image::picker::{Picker, ProtocolType};
use ratatui_image::protocol::StatefulProtocol;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

/// Path `assets/source/galaxy/galaxy.png` absolut (robust terlepas dari cwd proses, pola sama
/// `app.rs`'s `sprites_root()`).
pub fn galaxy_png_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/source/galaxy/galaxy.png")
}

/// Deteksi dukungan grafis terminal utk galaxy pixel. Dibungkus (bukan pakai `Picker` langsung)
/// biar `supports_pixel()` jadi satu titik keputusan yg jelas & testable.
///
/// M17.2: "Load galaxy PNG -> StatefulImage, render di area" — `cache` (SATU slot, bukan
/// `HashMap` per-id spt `Portraits` — galaxy pixel cuma py SATU gambar sumber per sesi,
/// beda dari portrait yg banyak NPC) dibungkus `RefCell` biar `render()` bisa dipanggil via
/// `&self` (pola sama `Portraits`, panel view tak perlu `&mut App` cuma utk gambar galaxy).
pub struct GalaxyPixel {
    picker: Picker,
    cache: RefCell<Option<StatefulProtocol>>,
}

impl GalaxyPixel {
    /// Deteksi protokol terminal nyata (Sixel/Kitty/iTerm2); gagal/headless -> fallback half-block.
    pub fn detect() -> Self {
        let picker = Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks());
        GalaxyPixel {
            picker,
            cache: RefCell::new(None),
        }
    }

    /// Picker half-block deterministik (headless/test, atau terminal tanpa protokol grafis).
    pub fn halfblocks() -> Self {
        GalaxyPixel {
            picker: Picker::halfblocks(),
            cache: RefCell::new(None),
        }
    }

    /// `true` bila terminal genuinely mendukung protokol PIKSEL (Sixel/Kitty/iTerm2) -- BUKAN
    /// cuma `Halfblocks` (itu fallback teks/karakter, bukan grafis nyata biarpun sama2 lewat
    /// `ratatui-image`). Dipakai M17.3 memutuskan pixel vs backdrop procedural.
    pub fn supports_pixel(&self) -> bool {
        !matches!(self.picker.protocol_type(), ProtocolType::Halfblocks)
    }

    pub fn picker(&self) -> &Picker {
        &self.picker
    }

    /// Sudah ter-decode & ter-cache (tak perlu decode ulang tiap frame — M17.4).
    pub fn is_loaded(&self) -> bool {
        self.cache.borrow().is_some()
    }

    /// Pastikan galaxy PNG di `path` ter-decode (lazy, sekali). `false` bila gagal baca/decode.
    fn ensure(&self, path: &Path) -> bool {
        if self.cache.borrow().is_some() {
            return true;
        }
        let Some(img) = image::ImageReader::open(path)
            .ok()
            .and_then(|r| r.decode().ok())
        else {
            return false;
        };
        let proto = self.picker.new_resize_protocol(img);
        *self.cache.borrow_mut() = Some(proto);
        true
    }

    /// Render galaxy PNG di `path` ke `area` via `StatefulImage`. Decode+cache sekali (M17.4);
    /// `false` bila file hilang/korup (caller lakukan fallback ke backdrop procedural — M17.3).
    pub fn render(&self, f: &mut Frame, area: Rect, path: &Path) -> bool {
        if !self.ensure(path) {
            return false;
        }
        let mut cache = self.cache.borrow_mut();
        if let Some(proto) = cache.as_mut() {
            f.render_stateful_widget(StatefulImage::<StatefulProtocol>::default(), area, proto);
        }
        true
    }

    /// M17.3: "Fallback ke backdrop procedural bila tak didukung/headless" — **gap real
    /// ditemukan**: `render()` (M17.2) HANYA fallback saat file gagal dimuat, TAK PERNAH cek
    /// `supports_pixel()` — di terminal headless/tanpa protokol grafis (`Halfblocks`),
    /// `render()` MASIH akan mencoba gambar via half-block (ratatui-image bisa render
    /// Halfblocks jg, tp itu BUKAN "galaxy pixel" genuine, checklist minta backdrop PROCEDURAL
    /// [`galaxy_anim`] sbg fallback, bukan half-block image approximation). Fungsi ini
    /// SATU titik keputusan: pixel asli HANYA dicoba kalau `supports_pixel()` true DAN file
    /// genuinely berhasil dimuat; else `fallback` (closure caller, biasanya `GalaxyAnim::render`)
    /// dipanggil. `true` = pixel asli dipakai, `false` = fallback dipakai.
    pub fn render_or_fallback(
        &self,
        f: &mut Frame,
        area: Rect,
        path: &Path,
        fallback: impl FnOnce(&mut Frame, Rect),
    ) -> bool {
        if self.supports_pixel() && self.render(f, area, path) {
            return true;
        }
        fallback(f, area);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// M17.1: "Deteksi dukungan sixel/kitty via `ratatui-image` Picker" -- dites LANGSUNG:
    /// `halfblocks()` (headless/test genuine, tak py protokol grafis) HARUS `supports_pixel()
    /// == false` -- kalau lupa exclude Halfblocks dari deteksi, headless CI akan salah kira
    /// py dukungan pixel padahal cuma fallback teks.
    #[test]
    fn halfblocks_picker_reports_no_pixel_support() {
        let g = GalaxyPixel::halfblocks();
        assert!(
            !g.supports_pixel(),
            "Picker::halfblocks() HARUS dilaporkan TAK mendukung pixel asli (itu fallback \
             teks/karakter, bukan grafis nyata)"
        );
    }

    /// M17.1: `detect()` di lingkungan test/headless (tak py stdio TTY grafis nyata) HARUS
    /// genuinely jatuh ke fallback half-block (via `unwrap_or_else`), bukan panic/hang.
    #[test]
    fn detect_falls_back_safely_in_headless_test_env() {
        let g = GalaxyPixel::detect();
        // Tak diasumsikan hasilnya PASTI false (CI grafis eksotis teorinya bisa lolos), tp
        // pemanggilan HARUS aman (tak panic) -- itu inti gap M17.1 (dulu TAK ADA modul ini
        // sama sekali, jadi tak ada cara aman deteksi tanpa risiko panic di headless).
        let _ = g.supports_pixel();
    }

    fn galaxy_png_path() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/source/galaxy/galaxy.png")
    }

    /// M17.2: "Load galaxy PNG -> StatefulImage, render di area" -- dites LANGSUNG: render
    /// dari file ASLI (`assets/source/galaxy/galaxy.png`, sumber M08) harus genuinely isi
    /// buffer (bukan cuma "tak panic") + cache HARUS kepakai (M17.4, `is_loaded()` true
    /// stlh render pertama).
    #[test]
    fn halfblocks_render_loads_and_fills_buffer() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let g = GalaxyPixel::halfblocks();
        let path = galaxy_png_path();
        let mut term = Terminal::new(TestBackend::new(60, 24)).unwrap();
        let ok = {
            let mut result = false;
            term.draw(|f| result = g.render(f, f.area(), &path))
                .unwrap();
            result
        };
        assert!(
            ok,
            "render() harus berhasil utk file galaxy.png yg genuinely ada"
        );
        assert!(g.is_loaded(), "cache harus terisi stlh render pertama");

        let buf = term.backend().buffer();
        let non_space = (0..24u16)
            .flat_map(|y| (0..60u16).map(move |x| (x, y)))
            .filter(|&(x, y)| buf[(x, y)].symbol() != " ")
            .count();
        assert!(
            non_space > 100,
            "galaxy pixel half-block harus isi banyak sel, got {non_space}"
        );
    }

    /// M17.4: "Cache image protocol (no reload tiap frame)" -- render 2x HARUS tetap 1 decode
    /// (dibuktikan via `is_loaded()` tetap true & tak panic/re-decode kedua kalinya).
    #[test]
    fn second_render_reuses_cache_without_reload() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let g = GalaxyPixel::halfblocks();
        let path = galaxy_png_path();
        let mut term = Terminal::new(TestBackend::new(40, 16)).unwrap();
        term.draw(|f| {
            g.render(f, f.area(), &path);
        })
        .unwrap();
        assert!(g.is_loaded());
        term.draw(|f| {
            g.render(f, f.area(), &path);
        })
        .unwrap();
        assert!(
            g.is_loaded(),
            "cache tetap terisi (tak dibuang) stlh render kedua"
        );
    }

    /// M17.2/3: file hilang/korup HARUS `false` (bukan panic) -- caller (M17.3) pakai ini
    /// sbg sinyal fallback ke backdrop procedural.
    #[test]
    fn missing_file_returns_false_without_panic() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let g = GalaxyPixel::halfblocks();
        let mut term = Terminal::new(TestBackend::new(20, 10)).unwrap();
        let ok = {
            let mut result = true;
            term.draw(|f| {
                result = g.render(f, f.area(), Path::new("/no/such/galaxy.png"));
            })
            .unwrap();
            result
        };
        assert!(!ok, "file hilang HARUS genuinely return false, bukan panic");
        assert!(!g.is_loaded(), "cache tak boleh terisi kalau load gagal");
    }

    /// M17.3: `GalaxyPixel::halfblocks()` (`supports_pixel()==false`, pola SAMA lingkungan
    /// headless/CI nyata) HARUS genuinely panggil `fallback` (BUKAN coba render pixel dulu) —
    /// biarpun file PNG genuinely ADA & valid, krn terminal tak py protokol grafis ASLI.
    #[test]
    fn render_or_fallback_uses_fallback_when_pixel_unsupported_even_if_file_exists() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let g = GalaxyPixel::halfblocks();
        let path = galaxy_png_path();
        let mut term = Terminal::new(TestBackend::new(20, 10)).unwrap();
        let mut fallback_called = false;
        let used_pixel = {
            let mut result = true;
            term.draw(|f| {
                result = g.render_or_fallback(f, f.area(), &path, |_, _| {
                    fallback_called = true;
                });
            })
            .unwrap();
            result
        };
        assert!(
            !used_pixel,
            "supports_pixel()==false HARUS genuinely pakai fallback, bukan pixel"
        );
        assert!(
            fallback_called,
            "closure fallback HARUS genuinely dipanggil (bukan cuma return false tanpa render \
             apa pun)"
        );
        assert!(
            !g.is_loaded(),
            "file TAK BOLEH sempat di-load kalau supports_pixel() sudah false di awal"
        );
    }

    /// M17.3: file hilang/korup jg HARUS trigger fallback (bukan cuma soal dukungan terminal).
    #[test]
    fn render_or_fallback_uses_fallback_when_file_missing() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let g = GalaxyPixel::halfblocks();
        let mut term = Terminal::new(TestBackend::new(20, 10)).unwrap();
        let mut fallback_called = false;
        term.draw(|f| {
            g.render_or_fallback(f, f.area(), Path::new("/no/such/galaxy.png"), |_, _| {
                fallback_called = true;
            });
        })
        .unwrap();
        assert!(
            fallback_called,
            "file hilang HARUS genuinely trigger fallback"
        );
    }
}
