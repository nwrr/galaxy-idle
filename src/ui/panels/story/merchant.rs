//! Panel MERCHANT (`view="merchant"`, `m` key). M1 fix: dulu genuinely `draw_placeholder`
//! ("(TODO)") — `state.merchant`/`MerchantOffer` sudah nyata+dites sejak M13.5 (dipakai footer
//! MARKET ringkas), tp tak py view detail sendiri utk NAVIGASI+BELI. Beli via
//! `game::events::accept_merchant_offer` (BARU jg M1 fix — dulu tak ada fungsi consumer sama
//! sekali, offer cuma ke-render, tak pernah genuinely bisa "diambil").

use crate::app::App;
use crate::game::state::MerchantOffer;
use crate::ui::{fmt_hms, offer_text, theme};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Wrap};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let title = ratatui::text::Span::styled(" MERCHANT ", Style::default().fg(th.header).bold());
    let outer = Block::bordered()
        .title(title)
        .border_style(Style::default().fg(th.dim));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(inner);

    let m = &app.state.merchant;
    if !m.active {
        let left = m.restock_at_tick.saturating_sub(app.state.tick);
        f.render_widget(
            Paragraph::new(Line::styled(
                "Tak ada merchant berkunjung saat ini.",
                Style::default().fg(th.dim),
            )),
            rows[0],
        );
        f.render_widget(
            Paragraph::new(Line::styled(
                format!("Kunjungan berikut dlm {}", fmt_hms(left)),
                Style::default().fg(th.dim),
            )),
            rows[1],
        );
        return;
    }

    let left = m.expires_at_tick.saturating_sub(app.state.tick);
    f.render_widget(
        Paragraph::new(Line::styled(
            format!(
                "Window tutup dlm {}  |  Warp Cores: {}",
                fmt_hms(left),
                app.state.prestige.warp_cores
            ),
            Style::default().fg(th.header),
        )),
        rows[0],
    );

    let lines: Vec<Line> = m
        .stock
        .iter()
        .enumerate()
        .map(|(i, o)| {
            let owned = matches!(o, MerchantOffer::Blueprint { owned: true, .. });
            let mut text = offer_text(app, o);
            if owned {
                text.push_str(" [OWNED]");
            }
            let active = i == app.sel;
            let prefix = if active { "\u{25ba} " } else { "  " };
            let style = if owned {
                Style::default().fg(th.dim)
            } else if active {
                Style::default()
                    .fg(th.focus)
                    .add_modifier(Modifier::REVERSED)
            } else {
                Style::default().fg(th.text)
            };
            Line::styled(format!("{prefix}{text}"), style)
        })
        .collect();
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), rows[1]);

    f.render_widget(
        Paragraph::new(Line::styled(
            "[j/k] pilih  [Enter] beli  [Esc] kembali",
            Style::default().fg(th.dim),
        )),
        rows[2],
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::events::restock_merchant;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn rendered_text(app: &App) -> String {
        let mut term = Terminal::new(TestBackend::new(80, 24)).unwrap();
        term.draw(|f| render(f, app, f.area())).unwrap();
        crate::ui::buffer_to_text(term.backend().buffer()).join("\n")
    }

    #[test]
    fn shows_closed_message_when_inactive() {
        let app = App::demo();
        let text = rendered_text(&app);
        assert!(
            text.contains("Tak ada merchant"),
            "harus tampilkan status tutup: {text:?}"
        );
    }

    #[test]
    fn shows_real_offers_when_active() {
        let mut app = App::demo();
        restock_merchant(&mut app.state.merchant, &app.content, app.state.tick);
        let text = rendered_text(&app);
        assert!(
            text.contains("Warp Cores"),
            "harus tampilkan warp cores: {text:?}"
        );
        assert!(!app.state.merchant.stock.is_empty());
        let first = offer_text(&app, &app.state.merchant.stock[0]);
        assert!(text.contains(&first), "harus render offer NYATA: {text:?}");
    }

    #[test]
    fn selected_offer_marker_follows_sel() {
        let mut app = App::demo();
        restock_merchant(&mut app.state.merchant, &app.content, app.state.tick);
        app.sel = 0;
        let text = rendered_text(&app);
        assert!(
            text.contains("\u{25ba}"),
            "harus tandai offer terpilih: {text:?}"
        );
    }
}
