//! Rasterizer `Buffer` → PNG (`image::RgbaImage`) untuk screenshot Phase 2.
//!
//! Tiap sel `Buffer` digambar sebagai satu blok `CELL_W × CELL_H` piksel: latar diisi warna bg
//! sel, lalu glyph (font monospace embedded, lihat `assets/fonts/`) dirender di atasnya dengan
//! warna fg. Box-drawing & half-block digambar programatik (M01.7/M01.8), bukan lewat glyph font.
//! Skeleton M01.3 — fungsi gambar (glyph/box/half-block) ditambah M01.5+.
#![allow(dead_code)]

use crate::ui::theme::Theme;
use ab_glyph::{Font as _, FontRef, PxScale, ScaleFont as _, point};
use image::{Rgba, RgbaImage};
use ratatui::buffer::Buffer;
use ratatui::style::Color;
use std::sync::OnceLock;

/// Lebar satu sel dalam piksel (font monospace ~1:2 rasio lebar:tinggi).
pub const CELL_W: u32 = 10;
/// Tinggi satu sel dalam piksel.
pub const CELL_H: u32 = 20;

/// Ukuran font dalam piksel (sedikit lebih kecil dari `CELL_H` agar ascender/descender tak
/// terpotong tepi sel). Nilai final disetel lewat pass visual M02/M09.
const FONT_SCALE_PX: f32 = 18.0;

/// Bytes TTF embedded (lihat `assets/fonts/LICENSE.md`).
const FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/DejaVuSansMono.ttf");

/// Font monospace dipakai rasterizer, dimuat sekali (embedded, tak pernah gagal di runtime
/// karena divalidasi lewat unit test `font_loads`).
pub fn font() -> &'static FontRef<'static> {
    static FONT: OnceLock<FontRef<'static>> = OnceLock::new();
    FONT.get_or_init(|| {
        FontRef::try_from_slice(FONT_BYTES).expect("assets/fonts/DejaVuSansMono.ttf harus valid")
    })
}

/// Rasterisasi `Buffer` penuh → `RgbaImage` berukuran `width*CELL_W × height*CELL_H`. Tiap sel
/// dipetakan lewat `draw_glyph` (yang otomatis mendelegasikan ke half-block/box-drawing bila
/// perlu). Karakter lebar-ganda/multi-`char` (mis. emoji, CJK) aman diambil char pertama saja —
/// sesuai kesepakatan `buffer_to_text` (fallback 1 sel, M01.10 menegaskan tak overflow tetangga).
pub fn buffer_to_rgba(buf: &Buffer, theme: &Theme) -> RgbaImage {
    let area = buf.area();
    let mut img = RgbaImage::new(area.width as u32 * CELL_W, area.height as u32 * CELL_H);
    for row in 0..area.height {
        for col in 0..area.width {
            let cell = &buf[(col, row)];
            let ch = cell.symbol().chars().next().unwrap_or(' ');
            let fg = color_to_rgb(cell.fg, theme, true);
            let bg = color_to_rgb(cell.bg, theme, false);
            draw_glyph(&mut img, ch, col as u32, row as u32, fg, bg);
        }
    }
    img
}

/// Isi seluruh sel `(col,row)` dengan warna bg (opaque), lalu raster glyph `ch` di atasnya
/// berwarna fg (anti-alias via coverage `ab_glyph`). Half-block `▀▄█` digambar programatik
/// (blok solid, bukan glyph font — lihat `draw_halfblock`). Glyph tanpa outline (spasi,
/// `.notdef`, atau simbol tak tersedia di font) aman diabaikan — sel tetap terisi bg.
pub fn draw_glyph(img: &mut RgbaImage, ch: char, col: u32, row: u32, fg: [u8; 3], bg: [u8; 3]) {
    let x0 = col * CELL_W;
    let y0 = row * CELL_H;

    if draw_halfblock(img, ch, x0, y0, fg, bg) {
        return;
    }
    if draw_boxdrawing(img, ch, x0, y0, fg, bg) {
        return;
    }
    fill_cell(img, x0, y0, bg);

    let f = font();
    let glyph_id = f.glyph_id(ch);
    if ch == ' ' || glyph_id.0 == 0 {
        return;
    }

    let scaled = f.as_scaled(PxScale::from(FONT_SCALE_PX));
    let baseline = point(x0 as f32, y0 as f32 + scaled.ascent());
    let glyph = glyph_id.with_scale_and_position(FONT_SCALE_PX, baseline);
    let Some(outlined) = f.outline_glyph(glyph) else {
        return;
    };
    let bounds = outlined.px_bounds();
    // Klip ketat ke kotak sel sendiri: glyph lebar/miring/di luar metrik normal (font apa pun,
    // termasuk substitusi) tak boleh menumpahkan ink ke sel tetangga.
    let (cell_x0, cell_y0) = (x0 as i32, y0 as i32);
    let (cell_x1, cell_y1) = ((x0 + CELL_W) as i32, (y0 + CELL_H) as i32);
    outlined.draw(|gx, gy, coverage| {
        let px = bounds.min.x as i32 + gx as i32;
        let py = bounds.min.y as i32 + gy as i32;
        if px < cell_x0 || py < cell_y0 || px >= cell_x1 || py >= cell_y1 {
            return;
        }
        blend_fg(img, px as u32, py as u32, fg, coverage);
    });
}

/// Gambar half-block `▀` (atas)/`▄` (bawah)/`█` (penuh) sbg blok solid pixel-perfect (bukan
/// glyph font — DejaVu punya glyph block tapi kita gambar sendiri agar batas piksel presisi
/// & konsisten lintas ukuran sel). Return `true` bila `ch` ditangani di sini.
fn draw_halfblock(
    img: &mut RgbaImage,
    ch: char,
    x0: u32,
    y0: u32,
    fg: [u8; 3],
    bg: [u8; 3],
) -> bool {
    match ch {
        '█' => {
            fill_rect(img, x0, y0, CELL_W, CELL_H, fg);
            true
        }
        '▀' => {
            let top_h = CELL_H / 2;
            fill_rect(img, x0, y0, CELL_W, top_h, fg);
            fill_rect(img, x0, y0 + top_h, CELL_W, CELL_H - top_h, bg);
            true
        }
        '▄' => {
            let top_h = CELL_H / 2;
            fill_rect(img, x0, y0, CELL_W, top_h, bg);
            fill_rect(img, x0, y0 + top_h, CELL_W, CELL_H - top_h, fg);
            true
        }
        _ => false,
    }
}

/// Tebal garis box-drawing dalam piksel.
const LINE_W: u32 = 2;

/// Gambar box-drawing dasar (`│─┌┐└┘├┤┬┴┼╴╶`) sbg garis tipis pixel-perfect yang bertemu presisi
/// di tengah sel — bukan lewat glyph font (sama alasan dgn `draw_halfblock`: sambungan garis
/// antar-sel harus presisi piksel & konsisten di semua ukuran, tak bisa diandalkan dari hinting
/// font). Return `true` bila `ch` ditangani di sini.
fn draw_boxdrawing(
    img: &mut RgbaImage,
    ch: char,
    x0: u32,
    y0: u32,
    fg: [u8; 3],
    bg: [u8; 3],
) -> bool {
    // (atas, bawah, kiri, kanan): sisi mana yang punya segmen garis dari tengah sel.
    let (up, down, left, right) = match ch {
        '│' => (true, true, false, false),
        '─' => (false, false, true, true),
        '┌' => (false, true, false, true),
        '┐' => (false, true, true, false),
        '└' => (true, false, false, true),
        '┘' => (true, false, true, false),
        '├' => (true, true, false, true),
        '┤' => (true, true, true, false),
        '┬' => (false, true, true, true),
        '┴' => (true, false, true, true),
        '┼' => (true, true, true, true),
        '╴' => (false, false, true, false),
        '╶' => (false, false, false, true),
        _ => return false,
    };

    fill_cell(img, x0, y0, bg);
    let cx = CELL_W / 2;
    let cy = CELL_H / 2;
    let half_thick = LINE_W / 2;
    let vx = x0 + cx.saturating_sub(half_thick);
    let hy = y0 + cy.saturating_sub(half_thick);

    if up {
        fill_rect(img, vx, y0, LINE_W, cy, fg);
    }
    if down {
        fill_rect(img, vx, y0 + cy, LINE_W, CELL_H - cy, fg);
    }
    if left {
        fill_rect(img, x0, hy, cx, LINE_W, fg);
    }
    if right {
        fill_rect(img, x0 + cx, hy, CELL_W - cx, LINE_W, fg);
    }
    true
}

fn fill_cell(img: &mut RgbaImage, x0: u32, y0: u32, color: [u8; 3]) {
    fill_rect(img, x0, y0, CELL_W, CELL_H, color);
}

fn fill_rect(img: &mut RgbaImage, x0: u32, y0: u32, w: u32, h: u32, color: [u8; 3]) {
    let x1 = (x0 + w).min(img.width());
    let y1 = (y0 + h).min(img.height());
    for y in y0.min(img.height())..y1 {
        for x in x0.min(img.width())..x1 {
            img.put_pixel(x, y, Rgba([color[0], color[1], color[2], 255]));
        }
    }
}

/// Blend pixel fg ke atas warna sudah ada (bg) dengan alpha `coverage` (0.0..=1.0).
fn blend_fg(img: &mut RgbaImage, x: u32, y: u32, fg: [u8; 3], coverage: f32) {
    if x >= img.width() || y >= img.height() || coverage <= 0.0 {
        return;
    }
    let c = coverage.clamp(0.0, 1.0);
    let under = *img.get_pixel(x, y);
    let mix = |bgc: u8, fgc: u8| -> u8 { (bgc as f32 * (1.0 - c) + fgc as f32 * c).round() as u8 };
    img.put_pixel(
        x,
        y,
        Rgba([
            mix(under[0], fg[0]),
            mix(under[1], fg[1]),
            mix(under[2], fg[2]),
            255,
        ]),
    );
}

/// Ubah `ratatui::Color` → RGB piksel nyata.
///
/// `is_fg` menentukan resolusi `Color::Reset`: fg → `theme.text`, bg → `theme.bg`.
/// `Rgb`/`Indexed` truecolor & 256-warna diteruskan langsung; 16 warna bernama dipetakan ke
/// palet ANSI standar (perkiraan umum — rendering terminal nyata bisa beda tema, lihat
/// `PROGRESS.md` §Catatan).
pub fn color_to_rgb(color: Color, theme: &Theme, is_fg: bool) -> [u8; 3] {
    match color {
        Color::Reset => color_to_rgb(if is_fg { theme.text } else { theme.bg }, theme, is_fg),
        Color::Rgb(r, g, b) => [r, g, b],
        Color::Indexed(i) => indexed_to_rgb(i),
        named => named16_to_rgb(named),
    }
}

/// Palet 16-warna ANSI standar (VGA/xterm klasik).
fn named16_to_rgb(color: Color) -> [u8; 3] {
    match color {
        Color::Black => [0x00, 0x00, 0x00],
        Color::Red => [0x80, 0x00, 0x00],
        Color::Green => [0x00, 0x80, 0x00],
        Color::Yellow => [0x80, 0x80, 0x00],
        Color::Blue => [0x00, 0x00, 0x80],
        Color::Magenta => [0x80, 0x00, 0x80],
        Color::Cyan => [0x00, 0x80, 0x80],
        Color::Gray => [0xc0, 0xc0, 0xc0],
        Color::DarkGray => [0x80, 0x80, 0x80],
        Color::LightRed => [0xff, 0x00, 0x00],
        Color::LightGreen => [0x00, 0xff, 0x00],
        Color::LightYellow => [0xff, 0xff, 0x00],
        Color::LightBlue => [0x00, 0x00, 0xff],
        Color::LightMagenta => [0xff, 0x00, 0xff],
        Color::LightCyan => [0x00, 0xff, 0xff],
        Color::White => [0xff, 0xff, 0xff],
        // Reset/Rgb/Indexed tak sampai sini (ditangani di color_to_rgb); fallback aman.
        _ => [0xff, 0xff, 0xff],
    }
}

/// Palet 256-warna xterm: 0–15 dasar, 16–231 kubus 6×6×6, 232–255 grayscale.
fn indexed_to_rgb(i: u8) -> [u8; 3] {
    const BASE16: [Color; 16] = [
        Color::Black,
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::Gray,
        Color::DarkGray,
        Color::LightRed,
        Color::LightGreen,
        Color::LightYellow,
        Color::LightBlue,
        Color::LightMagenta,
        Color::LightCyan,
        Color::White,
    ];
    if i < 16 {
        return named16_to_rgb(BASE16[i as usize]);
    }
    if i >= 232 {
        let level = 8 + (i - 232) as u32 * 10;
        return [level as u8, level as u8, level as u8];
    }
    let cube = i - 16;
    let r = cube / 36;
    let g = (cube / 6) % 6;
    let b = cube % 6;
    let level = |v: u8| if v == 0 { 0u8 } else { 55 + 40 * v };
    [level(r), level(g), level(b)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::state::ThemeChoice;
    use crate::ui::theme;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::style::Style;
    use ratatui::widgets::Paragraph;

    #[test]
    fn font_loads_with_sane_metrics() {
        let f = font();
        assert!(f.units_per_em().is_some());
        assert!(f.ascent_unscaled() > 0.0);
        // 'A' harus punya outline nyata (bukan glyph 0 / .notdef).
        let gid = f.glyph_id('A');
        assert_ne!(gid.0, 0, "font harus punya glyph utk 'A'");
    }

    #[test]
    fn font_load_is_cached_across_calls() {
        let a = font() as *const FontRef<'static>;
        let b = font() as *const FontRef<'static>;
        assert_eq!(
            a, b,
            "font() harus mengembalikan instance yang sama (OnceLock)"
        );
    }

    #[test]
    fn cell_size_matches_expected_monospace_ratio() {
        assert_eq!(CELL_W, 10);
        assert_eq!(CELL_H, 20);
    }

    #[test]
    fn rgb_passes_through_unchanged() {
        let t = theme::theme(ThemeChoice::Default);
        assert_eq!(color_to_rgb(Color::Rgb(12, 34, 56), &t, true), [12, 34, 56]);
    }

    #[test]
    fn reset_resolves_to_theme_text_or_bg() {
        let t = theme::theme(ThemeChoice::Default);
        assert_eq!(
            color_to_rgb(Color::Reset, &t, true),
            color_to_rgb(t.text, &t, true)
        );
        assert_eq!(
            color_to_rgb(Color::Reset, &t, false),
            color_to_rgb(t.bg, &t, false)
        );
    }

    #[test]
    fn indexed_base16_matches_named() {
        let t = theme::theme(ThemeChoice::Default);
        assert_eq!(
            color_to_rgb(Color::Indexed(1), &t, true),
            color_to_rgb(Color::Red, &t, true)
        );
        assert_eq!(
            color_to_rgb(Color::Indexed(15), &t, true),
            color_to_rgb(Color::White, &t, true)
        );
    }

    #[test]
    fn indexed_cube_extremes() {
        let t = theme::theme(ThemeChoice::Default);
        // 16 = kubus (0,0,0) = hitam; 231 = kubus (5,5,5) = putih.
        assert_eq!(color_to_rgb(Color::Indexed(16), &t, true), [0, 0, 0]);
        assert_eq!(color_to_rgb(Color::Indexed(231), &t, true), [255, 255, 255]);
    }

    #[test]
    fn indexed_grayscale_ramp() {
        let t = theme::theme(ThemeChoice::Default);
        assert_eq!(color_to_rgb(Color::Indexed(232), &t, true), [8, 8, 8]);
        assert_eq!(color_to_rgb(Color::Indexed(255), &t, true), [238, 238, 238]);
    }

    #[test]
    fn draw_glyph_fills_cell_with_bg() {
        let mut img = RgbaImage::new(CELL_W, CELL_H);
        draw_glyph(&mut img, ' ', 0, 0, [255, 255, 255], [10, 20, 30]);
        // Spasi: sel terisi bg penuh, tak ada glyph ink sama sekali.
        for y in 0..CELL_H {
            for x in 0..CELL_W {
                assert_eq!(*img.get_pixel(x, y), Rgba([10, 20, 30, 255]));
            }
        }
    }

    #[test]
    fn draw_glyph_paints_ink_for_visible_char() {
        let mut img = RgbaImage::new(CELL_W, CELL_H);
        draw_glyph(&mut img, 'A', 0, 0, [255, 255, 255], [0, 0, 0]);
        // Sel bukan spasi → ada minimal 1 piksel condong ke fg (coverage>0 di suatu titik).
        let has_ink = img.pixels().any(|p| p[0] > 0 || p[1] > 0 || p[2] > 0);
        assert!(has_ink, "glyph 'A' harus meninggalkan jejak fg di sel");
    }

    #[test]
    fn draw_glyph_places_cell_at_col_row_offset() {
        let mut img = RgbaImage::new(CELL_W * 2, CELL_H * 2);
        draw_glyph(&mut img, ' ', 1, 1, [0, 0, 0], [77, 88, 99]);
        // Sel (0,0) tak tersentuh; sel (1,1) terisi bg baru.
        assert_eq!(*img.get_pixel(0, 0), Rgba([0, 0, 0, 0]));
        assert_eq!(*img.get_pixel(CELL_W, CELL_H), Rgba([77, 88, 99, 255]));
    }

    #[test]
    fn draw_glyph_unknown_char_falls_back_to_bg_only() {
        let mut img = RgbaImage::new(CELL_W, CELL_H);
        // Karakter privat-use jarang tersedia di font apa pun → aman, tak panic.
        draw_glyph(&mut img, '\u{E000}', 0, 0, [255, 255, 255], [1, 2, 3]);
        assert_eq!(*img.get_pixel(0, 0), Rgba([1, 2, 3, 255]));
    }

    #[test]
    fn full_block_fills_whole_cell_with_fg() {
        let mut img = RgbaImage::new(CELL_W, CELL_H);
        draw_glyph(&mut img, '█', 0, 0, [200, 100, 50], [0, 0, 0]);
        for y in 0..CELL_H {
            for x in 0..CELL_W {
                assert_eq!(*img.get_pixel(x, y), Rgba([200, 100, 50, 255]));
            }
        }
    }

    #[test]
    fn upper_half_block_top_fg_bottom_bg() {
        let mut img = RgbaImage::new(CELL_W, CELL_H);
        draw_glyph(&mut img, '▀', 0, 0, [255, 0, 0], [0, 255, 0]);
        let top_h = CELL_H / 2;
        assert_eq!(*img.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(*img.get_pixel(0, top_h - 1), Rgba([255, 0, 0, 255]));
        assert_eq!(*img.get_pixel(0, top_h), Rgba([0, 255, 0, 255]));
        assert_eq!(*img.get_pixel(0, CELL_H - 1), Rgba([0, 255, 0, 255]));
    }

    #[test]
    fn lower_half_block_top_bg_bottom_fg() {
        let mut img = RgbaImage::new(CELL_W, CELL_H);
        draw_glyph(&mut img, '▄', 0, 0, [255, 0, 0], [0, 255, 0]);
        let top_h = CELL_H / 2;
        assert_eq!(*img.get_pixel(0, 0), Rgba([0, 255, 0, 255]));
        assert_eq!(*img.get_pixel(0, top_h - 1), Rgba([0, 255, 0, 255]));
        assert_eq!(*img.get_pixel(0, top_h), Rgba([255, 0, 0, 255]));
        assert_eq!(*img.get_pixel(0, CELL_H - 1), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn halfblock_covers_every_row_no_gap() {
        // Pixel-perfect: setiap baris sel harus terisi salah satu warna (tak ada baris kosong).
        let mut img = RgbaImage::new(CELL_W, CELL_H);
        draw_glyph(&mut img, '▀', 0, 0, [9, 9, 9], [1, 1, 1]);
        for y in 0..CELL_H {
            let p = *img.get_pixel(0, y);
            assert!(p == Rgba([9, 9, 9, 255]) || p == Rgba([1, 1, 1, 255]));
        }
    }

    #[test]
    fn box_cross_draws_both_axes_but_leaves_corners_bg() {
        let mut img = RgbaImage::new(CELL_W, CELL_H);
        draw_glyph(&mut img, '┼', 0, 0, [255, 255, 255], [0, 0, 0]);
        let fgpx = Rgba([255, 255, 255, 255]);
        let bgpx = Rgba([0, 0, 0, 255]);
        // Garis vertikal & horizontal lewat tengah sel (col~4-5, row~9-10).
        assert_eq!(*img.get_pixel(4, 0), fgpx);
        assert_eq!(*img.get_pixel(4, CELL_H - 1), fgpx);
        assert_eq!(*img.get_pixel(0, 9), fgpx);
        assert_eq!(*img.get_pixel(CELL_W - 1, 9), fgpx);
        // Sudut sel (jauh dari garis) tetap bg — `┼` tak punya diagonal.
        assert_eq!(*img.get_pixel(0, 0), bgpx);
        assert_eq!(*img.get_pixel(CELL_W - 1, 0), bgpx);
    }

    #[test]
    fn box_corner_top_left_only_draws_down_and_right() {
        let mut img = RgbaImage::new(CELL_W, CELL_H);
        draw_glyph(&mut img, '┌', 0, 0, [255, 255, 255], [0, 0, 0]);
        let fgpx = Rgba([255, 255, 255, 255]);
        let bgpx = Rgba([0, 0, 0, 255]);
        // Tak ada segmen ke atas/kiri → pojok kiri-atas tetap bg.
        assert_eq!(*img.get_pixel(0, 0), bgpx);
        // Segmen turun (bawah tengah) & kanan (tengah kanan) terisi fg.
        assert_eq!(*img.get_pixel(4, CELL_H - 1), fgpx);
        assert_eq!(*img.get_pixel(CELL_W - 1, 9), fgpx);
    }

    #[test]
    fn box_horizontal_line_spans_full_width_one_row() {
        let mut img = RgbaImage::new(CELL_W, CELL_H);
        draw_glyph(&mut img, '─', 0, 0, [255, 255, 255], [0, 0, 0]);
        for x in 0..CELL_W {
            assert_eq!(*img.get_pixel(x, 9), Rgba([255, 255, 255, 255]));
        }
        assert_eq!(*img.get_pixel(0, 0), Rgba([0, 0, 0, 255]));
    }

    #[test]
    fn buffer_to_rgba_has_expected_dimensions() {
        let term = Terminal::new(TestBackend::new(3, 2)).unwrap();
        let t = theme::theme(ThemeChoice::Default);
        let img = buffer_to_rgba(term.backend().buffer(), &t);
        assert_eq!(img.width(), 3 * CELL_W);
        assert_eq!(img.height(), 2 * CELL_H);
    }

    #[test]
    fn buffer_to_rgba_uses_cell_colors_at_right_cell_offset() {
        let mut term = Terminal::new(TestBackend::new(4, 2)).unwrap();
        term.draw(|f| {
            let p = Paragraph::new("X").style(
                Style::default()
                    .fg(Color::Rgb(200, 50, 10))
                    .bg(Color::Rgb(1, 2, 3)),
            );
            f.render_widget(p, f.area());
        })
        .unwrap();
        let t = theme::theme(ThemeChoice::Default);
        let img = buffer_to_rgba(term.backend().buffer(), &t);
        // Sel (0,0) = 'X' → pojok sel tetap bg custom (glyph ink di tengah, bukan di pojok).
        assert_eq!(*img.get_pixel(0, 0), Rgba([1, 2, 3, 255]));
        // Sel (1,0) kosong tapi bg style Paragraph tetap terisi ke seluruh area.
        assert_eq!(*img.get_pixel(CELL_W, 0), Rgba([1, 2, 3, 255]));
        // Sel (0,1) baris kedua → offset y benar (bukan hasil tumpang-tindih baris 0).
        assert_eq!(*img.get_pixel(0, CELL_H), Rgba([1, 2, 3, 255]));
    }

    /// M01.11 — Buffer kecil berisi ketiga jenis konten sekaligus (teks, box-drawing, half-block)
    /// dalam 1 render, verifikasi dimensi & warna tiap jenis sel benar. Menutup kriteria
    /// "Selesai bila" milestone M01.
    #[test]
    fn buffer_to_rgba_renders_text_box_and_halfblock_together() {
        use ratatui::layout::Rect;

        let area = Rect::new(0, 0, 3, 1);
        let mut buf = Buffer::empty(area);
        // Sel 0: teks 'A' fg merah / bg biru. Sel 1: box-drawing '┼' fg hijau / bg hitam.
        // Sel 2: half-block '█' fg kuning / bg tak relevan (penuh fg).
        buf[(0, 0)]
            .set_symbol("A")
            .set_fg(Color::Rgb(255, 0, 0))
            .set_bg(Color::Rgb(0, 0, 255));
        buf[(1, 0)]
            .set_symbol("┼")
            .set_fg(Color::Rgb(0, 255, 0))
            .set_bg(Color::Rgb(0, 0, 0));
        buf[(2, 0)]
            .set_symbol("█")
            .set_fg(Color::Rgb(255, 255, 0))
            .set_bg(Color::Rgb(0, 0, 0));

        let t = theme::theme(ThemeChoice::Default);
        let img = buffer_to_rgba(&buf, &t);

        // Dimensi: 3 sel lebar × 1 sel tinggi.
        assert_eq!(img.width(), 3 * CELL_W);
        assert_eq!(img.height(), CELL_H);

        // Sel 0 ('A'): pojok sel = bg biru murni (glyph ink di tengah, bukan pojok).
        assert_eq!(*img.get_pixel(0, 0), Rgba([0, 0, 255, 255]));
        // Sel 1 ('┼'): garis vertikal tengah (col lokal ~4-5 → global CELL_W+4) = fg hijau.
        assert_eq!(*img.get_pixel(CELL_W + 4, 0), Rgba([0, 255, 0, 255]));
        // Sel 1 pojok (jauh dari garis silang) tetap bg hitam.
        assert_eq!(*img.get_pixel(CELL_W, 0), Rgba([0, 0, 0, 255]));
        // Sel 2 ('█'): seluruh sel fg kuning solid, termasuk pojok.
        assert_eq!(*img.get_pixel(2 * CELL_W, 0), Rgba([255, 255, 0, 255]));
        assert_eq!(
            *img.get_pixel(2 * CELL_W + CELL_W - 1, CELL_H - 1),
            Rgba([255, 255, 0, 255])
        );
    }

    #[test]
    fn draw_glyph_ink_never_bleeds_into_neighbor_cell() {
        // Sel kiri diisi karakter lebar ('W' — advance mendekati batas sel di font monospace);
        // sel kanan wajib tetap murni bg-nya sendiri, tanpa ink dari sel kiri sedikit pun.
        let mut img = RgbaImage::new(CELL_W * 2, CELL_H);
        draw_glyph(&mut img, 'W', 0, 0, [255, 0, 0], [0, 0, 0]);
        draw_glyph(&mut img, ' ', 1, 0, [255, 0, 0], [10, 20, 30]);
        for y in 0..CELL_H {
            for x in CELL_W..(CELL_W * 2) {
                assert_eq!(*img.get_pixel(x, y), Rgba([10, 20, 30, 255]));
            }
        }
    }

    #[test]
    fn draw_glyph_unknown_wide_char_stays_within_own_cell() {
        // Karakter CJK (tak ada di DejaVu Sans Mono → glyph_id 0, fallback bg) tetap tak
        // menyentuh sel tetangga sama sekali.
        let mut img = RgbaImage::new(CELL_W * 2, CELL_H);
        draw_glyph(&mut img, '中', 0, 0, [255, 255, 255], [7, 7, 7]);
        draw_glyph(&mut img, ' ', 1, 0, [255, 255, 255], [8, 8, 8]);
        for y in 0..CELL_H {
            for x in 0..CELL_W {
                assert_eq!(*img.get_pixel(x, y), Rgba([7, 7, 7, 255]));
            }
            for x in CELL_W..(CELL_W * 2) {
                assert_eq!(*img.get_pixel(x, y), Rgba([8, 8, 8, 255]));
            }
        }
    }
}
