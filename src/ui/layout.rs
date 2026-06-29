//! Breakpoint responsif: pilih `LayoutMode` dari ukuran terminal (`06-ui.md` §Responsive).
//!
//! Full (sidebar + detail) ≥100×30 · Compact (sidebar tipis) ≥70w · Minimal (tab) sisanya.
//!
//! NOTE(scaffold): `allow(dead_code)` — dipakai `ui::draw` (M3 panels).
#![allow(dead_code)]

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayoutMode {
    Full,
    Compact,
    Minimal,
}

/// Tentukan mode dari lebar×tinggi terminal. Dipanggil tiap render (resize-aware).
pub fn layout_mode(width: u16, height: u16) -> LayoutMode {
    if width >= 100 && height >= 30 {
        LayoutMode::Full
    } else if width >= 70 {
        LayoutMode::Compact
    } else {
        LayoutMode::Minimal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breakpoints() {
        assert_eq!(layout_mode(120, 40), LayoutMode::Full);
        assert_eq!(layout_mode(100, 30), LayoutMode::Full);
        assert_eq!(layout_mode(99, 40), LayoutMode::Compact); // lebar < 100
        assert_eq!(layout_mode(120, 29), LayoutMode::Compact); // tinggi < 30
        assert_eq!(layout_mode(80, 30), LayoutMode::Compact);
        assert_eq!(layout_mode(70, 24), LayoutMode::Compact);
        assert_eq!(layout_mode(69, 40), LayoutMode::Minimal);
        assert_eq!(layout_mode(60, 24), LayoutMode::Minimal);
    }
}
