//! Event saat travel (`05-events.md`) — `GameEvent`, `EventQueue`.
//!
//! Skeleton M2 — varian `GameEvent` lengkap + trigger/resolusi diisi M9.
//! Sekarang cukup tipe agar `GameState` punya `EventQueue`.
#![allow(dead_code)]

use std::collections::VecDeque;

/// Event yang menunggu direview/diresolusi player. Varian diisi M9 (`05`).
#[derive(Clone, Debug)]
pub enum GameEvent {
    /// Placeholder sampai katalog event diimplementasi (M9).
    Pending,
}

#[derive(Clone, Debug, Default)]
pub struct EventQueue {
    pub pending: VecDeque<GameEvent>,
}
