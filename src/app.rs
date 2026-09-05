//! `App` (state + content) dan event loop utama.
//!
//! Event loop ratatui penuh (input non-blocking, tick 1Hz, render decoupled) diisi M3 lanjutan.
//! Sekarang: `App` + `App::demo()` (untuk snapshot headless) tersedia.
#![allow(dead_code)]

use crate::balance::{
    AUTOSAVE_INTERVAL_SECS, MAX_TICKS_PER_FRAME, RENDER_INTERVAL_MS, TICK_DURATION_SECS,
};
use crate::content::load_content;
use crate::game::defs::{BuildingId, Content, RecipeId, ResourceId};
use crate::game::state::{
    Biome, Factory, FactoryId, FactoryKind, GalaxyId, GameState, NodeId, Planet, PlanetId,
    ResourceMap, ResourceNode, SAVE_VERSION, Ship, UnlockReq,
};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use std::time::{Duration, Instant};

/// M4: berapa frame render border `outer_block` disorot sesudah view berpindah (~0.3s di
/// `RENDER_INTERVAL_MS` 75ms/frame ≈13fps -- lihat `balance::RENDER_INTERVAL_MS`).
const VIEW_TRANSITION_FRAMES: u8 = 4;

/// Aplikasi: state game + registry konten read-only + view aktif.
pub struct App {
    pub state: GameState,
    pub content: Content,
    /// View aktif (main_menu | galaxy_map | planet_view | research | merchant | warp).
    pub view: String,
    /// Indeks terpilih dalam list view aktif (mis. slot factory di planet_view).
    pub sel: usize,
    /// Bidang bintang galaksi spiral menu (kosmetik, read-only).
    pub anim: crate::ui::galaxy_anim::GalaxyAnim,
    /// Waktu animasi (detik) sejak masuk menu; `0.0` saat snapshot deterministik.
    pub anim_secs: f64,
    /// Sistem partikel kosmetik (engine exhaust saat Traveling, dll).
    pub particles: crate::ui::particles::ParticleSystem,
    /// Indeks emitter engine-exhaust di `particles`.
    pub exhaust_idx: usize,
    /// Cache sprite celestial (planet/star/ship), lazy-parse `.ans` per `(base,size,depth)`.
    /// `App::demo()` set depth tetap (`Tc`, deterministik); `run()` menaikkan ke
    /// `ColorDepth::detect()` (deteksi terminal nyata) sebelum masuk event loop.
    pub sprites: crate::ui::sprite::Sprites,
    /// Cache portrait karakter via `ratatui-image`. `App::demo()` set `halfblocks()`
    /// (deterministik, no I/O); `run()` menaikkan ke `Portraits::detect()` (query protokol
    /// terminal nyata, fallback half-block bila gagal — tak pernah panic) sebelum event loop.
    pub portraits: crate::ui::portrait::Portraits,
    /// M17.5: cache+detect galaxy pixel (Sixel/Kitty/iTerm2), pola SAMA `portraits`. `demo()`
    /// set `halfblocks()` (deterministik); `run()` menaikkan ke `detect()` sblm event loop.
    /// **M20.8 follow-up (galaxy_sim Phase 5) menggantikan INI+`anim` sbg backdrop UTAMA** --
    /// medan tetap ada sbg fallback dlm `galaxy_sim`'s desain, tp tak lg dipanggil LANGSUNG
    /// dr `main_view.rs`/`galaxy_map.rs` (lihat `galaxy_sim` field).
    pub galaxy_pixel: crate::ui::galaxy_pixel::GalaxyPixel,
    /// M20.8 follow-up: density-wave galaxy simulation (GPU wgpu + CPU rayon fallback,
    /// diadaptasi `andromeda-simulation-tui`) — MENGGANTIKAN `anim`+`galaxy_pixel` sbg
    /// backdrop UTAMA di `main_view.rs`+`galaxy_map.rs`'s mode Backdrop. `demo()` set
    /// `GalaxySimView::demo()` (paksa CPU, deterministik); `run()` naikkan ke `detect()`
    /// (probe GPU nyata) sblm event loop, pola SAMA `sprites`/`portraits`/`galaxy_pixel`.
    pub galaxy_sim: crate::ui::galaxy_sim_view::GalaxySimView,
    /// M17.5: toggle manual pixel<->procedural (`g`+`Shift` di MAIN VIEW, lihat `handle_key`).
    /// `false` (default) = AUTO (pixel bila didukung terminal, else procedural — keputusan
    /// `GalaxyPixel::render_or_fallback`'s `supports_pixel()`). `true` = paksa procedural
    /// biarpun terminal genuinely mendukung pixel (user mungkin lbh suka animasi spiral).
    pub force_procedural_galaxy: bool,
    /// M18.4: posisi kursor BEBAS di starmap grid (koordinat dunia, `galaxy_map.rs`'s
    /// `WORLD_W`/`WORLD_H`) — TERPISAH dari `sel` (indeks planet, dipakai research/planet_view)
    /// krn kursor grid HARUS bisa berdiri di sel KOSONG (bukan cuma lompat antar planet).
    /// Default `(0,0)` (pojok dunia, sama titik viewport M18.1 mulai).
    pub galaxy_cursor: (i32, i32),
    /// M19.9: "Filter (faksi/tile) toggle" — game TAK PY faksi (dikonfirmasi berulang M18.2/
    /// 8/19.1 dst, `game/state.rs` tak py field faction sama sekali), diganti filter STATUS
    /// tile [pola SAMA substitusi "owner"->"status" yg konsisten sesi ini]. Murni VISUAL
    /// (hide tile tak cocok filter dr grid, GANTI jd tekstur latar biasa) — TAK mengubah
    /// mekanik/aksi (scan/send/enter tetap jalan normal di tile tersembunyi bila kursor
    /// digerakkan ke situ). Default `All` (tampilkan semua, sama perilaku sblm M19.9).
    pub galaxy_filter: crate::ui::panels::galaxy_map::GalaxyFilter,
    /// M20.1: "Mode backdrop vs starmap (tab/key) dalam galaxy view" — toggle `Tab` DALAM
    /// `galaxy_map`, reuse LANGSUNG logic pixel/procedural `main_view.rs` (bukan view
    /// terpisah). Default `Starmap` (perilaku SAMA sblm M20.1, tak ada regresi).
    pub galaxy_map_mode: crate::ui::panels::galaxy_map::GalaxyMapMode,
    /// M14.12: build picker aktif (slot factory kosong terpilih + navigasi list kandidat
    /// building). `None` = tak sedang membangun. Selagi `Some`, SEMUA key input dialihkan ke
    /// `build_picker_keys` (lihat `handle_key`) — bukan campur logic navigasi planet biasa.
    pub build_picker: Option<BuildPicker>,
    /// M1#6 fix: **gap real ditemukan** -- build/upgrade/research/travel/trade dulu SEMUA
    /// `let _ = ...` (gagal DIABAIKAN total, tak ada sinyal player-facing sama sekali; sukses
    /// pun tak py konfirmasi eksplisit, cuma "state berubah, semoga kelihatan"). Sesi/UI-only
    /// (BUKAN `GameState` -- tak disimpan, sama kelas `sel`/`anim_secs`). `(ok, message)`;
    /// waktu-based fade-out (biar hilang otomatis) itu scope M4 (Action Feedback & Animation
    /// Polish) -- di sini cuma bertahan sampai aksi BERIKUTNYA menimpa (real, bukan timer).
    pub last_action: Option<(bool, String)>,
    /// M2: indeks terpilih di Title screen (`panels::flow::title`'s list New Game/Continue/
    /// Settings/Help/Quit). Sesi/UI-only, sama kelas `sel`/`anim_secs` — tak disimpan.
    pub title_sel: usize,
    /// M2: modal konfirmasi overwrite New Game aktif (`Some` = tampilkan Yes/No, cegah New
    /// Game diam2 timpa save lama tanpa peringatan). Pola SAMA `build_picker` (exclusive-input
    /// modal, semua key dialihkan sampai modal ditutup).
    pub new_game_confirm: bool,
    /// M3: step tutorial yg hint-nya DISEMBUNYIKAN player via `T` (dismissable, per plan's
    /// "contextual, dismissable hint panel"). Sesi/UI-only. Reset otomatis begitu
    /// `state.tutorial_step` MAJU ke step lain (`Some(step)` lama tak lagi cocok step baru,
    /// jadi hint step BARU tetap tampil default — dismiss cuma berlaku utk step yg SEDANG
    /// disembunyikan, bukan "matikan tutorial selamanya").
    pub tutorial_hint_dismissed_for: Option<u8>,
    /// M4: sisa frame render "transition flourish" border (dihitung mundur `event_loop` tiap
    /// frame, disulut saat `view` berubah dr frame sebelumnya). `0` = normal, `>0` = border
    /// `outer_block` disorot (`ui::mod`). Gated ke FRAME (bukan real time) --
    /// `demo()`/snapshot/screenshot-bin selalu `0` (tak pernah lewat `event_loop`), jd nol
    /// resiko regresi golden snapshot.
    pub view_transition_frames: u8,
}

/// M14.12: state picker building — slot tujuan (factory kosong yg dipilih sblm buka picker) +
/// indeks terpilih dalam list kandidat (`actions::buildable_options`).
#[derive(Clone, Copy, Debug)]
pub struct BuildPicker {
    pub slot: usize,
    pub sel: usize,
}

/// Path `assets/sprites` absolut (robust terlepas dari cwd proses).
fn sprites_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/sprites")
}

impl App {
    /// M2: "New Game" NYATA — beda dr `demo()` (single-planet, resource pra-isi utk snapshot):
    /// dunia Sol System PENUH via `game::world::load_milky_way` (24 body nyata dr
    /// `data/milky_way.ron`), `GameState` fresh (0 credits, stockpile kosong, no research
    /// aktif). Reuse scaffolding kosmetik `demo()` (particles/sprites/portraits/dll — semua
    /// deterministik/lazy, aman dipakai ulang) lalu timpa `state` + `view`; `demo()` sendiri
    /// TAK disentuh/diubah (tetap fixture test/snapshot/screenshot-bin, per plan M2's guardrail).
    pub fn new_game() -> App {
        let mut app = App::demo();
        let dir = std::path::Path::new("data");
        let galaxy0 = crate::game::world::load_milky_way(dir, &app.content)
            .expect("load data/milky_way.ron utk New Game");
        let seed = crate::game::world::procgen::galaxy_seed(0xA17, 1, 0);
        let frontier = crate::game::state::Galaxy {
            id: GalaxyId(1),
            name: crate::game::world::procgen::galaxy_name(seed),
            level: 1,
            kind: crate::game::state::GalaxyKind::Procedural {
                seed,
                visited: Default::default(),
            },
            planets: vec![],
        };
        app.state = GameState {
            version: SAVE_VERSION,
            last_saved_unix: 0,
            tick: 0,
            credits: 0.0,
            inventory: Default::default(),
            galaxies: vec![galaxy0, frontier],
            active_galaxy: GalaxyId(0),
            anchor_galaxy: GalaxyId(0),
            ship: Ship::default(),
            research: Default::default(),
            prestige: Default::default(),
            merchant: Default::default(),
            events: Default::default(),
            settings: Default::default(),
            tutorial_step: Some(0),
            quests: Default::default(),
        };
        app.view = "planet_view".into();
        app.sel = 0;
        app.last_action = None;
        app
    }

    /// Bangun state contoh deterministik (resource terisi) untuk snapshot & demo.
    pub fn demo() -> App {
        let content = load_content(std::path::Path::new("data")).expect("load data/ untuk demo");
        let mut stockpile = ResourceMap::new();
        let seed: [(&str, f64); 7] = [
            ("energy", 1240.0),
            ("iron", 850.0),
            ("water", 430.0),
            ("alloys", 120.0),
            ("consumer_goods", 85.0),
            ("rare_crystals", 12.0),
            ("dark_matter", 3.0),
        ];
        for (name, val) in seed {
            if let Some(h) = content.resources.id(name) {
                stockpile.insert(ResourceId(h), val);
            }
        }
        let rid = |n: &str| ResourceId(content.resources.id(n).unwrap());
        let bid = |n: &str| BuildingId(content.buildings.id(n).unwrap());
        let nodes = vec![
            ResourceNode {
                id: NodeId(0),
                resource: rid("iron"),
                richness: 1.5,
                level: 4,
            },
            ResourceNode {
                id: NodeId(1),
                resource: rid("carbon"),
                richness: 1.2,
                level: 2,
            },
            ResourceNode {
                id: NodeId(2),
                resource: rid("water"),
                richness: 2.0,
                level: 3,
            },
        ];
        let factory_slots = vec![
            Some(Factory {
                id: FactoryId(0),
                building: bid("mining_drill"),
                kind: FactoryKind::Extractor { node: NodeId(0) },
                level: 4,
                enabled: true,
            }),
            Some(Factory {
                id: FactoryId(1),
                building: bid("steel_mill_bld"),
                kind: FactoryKind::Refinery {
                    recipe: RecipeId(content.recipes.id("steel_mill").unwrap()),
                },
                level: 2,
                enabled: true,
            }),
            None,
            Some(Factory {
                id: FactoryId(2),
                building: bid("research_lab"),
                kind: FactoryKind::ResearchLab,
                level: 2,
                enabled: true,
            }),
        ];
        let ship = Ship {
            warp_tier: 3,
            engine: 2,
            cargo: 1,
            ..Ship::default()
        };
        let earth = Planet {
            id: PlanetId(0),
            name: "Earth".into(),
            tier: 1,
            biome: Biome::Terran,
            distance: 1.0,
            unlocked: true,
            unlock_req: UnlockReq::None,
            nodes,
            factory_slots,
            stockpile,
            stockpile_cap: 100_000.0,
        };
        let state = GameState {
            version: SAVE_VERSION,
            last_saved_unix: 0,
            tick: 0,
            credits: 0.0,
            inventory: std::collections::HashMap::new(),
            galaxies: vec![
                crate::game::state::Galaxy {
                    id: GalaxyId(0),
                    name: "Milky Way".into(),
                    level: 0,
                    kind: crate::game::state::GalaxyKind::Fixed,
                    planets: vec![earth],
                },
                // Sektor frontier ProcGen (Lvl 1) untuk demo Galaxy Map — deterministik dari seed.
                {
                    let seed = crate::game::world::procgen::galaxy_seed(0xA17, 1, 0);
                    crate::game::state::Galaxy {
                        id: GalaxyId(1),
                        name: crate::game::world::procgen::galaxy_name(seed),
                        level: 1,
                        kind: crate::game::state::GalaxyKind::Procedural {
                            seed,
                            visited: Default::default(),
                        },
                        planets: vec![],
                    }
                },
            ],
            active_galaxy: GalaxyId(0),
            anchor_galaxy: GalaxyId(0),
            ship,
            // Demo: riset aktif manu_steel ~60% (Data 300/500, 72/120s) untuk snapshot research.
            research: crate::game::state::ResearchState {
                data: 50.0,
                completed: Default::default(),
                active: Some(crate::game::state::ActiveResearch {
                    tech_id: "manu_steel".into(),
                    data_invested: 300.0,
                    elapsed_secs: 72.0,
                }),
            },
            prestige: Default::default(),
            merchant: Default::default(),
            events: Default::default(),
            settings: Default::default(),
            tutorial_step: None,
            quests: Default::default(),
        };
        let mut particles = crate::ui::particles::ParticleSystem::new(0x009A_12E5);
        let mut exhaust = crate::ui::particles::Emitter::exhaust((0.5, 0.6));
        exhaust.enabled = false; // hanya aktif saat Traveling (di event loop)
        let exhaust_idx = particles.add_emitter(exhaust);
        App {
            state,
            content,
            view: "main_menu".into(),
            sel: 0,
            anim: crate::ui::galaxy_anim::GalaxyAnim::default_field(),
            anim_secs: 0.0,
            particles,
            exhaust_idx,
            // Deterministik (bukan deteksi terminal nyata) — cocok tujuan `demo()`: "state contoh
            // deterministik utk snapshot & demo". `run()` menaikkan ke deteksi nyata sesudahnya;
            // test/headless (bin `screenshot`/`snapshot`, `cargo test`) tetap dpt fixed & stabil.
            sprites: crate::ui::sprite::Sprites::with_depth(
                sprites_root(),
                crate::ui::sprite::ColorDepth::Tc,
            ),
            portraits: crate::ui::portrait::Portraits::halfblocks(),
            galaxy_pixel: crate::ui::galaxy_pixel::GalaxyPixel::halfblocks(),
            galaxy_sim: crate::ui::galaxy_sim_view::GalaxySimView::demo(),
            force_procedural_galaxy: false,
            galaxy_cursor: (0, 0),
            galaxy_filter: crate::ui::panels::galaxy_map::GalaxyFilter::All,
            galaxy_map_mode: crate::ui::panels::galaxy_map::GalaxyMapMode::Starmap,
            build_picker: None,
            last_action: None,
            title_sel: 0,
            new_game_confirm: false,
            tutorial_hint_dismissed_for: None,
            view_transition_frames: 0,
        }
    }

    /// Peta `Biome` → nama base sprite celestial (folder `assets/sprites/<base>/`).
    pub fn sprite_for_biome(biome: Biome) -> &'static str {
        match biome {
            Biome::IronWorld => "planet_ironworld",
            Biome::OceanPlanet => "planet_ocean",
            Biome::GasGiant => "planet_gasgiant",
            Biome::DeadWorld => "planet_deadworld",
            Biome::CrystalWorld => "planet_crystalworld",
            Biome::Terran => "planet_terran",
            Biome::AsteroidBelt => "planet_asteroidbelt",
            Biome::IceWorld => "planet_iceworld",
            Biome::LavaWorld => "planet_lavaworld",
        }
    }

    /// ID portrait (path relatif ke `assets/characters/`, tanpa ekstensi `.png`) deterministik
    /// per konteks. `"captain"` = identitas tetap (kapten kapal pemain, tak pernah ganti wajah).
    /// Konteks lain (mis. `"merchant"`) pakai `seed` (mis. `restock_at_tick`) → karakter berbeda
    /// per kunjungan, tapi stabil selama seed sama (bukan acak tiap render/frame). Konteks tak
    /// dikenal jatuh ke fallback aman (bukan panic).
    pub fn portrait_id(ctx: &str, seed: u64) -> &'static str {
        const MERCHANT_POOL: [&str; 7] = [
            "alien/char_alien_01",
            "alien/char_alien_03",
            "alien/char_alien_05",
            "male/char_male_02",
            "male/char_male_04",
            "female/char_female_02",
            "female/char_female_04",
        ];
        match ctx {
            "captain" => "male/char_male_01",
            "merchant" => MERCHANT_POOL[(seed as usize) % MERCHANT_POOL.len()],
            _ => "alien/char_alien_01",
        }
    }

    /// Galaksi+planet aktif (galaksi sekarang, planet pertama yang unlocked). `pub(crate)`
    /// M14.9: dipakai jg `panels::planet`'s cost+afford preview (butuh index, bukan cuma ref).
    pub(crate) fn active_pi(&self) -> Option<(usize, usize)> {
        let gi = self
            .state
            .galaxies
            .iter()
            .position(|g| g.id == self.state.active_galaxy)?;
        let pi = self.state.galaxies[gi]
            .planets
            .iter()
            .position(|p| p.unlocked)?;
        Some((gi, pi))
    }

    /// Ganti view + dispatch navigasi/aksi list (`06` §Keybindings).
    /// M15.8: `pub(crate)` (dulu private) — `panels::research`'s test butuh panggil LANGSUNG
    /// (bukan set `app.sel` manual) utk verifikasi jembatan end-to-end key-press→render genuine,
    /// pola sama `active_pi` (M14.9) yg diperlebar visibility utk reuse cross-module.
    pub(crate) fn handle_key(&mut self, code: KeyCode) {
        // M14.12: build picker aktif → SEMUA input dialihkan ke picker eksklusif (bukan
        // dicampur global g/p/r/m/w/Esc — Esc di sini HARUS batalkan picker, bukan lompat ke
        // main_menu, jadi harus di-intercept SBLM match global di bawah).
        if self.build_picker.is_some() {
            self.build_picker_keys(code);
            return;
        }
        // M2 item 1: Splash HARUS di-intercept SBLM match global g/p/r/m/w/S/? di bawah --
        // kalau tak, tombol `g`/`p`/dst saat splash akan diam2 lompat ke game view (match
        // global tak bersyarat), bukan lanjut ke Title dulu spt seharusnya (sama kelas bug
        // Backdrop di atas).
        if self.view == "splash" {
            self.view = "title".into();
            return;
        }
        // M2 item 2: Title's New Game overwrite-confirm modal — exclusive input SAMA
        // build_picker (dicek SEBELUM `title_keys` sendiri, biar Esc/Enter di modal tak
        // ketimpa navigasi list Title di baliknya).
        if self.view == "title" && self.new_game_confirm {
            match code {
                KeyCode::Enter => {
                    self.new_game_confirm = false;
                    *self = App::new_game();
                }
                KeyCode::Esc => self.new_game_confirm = false,
                _ => {}
            }
            return;
        }
        if self.view == "title" {
            self.title_keys(code);
            return;
        }
        // M20.8 follow-up Phase 6: mode Backdrop's pan/zoom/reset (z/x/r/hjkl) HARUS di-
        // intercept SBLM match global di bawah -- **bug real ditemukan**: global `r`
        // (`view="research"`) diam2 menang atas Backdrop's `r` (reset_view), krn match
        // global TAK BERSYARAT dan reset `self.view` SEBELUM galaxy_map_keys sempat jalan
        // (dites `backdrop_pan_zoom_reset_keys_wired_end_to_end`, gagal PERSIS krn ini
        // sblm diperbaiki -- bukan diasumsikan aman spt guard build_picker di atas).
        if self.view == "galaxy_map"
            && self.galaxy_map_mode == crate::ui::panels::galaxy_map::GalaxyMapMode::Backdrop
        {
            self.galaxy_map_keys(code);
            return;
        }
        match code {
            KeyCode::Char('g') => self.view = "galaxy_map".into(),
            KeyCode::Char('p') => self.view = "planet_view".into(),
            KeyCode::Char('r') => self.view = "research".into(),
            KeyCode::Char('m') => self.view = "merchant".into(),
            KeyCode::Char('w') => self.view = "warp".into(),
            // M1 fix: Settings NYATA (dulu `menu.rs`'s entri "Settings" py view id `None`,
            // mati total). `06-ui.md` §Keybindings sebut `s` polos, tp `s` lowercase SUDAH
            // dipakai `galaxy_map_keys`'s Starmap mode (scan planet, mekanik NYATA+dites) --
            // pakai lowercase di sini akan diam2 patahkan scan (match global tak bersyarat,
            // jalan SBLM dispatch view-specific). `S` besar dipilih, pola SAMA `G`/`g` di bawah.
            KeyCode::Char('S') => self.view = "settings".into(),
            // M1 fix: `?:Help` di `shortcuts.rs` dulu genuinely fake -- key ini tak PERNAH
            // di-handle sama sekali (dicek: tak ada match arm `?` manapun sblm ini).
            KeyCode::Char('?') => self.view = "help".into(),
            // M3: dismiss hint tutorial step AKTIF (`T` besar, `t` kecil sudah dipakai
            // galaxy_map's travel -- pola SAMA `S`/`G` di atas). Cuma sembunyikan step
            // SEKARANG; step berikutnya (`tutorial_step` maju) otomatis tampil lg (lihat
            // dokumentasi field `tutorial_hint_dismissed_for`).
            KeyCode::Char('T') => self.tutorial_hint_dismissed_for = self.state.tutorial_step,
            // M17.5: toggle manual pixel<->procedural di MAIN VIEW (`G` besar, `g` kecil sudah
            // dipakai navigasi ke galaxy_map -- dicek dulu tak bentrok binding manapun).
            KeyCode::Char('G') if self.view == "main_menu" => {
                self.force_procedural_galaxy = !self.force_procedural_galaxy;
            }
            KeyCode::Esc => self.view = "main_menu".into(),
            _ => {}
        }
        if self.view == "planet_view" {
            self.planet_keys(code);
        } else if self.view == "galaxy_map" {
            self.galaxy_map_keys(code);
        } else if self.view == "research" {
            self.research_keys(code);
        } else if self.view == "settings" {
            self.settings_keys(code);
        } else if self.view == "merchant" {
            self.merchant_keys(code);
        } else if self.view == "warp" {
            self.warp_keys(code);
        }
    }

    /// M1 fix: Merchant dulu genuinely `draw_placeholder` -- j/k pilih offer (`self.sel`, field
    /// SAMA dipakai planet_view/research/galaxy_map, bukan field baru), Enter beli via
    /// `game::events::accept_merchant_offer` (BARU, dulu tak ada consumer sama sekali). Gagal
    /// (inactive/insufficient) diabaikan senyap -- pola SAMA `research_keys`/`planet_keys`'s
    /// upgrade/build (precondition TERLIHAT di teks offer SEBELUM Enter ditekan); audit+
    /// feedback nyata semua aksi silent begini menyusul item TERPISAH (M1#6).
    fn merchant_keys(&mut self, code: KeyCode) {
        let count = self.state.merchant.stock.len();
        match code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.sel = (self.sel + 1).min(count.saturating_sub(1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.sel = self.sel.saturating_sub(1);
            }
            KeyCode::Enter => {
                match crate::game::events::accept_merchant_offer(&mut self.state, self.sel) {
                    Ok(()) => {
                        self.feedback(true, "Transaksi berhasil.");
                        self.celebrate();
                    }
                    Err(e) => self.feedback(false, format!("Transaksi gagal: {e:?}")),
                }
            }
            _ => {}
        }
    }

    /// M4 fix: **gap real ditemukan** — `panels::warp::render` sudah menampilkan hint
    /// `"[1] INITIATE WARP JUMP"` sejak view ini ada, tp `handle_key` TAK PERNAH dispatch ke
    /// view "warp" sama sekali (dicek if-else chain) -- menekan `1` genuinely no-op total,
    /// persis kelas bug M1 (advertised tp mati) yg lolos dr audit M1 krn bukan `let _ = ...`
    /// call site, tp mekanik yg belum pernah di-wire ke key SAMA SEKALI. `2` kembali ke
    /// `main_menu` (SAMA hint teks "[2] Back", konsisten Esc global yg jg ke main_menu).
    fn warp_keys(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('1') => {
                match crate::game::prestige::warp_jump(&mut self.state, &self.content) {
                    Ok(report) => {
                        self.feedback(
                            true,
                            format!(
                                "Warp jump berhasil! +{} Warp Cores (Lvl {})",
                                report.cores_gained, report.new_level
                            ),
                        );
                        self.celebrate();
                    }
                    Err(e) => self.feedback(false, format!("Warp jump gagal: {e:?}")),
                }
            }
            KeyCode::Char('2') => self.view = "main_menu".into(),
            _ => {}
        }
    }

    /// M1 fix: Settings — `j/k` siklus tema (Default -> HighContrast -> Mono -> ..), berlaku
    /// SEKETIKA (dibaca `ui::theme::theme` semua panel lain via `state.settings.theme`, bukan
    /// sistem preview terpisah). `Esc` kembali `main_menu` via match global di atas (generik,
    /// tak perlu override di sini -- pola SAMA view lain yg tak override Esc).
    fn settings_keys(&mut self, code: KeyCode) {
        use crate::ui::panels::flow::settings::THEME_ORDER;
        let cur = THEME_ORDER
            .iter()
            .position(|t| *t == self.state.settings.theme)
            .unwrap_or(0);
        match code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.settings.theme = THEME_ORDER[(cur + 1) % THEME_ORDER.len()];
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.settings.theme =
                    THEME_ORDER[(cur + THEME_ORDER.len() - 1) % THEME_ORDER.len()];
            }
            _ => {}
        }
    }

    /// M2: navigasi list Title (New Game/Continue/Settings/Help/Quit). Enter: New Game →
    /// modal konfirmasi bila save ADA (cegah timpa diam2), langsung `App::new_game()` bila
    /// tak ada save; Continue → `save::read` (gagal = feedback visible, TETAP di Title, tak
    /// panic, per plan's "read errors handled gracefully"); Settings/Help → view nyata sudah
    /// ada (M1); Quit ditangani `event_loop` (`q`/Ctrl+C sudah global, bukan lewat sini).
    fn title_keys(&mut self, code: KeyCode) {
        use crate::ui::panels::flow::title::ENTRIES;
        match code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.title_sel = (self.title_sel + 1) % ENTRIES.len();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.title_sel = (self.title_sel + ENTRIES.len() - 1) % ENTRIES.len();
            }
            KeyCode::Enter => match ENTRIES[self.title_sel] {
                "New Game" => {
                    if crate::ui::panels::flow::title::save_exists() {
                        self.new_game_confirm = true;
                    } else {
                        *self = App::new_game();
                    }
                }
                "Continue" => match crate::save::read(&self.content) {
                    Ok(state) => {
                        self.state = state;
                        self.view = "planet_view".into();
                        self.sel = 0;
                    }
                    Err(e) => self.feedback(false, format!("Gagal load save: {e:?}")),
                },
                "Settings" => self.view = "settings".into(),
                "Help" => self.view = "help".into(),
                "Quit" => {}
                _ => {}
            },
            _ => {}
        }
    }

    /// M15.8: **bug real ditemukan** — view "research" TAK PUNYA key handler SAMA SEKALI sblm
    /// ini (dicek `handle_key`'s if-else chain: cuma planet_view/galaxy_map disebut). Footer
    /// research view "[j/k]Pilih" (sudah ada sejak M15.1) 100% FAKE — j/k/Enter genuinely
    /// no-op total, pola SAMA kelas M14.7's "3 keybind fake no-op". j/k/panah skrng genuinely
    /// gerakkan `app.sel` (field SAMA dipakai planet_view/galaxy_map — bukan field baru),
    /// diklem ke total tech (SAMA count `panels::research::all_techs_ordered` pakai, tp
    /// dihitung LANGSUNG di sini krn fungsi itu private ke modul panel).
    fn research_keys(&mut self, code: KeyCode) {
        let count = self.content.techs.iter().count();
        match code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.sel = (self.sel + 1).min(count.saturating_sub(1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.sel = self.sel.saturating_sub(1);
            }
            // M15.9: "aksi mulai riset terpilih + feedback" — tech id diambil dari `all_techs_
            // ordered` (SAMA urutan dipakai render/selektor, M15.4/8), `start_research` dipanggil
            // LANGSUNG (bukan reka mekanik baru, fungsi ini sudah ada+dites `game::research`).
            // Err (mis. riset lain msh aktif, prereq blm selesai) DIABAIKAN senyap — pola SAMA
            // PERSIS `planet_keys`'s upgrade/build (`let _ = ...`) — precondition (status marker
            // `[--]`/`[OK]`/`[>>]`/`[DN]`, M15.2) SUDAH terlihat SEBELUM Enter ditekan, ACTIVE
            // section (M15.3) genuinely update SEKETIKA kalau sukses — feedback via state
            // langsung, bukan sistem toast baru (pola sama M14.15's "sudah terpenuhi").
            KeyCode::Enter => {
                let ordered = crate::ui::panels::research::all_techs_ordered(&self.content);
                if let Some(t) = ordered.get(self.sel.min(ordered.len().saturating_sub(1))) {
                    let id = t.id.clone();
                    match crate::game::research::start_research(&mut self.state, &self.content, &id)
                    {
                        Ok(()) => {
                            self.feedback(true, format!("Riset dimulai: {id}"));
                            self.celebrate();
                        }
                        Err(e) => self.feedback(false, format!("Gagal mulai riset: {e:?}")),
                    }
                }
            }
            _ => {}
        }
    }

    /// M18.4: Navigasi Galaxy Map — kursor BEBAS `galaxy_cursor` gerak per-sel h/j/k/l +
    /// panah (BUKAN lompat antar planet spt `sel` dulu — kursor BISA berdiri di sel kosong).
    /// `s` = scan/materialisasi planet DI POSISI KURSOR (via `planet_idx_at`), no-op bila
    /// kursor di sel kosong (tak ada planet utk discan situ).
    fn galaxy_map_keys(&mut self, code: KeyCode) {
        // M20.8 follow-up (galaxy_sim Phase 6): mode `Backdrop` py KEYSET SENDIRI (pan/zoom/
        // reset kamera), TERPISAH TOTAL dr Starmap's hjkl-cursor-movement -- dicek PALING
        // AWAL + `return` (bukan lanjut ke match Starmap di bawah), pola SAMA "1 mode fokus
        // per waktu" render()'s precedent (M20.1/M14.12). `h/j/k/l`+panah DIPAKAI ULANG utk
        // pan (aman krn Starmap's cursor logic di-skip TOTAL saat Backdrop aktif, TAK ADA
        // konflik genuine -- beda tombol drpd yg SAMA dipakai 2 arti berbeda tergantung mode).
        if self.galaxy_map_mode == crate::ui::panels::galaxy_map::GalaxyMapMode::Backdrop {
            const PAN_STEP: f32 = 4.0;
            match code {
                // M20.1's Tab-toggle-balik-ke-Starmap HARUS TETAP jalan di sini -- letaknya
                // ASLI di match Starmap-scoped DI BAWAH (skrng genuinely unreachable krn
                // `return` ini) -- **bug ditemukan LANGSUNG via test SUDAH ADA**
                // (`tab_key_toggles_galaxy_map_mode_only_in_galaxy_map_view`, gagal PERSIS
                // krn ini SEBELUM diperbaiki), bukan diasumsikan aman.
                KeyCode::Tab => self.galaxy_map_mode = self.galaxy_map_mode.toggle(),
                // `Esc` global (`view="main_menu"`) jg unreachable skrng (guard baru di
                // `handle_key` skip match global TOTAL saat Backdrop aktif) -- HARUS
                // dipertahankan genuine di sini, bukan cuma Tab. M20.8 follow-up Phase 7:
                // `Esc` SAAT fokus POI aktif cuma batalkan fokus dulu (kembali Overview),
                // BARU `Esc` KEDUA keluar view -- pola umum "escape bertingkat" (mis. modal
                // sblm keluar halaman), bukan diam2 loncat 2 state sekaligus.
                KeyCode::Esc => {
                    if self.galaxy_sim.is_focused() {
                        self.galaxy_sim.unfocus();
                    } else {
                        self.view = "main_menu".into();
                    }
                }
                // Phase 7: `Enter` siklus fokus POI (Overview -> POI1 -> POI2 -> POI3 ->
                // Overview) -- dekoratif murni, TAK terikat mekanik `s`/aksi Starmap lain.
                KeyCode::Enter => self.galaxy_sim.focus_next(),
                KeyCode::Char('h') | KeyCode::Left => self.galaxy_sim.pan(-PAN_STEP, 0.0),
                KeyCode::Char('l') | KeyCode::Right => self.galaxy_sim.pan(PAN_STEP, 0.0),
                KeyCode::Char('k') | KeyCode::Up => self.galaxy_sim.pan(0.0, -PAN_STEP),
                KeyCode::Char('j') | KeyCode::Down => self.galaxy_sim.pan(0.0, PAN_STEP),
                KeyCode::Char('z') => self.galaxy_sim.zoom_by(1.2),
                KeyCode::Char('x') => self.galaxy_sim.zoom_by(1.0 / 1.2),
                KeyCode::Char('r') => self.galaxy_sim.reset_view(),
                _ => {}
            }
            return;
        }
        let Some(fg) = crate::ui::panels::galaxy_map::frontier(self) else {
            return;
        };
        let fid = fg.id;
        let (x, y) = self.galaxy_cursor;
        match code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.galaxy_cursor = (x, (y + 1).min(crate::ui::panels::galaxy_map::WORLD_H - 1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.galaxy_cursor = (x, (y - 1).max(0));
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.galaxy_cursor = ((x - 1).max(0), y);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.galaxy_cursor = ((x + 1).min(crate::ui::panels::galaxy_map::WORLD_W - 1), y);
            }
            KeyCode::Char('s') => {
                if let Some(idx) = crate::ui::panels::galaxy_map::planet_idx_at(fg, (x, y))
                    && let Some(g) = self.state.galaxies.iter_mut().find(|g| g.id == fid)
                {
                    crate::game::world::procgen::materialize_planet(g, idx, &self.content);
                }
            }
            // M19.6: "Aksi Enter masuk planet/sistem terpilih" — planet DI POSISI KURSOR harus
            // genuinely `unlocked` (ship SUDAH tiba, `sim/tick.rs`'s `advance_travel` yg set
            // `true` -- planet blm-unlocked/blm-materialize tak bisa dimasuki, no-op senyap,
            // pola SAMA `s`). Dipakai `state.active_galaxy` [field mekanik SUDAH ADA, dibedakan
            // dari `anchor_galaxy` di `prestige.rs` -- BUKAN state UI baru, "masuk sistem" =
            // genuinely ganti sistem AKTIF, bukan cuma overlay tampilan] BUKAN field baru --
            // `active_pi()` (dipakai `planet_view`) otomatis ikut planet pertama unlocked di
            // galaxy BARU ini stlh pindah.
            KeyCode::Enter => {
                if let Some(idx) = crate::ui::panels::galaxy_map::planet_idx_at(fg, (x, y))
                    && let Some(p) = crate::ui::panels::galaxy_map::planet_by_idx(fg, idx)
                    && p.unlocked
                {
                    self.state.active_galaxy = fid;
                    self.view = "planet_view".into();
                }
            }
            // M19.7: "Aksi kirim ship ke tile (set Traveling)" — **gap real ditemukan**:
            // `game::ship::travel_to` SUDAH ADA+dites [`ship.rs`] TAPI TAK PERNAH dipanggil dr
            // UI manapun (dikonfirmasi grep: cuma dipakai test-nya sendiri) -- wiring pertama
            // ke input nyata. `travel_to(state,content,pi)` butuh `pi` = POSISI planet di
            // `state.active_galaxy`'s planets vec (BUKAN idx grid) -- kalau frontier != active
            // galaxy [kasus umum, planet BARU discan blm pernah "dimasuki"], `active_galaxy`
            // di-set ke frontier DULU [pola SAMA `Enter`/M19.6, field mekanik SUDAH ADA] biar
            // `travel_to` genuinely bisa temukan planet ini. Kegagalan internal `travel_to`
            // [ship sibuk/locked/WarpTier kurang] DIABAIKAN senyap -- pola SAMA `research_
            // keys`'s Enter [M15.9] & `s`'s scan, precondition/status SUDAH kelihatan di
            // SELECTED TILE (M19.1) & footer (M19.5) sblm `t` ditekan.
            KeyCode::Char('t') => {
                if let Some(idx) = crate::ui::panels::galaxy_map::planet_idx_at(fg, (x, y))
                    && let Some(p) = crate::ui::panels::galaxy_map::planet_by_idx(fg, idx)
                    && !p.unlocked
                {
                    self.state.active_galaxy = fid;
                    if let Some(gi) = self.state.galaxies.iter().position(|g| g.id == fid)
                        && let Some(pi) = self.state.galaxies[gi]
                            .planets
                            .iter()
                            .position(|p| p.id == crate::game::state::PlanetId(idx))
                    {
                        match crate::game::ship::travel_to(&mut self.state, &self.content, pi) {
                            Ok(()) => {
                                self.feedback(true, "Ship berangkat travel.");
                                self.celebrate();
                            }
                            Err(e) => self.feedback(false, format!("Travel gagal: {e:?}")),
                        }
                    }
                }
            }
            // M19.9: "Filter (faksi/tile) toggle" — game TAK PY faksi (dikonfirmasi berulang
            // sesi ini), diganti filter STATUS tile. Murni VISUAL (`galaxy_filter` cuma dibaca
            // `galaxy_map.rs`'s render, TAK mempengaruhi `can_scan`/`can_send`/`can_enter` atau
            // aksi lain -- kursor tetap bebas gerak ke tile "tersembunyi", aksi tetap normal).
            KeyCode::Char('f') => {
                self.galaxy_filter = self.galaxy_filter.next();
            }
            // M20.1: "Mode backdrop vs starmap (tab/key) dalam galaxy view" -- toggle
            // Starmap<->Backdrop DALAM view ini (bukan pindah view lain), murni tampilan.
            KeyCode::Tab => {
                self.galaxy_map_mode = self.galaxy_map_mode.toggle();
            }
            _ => {}
        }
    }

    /// Navigasi & aksi pada planet_view: j/k pilih slot, 1 upgrade node, 2/Enter upgrade
    /// factory, 3 buka build picker (M14.12) bila slot terpilih kosong.
    fn planet_keys(&mut self, code: KeyCode) {
        let Some((gi, pi)) = self.active_pi() else {
            return;
        };
        let slots = self.state.galaxies[gi].planets[pi].factory_slots.len();
        let nodes = self.state.galaxies[gi].planets[pi].nodes.len();
        let slot_empty = self.state.galaxies[gi].planets[pi].factory_slots[self.sel].is_none();
        match code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.sel = (self.sel + 1).min(slots.saturating_sub(1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.sel = self.sel.saturating_sub(1);
            }
            KeyCode::Char('2') | KeyCode::Enter => {
                match crate::game::actions::upgrade_factory(
                    &mut self.state,
                    &self.content,
                    pi,
                    self.sel,
                ) {
                    Ok(()) => {
                        self.feedback(true, "Factory di-upgrade.");
                        self.celebrate();
                    }
                    Err(e) => self.feedback(false, format!("Upgrade gagal: {e:?}")),
                }
            }
            KeyCode::Char('1') if nodes > 0 => {
                let node = self.sel.min(nodes - 1);
                match crate::game::actions::upgrade_node(&mut self.state, pi, node) {
                    Ok(()) => {
                        self.feedback(true, "Node di-upgrade.");
                        self.celebrate();
                    }
                    Err(e) => self.feedback(false, format!("Upgrade node gagal: {e:?}")),
                }
            }
            KeyCode::Char('3') if slot_empty => {
                self.build_picker = Some(BuildPicker {
                    slot: self.sel,
                    sel: 0,
                });
            }
            _ => {}
        }
    }

    /// M14.12: navigasi + konfirmasi/batal build picker. j/k pilih kandidat building
    /// (`actions::buildable_options`), Enter bangun (`actions::build_picked`) lalu tutup
    /// picker (sukses ATAU gagal — error `build_picked` diabaikan sama pola `upgrade_node`/
    /// `upgrade_factory`, konsisten "coba, kalau gagal ya gagal senyap", BUKAN pola baru),
    /// Esc batal tanpa aksi.
    fn build_picker_keys(&mut self, code: KeyCode) {
        let Some(picker) = self.build_picker else {
            return;
        };
        let Some((_, pi)) = self.active_pi() else {
            self.build_picker = None;
            return;
        };
        let options = crate::game::actions::buildable_options(&self.state, &self.content, pi);
        match code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.build_picker = Some(BuildPicker {
                    sel: (picker.sel + 1).min(options.len().saturating_sub(1)),
                    ..picker
                });
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.build_picker = Some(BuildPicker {
                    sel: picker.sel.saturating_sub(1),
                    ..picker
                });
            }
            KeyCode::Enter if !options.is_empty() => {
                let building = options[picker.sel.min(options.len() - 1)];
                let name = self.content.buildings.get(building.0).name.clone();
                match crate::game::actions::build_picked(
                    &mut self.state,
                    &self.content,
                    pi,
                    picker.slot,
                    building,
                ) {
                    Ok(()) => {
                        self.feedback(true, format!("Dibangun: {name}"));
                        self.celebrate();
                    }
                    Err(e) => self.feedback(false, format!("Gagal membangun {name}: {e:?}")),
                }
                self.build_picker = None;
            }
            KeyCode::Esc => {
                self.build_picker = None;
            }
            _ => {}
        }
    }

    /// Simpan state SAAT INI ke `save::save_path()` (XDG). Dipakai autosave berkala, quit
    /// (`q`/Ctrl-C), dan aksi manual (`Ctrl+S`, dicek LANGSUNG di `event_loop` sblm dispatch
    /// `handle_key` -- `handle_key` sendiri tak terima modifier, pola SAMA quit's `Ctrl+C`
    /// yg sudah di-cek di situ, bukan lewat `handle_key`).
    pub fn manual_save(&self) -> Result<(), crate::save::SaveError> {
        crate::save::write(&self.state, &self.content)
    }

    /// M1#6: catat hasil aksi terbaru (ok/gagal + pesan) utk ditampilkan (baris status/footer
    /// tergantung breakpoint, `ui::mod`'s `feedback_line`). Timpa aksi sblmnya (bukan antrian --
    /// cukup utk "no more silent no-op", history multi-pesan itu scope beda/lbh besar).
    fn feedback(&mut self, ok: bool, message: impl Into<String>) {
        self.last_action = Some((ok, message.into()));
    }

    /// M4: burst partikel Sparkle di tengah panel MAIN — flourish visual dipanggil bareng
    /// `feedback(true, ...)` di tiap aksi sukses (build/upgrade/riset mulai/trade/warp jump).
    /// Reuse `ParticleSystem::burst` yg SUDAH ada (dulu cuma dipakai exhaust ship di
    /// `main_view.rs`) — bukan sistem partikel baru. `ui::mod`'s shell fns skrng meng-overlay
    /// `app.particles.render` di area MAIN tiap breakpoint (dulu cuma di `main_view.rs`, jd
    /// burst di planet_view/research/warp/merchant dulu TAK akan kelihatan sama sekali).
    fn celebrate(&mut self) {
        self.particles.burst(
            crate::ui::particles::ParticleKind::Sparkle,
            (0.5, 0.5),
            18,
            (0.15, 0.35),
        );
    }
}

/// Entry runtime: setup terminal, jalankan event loop, teardown bersih.
pub fn run() {
    let mut app = App::demo();
    // `demo()` deterministik (Tc/halfblocks) — gameplay nyata naikkan ke kapabilitas terminal
    // sungguhan sebelum masuk event loop.
    app.sprites = crate::ui::sprite::Sprites::new(sprites_root());
    app.portraits = crate::ui::portrait::Portraits::detect();
    app.galaxy_pixel = crate::ui::galaxy_pixel::GalaxyPixel::detect();
    app.galaxy_sim = crate::ui::galaxy_sim_view::GalaxySimView::detect();
    // M2: boot ke Splash → Title (New Game/Continue), bukan langsung ke `demo()`'s dashboard
    // `"main_menu"` -- `demo()`'s state scaffolding TETAP dipakai sbg placeholder sblm New
    // Game/Continue nyata dipilih (event_loop's `pre_game` guard cegah tick/save diam2 jalan
    // di atasnya sampai itu terjadi).
    app.view = "splash".into();
    if let Err(e) = run_loop(&mut app) {
        eprintln!("galaxy-idle: error terminal: {e}");
    }
}

fn run_loop(app: &mut App) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let result = event_loop(app, &mut term);

    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen)?;
    term.show_cursor()?;
    result
}

/// Loop dua-frekuensi: simulasi 1 Hz (catch-up dibatasi), render ~13 FPS (`07` §Game Loop).
fn event_loop<B: ratatui::backend::Backend>(
    app: &mut App,
    term: &mut Terminal<B>,
) -> io::Result<()> {
    let tick = Duration::from_secs_f64(TICK_DURATION_SECS);
    let render_interval = Duration::from_millis(RENDER_INTERVAL_MS);
    let mut accumulated = Duration::ZERO;
    let start = Instant::now();
    let mut last = Instant::now();
    let mut last_render = Instant::now() - render_interval;
    let mut last_autosave = Instant::now();
    let mut prev_view = app.view.clone();

    loop {
        if event::poll(Duration::from_millis(10))?
            && let Event::Key(k) = event::read()?
            && k.kind == KeyEventKind::Press
        {
            let quit = matches!(k.code, KeyCode::Char('q'))
                || (k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL));
            if quit {
                // Simpan sblm keluar -- `manual_save` best-effort (gagal simpan saat quit tak
                // py tempat utk ditampilkan lg, TUI sudah mau tutup); pola SAMA autosave. M2:
                // TAK disimpan di Splash/Title -- belum ada sesi game nyata dimulai (New Game/
                // Continue blm dipilih), simpan di sini akan diam2 menimpa save lama dgn state
                // scaffolding `demo()`'s stub (bug real yg dicegah, bukan cuma teoretis).
                if !matches!(app.view.as_str(), "splash" | "title")
                    && let Err(e) = app.manual_save()
                {
                    eprintln!("galaxy-idle: gagal simpan saat keluar: {e}");
                }
                break;
            }
            // `Ctrl+S` simpan manual -- dicek LANGSUNG di sini (bukan `handle_key`, yg tak
            // terima modifier), pola SAMA quit's `Ctrl+C` di atas. Raw mode (`enable_raw_mode`)
            // menonaktifkan flow-control software (IXON/IXOFF) di terminal POSIX, jd `Ctrl+S`
            // genuinely tiba sbg key event normal, bukan ter-freeze XOFF legacy.
            let manual_save =
                k.code == KeyCode::Char('s') && k.modifiers.contains(KeyModifiers::CONTROL);
            if manual_save {
                if let Err(e) = app.manual_save() {
                    eprintln!("galaxy-idle: gagal simpan manual: {e}");
                }
            } else {
                app.handle_key(k.code);
            }
        }

        let now = Instant::now();
        // M2: sim tick + autosave hanya jalan SETELAH New Game/Continue nyata dipilih -- di
        // Splash/Title, `app.state` msh `demo()`'s scaffolding placeholder (blm ada sesi
        // pemain), men-tick/autosave-nya cuma buang kerja + resiko menimpa save lama diam2.
        let pre_game = matches!(app.view.as_str(), "splash" | "title");
        if !pre_game {
            if now.duration_since(last_autosave).as_secs_f64() >= AUTOSAVE_INTERVAL_SECS {
                if let Err(e) = app.manual_save() {
                    eprintln!("galaxy-idle: autosave gagal: {e}");
                }
                last_autosave = now;
            }
            accumulated += now - last;
            let mut ticks: u32 = 0;
            let was_traveling = matches!(
                app.state.ship.status,
                crate::game::state::ShipStatus::Traveling { .. }
            );
            while accumulated >= tick && ticks < MAX_TICKS_PER_FRAME {
                crate::sim::tick::step(&mut app.state, &app.content);
                app.state.tick += 1;
                accumulated -= tick;
                ticks += 1;
            }
            if ticks == MAX_TICKS_PER_FRAME {
                accumulated = Duration::ZERO; // sisa → offline (M7); cegah spiral.
            }
            // M4: flourish "ship arrival" — dites di SINI (event_loop, bukan `sim::tick`) krn
            // `sim::tick::step` sengaja cuma py akses `GameState` (batas "UI tak pernah
            // dimutasi state" -- particles itu sendiri kosmetik `App`-only, bukan gameplay,
            // tp fungsi sim murni tetap tak boleh py ketergantungan ke `App`). Diff status
            // SEBELUM vs SESUDAH batch tick: Traveling->Idle persis 1 frame = tiba.
            let now_idle = matches!(app.state.ship.status, crate::game::state::ShipStatus::Idle);
            if was_traveling && now_idle {
                app.celebrate();
            }
        }
        last = now;

        if now - last_render >= render_interval {
            app.anim_secs = (now - start).as_secs_f64();
            // M4: transition flourish — view berubah sejak frame render terakhir → sulut N
            // frame highlight border (`ui::mod`'s `outer_block`); kalau tidak, hitung mundur.
            // Frame-gated (bukan real-time timer) sesuai plan's "tak perlu timer, cukup gate
            // ke frame/state" -- otomatis nol utk snapshot/screenshot-bin (tak lewat sini).
            if app.view != prev_view {
                app.view_transition_frames = VIEW_TRANSITION_FRAMES;
                prev_view = app.view.clone();
            } else if app.view_transition_frames > 0 {
                app.view_transition_frames -= 1;
            }
            // Engine exhaust aktif saat ship Traveling; update partikel dgn dt nyata.
            let traveling = matches!(
                app.state.ship.status,
                crate::game::state::ShipStatus::Traveling { .. }
            );
            app.particles
                .set_emitter(app.exhaust_idx, traveling, (0.5, 0.6));
            app.particles.update((now - last_render).as_secs_f32());
            let view = app.view.clone();
            term.draw(|f| crate::ui::draw_view(f, app, &view))?;
            last_render = now;
        }
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn any_key_on_splash_advances_to_title_not_a_global_shortcut() {
        // M2 item 1: pastikan guard splash BENAR-BENAR mendahului match global -- tombol `g`
        // (yg biasanya lompat ke galaxy_map) di splash HARUS ke "title", bukan "galaxy_map".
        let mut app = App::demo();
        app.view = "splash".into();
        app.handle_key(KeyCode::Char('g'));
        assert_eq!(app.view, "title");
    }

    /// `XDG_DATA_HOME` is process-global — `cargo test` runs threads in parallel by default,
    /// so any two tests touching it race each other (found the hard way: these title tests
    /// flaked against each other + `manual_save_writes_and_roundtrips` until serialized behind
    /// this lock). Holds the lock for the whole closure so `save_path()` stays stable across
    /// all of `f()`, not just the env-var mutation itself.
    ///
    /// Dir dikunci per-PANGGILAN (counter), BUKAN per-thread: libtest pakai thread pool dan
    /// MEMAKAI ULANG thread utk test berikutnya, jadi keying by `ThreadId` bikin dua test
    /// berbagi dir — `save.json` sisa test sebelumnya bocor ke test yg justru butuh dir KOSONG
    /// (`title_new_game_without_existing_save_starts_immediately` flake persis krn ini). Dir
    /// dihapus sebelum + sesudah `f()` biar tiap panggilan mulai dari nol.
    pub(crate) fn with_temp_save_dir<T>(f: impl FnOnce() -> T) -> T {
        static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("galaxy_idle_test_title_{}_{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let prev = std::env::var_os("XDG_DATA_HOME");
        unsafe { std::env::set_var("XDG_DATA_HOME", &dir) };
        let r = f();
        match prev {
            Some(v) => unsafe { std::env::set_var("XDG_DATA_HOME", v) },
            None => unsafe { std::env::remove_var("XDG_DATA_HOME") },
        }
        let _ = std::fs::remove_dir_all(&dir);
        r
    }

    #[test]
    fn title_new_game_without_existing_save_starts_immediately() {
        with_temp_save_dir(|| {
            let mut app = App::demo();
            app.view = "title".into();
            app.title_sel = 0; // "New Game"
            app.handle_key(KeyCode::Enter);
            assert!(
                !app.new_game_confirm,
                "tanpa save lama, tak perlu modal konfirmasi"
            );
            assert_eq!(app.view, "planet_view");
            // World nyata (Sol System, 24 body) — BUKAN demo()'s 1-planet stub.
            assert_eq!(app.state.galaxies[0].planets.len(), 24);
        });
    }

    #[test]
    fn title_new_game_with_existing_save_shows_confirm_modal_first() {
        with_temp_save_dir(|| {
            let seed_app = App::demo();
            seed_app.manual_save().unwrap();
            let mut app = App::demo();
            app.view = "title".into();
            app.title_sel = 0;
            app.handle_key(KeyCode::Enter);
            assert!(
                app.new_game_confirm,
                "save sudah ada — harus tampilkan modal dulu"
            );
            assert_eq!(app.view, "title", "belum pindah view sblm konfirmasi");
            // Esc batalkan tanpa menimpa apa pun.
            app.handle_key(KeyCode::Esc);
            assert!(!app.new_game_confirm);
            assert_eq!(app.view, "title");
        });
    }

    #[test]
    fn title_new_game_confirm_enter_overwrites_and_starts() {
        with_temp_save_dir(|| {
            let seed_app = App::demo();
            seed_app.manual_save().unwrap();
            let mut app = App::demo();
            app.view = "title".into();
            app.new_game_confirm = true;
            app.handle_key(KeyCode::Enter);
            assert!(!app.new_game_confirm);
            assert_eq!(app.view, "planet_view");
        });
    }

    #[test]
    fn title_continue_without_save_sets_visible_failure_feedback() {
        with_temp_save_dir(|| {
            let mut app = App::demo();
            app.view = "title".into();
            app.title_sel = 1; // "Continue"
            app.handle_key(KeyCode::Enter);
            assert_eq!(app.view, "title", "gagal load tetap di Title, tak panic");
            assert_eq!(app.last_action.as_ref().map(|(ok, _)| *ok), Some(false));
        });
    }

    #[test]
    fn title_continue_with_real_save_loads_it() {
        with_temp_save_dir(|| {
            let mut seed_app = App::demo();
            seed_app.state.credits = 4242.0;
            seed_app.manual_save().unwrap();
            let mut app = App::demo();
            app.view = "title".into();
            app.title_sel = 1;
            app.handle_key(KeyCode::Enter);
            assert_eq!(app.view, "planet_view");
            assert_eq!(app.state.credits, 4242.0);
        });
    }

    #[test]
    fn title_settings_and_help_navigate_to_real_views() {
        let mut app = App::demo();
        app.view = "title".into();
        app.title_sel = 2;
        app.handle_key(KeyCode::Enter);
        assert_eq!(app.view, "settings");

        app.view = "title".into();
        app.title_sel = 3;
        app.handle_key(KeyCode::Enter);
        assert_eq!(app.view, "help");
    }

    #[test]
    fn t_key_dismisses_hint_only_for_current_step_not_forever() {
        let mut app = App::demo();
        app.state.tutorial_step = Some(0);
        app.handle_key(KeyCode::Char('T'));
        assert_eq!(app.tutorial_hint_dismissed_for, Some(0));
        // Step maju (mis. extractor jadi ada) -- dismiss lama TAK berlaku utk step baru.
        app.state.tutorial_step = Some(1);
        assert_ne!(app.tutorial_hint_dismissed_for, Some(1));
    }

    #[test]
    fn warp_1_key_actually_triggers_warp_jump_not_a_dead_hint() {
        // M4 fix: dulu `handle_key` TAK PERNAH dispatch ke view "warp" -- `1` genuinely no-op.
        let mut app = App::demo();
        app.view = "warp".into();
        app.handle_key(KeyCode::Char('1'));
        // `demo()`'s stockpile tak penuhi threshold Lvl 1 -> gagal REAL (bukan sukses palsu),
        // tp WAJIB ada feedback visible (bukan diam2 lagi).
        assert!(
            app.last_action.is_some(),
            "harus ada feedback, sukses atau gagal"
        );
    }

    #[test]
    fn warp_2_key_returns_to_main_menu() {
        let mut app = App::demo();
        app.view = "warp".into();
        app.handle_key(KeyCode::Char('2'));
        assert_eq!(app.view, "main_menu");
    }

    #[test]
    fn celebrate_spawns_visible_particles() {
        let mut app = App::demo();
        assert!(app.particles.is_empty());
        app.celebrate();
        assert!(
            !app.particles.is_empty(),
            "burst harus genuinely spawn partikel"
        );
    }

    #[test]
    fn manual_save_writes_and_roundtrips() {
        with_temp_save_dir(|| {
            let app = App::demo();
            app.manual_save().unwrap();
            let path = crate::save::save_path();
            assert!(path.exists());
            let loaded = crate::save::read(&app.content).unwrap();
            assert_eq!(loaded.tick, app.state.tick);
            assert_eq!(loaded.version, app.state.version);
        });
    }

    #[test]
    fn jk_navigation_clamps() {
        let mut app = App::demo();
        app.view = "planet_view".into();
        assert_eq!(app.sel, 0);
        app.handle_key(KeyCode::Char('j'));
        assert_eq!(app.sel, 1);
        for _ in 0..10 {
            app.handle_key(KeyCode::Char('j'));
        }
        assert_eq!(app.sel, 3); // 4 slot → indeks maks 3.
        app.handle_key(KeyCode::Char('k'));
        assert_eq!(app.sel, 2);
    }

    /// M15.8: **bug real ditemukan** — view "research" TAK PUNYA key handler sama sekali sblm
    /// ini, footer "[j/k]Pilih" (ADA sejak M15.1) 100% FAKE. Dites LANGSUNG: j/k genuinely
    /// gerakkan `app.sel`, diklem ke total tech demo (7: manu_steel/manu_titanium/ext_deep_core/
    /// ext_efficiency/aero_warp_mk2/aero_warp_mk3/astro_far_warp → indeks maks 6).
    #[test]
    fn research_jk_navigation_moves_and_clamps_selection() {
        let mut app = App::demo();
        app.view = "research".into();
        assert_eq!(app.sel, 0);
        app.handle_key(KeyCode::Char('j'));
        assert_eq!(app.sel, 1);
        for _ in 0..10 {
            app.handle_key(KeyCode::Char('j'));
        }
        assert_eq!(app.sel, 6); // 7 tech demo -> indeks maks 6.
        app.handle_key(KeyCode::Char('k'));
        assert_eq!(app.sel, 5);
    }

    /// M15.9: "aksi mulai riset terpilih" — Enter genuinely panggil `research::start_research`
    /// pd tech terpilih (`all_techs_ordered`'s indeks `sel`, urutan SAMA persis selektor/DETAIL).
    /// Dites LANGSUNG: kosongkan `active` dulu (demo default py `manu_steel` aktif, akan blokir
    /// start baru), navigasi ke `ext_efficiency` (indeks 1, `depends_on` kosong -> Available),
    /// Enter, konfirmasi `state.research.active` genuinely jd tech itu.
    #[test]
    fn research_enter_starts_selected_tech() {
        let mut app = App::demo();
        app.view = "research".into();
        app.state.research.active = None;

        app.handle_key(KeyCode::Char('j')); // sel=1 -> ext_efficiency (Available).
        app.handle_key(KeyCode::Enter);

        assert_eq!(
            app.state
                .research
                .active
                .as_ref()
                .map(|a| a.tech_id.as_str()),
            Some("ext_efficiency"),
            "Enter pd tech Available harus genuinely mulai riset tech ITU (bukan tech lain)"
        );
    }

    /// M15.9/M1#6: precondition (research LAIN msh aktif) SUDAH terlihat lwt status marker
    /// `[--]`/`[>>]` (M15.2) SEBELUM Enter ditekan; Enter pd tech tak eligible (mis. Locked,
    /// ATAU riset lain msh aktif) tak boleh ganti STATE (`active` lama tetap) — TAPI M1#6 fix:
    /// harus TETAP tampilkan feedback GAGAL yg visible (`app.last_action`), bukan lagi no-op
    /// TOTAL tanpa sinyal sama sekali (gap real: dulu genuinely `let _ = ...`, player tak tau
    /// keypress-nya ditolak atau tak berbuat apa-apa).
    #[test]
    fn research_enter_on_ineligible_tech_is_safe_noop() {
        let mut app = App::demo();
        app.view = "research".into();
        // demo default: manu_steel SUDAH aktif. sel=0 -> ext_deep_core (Locked, butuh manu_steel
        // SELESAI, msh Active bukan Done) — Enter di sini harus no-op senyap, `active` tak berubah.
        app.handle_key(KeyCode::Enter);

        assert_eq!(
            app.state
                .research
                .active
                .as_ref()
                .map(|a| a.tech_id.as_str()),
            Some("manu_steel"),
            "Enter pd tech Locked/ineligible harus no-op senyap, `active` LAMA tak boleh berubah"
        );
        assert_eq!(
            app.last_action.as_ref().map(|(ok, _)| *ok),
            Some(false),
            "gagal HARUS genuinely tercatat sbg feedback visible, bukan diabaikan total"
        );
    }

    #[test]
    fn research_enter_success_sets_visible_positive_feedback() {
        let mut app = App::demo();
        app.view = "research".into();
        app.state.research.active = None;
        app.handle_key(KeyCode::Char('j')); // sel=1 -> ext_efficiency (Available).
        app.handle_key(KeyCode::Enter);
        assert_eq!(
            app.last_action.as_ref().map(|(ok, _)| *ok),
            Some(true),
            "sukses HARUS genuinely tercatat sbg feedback visible"
        );
    }

    /// M1#6: merchant Enter tanpa offer valid (stock kosong, `active=false` demo default) harus
    /// genuinely gagal DENGAN feedback visible -- dulu `let _ = accept_merchant_offer(...)`.
    #[test]
    fn merchant_enter_without_active_offer_sets_visible_failure_feedback() {
        let mut app = App::demo();
        app.view = "merchant".into();
        assert!(!app.state.merchant.active);
        app.handle_key(KeyCode::Enter);
        assert_eq!(
            app.last_action.as_ref().map(|(ok, _)| *ok),
            Some(false),
            "merchant inactive -> Enter harus genuinely gagal + tercatat visible"
        );
    }

    #[test]
    fn merchant_enter_accepts_real_offer_sets_visible_success_feedback() {
        let mut app = App::demo();
        crate::game::events::restock_merchant(
            &mut app.state.merchant,
            &app.content,
            app.state.tick,
        );
        app.view = "merchant".into();
        app.sel = 0;
        app.handle_key(KeyCode::Enter);
        assert_eq!(
            app.last_action.as_ref().map(|(ok, _)| *ok),
            Some(true),
            "offer valid pertama HARUS genuinely diterima + tercatat visible"
        );
    }

    /// M14.14: `jk_navigation_clamps` di atas cuma dites BATAS ATAS (`min`) — batas BAWAH
    /// (`saturating_sub`, k di sel=0) TAK PERNAH dites eksplisit (aman by construction krn
    /// `usize` tak bisa underflow, tp "aman krn tipe data" beda dari "dikonfirmasi behaviour
    /// nyata benar"). Dites lgs: `k` berulang di sel=0 HARUS tetap 0, tak panic/wrap negatif.
    #[test]
    fn k_navigation_stays_clamped_at_lower_bound() {
        let mut app = App::demo();
        app.view = "planet_view".into();
        assert_eq!(app.sel, 0);
        for _ in 0..10 {
            app.handle_key(KeyCode::Char('k'));
        }
        assert_eq!(
            app.sel, 0,
            "k berulang di sel=0 harus tetap 0, tak underflow"
        );
    }

    #[test]
    fn key_upgrades_selected_factory() {
        let mut app = App::demo();
        app.view = "planet_view".into();
        app.state.credits = 1000.0;
        app.sel = 1; // slot steel_mill_bld L2.
        app.handle_key(KeyCode::Char('2'));
        let lvl = app.state.galaxies[0].planets[0].factory_slots[1]
            .unwrap()
            .level;
        assert_eq!(lvl, 3);
        assert!(app.state.credits < 1000.0); // biaya terpotong.
    }

    /// M14.14: `app.sel` SATU cursor dipakai bareng utk NODES (`.min(nodes-1)` re-clamp, tombol
    /// `1`) DAN FACTORY SLOTS (langsung, tombol `2`) — demo Earth py 3 node vs 4 slot (PANJANG
    /// BEDA). Dicek KONSISTEN: node yg di-upgrade tombol `1` di `sel` MANA PUN HARUS SELALU
    /// PERSIS node yg ditandai `►` di `nodes_lines` (M14.3's clamp formula) — TAK PERNAH
    /// upgrade node "salah" krn sel > nodes.len()-1. Dites SELURUH rentang sel 0..slots (bukan
    /// cuma 1 titik) — 4 iterasi, tiap kali reset credits+konfirmasi node target PERSIS
    /// `sel.min(nodes-1)`, node LAIN tak ikut ter-upgrade.
    #[test]
    fn node_upgrade_target_matches_visual_marker_across_full_sel_range() {
        let nodes_len = 3usize; // Iron/Carbon/Water (demo Earth).
        for sel in 0..4usize {
            let mut app = App::demo();
            app.view = "planet_view".into();
            app.state.credits = 1_000_000.0;
            app.sel = sel;
            let before: Vec<u32> = app.state.galaxies[0].planets[0]
                .nodes
                .iter()
                .map(|n| n.level)
                .collect();
            app.handle_key(KeyCode::Char('1'));
            let after: Vec<u32> = app.state.galaxies[0].planets[0]
                .nodes
                .iter()
                .map(|n| n.level)
                .collect();
            let expected_target = sel.min(nodes_len - 1);
            for (i, (b, a)) in before.iter().zip(after.iter()).enumerate() {
                if i == expected_target {
                    assert_eq!(
                        *a,
                        b + 1,
                        "sel={sel}: node {i} (target diharapkan) harus naik level"
                    );
                } else {
                    assert_eq!(
                        a, b,
                        "sel={sel}: node {i} (BUKAN target) TAK boleh ikut ter-upgrade"
                    );
                }
            }
        }
    }

    /// M14.14: simetris dgn `node_upgrade_target_matches_visual_marker_across_full_sel_range` —
    /// FACTORY slot (tombol `2`) TAK re-clamp (`sel` LANGSUNG, beda dari node), jadi konsisten
    /// LEBIH sederhana dites: `sel` MANA PUN (slot terisi) HARUS upgrade slot PERSIS itu, slot
    /// LAIN tak ikut. Slot 2 (Empty) dilewati (upgrade_factory genuinely no-op di slot kosong,
    /// diverifikasi TERPISAH `key_upgrades_selected_factory`/`build_picker` test).
    #[test]
    fn factory_upgrade_target_matches_visual_marker_across_full_sel_range() {
        for sel in [0usize, 1, 3] {
            let mut app = App::demo();
            app.view = "planet_view".into();
            app.state.credits = 1_000_000.0;
            app.sel = sel;
            let before: Vec<Option<u32>> = app.state.galaxies[0].planets[0]
                .factory_slots
                .iter()
                .map(|s| s.map(|f| f.level))
                .collect();
            app.handle_key(KeyCode::Char('2'));
            let after: Vec<Option<u32>> = app.state.galaxies[0].planets[0]
                .factory_slots
                .iter()
                .map(|s| s.map(|f| f.level))
                .collect();
            for (i, (b, a)) in before.iter().zip(after.iter()).enumerate() {
                if i == sel {
                    assert_eq!(
                        *a,
                        b.map(|v| v + 1),
                        "sel={sel}: slot {i} (target) harus naik level"
                    );
                } else {
                    assert_eq!(a, b, "sel={sel}: slot {i} (BUKAN target) TAK boleh berubah");
                }
            }
        }
    }

    /// M14.12: slot 2 (demo Earth) EMPTY (Mining Drill=0, Steel Mill=1, Empty=2, Research
    /// Lab=3) — tekan `3` di slot kosong HARUS buka picker (`build_picker` jd `Some`).
    #[test]
    fn build_picker_opens_on_empty_slot() {
        let mut app = App::demo();
        app.view = "planet_view".into();
        app.sel = 2;
        assert!(app.build_picker.is_none());
        app.handle_key(KeyCode::Char('3'));
        let picker = app.build_picker.expect("slot kosong harus buka picker");
        assert_eq!(picker.slot, 2);
        assert_eq!(picker.sel, 0);
    }

    /// M14.12: tombol `3` di slot TERISI (Mining Drill, slot 0) TAK boleh buka picker — guard
    /// `slot_empty` di `planet_keys` harus mencegah, sama disiplin M14.7 (aksi cuma match kondisi
    /// nyata, bukan selalu aktif).
    #[test]
    fn build_picker_does_not_open_on_occupied_slot() {
        let mut app = App::demo();
        app.view = "planet_view".into();
        app.sel = 0;
        app.handle_key(KeyCode::Char('3'));
        assert!(
            app.build_picker.is_none(),
            "slot terisi tak boleh buka build picker"
        );
    }

    /// M14.12: navigasi j/k di picker geser `sel` (bukan `app.sel` utama — dites picker.sel
    /// terpisah). Esc batal TANPA membangun apa pun DAN tak lompat ke main_menu (dicek
    /// `handle_key`'s intercept `if self.build_picker.is_some()` bekerja sblm match global Esc).
    #[test]
    fn build_picker_navigation_and_cancel() {
        let mut app = App::demo();
        app.view = "planet_view".into();
        app.sel = 2;
        app.handle_key(KeyCode::Char('3'));
        app.handle_key(KeyCode::Char('j'));
        assert_eq!(app.build_picker.unwrap().sel, 1, "j harus geser sel picker");
        app.handle_key(KeyCode::Char('k'));
        assert_eq!(app.build_picker.unwrap().sel, 0, "k harus geser balik");

        app.handle_key(KeyCode::Esc);
        assert!(app.build_picker.is_none(), "Esc harus batalkan picker");
        assert_eq!(
            app.view, "planet_view",
            "Esc saat picker aktif TAK boleh lompat ke main_menu"
        );
        assert!(
            app.state.galaxies[0].planets[0].factory_slots[2].is_none(),
            "batal (Esc) TAK boleh membangun apa pun"
        );
    }

    /// M14.12: Enter dgn credits cukup HARUS genuinely membangun (slot 2 terisi factory baru),
    /// potong credits, tutup picker. Dites end-to-end (bukan cuma unit `build_picked` terpisah)
    /// utk buktikan `handle_key` routing+picker+actions genuinely terhubung.
    #[test]
    fn build_picker_confirms_and_builds() {
        let mut app = App::demo();
        app.view = "planet_view".into();
        app.state.credits = 100_000.0;
        app.sel = 2;
        app.handle_key(KeyCode::Char('3'));
        app.handle_key(KeyCode::Enter);

        assert!(app.build_picker.is_none(), "picker harus tutup stlh bangun");
        assert!(
            app.state.galaxies[0].planets[0].factory_slots[2].is_some(),
            "slot 2 harus terisi factory baru stlh Enter"
        );
        assert!(app.state.credits < 100_000.0, "credits harus terpotong");
    }

    #[test]
    fn demo_holds_sprites_lazily_empty_until_rendered() {
        let app = App::demo();
        // Lazy: belum ada render → cache kosong (tak parse .ans di awal, cuma saat dipakai).
        assert!(app.sprites.is_empty());
    }

    #[test]
    fn demo_sprites_depth_is_deterministic_tc() {
        use crate::ui::sprite::ColorDepth;
        // App::demo() harus set depth tetap (bukan deteksi terminal) → stabil lintas mesin/CI.
        let app = App::demo();
        assert_eq!(app.sprites.depth(), ColorDepth::Tc);
    }

    #[test]
    fn sprites_field_overridable_with_explicit_depth_for_test() {
        use crate::ui::sprite::{ColorDepth, Sprites};
        let mut app = App::demo();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/sprites");
        app.sprites = Sprites::with_depth(root, ColorDepth::C16);
        assert_eq!(app.sprites.depth(), ColorDepth::C16);
    }

    #[test]
    fn demo_builds_portraits_without_panic_headless() {
        // Portraits::detect() query stdio lalu fallback aman bila gagal/headless — tak boleh
        // panic/hang saat App::demo() dipanggil dari test runner (tanpa tty interaktif nyata).
        let app = App::demo();
        assert!(app.portraits.is_empty());
    }

    #[test]
    fn portraits_field_overridable_with_halfblocks_for_test() {
        use crate::ui::portrait::Portraits;
        let mut app = App::demo();
        app.portraits = Portraits::halfblocks();
        assert!(app.portraits.is_empty());
    }

    #[test]
    fn sprite_for_biome_maps_all_variants_distinctly() {
        let all = [
            Biome::IronWorld,
            Biome::OceanPlanet,
            Biome::GasGiant,
            Biome::DeadWorld,
            Biome::CrystalWorld,
            Biome::Terran,
            Biome::AsteroidBelt,
            Biome::IceWorld,
            Biome::LavaWorld,
        ];
        let names: Vec<&str> = all.iter().map(|b| App::sprite_for_biome(*b)).collect();
        // Semua nama dimulai "planet_" & unik satu sama lain (tak ada 2 biome ke base sama).
        for n in &names {
            assert!(
                n.starts_with("planet_"),
                "base sprite '{n}' harus prefix planet_"
            );
        }
        let mut sorted = names.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            names.len(),
            "tiap biome harus punya base sprite unik"
        );
    }

    #[test]
    fn sprite_for_biome_matches_existing_asset_folder() {
        // Base sprite yang dipetakan harus benar-benar ada di assets/sprites/ (bukan nama karang).
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/sprites");
        for b in [
            Biome::IronWorld,
            Biome::OceanPlanet,
            Biome::GasGiant,
            Biome::DeadWorld,
            Biome::CrystalWorld,
            Biome::Terran,
            Biome::AsteroidBelt,
            Biome::IceWorld,
            Biome::LavaWorld,
        ] {
            let base = App::sprite_for_biome(b);
            assert!(
                root.join(base).is_dir(),
                "folder assets/sprites/{base}/ harus ada"
            );
        }
    }

    #[test]
    fn portrait_id_captain_is_fixed_regardless_of_seed() {
        assert_eq!(
            App::portrait_id("captain", 0),
            App::portrait_id("captain", 999)
        );
    }

    #[test]
    fn portrait_id_same_seed_is_deterministic() {
        // Konteks+seed sama harus selalu hasil sama (dipanggil dua kali).
        assert_eq!(
            App::portrait_id("merchant", 42),
            App::portrait_id("merchant", 42)
        );
    }

    #[test]
    fn portrait_id_merchant_varies_across_seeds() {
        let a = App::portrait_id("merchant", 0);
        let b = App::portrait_id("merchant", 1);
        assert_ne!(a, b, "seed beda harus bisa hasil karakter berbeda");
    }

    #[test]
    fn portrait_id_unknown_context_falls_back_safely() {
        let id = App::portrait_id("nonexistent_ctx", 7);
        assert!(!id.is_empty());
    }

    #[test]
    fn portrait_id_all_outputs_match_existing_asset_file() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/characters");
        for ctx in ["captain", "merchant", "unknown"] {
            for seed in 0..7u64 {
                let id = App::portrait_id(ctx, seed);
                let path = root.join(format!("{id}.png"));
                assert!(
                    path.is_file(),
                    "portrait '{id}' (ctx={ctx}) harus ada file PNG-nya"
                );
            }
        }
    }

    /// M17.5: "Toggle pixel/procedural (key atau auto)" — `G` (besar) di MAIN VIEW HARUS
    /// genuinely balik `force_procedural_galaxy` (default `false` = auto). Dites LANGSUNG via
    /// `handle_key` (jembatan end-to-end key-press asli, bukan set field manual).
    #[test]
    fn shift_g_toggles_force_procedural_galaxy_only_in_main_menu() {
        let mut app = App::demo();
        assert_eq!(app.view, "main_menu");
        assert!(!app.force_procedural_galaxy);
        app.handle_key(KeyCode::Char('G'));
        assert!(
            app.force_procedural_galaxy,
            "G di main_menu harus genuinely toggle ON"
        );
        app.handle_key(KeyCode::Char('G'));
        assert!(
            !app.force_procedural_galaxy,
            "G kedua harus toggle balik OFF"
        );
    }

    /// M17.5: `G` di view LAIN (bukan main_menu) HARUS TAK berefek -- toggle ini spesifik
    /// backdrop MAIN VIEW, bukan global (view lain tak py galaxy backdrop sama sekali).
    #[test]
    fn shift_g_does_nothing_outside_main_menu() {
        let mut app = App::demo();
        app.view = "planet_view".into();
        app.handle_key(KeyCode::Char('G'));
        assert!(
            !app.force_procedural_galaxy,
            "G di luar main_menu HARUS tak berefek pd toggle galaxy"
        );
    }

    /// M18.4: "Navigasi kursor h/j/k/l + panah" — dites LANGSUNG via `handle_key` (jembatan
    /// end-to-end asli): kursor BEBAS gerak per-sel di 4 arah, BUKAN lompat antar planet
    /// (beda dari `sel`'s perilaku lama M18.3).
    #[test]
    fn galaxy_cursor_moves_freely_with_hjkl_and_arrows() {
        let mut app = App::demo();
        app.view = "galaxy_map".into();
        assert_eq!(app.galaxy_cursor, (0, 0));
        app.handle_key(KeyCode::Char('l'));
        assert_eq!(app.galaxy_cursor, (1, 0), "l harus gerak +1 sumbu x");
        app.handle_key(KeyCode::Char('j'));
        assert_eq!(app.galaxy_cursor, (1, 1), "j harus gerak +1 sumbu y");
        app.handle_key(KeyCode::Down);
        assert_eq!(app.galaxy_cursor, (1, 2), "panah bawah harus setara j");
        app.handle_key(KeyCode::Char('h'));
        assert_eq!(app.galaxy_cursor, (0, 2), "h harus gerak -1 sumbu x");
        app.handle_key(KeyCode::Char('k'));
        assert_eq!(app.galaxy_cursor, (0, 1), "k harus gerak -1 sumbu y");
    }

    /// M18.4: kursor tak boleh keluar batas dunia (0..WORLD_W, 0..WORLD_H) -- cegah panic
    /// index kalau nanti dipakai lookup grid (pondasi M18.9's clamp lebih lengkap).
    #[test]
    fn galaxy_cursor_clamps_at_world_edges() {
        let mut app = App::demo();
        app.view = "galaxy_map".into();
        // Pojok kiri-atas (0,0) -- h/k tak boleh jadi negatif.
        app.handle_key(KeyCode::Char('h'));
        app.handle_key(KeyCode::Char('k'));
        assert_eq!(
            app.galaxy_cursor,
            (0, 0),
            "tak boleh keluar batas dunia sisi (0,0)"
        );

        // Pojok kanan-bawah -- l/j tak boleh lewati WORLD_W-1/WORLD_H-1.
        use crate::ui::panels::galaxy_map::{WORLD_H, WORLD_W};
        app.galaxy_cursor = (WORLD_W - 1, WORLD_H - 1);
        app.handle_key(KeyCode::Char('l'));
        app.handle_key(KeyCode::Char('j'));
        assert_eq!(
            app.galaxy_cursor,
            (WORLD_W - 1, WORLD_H - 1),
            "tak boleh keluar batas dunia sisi (WORLD_W-1,WORLD_H-1)"
        );
    }

    /// M18.4: `s` (scan) di posisi kursor genuinely materialize planet DI SITU (via
    /// `planet_idx_at`, bukan `app.sel` lg) — kursor BEBAS bisa ke sel kosong (no-op `s`).
    #[test]
    fn scan_action_uses_cursor_position_not_stale_sel_index() {
        let mut app = App::demo();
        app.view = "galaxy_map".into();
        let g = crate::ui::panels::galaxy_map::frontier(&app).unwrap();
        let seed = match &g.kind {
            crate::game::state::GalaxyKind::Procedural { seed, .. } => *seed,
            crate::game::state::GalaxyKind::Fixed => 0,
        };
        let planet0_pos = crate::ui::panels::galaxy_map::planet_grid_pos(seed, 0);
        app.galaxy_cursor = planet0_pos;
        app.handle_key(KeyCode::Char('s'));
        let g = crate::ui::panels::galaxy_map::frontier(&app).unwrap();
        match &g.kind {
            crate::game::state::GalaxyKind::Procedural { visited, .. } => {
                assert!(
                    visited.contains(&0),
                    "planet idx 0 (di posisi kursor) harus genuinely termaterialisasi"
                );
            }
            _ => panic!("expected Procedural galaxy di demo"),
        }
    }

    /// M20.5: "Navigasi galaxy bebas-panic (unit + smoke)" — **gap real ditemukan**: unit
    /// test lama (`galaxy_map.rs`) cek skenario STATIS terpisah (1 render, 1 kondisi tetap),
    /// TAK ADA yg simulasikan SESI navigasi PANJANG (banyak key-press berurutan, gerak
    /// kursor SAMPAI tepi dunia lalu balik, semua aksi dicoba berulang) -- pola "smoke"
    /// [`tests/smoke.rs`'s gaya] tp HARUS taruh di sini krn `handle_key` `pub(crate)` [tak
    /// bisa diakses dr `tests/` external crate]. Dites di 3 breakpoint (Full/Compact/
    /// Minimal), kursor digerakkan MELEWATI WORLD_W/H (M18.9's overflow precedent) dua arah,
    /// SEMUA key (h/j/k/l/panah/s/t/Enter/f/Tab) dicoba berulang sepanjang jalan.
    #[test]
    fn galaxy_navigation_survives_extended_key_sequence_without_panic() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        for (w, h) in [(120u16, 40u16), (80, 30), (60, 24)] {
            let mut app = App::demo();
            app.view = "galaxy_map".into();
            let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();

            let keys = [
                KeyCode::Char('l'),
                KeyCode::Char('j'),
                KeyCode::Char('h'),
                KeyCode::Char('k'),
                KeyCode::Right,
                KeyCode::Down,
                KeyCode::Left,
                KeyCode::Up,
                KeyCode::Char('s'),
                KeyCode::Char('t'),
                KeyCode::Enter,
                KeyCode::Char('f'),
                KeyCode::Tab,
            ];
            // Gerak jauh ke satu tepi dunia (130 langkah > WORLD_W=120, buktikan clamp
            // genuinely aman di batas), lalu jalankan SEMUA aksi berulang sambil render.
            for _ in 0..130 {
                app.handle_key(KeyCode::Char('l'));
                app.handle_key(KeyCode::Char('j'));
            }
            for &k in keys.iter().cycle().take(52) {
                app.handle_key(k);
                term.draw(|f| crate::ui::draw_view(f, &app, "galaxy_map"))
                    .unwrap();
            }
            // Gerak balik ke tepi BERLAWANAN + ulangi semua aksi.
            for _ in 0..130 {
                app.handle_key(KeyCode::Char('h'));
                app.handle_key(KeyCode::Char('k'));
            }
            for &k in keys.iter().cycle().take(52) {
                app.handle_key(k);
                term.draw(|f| crate::ui::draw_view(f, &app, "galaxy_map"))
                    .unwrap();
            }
        }
    }

    /// M20.8 follow-up Phase 6: pan/zoom/reset keys HARUS genuinely HANYA aktif saat mode
    /// `Backdrop`, via `App::handle_key` end-to-end (bukan panggil `galaxy_sim.pan()` LANGSUNG
    /// -- itu cuma buktikan method-nya sendiri, BUKAN jembatan key->aksi genuinely tersambung).
    #[test]
    fn backdrop_pan_zoom_reset_keys_wired_end_to_end() {
        let mut app = App::demo();
        app.view = "galaxy_map".into();
        assert_eq!(
            app.galaxy_map_mode,
            crate::ui::panels::galaxy_map::GalaxyMapMode::Starmap
        );
        // Render SEKALI dulu (biar `galaxy_sim`'s dims/zoom-fit genuinely terisi) sblm cek
        // efek pan/zoom -- tanpa ini `last_dims` msh (0,0), zoom blm di-fit, test genuinely
        // tak berarti apa2.
        let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 40)).unwrap();
        term.draw(|f| crate::ui::draw_view(f, &app, "galaxy_map"))
            .unwrap();

        app.handle_key(KeyCode::Tab); // Starmap -> Backdrop.
        assert_eq!(
            app.galaxy_map_mode,
            crate::ui::panels::galaxy_map::GalaxyMapMode::Backdrop
        );
        term.draw(|f| crate::ui::draw_view(f, &app, "galaxy_map"))
            .unwrap();

        let before = app.galaxy_sim.camera_snapshot();
        app.handle_key(KeyCode::Char('l')); // pan.
        let after_pan = app.galaxy_sim.camera_snapshot();
        assert_ne!(
            before.view_center, after_pan.view_center,
            "`l` di mode Backdrop HARUS genuinely pan kamera (bukan gerak galaxy_cursor)"
        );

        app.handle_key(KeyCode::Char('z')); // zoom in.
        let after_zoom = app.galaxy_sim.camera_snapshot();
        assert!(
            after_zoom.zoom > after_pan.zoom,
            "`z` HARUS genuinely zoom in"
        );

        app.handle_key(KeyCode::Char('r')); // reset.
        let after_reset = app.galaxy_sim.camera_snapshot();
        assert_eq!(
            after_reset.view_center,
            glam::Vec2::ZERO,
            "`r` HARUS genuinely reset view_center ke (0,0)"
        );

        app.handle_key(KeyCode::Tab); // Backdrop -> Starmap (toggle balik HARUS TETAP jalan).
        assert_eq!(
            app.galaxy_map_mode,
            crate::ui::panels::galaxy_map::GalaxyMapMode::Starmap,
            "Tab HARUS genuinely toggle balik ke Starmap dr dlm Backdrop mode"
        );
    }

    /// M20.8 follow-up Phase 7: `Enter`/`Esc` fokus POI HARUS genuinely tersambung end-to-end
    /// via `App::handle_key` (bukan panggil `galaxy_sim.focus_next()` LANGSUNG), termasuk
    /// "escape bertingkat" (`Esc` PERTAMA batal fokus, `Esc` KEDUA baru keluar view).
    #[test]
    fn backdrop_enter_focuses_poi_and_esc_unfocuses_before_exiting_view() {
        let mut app = App::demo();
        app.view = "galaxy_map".into();
        let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 40)).unwrap();
        term.draw(|f| crate::ui::draw_view(f, &app, "galaxy_map"))
            .unwrap();
        app.handle_key(KeyCode::Tab); // Starmap -> Backdrop.
        term.draw(|f| crate::ui::draw_view(f, &app, "galaxy_map"))
            .unwrap();

        assert!(!app.galaxy_sim.is_focused());
        app.handle_key(KeyCode::Enter);
        assert!(
            app.galaxy_sim.is_focused(),
            "`Enter` di mode Backdrop HARUS genuinely fokus POI pertama"
        );

        app.handle_key(KeyCode::Esc); // Esc PERTAMA: batal fokus, TETAP di galaxy_map.
        assert!(!app.galaxy_sim.is_focused());
        assert_eq!(
            app.view, "galaxy_map",
            "Esc PERTAMA HARUS genuinely cuma batal fokus"
        );

        app.handle_key(KeyCode::Esc); // Esc KEDUA: baru keluar view.
        assert_eq!(
            app.view, "main_menu",
            "Esc KEDUA (tanpa fokus aktif) HARUS genuinely keluar view spt biasa"
        );
    }
}
