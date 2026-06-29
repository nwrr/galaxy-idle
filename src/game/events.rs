//! Event saat travel + Void Merchant (`05-events.md`, `04-prestige.md` §4).
//!
//! Model risk/reward (no-combat): `GameEvent` punya `options`; player memilih `EventOption` →
//! `EventEffect` diterapkan ke state. Saat ship `Traveling`, sistem me-`roll` event tiap
//! `EVENT_ROLL_INTERVAL_TICKS` (peluang naik dgn jarak & scanner). Void Merchant = roaming:
//! di-restock saat muncul, `active` sampai `expires_at_tick`.
//!
//! Transient (tak disimpan; `save/dto.rs` default saat load). RNG event = deterministik dari
//! `tick` (asumsi, lebih sederhana & testable drpd sumber non-deterministik di `07`).
#![allow(dead_code)]

use crate::balance::{
    BASE_EVENT_CHANCE, EVENT_DIST_FACTOR, EVENT_ROLL_INTERVAL_TICKS, EVENT_SCANNER_FACTOR,
    MERCHANT_RESTOCK_SECS, MERCHANT_WINDOW_SECS, NEBULA_TRAVEL_PENALTY, TICK_DURATION_SECS,
    WRECK_SALVAGE_SECS,
};
use crate::game::defs::{Content, ResourceId};
use crate::game::state::{GameState, Merchant, MerchantOffer, ShipStatus};
use crate::rng::{SplitMix64, derive};
use std::collections::VecDeque;

/// Kategori event → warna log (`05` §Tipe).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EventCategory {
    Hazard,
    Loot,
    Trade,
    Neutral,
}

/// Jenis event (subset terimplementasi; sisanya katalog lanjutan `05`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EventKind {
    Nebula,
    ShipWreck,
    AlienCaravan,
    VoidMerchant,
}

/// Syarat memilih sebuah opsi (minimal; `05` §EventReq).
#[derive(Clone, Debug)]
pub enum EventReq {
    EnergyCost(f64),
    ScannerLevel(u32),
}

/// Resolusi opsi (subset; `TempProductionBuff` ditunda — butuh field buff di state).
#[derive(Clone, Debug)]
pub enum EventEffect {
    GainResources(Vec<(ResourceId, f64)>),
    LoseResources(Vec<(ResourceId, f64)>),
    AdjustTravelTime { delta_secs: f64 },
    Nothing,
}

#[derive(Clone, Debug)]
pub struct EventOption {
    pub label: String,
    pub requires: Option<EventReq>,
    pub duration_secs: f64,
    pub effect: EventEffect,
}

#[derive(Clone, Debug)]
pub struct GameEvent {
    pub id: u64,
    pub kind: EventKind,
    pub category: EventCategory,
    pub created_tick: u64,
    pub expires_tick: Option<u64>,
    pub options: Vec<EventOption>,
}

#[derive(Clone, Debug, Default)]
pub struct EventQueue {
    pub pending: VecDeque<GameEvent>,
    next_id: u64,
    ticks_since_roll: u32,
}

/// Peluang event per roll (clamp [0,1]) — `05` §Trigger.
pub fn event_chance(distance: f64, scanner: f64) -> f64 {
    let c = BASE_EVENT_CHANCE
        * (1.0 + EVENT_DIST_FACTOR * distance)
        * (1.0 + EVENT_SCANNER_FACTOR * scanner);
    c.clamp(0.0, 1.0)
}

impl EventQueue {
    /// Pilih satu jenis event dari selector + bangun opsinya. Effect merujuk id konkret.
    fn make_event(&mut self, kind: EventKind, content: &Content, tick: u64) -> GameEvent {
        self.next_id += 1;
        let res = |name: &str| content.resources.id(name).map(ResourceId);
        let (category, options, ttl) = match kind {
            EventKind::Nebula => (
                EventCategory::Hazard,
                vec![
                    EventOption {
                        label: "Reroute".into(),
                        requires: Some(EventReq::EnergyCost(50.0)),
                        duration_secs: 0.0,
                        effect: EventEffect::Nothing,
                    },
                    EventOption {
                        label: "Push through".into(),
                        requires: None,
                        duration_secs: 0.0,
                        effect: EventEffect::AdjustTravelTime {
                            delta_secs: WRECK_SALVAGE_SECS * NEBULA_TRAVEL_PENALTY,
                        },
                    },
                ],
                Some(120),
            ),
            EventKind::ShipWreck => (
                EventCategory::Loot,
                vec![
                    EventOption {
                        label: "Salvage".into(),
                        requires: None,
                        duration_secs: WRECK_SALVAGE_SECS,
                        effect: gain(&res, &[("iron", 200.0), ("carbon", 50.0)]),
                    },
                    EventOption {
                        label: "Ignore".into(),
                        requires: None,
                        duration_secs: 0.0,
                        effect: EventEffect::Nothing,
                    },
                ],
                Some(300),
            ),
            EventKind::AlienCaravan => (
                EventCategory::Trade,
                vec![
                    EventOption {
                        label: "Trade".into(),
                        requires: None,
                        duration_secs: 0.0,
                        effect: trade(&res, "iron", 500.0, "titanium", 10.0),
                    },
                    EventOption {
                        label: "Decline".into(),
                        requires: None,
                        duration_secs: 0.0,
                        effect: EventEffect::Nothing,
                    },
                ],
                None,
            ),
            EventKind::VoidMerchant => (
                EventCategory::Trade,
                vec![EventOption {
                    label: "Acknowledge".into(),
                    requires: None,
                    duration_secs: 0.0,
                    effect: EventEffect::Nothing,
                }],
                Some(MERCHANT_WINDOW_SECS as u64),
            ),
        };
        GameEvent {
            id: self.next_id,
            kind,
            category,
            created_tick: tick,
            expires_tick: ttl.map(|t| tick + t),
            options,
        }
    }

    /// Buang event yang kedaluwarsa (`expires_tick < tick`).
    pub fn expire(&mut self, tick: u64) {
        self.pending
            .retain(|e| e.expires_tick.is_none_or(|t| t >= tick));
    }

    /// Ambil + buang event ke-`index`, kembalikan effect opsi ke-`option`.
    pub fn take_option(&mut self, index: usize, option: usize) -> Option<EventEffect> {
        let ev = self.pending.get(index)?;
        let effect = ev.options.get(option)?.effect.clone();
        self.pending.remove(index);
        Some(effect)
    }
}

fn gain(res: &impl Fn(&str) -> Option<ResourceId>, items: &[(&str, f64)]) -> EventEffect {
    let v: Vec<_> = items
        .iter()
        .filter_map(|(n, a)| res(n).map(|r| (r, *a)))
        .collect();
    if v.is_empty() {
        EventEffect::Nothing
    } else {
        EventEffect::GainResources(v)
    }
}

fn trade(
    res: &impl Fn(&str) -> Option<ResourceId>,
    lose: &str,
    lose_amt: f64,
    win: &str,
    win_amt: f64,
) -> EventEffect {
    match (res(lose), res(win)) {
        (Some(l), Some(w)) => EventEffect::GainResources(vec![(w, win_amt), (l, -lose_amt)]),
        _ => EventEffect::Nothing,
    }
}

/// Fase tick 6: roll event saat travel + buang yang kedaluwarsa.
pub fn tick_travel_events(state: &mut GameState, content: &Content) {
    let tick = state.tick;
    state.events.expire(tick);
    let ShipStatus::Traveling { to, .. } = state.ship.status else {
        state.events.ticks_since_roll = 0;
        return;
    };
    state.events.ticks_since_roll += 1;
    if state.events.ticks_since_roll < EVENT_ROLL_INTERVAL_TICKS {
        return;
    }
    state.events.ticks_since_roll = 0;

    let distance = state
        .galaxies
        .iter()
        .flat_map(|g| g.planets.iter())
        .find(|p| p.id == to)
        .map(|p| p.distance)
        .unwrap_or(1.0);
    let mut rng = SplitMix64::new(derive(tick, "event"));
    if rng.next_f64() >= event_chance(distance, state.ship.scanner as f64) {
        return;
    }
    let kind = match rng.below(4) {
        0 => EventKind::Nebula,
        1 => EventKind::ShipWreck,
        2 => EventKind::AlienCaravan,
        _ => EventKind::VoidMerchant,
    };
    if kind == EventKind::VoidMerchant {
        restock_merchant(&mut state.merchant, content, tick);
    }
    let ev = state.events.make_event(kind, content, tick);
    state.events.pending.push_back(ev);
}

/// Resolusi: terapkan effect opsi terpilih ke state (loot → planet pertama yg unlocked).
pub fn resolve_event(state: &mut GameState, index: usize, option: usize) -> bool {
    let Some(effect) = state.events.take_option(index, option) else {
        return false;
    };
    apply_effect(state, &effect);
    true
}

/// Terapkan satu `EventEffect` ke state. Resource masuk/keluar stockpile planet aktif pertama.
pub fn apply_effect(state: &mut GameState, effect: &EventEffect) {
    match effect {
        EventEffect::GainResources(rs) => adjust_stockpile(state, rs, 1.0),
        EventEffect::LoseResources(rs) => adjust_stockpile(state, rs, -1.0),
        EventEffect::AdjustTravelTime { delta_secs } => {
            if let ShipStatus::Traveling {
                to,
                total_secs,
                elapsed_secs,
            } = state.ship.status
            {
                state.ship.status = ShipStatus::Traveling {
                    to,
                    total_secs: (total_secs + delta_secs).max(elapsed_secs),
                    elapsed_secs,
                };
            }
        }
        EventEffect::Nothing => {}
    }
}

fn adjust_stockpile(state: &mut GameState, rs: &[(ResourceId, f64)], sign: f64) {
    let active = state.active_galaxy;
    let Some(g) = state.galaxies.iter_mut().find(|g| g.id == active) else {
        return;
    };
    let Some(p) = g.planets.iter_mut().find(|p| p.unlocked) else {
        return;
    };
    for (r, amt) in rs {
        let cur = p.stockpile.get(r).copied().unwrap_or(0.0);
        let next = (cur + sign * amt).clamp(0.0, p.stockpile_cap);
        p.stockpile.insert(*r, next);
    }
}

/// Restock Void Merchant + aktifkan window (`04` §4). Stock: jual/beli + 1 blueprint.
pub fn restock_merchant(merchant: &mut Merchant, content: &Content, tick: u64) {
    let mut stock = Vec::new();
    if let Some(r) = content.resources.id("iron") {
        stock.push(MerchantOffer::SellResource {
            resource: ResourceId(r),
            amount: 100.0,
            gain_credits: 200.0,
        });
    }
    if let Some(r) = content.resources.id("antimatter") {
        stock.push(MerchantOffer::BuyResource {
            resource: ResourceId(r),
            amount: 1.0,
            cost_cores: 1,
        });
    }
    if let Some(rec) = content.recipes.iter().position(|r| r.requires_blueprint) {
        stock.push(MerchantOffer::Blueprint {
            recipe: crate::game::defs::RecipeId(rec as u32),
            cost_cores: 2,
            owned: false,
        });
    }
    let secs_to_ticks = |s: f64| (s / TICK_DURATION_SECS) as u64;
    merchant.active = true;
    merchant.stock = stock;
    merchant.expires_at_tick = tick + secs_to_ticks(MERCHANT_WINDOW_SECS);
    merchant.restock_at_tick = tick + secs_to_ticks(MERCHANT_RESTOCK_SECS);
}

/// Fase tick 8: nonaktifkan merchant yang habis window-nya.
pub fn update_merchant(merchant: &mut Merchant, tick: u64) {
    if merchant.active && tick >= merchant.expires_at_tick {
        merchant.active = false;
        merchant.stock.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::load_content;
    use crate::game::state::{
        Biome, Galaxy, GalaxyId, GalaxyKind, Planet, PlanetId, ResourceMap, Ship, ShipStatus,
        UnlockReq,
    };

    fn content() -> Content {
        load_content(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data")).unwrap()
    }

    fn state(content: &Content) -> GameState {
        let planet = Planet {
            id: PlanetId(0),
            name: "Earth".into(),
            tier: 1,
            biome: Biome::Terran,
            distance: 5.0,
            unlocked: true,
            unlock_req: UnlockReq::None,
            nodes: vec![],
            factory_slots: vec![],
            stockpile: ResourceMap::new(),
            stockpile_cap: 100_000.0,
        };
        let _ = content;
        GameState {
            version: 1,
            last_saved_unix: 0,
            tick: 0,
            credits: 0.0,
            inventory: std::collections::HashMap::new(),
            galaxies: vec![Galaxy {
                id: GalaxyId(0),
                name: "MW".into(),
                level: 0,
                kind: GalaxyKind::Fixed,
                planets: vec![planet],
            }],
            active_galaxy: GalaxyId(0),
            anchor_galaxy: GalaxyId(0),
            ship: Ship::default(),
            research: Default::default(),
            prestige: Default::default(),
            merchant: Default::default(),
            events: Default::default(),
            settings: Default::default(),
        }
    }

    #[test]
    fn chance_rises_and_clamps() {
        assert!(event_chance(10.0, 5.0) > event_chance(1.0, 0.0));
        assert!(event_chance(1e9, 1e9) <= 1.0);
        assert!(event_chance(0.0, 0.0) >= 0.0);
    }

    #[test]
    fn resolve_gain_adds_to_stockpile() {
        let c = content();
        let mut st = state(&c);
        let iron = ResourceId(c.resources.id("iron").unwrap());
        let ev = st.events.make_event(EventKind::ShipWreck, &c, 0);
        st.events.pending.push_back(ev);
        assert!(resolve_event(&mut st, 0, 0)); // Salvage
        assert!(st.galaxies[0].planets[0].stockpile[&iron] >= 200.0);
        assert!(st.events.pending.is_empty());
    }

    #[test]
    fn trade_nets_positive_and_negative() {
        let c = content();
        let mut st = state(&c);
        let iron = ResourceId(c.resources.id("iron").unwrap());
        st.galaxies[0].planets[0].stockpile.insert(iron, 1000.0);
        let ev = st.events.make_event(EventKind::AlienCaravan, &c, 0);
        st.events.pending.push_back(ev);
        assert!(resolve_event(&mut st, 0, 0)); // Trade: -500 iron, +10 titanium
        assert!((st.galaxies[0].planets[0].stockpile[&iron] - 500.0).abs() < 1e-9);
    }

    #[test]
    fn expire_drops_old_events() {
        let c = content();
        let mut st = state(&c);
        let ev = st.events.make_event(EventKind::Nebula, &c, 0); // ttl 120
        st.events.pending.push_back(ev);
        st.events.expire(119);
        assert_eq!(st.events.pending.len(), 1);
        st.events.expire(121);
        assert!(st.events.pending.is_empty());
    }

    #[test]
    fn merchant_restock_then_expire() {
        let c = content();
        let mut m = Merchant::default();
        restock_merchant(&mut m, &c, 0);
        assert!(m.active && !m.stock.is_empty());
        let expiry = m.expires_at_tick;
        update_merchant(&mut m, expiry - 1);
        assert!(m.active);
        update_merchant(&mut m, expiry);
        assert!(!m.active && m.stock.is_empty());
    }

    #[test]
    fn roll_only_fires_on_interval_while_traveling() {
        let c = content();
        let mut st = state(&c);
        st.ship.scanner = 5;
        st.ship.status = ShipStatus::Traveling {
            to: PlanetId(0),
            total_secs: 10_000.0,
            elapsed_secs: 0.0,
        };
        // Sebelum interval tercapai: tak ada roll.
        for t in 0..(EVENT_ROLL_INTERVAL_TICKS - 1) {
            st.tick = t as u64;
            tick_travel_events(&mut st, &c);
        }
        assert!(st.events.pending.is_empty());
        assert_eq!(st.events.ticks_since_roll, EVENT_ROLL_INTERVAL_TICKS - 1);
    }
}
