//! Portrait karakter (NPC merchant, alien encounter, crew) sebagai gambar PNG via `ratatui-image`.
//! Spec: `07-architecture.md` §Portrait, `06-ui.md` §Portrait Karakter.
//!
//! `Picker` (deteksi protokol terminal) dibuat sekali; tiap portrait di-decode → `StatefulProtocol`
//! di-**cache per id** (jangan decode ulang tiap frame) → digambar via `StatefulImage`. Terminal
//! dengan Sixel/Kitty/iTerm2 → gambar tajam; terminal biasa → fallback Unicode half-block (`▀`).
//! File hilang/korup → kotak teks fallback (tak crash). `clear()` membuang cache saat panel tutup.
#![allow(dead_code)]

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Paragraph};
use ratatui_image::StatefulImage;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use std::collections::HashMap;
use std::path::Path;

/// Cache portrait + picker protokol terminal.
pub struct Portraits {
    picker: Picker,
    cache: HashMap<String, StatefulProtocol>,
}

impl Portraits {
    /// Deteksi protokol terminal (Sixel/Kitty/iTerm2); gagal/headless → fallback half-block.
    pub fn detect() -> Self {
        let picker = Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks());
        Portraits {
            picker,
            cache: HashMap::new(),
        }
    }

    /// Picker half-block deterministik (headless/test, atau terminal tanpa protokol grafis).
    pub fn halfblocks() -> Self {
        Portraits {
            picker: Picker::halfblocks(),
            cache: HashMap::new(),
        }
    }

    /// Apakah id sudah ter-decode & ter-cache.
    pub fn has(&self, id: &str) -> bool {
        self.cache.contains_key(id)
    }

    /// Jumlah portrait dalam cache.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Buang semua protokol ter-cache (panggil saat panel portrait ditutup).
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Pastikan `id` ter-decode dari `path` (lazy). `false` bila gagal decode/baca.
    fn ensure(&mut self, id: &str, path: &Path) -> bool {
        if self.cache.contains_key(id) {
            return true;
        }
        let Some(img) = image::ImageReader::open(path)
            .ok()
            .and_then(|r| r.decode().ok())
        else {
            return false;
        };
        let proto = self.picker.new_resize_protocol(img);
        self.cache.insert(id.to_string(), proto);
        true
    }

    /// Render portrait `id` (`path`) ke `area`. Decode+cache sekali; kotak fallback bila gagal.
    pub fn render(&mut self, f: &mut Frame, area: Rect, id: &str, path: &Path) {
        if !self.ensure(id, path) {
            let fb = Paragraph::new(format!("[portrait?\n {id}]"))
                .block(Block::bordered().title(" NPC "))
                .style(Style::default().dim());
            f.render_widget(fb, area);
            return;
        }
        if let Some(proto) = self.cache.get_mut(id) {
            f.render_stateful_widget(StatefulImage::<StatefulProtocol>::default(), area, proto);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn portrait_path() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/characters/alien/char_alien_01.png")
    }

    #[test]
    fn halfblocks_render_fills_buffer_and_caches() {
        let mut p = Portraits::halfblocks();
        let path = portrait_path();
        let mut term = Terminal::new(TestBackend::new(44, 32)).unwrap();
        term.draw(|f| p.render(f, f.area(), "char_alien_01", &path))
            .unwrap();
        assert!(p.has("char_alien_01"));
        assert_eq!(p.len(), 1);
        let buf = term.backend().buffer();
        let mut non_space = 0;
        for y in 0..32u16 {
            for x in 0..44u16 {
                if buf[(x, y)].symbol() != " " {
                    non_space += 1;
                }
            }
        }
        assert!(
            non_space > 100,
            "half-block portrait harus isi banyak sel, got {non_space}"
        );
    }

    #[test]
    fn second_render_reuses_cache() {
        let mut p = Portraits::halfblocks();
        let path = portrait_path();
        let mut term = Terminal::new(TestBackend::new(20, 16)).unwrap();
        term.draw(|f| p.render(f, f.area(), "a", &path)).unwrap();
        term.draw(|f| p.render(f, f.area(), "a", &path)).unwrap();
        assert_eq!(p.len(), 1); // tetap satu entri (tak decode ulang)
    }

    #[test]
    fn missing_file_falls_back_without_caching() {
        let mut p = Portraits::halfblocks();
        let mut term = Terminal::new(TestBackend::new(20, 10)).unwrap();
        term.draw(|f| p.render(f, f.area(), "nope", Path::new("/no/such/portrait.png")))
            .unwrap();
        assert!(!p.has("nope"));
        assert!(p.is_empty());
    }

    #[test]
    fn clear_drops_cache() {
        let mut p = Portraits::halfblocks();
        let path = portrait_path();
        let mut term = Terminal::new(TestBackend::new(20, 16)).unwrap();
        term.draw(|f| p.render(f, f.area(), "a", &path)).unwrap();
        assert_eq!(p.len(), 1);
        p.clear();
        assert!(p.is_empty());
    }
}
