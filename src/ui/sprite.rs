//! Sprite celestial = ANSI half-block **berwarna** (`.ans`) → `ratatui` `Text` via `ansi-to-tui`.
//! Spec: `07-architecture.md` §Sprite Celestial, `06-ui.md`.
//!
//! Tiap base (`planet_*`, `star_sun`, `ship`) punya 9 varian `sprites/<base>/<size>.<depth>.ans`
//! (size ∈ {sm 20×10, md 40×20, lg 64×32} × depth ∈ {tc, 256, 16}). Runtime memilih **size** dari
//! luas `Rect` panel & **depth** dari kapabilitas terminal (deteksi sekali). `.ans` di-parse →
//! `Text` ber-`Style` lalu di-cache per `(base,size,depth)` (jangan parse ulang tiap frame) dan
//! digambar via `Paragraph`. File hilang/korup → turunkan depth lalu kotak teks fallback.
#![allow(dead_code)]

use ansi_to_tui::IntoText;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{Block, Paragraph};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

/// Ukuran sprite dalam sel: `lg` (Full), `md` (Compact), `sm` (Minimal).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SpriteSize {
    Sm,
    Md,
    Lg,
}

impl SpriteSize {
    /// Suffix nama file (`sm`/`md`/`lg`).
    pub fn suffix(self) -> &'static str {
        match self {
            SpriteSize::Sm => "sm",
            SpriteSize::Md => "md",
            SpriteSize::Lg => "lg",
        }
    }

    /// Box sel canonical (w, h): sm 20×10, md 40×20, lg 64×32.
    pub fn cells(self) -> (u16, u16) {
        match self {
            SpriteSize::Sm => (20, 10),
            SpriteSize::Md => (40, 20),
            SpriteSize::Lg => (64, 32),
        }
    }

    /// Pilih size terbesar yang muat di `area` (lg → md → sm). Gagal-aman ke `sm`.
    pub fn for_area(area: Rect) -> Self {
        for s in [SpriteSize::Lg, SpriteSize::Md] {
            let (w, h) = s.cells();
            if area.width >= w && area.height >= h {
                return s;
            }
        }
        SpriteSize::Sm
    }
}

/// Kedalaman warna terminal (turun bila aset depth lebih tinggi tak ada).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ColorDepth {
    Tc,
    C256,
    C16,
}

impl ColorDepth {
    /// Suffix nama file (`tc`/`256`/`16`).
    pub fn suffix(self) -> &'static str {
        match self {
            ColorDepth::Tc => "tc",
            ColorDepth::C256 => "256",
            ColorDepth::C16 => "16",
        }
    }

    /// Rantai fallback dari depth ini ke bawah (mis. `Tc → [Tc, C256, C16]`).
    fn chain(self) -> &'static [ColorDepth] {
        match self {
            ColorDepth::Tc => &[ColorDepth::Tc, ColorDepth::C256, ColorDepth::C16],
            ColorDepth::C256 => &[ColorDepth::C256, ColorDepth::C16],
            ColorDepth::C16 => &[ColorDepth::C16],
        }
    }

    /// Deteksi env: `COLORTERM=truecolor|24bit` → Tc; `TERM` ada `256` → C256; selain itu C16.
    pub fn detect() -> Self {
        if let Ok(ct) = std::env::var("COLORTERM") {
            let ct = ct.to_ascii_lowercase();
            if ct.contains("truecolor") || ct.contains("24bit") {
                return ColorDepth::Tc;
            }
        }
        match std::env::var("TERM") {
            Ok(t) if t.contains("256") => ColorDepth::C256,
            _ => ColorDepth::C16,
        }
    }
}

/// Cache sprite celestial: parse `.ans` → `Text` per `(base,size,depth)`, depth dipilih sekali.
/// Cache dibungkus `RefCell` agar `render` bisa dipanggil lewat `&App` (panel view tak butuh
/// `&mut App` cuma utk gambar sprite).
pub struct Sprites {
    root: PathBuf,
    depth: ColorDepth,
    cache: RefCell<HashMap<String, Text<'static>>>,
}

impl Sprites {
    /// `root` = folder `assets/sprites`; depth dideteksi dari terminal.
    pub fn new(root: PathBuf) -> Self {
        Sprites {
            root,
            depth: ColorDepth::detect(),
            cache: RefCell::new(HashMap::new()),
        }
    }

    /// Seperti [`new`], tapi depth eksplisit (deterministik untuk test/headless).
    pub fn with_depth(root: PathBuf, depth: ColorDepth) -> Self {
        Sprites {
            root,
            depth,
            cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn depth(&self) -> ColorDepth {
        self.depth
    }

    pub fn len(&self) -> usize {
        self.cache.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.borrow().is_empty()
    }

    pub fn clear(&self) {
        self.cache.borrow_mut().clear();
    }

    fn key(base: &str, size: SpriteSize, depth: ColorDepth) -> String {
        format!("{base}.{}.{}", size.suffix(), depth.suffix())
    }

    /// Parse `base`/`size` pada depth efektif (turun bila perlu). `None` bila semua varian gagal.
    fn ensure(&self, base: &str, size: SpriteSize) -> Option<Text<'static>> {
        let key = Self::key(base, size, self.depth);
        if let Some(t) = self.cache.borrow().get(&key) {
            return Some(t.clone());
        }
        let text = self.depth.chain().iter().find_map(|d| {
            let path = self
                .root
                .join(base)
                .join(format!("{}.{}.ans", size.suffix(), d.suffix()));
            let bytes = std::fs::read(&path).ok()?;
            bytes.into_text().ok()
        })?;
        self.cache.borrow_mut().insert(key, text.clone());
        Some(text)
    }

    /// Render sprite `base` ke `area`; size dipilih dari luas `area`. Kotak fallback bila gagal.
    pub fn render(&self, f: &mut Frame, area: Rect, base: &str) {
        let size = SpriteSize::for_area(area);
        match self.ensure(base, size) {
            Some(text) => f.render_widget(Paragraph::new(text), area),
            None => {
                let fb = Paragraph::new(format!("[sprite?\n {base}]"))
                    .block(Block::bordered())
                    .style(Style::default().dim());
                f.render_widget(fb, area);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn sprites_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/sprites")
    }

    #[test]
    fn size_for_area_picks_largest_fitting() {
        assert_eq!(
            SpriteSize::for_area(Rect::new(0, 0, 70, 40)),
            SpriteSize::Lg
        );
        assert_eq!(
            SpriteSize::for_area(Rect::new(0, 0, 50, 25)),
            SpriteSize::Md
        );
        assert_eq!(SpriteSize::for_area(Rect::new(0, 0, 10, 5)), SpriteSize::Sm);
    }

    #[test]
    fn depth_chain_descends() {
        assert_eq!(
            ColorDepth::C256.chain(),
            &[ColorDepth::C256, ColorDepth::C16]
        );
        assert_eq!(ColorDepth::C16.chain(), &[ColorDepth::C16]);
    }

    #[test]
    fn render_colored_sprite_fills_and_caches() {
        let s = Sprites::with_depth(sprites_root(), ColorDepth::Tc);
        let mut term = Terminal::new(TestBackend::new(64, 32)).unwrap();
        term.draw(|f| s.render(f, f.area(), "planet_ocean"))
            .unwrap();
        assert_eq!(s.len(), 1);
        let buf = term.backend().buffer();
        let mut colored = 0;
        for y in 0..32u16 {
            for x in 0..64u16 {
                if buf[(x, y)].fg != ratatui::style::Color::Reset {
                    colored += 1;
                }
            }
        }
        assert!(
            colored > 100,
            "sprite half-block harus berwarna, got {colored}"
        );
    }

    #[test]
    fn second_render_reuses_cache() {
        let s = Sprites::with_depth(sprites_root(), ColorDepth::Tc);
        let mut term = Terminal::new(TestBackend::new(64, 32)).unwrap();
        term.draw(|f| s.render(f, f.area(), "planet_lavaworld"))
            .unwrap();
        term.draw(|f| s.render(f, f.area(), "planet_lavaworld"))
            .unwrap();
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn missing_base_falls_back_without_caching() {
        let s = Sprites::with_depth(sprites_root(), ColorDepth::Tc);
        let mut term = Terminal::new(TestBackend::new(20, 10)).unwrap();
        term.draw(|f| s.render(f, f.area(), "planet_nope")).unwrap();
        assert!(s.is_empty());
    }

    #[test]
    fn depth_falls_back_when_variant_absent() {
        // semua base punya .16; minta Tc tetap dapat (chain turun ke yang ada).
        let s = Sprites::with_depth(sprites_root(), ColorDepth::Tc);
        let mut term = Terminal::new(TestBackend::new(20, 10)).unwrap();
        term.draw(|f| s.render(f, f.area(), "star_sun")).unwrap();
        assert_eq!(s.len(), 1);
    }
}
