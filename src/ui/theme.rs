//! Palet warna ANSI per `ThemeChoice` (`06-ui.md` §Color Scheme). Color = status, bukan dekorasi.
//!
//! NOTE(scaffold): `allow(dead_code)` — sebagian warna dipakai panel M4+.
#![allow(dead_code)]

use crate::game::state::ThemeChoice;
use ratatui::style::Color;

/// Warna semantik UI. Default = mapping `06`; varian lain untuk aksesibilitas.
pub struct Theme {
    pub header: Color,
    pub sidebar: Color,
    pub basic: Color,
    pub advanced: Color,
    pub rare: Color,
    pub focus: Color,
    pub dim: Color,
    pub alert: Color,
    pub good: Color,
    pub neutral: Color,
}

/// Bangun palet dari pilihan tema.
pub fn theme(choice: ThemeChoice) -> Theme {
    match choice {
        ThemeChoice::Default => Theme {
            header: Color::Cyan,
            sidebar: Color::Yellow,
            basic: Color::Green,
            advanced: Color::Blue,
            rare: Color::Magenta,
            focus: Color::White,
            dim: Color::DarkGray,
            alert: Color::Red,
            good: Color::Green,
            neutral: Color::Yellow,
        },
        ThemeChoice::HighContrast => Theme {
            header: Color::White,
            sidebar: Color::White,
            basic: Color::LightGreen,
            advanced: Color::LightBlue,
            rare: Color::LightMagenta,
            focus: Color::Yellow,
            dim: Color::Gray,
            alert: Color::LightRed,
            good: Color::LightGreen,
            neutral: Color::LightYellow,
        },
        ThemeChoice::Mono => {
            let w = Color::White;
            Theme {
                header: w,
                sidebar: w,
                basic: w,
                advanced: w,
                rare: w,
                focus: w,
                dim: Color::DarkGray,
                alert: w,
                good: w,
                neutral: w,
            }
        }
    }
}
