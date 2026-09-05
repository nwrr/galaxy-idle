//! UI glue utk `galaxy_sim`: half-block truecolor `PixelWidget` (port dr `andromeda-
//! simulation-tui`'s `src/ui/pixels.rs`) + `GalaxySimView` (cache per-galaxy model + backend +
//! throttle recompute, pola SAMA `GalaxyPixel`/`Sprites`/`Portraits`'s cache-in-`RefCell`-
//! behind-`&self`). Menggantikan `galaxy_anim.rs`(spiral prosedural)+`galaxy_pixel.rs`(PNG
//! statis via sixel) sbg backdrop utama (M20.8 follow-up plan Phase 5).

use std::cell::{Cell, RefCell};

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::Widget;

use crate::galaxy_sim::camera::{self, Camera};
use crate::galaxy_sim::model::{GalaxyModel, OrbitParams, PoiOrbitKey, Uniforms};
use crate::galaxy_sim::poi::{self, Poi};
use crate::galaxy_sim::render::{SimBackend, create_backend, probe};
use crate::galaxy_sim::spec::{GalaxySpec, spec_for_seed};
use crate::galaxy_sim::{generate, orbit};

/// Laju simulasi: Myr (juta tahun) per detik NYATA -- port dr reference's `MYR_PER_SEC`,
/// dipilih PELAN [rotasi galaksi terasa halus, bukan "muter cepat" tak masuk akal] shg
/// panggil `t_myr = anim_secs * MYR_PER_SEC` di call-site (`main_view.rs`/`galaxy_map.rs`).
pub const MYR_PER_SEC: f64 = 1.5;

/// Taburan bintang latar depan (screen-space, diam saat kamera bergerak) -- deterministik
/// thd `(x,y,seed)` shg stabil antar frame. Port near-verbatim dari reference `Starfield`.
struct Starfield {
    w: u32,
    h: u32,
    rgb: Vec<[u8; 3]>,
}

fn hash(x: u32, y: u32, seed: u64) -> u32 {
    let mut h = (x as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (y as u64 + 1).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ seed.wrapping_mul(0x1656_67B1_9E37_79F9);
    h ^= h >> 30;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^= h >> 31;
    h as u32
}

impl Starfield {
    fn generate(w: u32, h: u32, seed: u64) -> Self {
        let mut sf = Self {
            w,
            h,
            rgb: vec![[0; 3]; (w * h) as usize],
        };
        let tint = |warm: bool, b: u32| -> [u8; 3] {
            let b = b as f32 / 255.0;
            if warm {
                [(255.0 * b) as u8, (196.0 * b) as u8, (128.0 * b) as u8]
            } else {
                [(158.0 * b) as u8, (196.0 * b) as u8, (255.0 * b) as u8]
            }
        };
        // ~1 bintang redup / 600 piksel.
        for y in 0..h {
            for x in 0..w {
                let hsh = hash(x, y, seed);
                let p = hsh & 0xffff;
                if p < 109 {
                    let b = 36 + ((hsh >> 16) & 63);
                    let warm = ((hsh >> 22) & 15) < 11;
                    sf.set(x, y, tint(warm, b));
                }
            }
        }
        sf
    }

    fn idx(&self, x: i64, y: i64) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.w as i64 || y >= self.h as i64 {
            None
        } else {
            Some((y as u32 * self.w + x as u32) as usize)
        }
    }

    fn set(&mut self, x: u32, y: u32, c: [u8; 3]) {
        if let Some(i) = self.idx(x as i64, y as i64) {
            self.rgb[i] = c;
        }
    }

    fn get(&self, x: u32, y: u32) -> [u8; 3] {
        if x < self.w && y < self.h {
            self.rgb[(y * self.w + x) as usize]
        } else {
            [0; 3]
        }
    }
}

/// Renderer half-block: tiap sel terminal `▀` = 2 piksel vertikal (fg=piksel atas, bg=piksel
/// bawah), warna 24-bit. Port near-verbatim dari reference `PixelWidget` (tanpa `Marker`,
/// POI overlay menyusul Phase 7).
struct PixelWidget<'a> {
    rgba: &'a [u8],
    w: u32,
    h: u32,
    backdrop: &'a Starfield,
}

impl PixelWidget<'_> {
    fn pixel(&self, x: u32, y: u32) -> (u8, u8, u8) {
        if x >= self.w || y >= self.h {
            return (0, 0, 0);
        }
        let i = ((y * self.w + x) * 4) as usize;
        if i + 2 >= self.rgba.len() {
            return (0, 0, 0);
        }
        let s = self.backdrop.get(x, y);
        (
            self.rgba[i].saturating_add(s[0]),
            self.rgba[i + 1].saturating_add(s[1]),
            self.rgba[i + 2].saturating_add(s[2]),
        )
    }
}

impl Widget for PixelWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for cy in 0..area.height {
            for cx in 0..area.width {
                let (tr, tg, tb) = self.pixel(cx as u32, cy as u32 * 2);
                let (br, bg_, bb) = self.pixel(cx as u32, cy as u32 * 2 + 1);
                let cell = &mut buf[(area.x + cx, area.y + cy)];
                cell.set_char('▀');
                cell.set_fg(Color::Rgb(tr, tg, tb));
                cell.set_bg(Color::Rgb(br, bg_, bb));
            }
        }
    }
}

/// Isi cache: model+spec galaksi SAAT INI (dibangun sekali per (seed,is_anchor), dipakai
/// ulang lintas frame sampai galaksi berganti).
struct CachedGalaxy {
    seed: u64,
    is_anchor: bool,
    spec: GalaxySpec,
    model_anchors: [glam::Vec3; 3],
    /// Orbit POI notable (Phase 7) -- lookup posisi utk fokus/follow kamera.
    poi_orbits: Vec<(PoiOrbitKey, OrbitParams)>,
    /// Katalog nama POI (Phase 7) -- anchor pakai nama literal, frontier seed-driven.
    pois: Vec<Poi>,
}

/// Status fokus kamera (Phase 7) -- `Overview` = pan/zoom bebas user; `Poi(key)` = kamera
/// diam2 mengikuti orbit POI tsb tiap frame (`tick_camera`), pan manual di-nonaktifkan
/// sementara (SIA-SIA -- akan langsung ditimpa follow di frame berikutnya).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FocusState {
    Overview,
    Poi(PoiOrbitKey),
}

/// State per-view: backend (CPU/GPU) + cache model galaksi + throttle recompute + buffer
/// RGBA frame terakhir (dipakai ulang di antara recompute, biar tak render TIAP tick 75ms).
pub struct GalaxySimView {
    backend: RefCell<Box<dyn SimBackend>>,
    fallback_reason: Option<String>,
    cached: RefCell<Option<CachedGalaxy>>,
    /// Kamera interaktif (pan/zoom, Phase 6) -- direset ke overview SETIAP galaksi ganti
    /// (`ensure_galaxy`), pola SAMA reference's "reset zoom on first valid width".
    camera: Cell<Camera>,
    starfield: RefCell<Option<Starfield>>,
    last_frame: RefCell<Option<Vec<u8>>>,
    last_dims: Cell<(u32, u32)>,
    tick: Cell<u32>,
    /// Jumlah tick (masing2 ~75ms, `RENDER_INTERVAL_MS`) antar recompute genuine -- diam2
    /// pakai buffer cache di antaranya. Default 4 (~300ms, ~3.3Hz) -- ditala visual via
    /// screenshot, bukan ditebak (M20.8 follow-up plan Phase 5's catatan eksplisit).
    recompute_every: u32,
    /// Jumlah bintang per galaksi -- viewport TUI (~78x40 px half-block) JAUH lbh kecil drpd
    /// reference's layar desktop (400k bintang), skala turun proporsional.
    star_count: u32,
    /// Fokus kamera aktif (Phase 7) -- `Overview` default.
    focus: Cell<FocusState>,
    /// `t_myr` render TERAKHIR -- dipakai hitung `dt` (detik nyata) utk easing `tick_camera`
    /// (`approach_v2`/`CAM_TAU`), TANPA baca wall-clock langsung (tetap fungsi murni dr
    /// `t_myr` yg dioper caller, konsisten pola proyek ini).
    last_t_myr: Cell<f32>,
}

impl GalaxySimView {
    fn new(force_cpu: bool) -> Self {
        let (backend, fallback_reason) = create_backend(probe(force_cpu), Vec::new());
        Self {
            backend: RefCell::new(backend),
            fallback_reason,
            cached: RefCell::new(None),
            camera: Cell::new(Camera::new(1.0, 35.0, 15.0, 44.0)),
            starfield: RefCell::new(None),
            last_frame: RefCell::new(None),
            last_dims: Cell::new((0, 0)),
            tick: Cell::new(0),
            recompute_every: 4,
            star_count: 6_000,
            focus: Cell::new(FocusState::Overview),
            last_t_myr: Cell::new(0.0),
        }
    }

    /// Deterministik, tanpa I/O/deteksi GPU -- dipakai `App::demo()` (test/screenshot/snapshot).
    pub fn demo() -> Self {
        Self::new(true)
    }

    /// Deteksi GPU nyata -- dipakai `App::run()` sblm masuk event loop, pola SAMA
    /// `Sprites::detect`/`Portraits::detect`/`GalaxyPixel::detect`.
    pub fn detect() -> Self {
        Self::new(false)
    }

    /// Label backend aktif (mis. "GPU: NVIDIA ... (Vulkan)" / "CPU: rayon 8 threads").
    pub fn backend_name(&self) -> String {
        self.backend.borrow().name()
    }

    #[cfg(test)]
    pub(crate) fn camera_snapshot(&self) -> Camera {
        self.camera.get()
    }

    fn ensure_galaxy(&self, seed: u64, is_anchor: bool) {
        let needs_rebuild = match &*self.cached.borrow() {
            Some(c) => c.seed != seed || c.is_anchor != is_anchor,
            None => true,
        };
        if !needs_rebuild {
            return;
        }
        let spec = spec_for_seed(seed, is_anchor);
        let model: GalaxyModel = generate::generate(&spec, seed, self.star_count);
        self.backend.borrow_mut().set_stars(&model.stars);
        // Kamera direset ke overview galaksi BARU (pan/zoom galaksi LAMA tak berarti lg utk
        // galaksi ini -- span/inklinasi/PA beda per spec, pola SAMA reference's app.rs
        // "reset zoom on first valid width" saat pindah galaksi). Zoom FIT genuine [pakai
        // dims aktual] dihitung ulang di `render()`'s cabang `dims_changed` -- di sini msh
        // placeholder [dims mgkn blm diketahui sblm render pertama], TAK bisa fit di sini.
        let cam = Camera::new(
            1.0,
            spec.view.inclination_deg,
            spec.view.position_angle_deg,
            spec.view.span_kpc,
        );
        self.camera.set(cam);
        let pois = poi::catalog(seed, is_anchor);
        *self.cached.borrow_mut() = Some(CachedGalaxy {
            seed,
            is_anchor,
            spec,
            model_anchors: model.anchors,
            poi_orbits: model.poi_orbits,
            pois,
        });
        // Galaksi ganti -> fokus lama (kunci POI galaksi SEBELUMNYA) tak lg berarti -> reset
        // ke Overview (bukan diam2 "mengikuti" POI galaksi BARU yg kebetulan py kunci sama).
        self.focus.set(FocusState::Overview);
        // Galaksi ganti -> paksa recompute SEGERA (bukan tunggu cadence), biar tak nampilkan
        // buffer galaksi LAMA sesaat.
        self.tick.set(0);
        *self.last_frame.borrow_mut() = None;
    }

    /// Geser kamera `(dx,dy)` piksel layar -- dipakai key pan (`app.rs`'s `galaxy_map_keys`,
    /// scoped mode Backdrop saja). Paksa recompute SEGERA (bukan tunggu cadence throttle),
    /// biar pan genuinely terasa RESPONSIF, bukan lag ~300ms. No-op saat FOKUS POI aktif --
    /// `tick_camera` akan langsung menimpanya frame berikutnya, pan manual SIA-SIA/membingungkan.
    pub fn pan(&self, dx: f32, dy: f32) {
        if self.focus.get() != FocusState::Overview {
            return;
        }
        let mut cam = self.camera.get();
        cam.pan_px(dx, dy);
        self.camera.set(cam);
        self.tick.set(0);
    }

    /// Pindah fokus ke POI BERIKUTNYA dlm katalog (Overview -> POI[0] -> POI[1] -> ... ->
    /// Overview, siklus). Dipakai key `Enter` (`app.rs`, scoped mode Backdrop).
    pub fn focus_next(&self) {
        let cached = self.cached.borrow();
        let Some(c) = cached.as_ref() else { return };
        if c.pois.is_empty() {
            return;
        }
        let next = match self.focus.get() {
            FocusState::Overview => FocusState::Poi(c.pois[0].orbit_key),
            FocusState::Poi(key) => {
                let idx = c.pois.iter().position(|p| p.orbit_key == key).unwrap_or(0);
                if idx + 1 < c.pois.len() {
                    FocusState::Poi(c.pois[idx + 1].orbit_key)
                } else {
                    FocusState::Overview
                }
            }
        };
        drop(cached);
        self.focus.set(next);
        self.tick.set(0);
    }

    /// Kembali ke Overview (batalkan fokus POI) -- dipakai key `Esc` (`app.rs`, scoped mode
    /// Backdrop, HANYA saat sedang fokus -- `Esc` TANPA fokus tetap keluar view spt biasa).
    pub fn unfocus(&self) {
        self.focus.set(FocusState::Overview);
        self.tick.set(0);
    }

    /// `true` bila sedang fokus ke suatu POI (dipakai `app.rs` memutuskan `Esc` batalkan
    /// fokus vs keluar view; `galaxy_map.rs` menampilkan nama POI aktif di footer).
    pub fn is_focused(&self) -> bool {
        self.focus.get() != FocusState::Overview
    }

    /// Nama POI yg sedang difokus, bila ada (utk label UI, mis. footer "Focus: <name>").
    pub fn focused_poi_name(&self) -> Option<String> {
        let FocusState::Poi(key) = self.focus.get() else {
            return None;
        };
        let cached = self.cached.borrow();
        cached
            .as_ref()?
            .pois
            .iter()
            .find(|p| p.orbit_key == key)
            .map(|p| p.name.clone())
    }

    /// Gerakkan kamera menuju posisi POI yg difokus (view-plane, easing frame-rate-independent
    /// via `approach_v2`/`CAM_TAU`) -- no-op saat Overview. Dipanggil tiap `render()` (BUKAN
    /// cuma saat `due`/recompute -- easing HARUS mulus tiap tick UI, bukan tersendat ikut
    /// cadence recompute background).
    fn tick_camera(&self, dt: f32, t_myr: f32) {
        let FocusState::Poi(key) = self.focus.get() else {
            return;
        };
        let cached = self.cached.borrow();
        let Some(c) = cached.as_ref() else { return };
        let Some(o) = c
            .poi_orbits
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, o)| *o)
        else {
            return;
        };
        let pos = orbit::orbit_position(&o, t_myr, c.spec.phys.pattern_omega, &c.model_anchors);
        drop(cached);
        let mut cam = self.camera.get();
        let target = cam.view_plane(pos);
        cam.view_center = camera::approach_v2(cam.view_center, target, dt, camera::CAM_TAU);
        self.camera.set(cam);
    }

    /// Zoom kali `factor` (>1 = zoom IN, <1 = zoom OUT), clamp bawah kecil (hindari zoom ke
    /// nol/negatif -- genuinely aman di SEMUA nilai, bukan cuma "biasanya positif").
    pub fn zoom_by(&self, factor: f32) {
        let mut cam = self.camera.get();
        cam.zoom = (cam.zoom * factor).max(0.01);
        self.camera.set(cam);
        self.tick.set(0);
    }

    /// Reset kamera ke overview (zoom+pan default galaksi AKTIF) -- pola SAMA reference's
    /// key `r`.
    pub fn reset_view(&self) {
        let (pw, ph) = self.last_dims.get();
        if let Some(c) = self.cached.borrow().as_ref() {
            let mut cam = Camera::new(
                1.0,
                c.spec.view.inclination_deg,
                c.spec.view.position_angle_deg,
                c.spec.view.span_kpc,
            );
            if pw > 0 {
                cam.zoom = cam.overview_zoom(pw.min(ph) as f32);
            }
            self.camera.set(cam);
            self.tick.set(0);
        }
    }

    /// Render galaksi `(seed,is_anchor)` ke `area`, waktu simulasi `t_myr`. Redraw genuine
    /// HANYA tiap `recompute_every` panggilan (throttle) -- di antaranya pakai buffer cache.
    pub fn render(&self, f: &mut Frame, area: Rect, seed: u64, is_anchor: bool, t_myr: f64) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        self.ensure_galaxy(seed, is_anchor);
        let (pw, ph) = (area.width as u32, area.height as u32 * 2);
        let (old_pw, old_ph) = self.last_dims.get();
        let dims_changed = (old_pw, old_ph) != (pw, ph);
        if dims_changed {
            self.backend.borrow_mut().resize(pw, ph);
            self.last_dims.set((pw, ph));
            self.tick.set(0); // ukuran ganti -> paksa recompute segera.
            // Fit zoom overview HANYA saat ini genuinely render PERTAMA (dims lama 0x0) --
            // resize di TENGAH sesi (mis. terminal diperbesar) HARUS pertahankan pan/zoom
            // user, bukan diam2 reset ke overview (mengejutkan bila user SUDAH pan/zoom).
            if old_pw == 0 || old_ph == 0 {
                let mut cam = self.camera.get();
                cam.zoom = cam.overview_zoom(pw.min(ph) as f32);
                self.camera.set(cam);
            }
            *self.starfield.borrow_mut() = Some(Starfield::generate(pw, ph, seed));
        }

        let dt = (t_myr as f32 - self.last_t_myr.get()).max(0.0) / MYR_PER_SEC as f32;
        self.last_t_myr.set(t_myr as f32);
        self.tick_camera(dt, t_myr as f32);

        // Saat FOKUS aktif, throttle recompute biasa DILEWATI -- kamera bergerak tiap frame
        // (easing `tick_camera` di atas), buffer HARUS ikut segar tiap frame jg (else galaksi
        // "diam" sesaat sementara kamera genuinely sudah bergerak, terasa patah/lag).
        let due = self.tick.get() == 0 || self.is_focused();
        self.tick
            .set((self.tick.get() + 1) % self.recompute_every.max(1));

        if due || self.last_frame.borrow().is_none() {
            let cached = self.cached.borrow();
            let c = cached
                .as_ref()
                .expect("ensure_galaxy baru saja mengisi cache");
            let u = self.build_uniforms(c, pw, ph, t_myr);
            if let Ok(out) = self.backend.borrow_mut().render(&u) {
                *self.last_frame.borrow_mut() = Some(out.to_vec());
            }
        }

        let frame = self.last_frame.borrow();
        let Some(rgba) = frame.as_ref() else { return };
        let starfield_guard = self.starfield.borrow();
        let Some(sf) = starfield_guard.as_ref() else {
            return;
        };
        f.render_widget(
            PixelWidget {
                rgba,
                w: pw,
                h: ph,
                backdrop: sf,
            },
            area,
        );
    }

    /// Bangun `Uniforms` dr spec+anchors ter-cache + `Camera` interaktif (Phase 6: pan/zoom
    /// genuine, gantikan proyeksi tetap Phase 5).
    fn build_uniforms(&self, c: &CachedGalaxy, pw: u32, ph: u32, t_myr: f64) -> Uniforms {
        let cam = self.camera.get();
        let n_ref = crate::galaxy_sim::model::N_REF
            .min(self.star_count as f32)
            .max(1.0);
        let dust_scale = (self.star_count as f32 / n_ref).sqrt();
        Uniforms {
            time_myr: t_myr as f32,
            pattern_omega: c.spec.phys.pattern_omega,
            zoom: cam.zoom,
            exposure: 1.0,
            view_center: [cam.view_center.x, cam.view_center.y],
            screen: [pw as f32, ph as f32],
            incl_cs: [cam.incl_cs.x, cam.incl_cs.y],
            pa_cs: [cam.pa_cs.x, cam.pa_cs.y],
            dust_k: [
                crate::galaxy_sim::model::DUST_BASE[0]
                    * crate::galaxy_sim::model::DUST_STRENGTH
                    * dust_scale,
                crate::galaxy_sim::model::DUST_BASE[1]
                    * crate::galaxy_sim::model::DUST_STRENGTH
                    * dust_scale,
                crate::galaxy_sim::model::DUST_BASE[2]
                    * crate::galaxy_sim::model::DUST_STRENGTH
                    * dust_scale,
                cam.cell_aspect,
            ],
            anchors: c.model_anchors.map(|a| [a.x, a.y, a.z, 0.0]),
            star_count: self.star_count,
            tonemap_denom: (1.0f32 * crate::galaxy_sim::model::L_REF).asinh(),
            _pad: [0; 2],
        }
    }
}

/// `(seed, is_anchor)` dr `Galaxy` -- `Fixed` (Milky Way/home) -> anchor literal; `Procedural`
/// -> seed-driven. Dipakai `main_view.rs`/`galaxy_map.rs`'s mode Backdrop.
pub fn galaxy_sim_key(g: &crate::game::state::Galaxy) -> (u64, bool) {
    match &g.kind {
        crate::game::state::GalaxyKind::Fixed => (0, true),
        crate::game::state::GalaxyKind::Procedural { seed, .. } => (*seed, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    /// `render` HARUS genuinely gambar sesuatu (sel non-default) di area wajar -- pola SAMA
    /// precedent `GalaxyPixel`/`Sprites`'s render test.
    #[test]
    fn render_fills_area_without_panic() {
        let view = GalaxySimView::demo();
        let mut term = Terminal::new(TestBackend::new(40, 20)).unwrap();
        term.draw(|f| {
            let area = f.area();
            view.render(f, area, 0, true, 0.0);
        })
        .unwrap();
        let buf = term.backend().buffer();
        let has_halfblock = (0..20u16).any(|y| (0..40u16).any(|x| buf[(x, y)].symbol() == "▀"));
        assert!(
            has_halfblock,
            "render HARUS genuinely gambar half-block, bukan area kosong"
        );
    }

    /// Area 0x0/ekstrem kecil HARUS genuinely TAK panic (skip aman) -- pola SAMA M12.9's
    /// precedent guard lebar minimum.
    #[test]
    fn render_handles_zero_and_tiny_area_without_panic() {
        let view = GalaxySimView::demo();
        let mut term = Terminal::new(TestBackend::new(3, 3)).unwrap();
        term.draw(|f| {
            view.render(f, Rect::new(0, 0, 0, 0), 0, true, 0.0);
            view.render(f, Rect::new(0, 0, 1, 1), 0, true, 0.0);
        })
        .unwrap();
    }

    /// Ganti galaksi (seed BEDA) HARUS genuinely trigger recompute SEGERA (bukan nunggu
    /// cadence lama nampilkan galaksi SALAH) -- dites LANGSUNG bandingkan 2 render beda seed.
    #[test]
    fn switching_galaxy_forces_immediate_recompute() {
        let view = GalaxySimView::demo();
        let render_seed = |seed: u64| -> Vec<ratatui::style::Color> {
            let mut term = Terminal::new(TestBackend::new(40, 20)).unwrap();
            term.draw(|f| {
                let area = f.area();
                view.render(f, area, seed, false, 0.0);
            })
            .unwrap();
            let buf = term.backend().buffer();
            (0..20u16)
                .flat_map(|y| (0..40u16).map(move |x| (x, y)))
                .map(|(x, y)| buf[(x, y)].fg)
                .collect()
        };
        let a = render_seed(1);
        let b = render_seed(2);
        assert_ne!(
            a, b,
            "seed BEDA HARUS genuinely hasilkan tampilan warna beda"
        );
    }

    /// Recompute HARUS genuinely THROTTLE -- render berulang TANPA ganti waktu/galaksi HARUS
    /// pakai buffer CACHE yg sama (bukan recompute tiap panggilan) sampai cadence lewat.
    #[test]
    fn recompute_is_throttled_not_every_call() {
        let view = GalaxySimView::demo();
        let mut term = Terminal::new(TestBackend::new(40, 20)).unwrap();
        // Panggilan pertama: recompute (tick baru mulai 0). Panggilan KEDUA (tick=1, msh
        // dlm cadence recompute_every=4): HARUS pakai frame CACHE persis sama (bukan
        // recompute dgn t_myr BEDA -- kalau recompute pasti beda krn t_myr beda jauh).
        term.draw(|f| {
            let area = f.area();
            view.render(f, area, 0, true, 0.0);
        })
        .unwrap();
        let frame1 = view.last_frame.borrow().clone();
        term.draw(|f| {
            let area = f.area();
            view.render(f, area, 0, true, 999.0); // t_myr JAUH beda -- beda kalau recompute.
        })
        .unwrap();
        let frame2 = view.last_frame.borrow().clone();
        assert_eq!(
            frame1, frame2,
            "panggilan KEDUA (msh dlm cadence throttle) HARUS genuinely pakai buffer cache \
             SAMA, bukan recompute dgn t_myr baru"
        );
    }

    /// M20.8 follow-up Phase 6: render PERTAMA (dims dr 0x0) HARUS genuinely fit zoom overview
    /// [BUKAN camera default `zoom=4.0` sembarangan] -- dites LANGSUNG bandingkan nilai zoom
    /// stlh render vs `Camera::new`'s default.
    #[test]
    fn first_render_fits_zoom_to_overview_not_default() {
        let view = GalaxySimView::demo();
        let mut term = Terminal::new(TestBackend::new(40, 20)).unwrap();
        term.draw(|f| {
            let area = f.area();
            view.render(f, area, 0, true, 0.0);
        })
        .unwrap();
        let cam = view.camera_snapshot();
        assert_ne!(
            cam.zoom, 4.0,
            "zoom HARUS genuinely di-fit, bukan default placeholder"
        );
        assert!(cam.zoom > 0.0 && cam.zoom.is_finite());
    }

    /// `pan` HARUS genuinely ubah `view_center` & paksa recompute SEGERA (bukan nunggu
    /// cadence) -- dites LANGSUNG bandingkan camera state + output render sblm/sesudah.
    #[test]
    fn pan_moves_view_center_and_forces_recompute() {
        let view = GalaxySimView::demo();
        let mut term = Terminal::new(TestBackend::new(40, 20)).unwrap();
        term.draw(|f| {
            let area = f.area();
            view.render(f, area, 0, true, 0.0);
        })
        .unwrap();
        let before = view.camera_snapshot().view_center;
        view.pan(50.0, 0.0);
        let after = view.camera_snapshot().view_center;
        assert_ne!(before, after, "pan HARUS genuinely ubah view_center");
        // Recompute SEGERA: render lagi harus genuinely pakai posisi kamera BARU (tick di-reset).
        let frame_before_pan_render = view.last_frame.borrow().clone();
        term.draw(|f| {
            let area = f.area();
            view.render(f, area, 0, true, 0.0);
        })
        .unwrap();
        let frame_after_pan_render = view.last_frame.borrow().clone();
        assert_ne!(
            frame_before_pan_render, frame_after_pan_render,
            "render stlh pan HARUS genuinely beda (recompute SEGERA, bukan tunggu cadence)"
        );
    }

    /// `zoom_by` HARUS genuinely ubah level zoom (dites LANGSUNG nilai numerik) & TAK PERNAH
    /// nol/negatif walau faktor ekstrem (`max(0.01)` guard) -- dites nilai ekstrem LANGSUNG.
    #[test]
    fn zoom_by_changes_zoom_and_stays_positive_at_extremes() {
        let view = GalaxySimView::demo();
        let mut term = Terminal::new(TestBackend::new(40, 20)).unwrap();
        term.draw(|f| view.render(f, f.area(), 0, true, 0.0))
            .unwrap();
        let z0 = view.camera_snapshot().zoom;
        view.zoom_by(2.0);
        let z1 = view.camera_snapshot().zoom;
        assert!(z1 > z0, "zoom_by(2.0) HARUS genuinely perbesar zoom");
        view.zoom_by(0.0001);
        let z2 = view.camera_snapshot().zoom;
        assert!(
            z2 > 0.0 && z2.is_finite(),
            "zoom HARUS genuinely tetap positif di faktor ekstrem kecil"
        );
    }

    /// `reset_view` HARUS genuinely kembalikan kamera ke overview (view_center=(0,0), zoom
    /// fit) stlh pan+zoom -- dites LANGSUNG bandingkan state SEBELUM pan/zoom vs SETELAH reset.
    #[test]
    fn reset_view_restores_overview_after_pan_and_zoom() {
        let view = GalaxySimView::demo();
        let mut term = Terminal::new(TestBackend::new(40, 20)).unwrap();
        term.draw(|f| view.render(f, f.area(), 0, true, 0.0))
            .unwrap();
        let original = view.camera_snapshot();

        view.pan(100.0, 50.0);
        view.zoom_by(3.0);
        let disturbed = view.camera_snapshot();
        assert_ne!(disturbed.view_center, original.view_center);
        assert_ne!(disturbed.zoom, original.zoom);

        view.reset_view();
        let restored = view.camera_snapshot();
        assert_eq!(
            restored.view_center,
            glam::Vec2::ZERO,
            "reset_view HARUS genuinely kembalikan view_center ke (0,0)"
        );
        assert!(
            (restored.zoom - original.zoom).abs() < 1e-4,
            "reset_view HARUS genuinely kembalikan zoom ke fit overview SAMA spt awal"
        );
    }

    /// `focus_next` HARUS genuinely siklus Overview -> POI[0] -> POI[1] -> POI[2] -> Overview
    /// (bukan cuma toggle 2 state) -- dites LANGSUNG nama tiap langkah, katalog anchor py
    /// urutan tetap ["Sol","Rigel","Orion Nebula"].
    #[test]
    fn focus_next_cycles_through_full_catalog_then_back_to_overview() {
        let view = GalaxySimView::demo();
        let mut term = Terminal::new(TestBackend::new(40, 20)).unwrap();
        term.draw(|f| view.render(f, f.area(), 0, true, 0.0))
            .unwrap();
        assert!(!view.is_focused());
        assert_eq!(view.focused_poi_name(), None);

        view.focus_next();
        assert_eq!(view.focused_poi_name().as_deref(), Some("Sol"));
        view.focus_next();
        assert_eq!(view.focused_poi_name().as_deref(), Some("Rigel"));
        view.focus_next();
        assert_eq!(view.focused_poi_name().as_deref(), Some("Orion Nebula"));
        view.focus_next();
        assert!(
            !view.is_focused(),
            "siklus penuh HARUS genuinely kembali ke Overview"
        );
        assert_eq!(view.focused_poi_name(), None);
    }

    /// `unfocus` HARUS genuinely kembalikan Overview dr fokus manapun (dites LANGSUNG, bukan
    /// diasumsikan dr `focus_next`'s siklus).
    #[test]
    fn unfocus_returns_to_overview() {
        let view = GalaxySimView::demo();
        let mut term = Terminal::new(TestBackend::new(40, 20)).unwrap();
        term.draw(|f| view.render(f, f.area(), 0, true, 0.0))
            .unwrap();
        view.focus_next();
        assert!(view.is_focused());
        view.unfocus();
        assert!(!view.is_focused());
        assert_eq!(view.focused_poi_name(), None);
    }

    /// Saat fokus AKTIF, kamera HARUS genuinely bergerak mendekati posisi POI antar frame
    /// (easing `tick_camera`) -- dites LANGSUNG `view_center` berubah lintas beberapa render
    /// dgn `t_myr` maju (POI berorbit, target posisi genuinely berubah).
    #[test]
    fn focused_camera_moves_toward_poi_across_frames() {
        let view = GalaxySimView::demo();
        let mut term = Terminal::new(TestBackend::new(40, 20)).unwrap();
        term.draw(|f| view.render(f, f.area(), 0, true, 0.0))
            .unwrap();
        view.focus_next(); // -> Sol.
        let c0 = view.camera_snapshot().view_center;
        // Render lintas beberapa "detik" simulasi (t_myr maju genuine) -- easing HARUS
        // genuinely gerakkan view_center menuju target tiap panggilan.
        let mut moved = false;
        for i in 1..20 {
            term.draw(|f| view.render(f, f.area(), 0, true, i as f64 * MYR_PER_SEC * 0.05))
                .unwrap();
            if view.camera_snapshot().view_center != c0 {
                moved = true;
                break;
            }
        }
        assert!(
            moved,
            "view_center HARUS genuinely bergerak saat fokus POI aktif"
        );
    }

    /// `pan` HARUS genuinely NO-OP saat fokus POI aktif (SIA-SIA, akan ditimpa follow) --
    /// dites LANGSUNG bandingkan view_center sblm/sesudah panggil `pan` saat fokus.
    #[test]
    fn pan_is_noop_while_focused() {
        let view = GalaxySimView::demo();
        let mut term = Terminal::new(TestBackend::new(40, 20)).unwrap();
        term.draw(|f| view.render(f, f.area(), 0, true, 0.0))
            .unwrap();
        view.focus_next();
        let before = view.camera_snapshot().view_center;
        view.pan(999.0, 999.0);
        let after = view.camera_snapshot().view_center;
        assert_eq!(
            before, after,
            "pan HARUS genuinely no-op saat fokus POI aktif"
        );
    }

    /// Ganti galaksi (seed BEDA) SAAT fokus aktif HARUS genuinely reset ke Overview (kunci POI
    /// galaksi LAMA tak berarti lg utk galaksi BARU) -- dites LANGSUNG, bukan diasumsikan aman.
    #[test]
    fn switching_galaxy_resets_focus_to_overview() {
        let view = GalaxySimView::demo();
        let mut term = Terminal::new(TestBackend::new(40, 20)).unwrap();
        term.draw(|f| view.render(f, f.area(), 1, false, 0.0))
            .unwrap();
        view.focus_next();
        assert!(view.is_focused());
        term.draw(|f| view.render(f, f.area(), 2, false, 0.0))
            .unwrap();
        assert!(
            !view.is_focused(),
            "ganti galaksi HARUS genuinely reset fokus ke Overview"
        );
    }
}
