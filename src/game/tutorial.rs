//! Onboarding (M3): `GameState.tutorial_step` (`0..=3`, `None`=selesai/dilewati). Pola pure-
//! function SAMA `research.rs` (`advance` dipanggil tiap tick, `sim::tick::step`) -- auto-maju
//! saat kondisi REAL terpenuhi (bukan dismiss manual), sesuai `plan/CHECKLIST.md` M3.

use crate::game::state::{FactoryKind, GameState, ShipStatus};

/// Deskripsi singkat step aktif, dipakai panel hint (UI baca-saja, tak dipakai logic ini).
pub fn step_hint(step: u8) -> &'static str {
    match step {
        0 => "Bangun extractor pertama di sebuah slot factory kosong.",
        1 => "Mulai riset pertama.",
        2 => "Kirim ship travel ke planet lain.",
        3 => "Lakukan warp jump pertama.",
        _ => "",
    }
}

fn step0_built_extractor(state: &GameState) -> bool {
    state.galaxies.iter().any(|g| {
        g.planets.iter().any(|p| {
            p.factory_slots
                .iter()
                .flatten()
                .any(|f| matches!(f.kind, FactoryKind::Extractor { .. }))
        })
    })
}

fn step1_started_research(state: &GameState) -> bool {
    state.research.active.is_some() || !state.research.completed.is_empty()
}

fn step2_sent_travel(state: &GameState) -> bool {
    matches!(state.ship.status, ShipStatus::Traveling { .. })
}

fn step3_warp_jumped(state: &GameState) -> bool {
    state.prestige.galaxy_level_reached > 0
}

/// Cek predikat step AKTIF, maju satu langkah bila terpenuhi (tak pernah lompat >1 step per
/// panggilan — tiap kondisi dites ulang tick berikutnya, cukup krn dipanggil tiap tick).
/// Return `true` bila step berubah (dipakai test; UI re-render sudah otomatis tiap tick).
pub fn advance(state: &mut GameState) -> bool {
    let Some(step) = state.tutorial_step else {
        return false;
    };
    let done = match step {
        0 => step0_built_extractor(state),
        1 => step1_started_research(state),
        2 => step2_sent_travel(state),
        3 => step3_warp_jumped(state),
        _ => false,
    };
    if !done {
        return false;
    }
    state.tutorial_step = if step >= 3 { None } else { Some(step + 1) };
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::load_content;
    use crate::game::defs::Content;
    use crate::game::state::{Factory, FactoryId};

    fn content() -> Content {
        load_content(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data")).unwrap()
    }

    fn state_at_step(step: u8) -> GameState {
        let mut app = crate::app::App::new_game();
        app.state.tutorial_step = Some(step);
        app.state
    }

    #[test]
    fn stays_put_when_predicate_unmet() {
        let mut state = state_at_step(0);
        assert!(!advance(&mut state));
        assert_eq!(state.tutorial_step, Some(0));
    }

    #[test]
    fn step0_advances_when_extractor_built() {
        let mut state = state_at_step(0);
        let c = content();
        let bid = crate::game::defs::BuildingId(c.buildings.id("mining_drill").unwrap());
        state.galaxies[0].planets[0].factory_slots[0] = Some(Factory {
            id: FactoryId(0),
            building: bid,
            kind: FactoryKind::Extractor {
                node: state.galaxies[0].planets[0].nodes[0].id,
            },
            level: 1,
            enabled: true,
        });
        assert!(advance(&mut state));
        assert_eq!(state.tutorial_step, Some(1));
    }

    #[test]
    fn step1_advances_when_research_started() {
        let mut state = state_at_step(1);
        state.research.active = Some(crate::game::state::ActiveResearch {
            tech_id: "x".into(),
            data_invested: 0.0,
            elapsed_secs: 0.0,
        });
        assert!(advance(&mut state));
        assert_eq!(state.tutorial_step, Some(2));
    }

    #[test]
    fn step2_advances_when_traveling() {
        let mut state = state_at_step(2);
        state.ship.status = ShipStatus::Traveling {
            to: crate::game::state::PlanetId(1),
            total_secs: 10.0,
            elapsed_secs: 0.0,
        };
        assert!(advance(&mut state));
        assert_eq!(state.tutorial_step, Some(3));
    }

    #[test]
    fn step3_completes_tutorial_on_warp_jump() {
        let mut state = state_at_step(3);
        state.prestige.galaxy_level_reached = 1;
        assert!(advance(&mut state));
        assert_eq!(
            state.tutorial_step, None,
            "step terakhir -> None, bukan lompat step 4"
        );
    }

    #[test]
    fn none_step_is_a_stable_noop() {
        let mut state = state_at_step(0);
        state.tutorial_step = None;
        assert!(!advance(&mut state));
        assert_eq!(state.tutorial_step, None);
    }
}
