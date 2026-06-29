//! `App` (state + content) dan event loop utama.
//!
//! Event loop ratatui penuh (input non-blocking, tick 1Hz, render decoupled) diisi M3 lanjutan.
//! Sekarang: `App` + `App::demo()` (untuk snapshot headless) tersedia.
#![allow(dead_code)]

use crate::balance::{MAX_TICKS_PER_FRAME, RENDER_INTERVAL_MS, TICK_DURATION_SECS};
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

/// Aplikasi: state game + registry konten read-only + view aktif.
pub struct App {
    pub state: GameState,
    pub content: Content,
    /// View aktif (main_menu | galaxy_map | planet_view | research | merchant | warp).
    pub view: String,
    /// Indeks terpilih dalam list view aktif (mis. slot factory di planet_view).
    pub sel: usize,
}

impl App {
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
            galaxies: vec![crate::game::state::Galaxy {
                id: GalaxyId(0),
                name: "Milky Way".into(),
                level: 0,
                kind: crate::game::state::GalaxyKind::Fixed,
                planets: vec![earth],
            }],
            active_galaxy: GalaxyId(0),
            anchor_galaxy: GalaxyId(0),
            ship,
            research: Default::default(),
            prestige: Default::default(),
            merchant: Default::default(),
            events: Default::default(),
            settings: Default::default(),
        };
        App {
            state,
            content,
            view: "main_menu".into(),
            sel: 0,
        }
    }

    /// Galaksi+planet aktif (galaksi sekarang, planet pertama yang unlocked).
    fn active_pi(&self) -> Option<(usize, usize)> {
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
    fn handle_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('g') => self.view = "galaxy_map".into(),
            KeyCode::Char('p') => self.view = "planet_view".into(),
            KeyCode::Char('r') => self.view = "research".into(),
            KeyCode::Char('m') => self.view = "merchant".into(),
            KeyCode::Char('w') => self.view = "warp".into(),
            KeyCode::Esc => self.view = "main_menu".into(),
            _ => {}
        }
        if self.view == "planet_view" {
            self.planet_keys(code);
        }
    }

    /// Navigasi & aksi pada planet_view: j/k pilih slot, 1 upgrade node, 2/Enter upgrade factory.
    fn planet_keys(&mut self, code: KeyCode) {
        let Some((gi, pi)) = self.active_pi() else {
            return;
        };
        let slots = self.state.galaxies[gi].planets[pi].factory_slots.len();
        let nodes = self.state.galaxies[gi].planets[pi].nodes.len();
        match code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.sel = (self.sel + 1).min(slots.saturating_sub(1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.sel = self.sel.saturating_sub(1);
            }
            KeyCode::Char('2') | KeyCode::Enter => {
                let _ = crate::game::actions::upgrade_factory(
                    &mut self.state,
                    &self.content,
                    pi,
                    self.sel,
                );
            }
            KeyCode::Char('1') if nodes > 0 => {
                let node = self.sel.min(nodes - 1);
                let _ = crate::game::actions::upgrade_node(&mut self.state, pi, node);
            }
            _ => {}
        }
    }
}

/// Entry runtime: setup terminal, jalankan event loop, teardown bersih.
pub fn run() {
    let mut app = App::demo();
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
    let mut last = Instant::now();
    let mut last_render = Instant::now() - render_interval;

    loop {
        if event::poll(Duration::from_millis(10))?
            && let Event::Key(k) = event::read()?
            && k.kind == KeyEventKind::Press
        {
            let quit = matches!(k.code, KeyCode::Char('q'))
                || (k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL));
            if quit {
                break;
            }
            app.handle_key(k.code);
        }

        let now = Instant::now();
        accumulated += now - last;
        last = now;
        let mut ticks: u32 = 0;
        while accumulated >= tick && ticks < MAX_TICKS_PER_FRAME {
            crate::sim::tick::step(&mut app.state, &app.content);
            app.state.tick += 1;
            accumulated -= tick;
            ticks += 1;
        }
        if ticks == MAX_TICKS_PER_FRAME {
            accumulated = Duration::ZERO; // sisa → offline (M7); cegah spiral.
        }

        if now - last_render >= render_interval {
            let view = app.view.clone();
            term.draw(|f| crate::ui::draw_view(f, app, &view))?;
            last_render = now;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
