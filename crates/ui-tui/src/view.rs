//! Drawing one frame: the sidebar on the left, sessions on the right, key hints at the bottom.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph};

use crate::palette::Palette;

/// The sidebar's width in columns: about the window's 260 px sidebar at its 13 px font.
const SIDEBAR_COLS: u16 = 32;

pub fn draw(frame: &mut Frame, p: &Palette) {
    let [body, status] = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());
    let [sidebar, sessions] = Layout::horizontal([Constraint::Length(SIDEBAR_COLS), Constraint::Min(0)]).areas(body);
    draw_sidebar(frame, sidebar, p);
    draw_sessions(frame, sessions, p);
    draw_status(frame, status, p);
}

/// The host lists, with Key Management at the bottom, as in the window.
fn draw_sidebar(frame: &mut Frame, area: Rect, p: &Palette) {
    let title = Span::styled(" CloudNode ", Style::new().fg(p.accent).add_modifier(Modifier::BOLD));
    let block = panel(p).title(Line::from(title)).style(Style::new().bg(p.sidebar_bg));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [lists, keys] = Layout::vertical([Constraint::Min(0), Constraint::Length(2)]).areas(inner);
    let lists_text = vec![
        header("Favorites", p),
        hint("  Star a host to keep it here.", p),
        Line::default(),
        header("Connected", p),
        hint("  No open sessions.", p),
        Line::default(),
        header("Hosts", p),
        hint("  No hosts yet.", p),
    ];
    frame.render_widget(Paragraph::new(lists_text), lists);
    frame.render_widget(Paragraph::new(vec![header("Keys", p), hint("  No keys yet.", p)]), keys);
}

/// Where sessions go. Until one is open it says how to start one.
fn draw_sessions(frame: &mut Frame, area: Rect, p: &Palette) {
    let block = panel(p).style(Style::new().bg(p.terminal_bg));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [_, middle, _] = Layout::vertical([Constraint::Fill(1), Constraint::Length(3), Constraint::Fill(1)]).areas(inner);
    let text = vec![
        Line::styled("CloudNode", Style::new().fg(p.ink_strong).add_modifier(Modifier::BOLD)),
        Line::styled("──", Style::new().fg(p.accent)),
        Line::styled("Pick a host in the sidebar to open a session.", Style::new().fg(p.muted)),
    ];
    frame.render_widget(Paragraph::new(text).alignment(Alignment::Center), middle);
}

/// The key hints on the bottom row.
fn draw_status(frame: &mut Frame, area: Rect, p: &Palette) {
    let line = Line::from(vec![
        Span::styled(" q ", Style::new().fg(p.on_accent).bg(p.accent).add_modifier(Modifier::BOLD)),
        Span::styled(" quit", Style::new().fg(p.ink)),
    ]);
    frame.render_widget(Paragraph::new(line).style(Style::new().bg(p.window_bg)), area);
}

/// A rounded panel outlined in the line colour.
fn panel(p: &Palette) -> Block<'static> {
    Block::bordered().border_type(BorderType::Rounded).border_style(Style::new().fg(p.line))
}

fn header(text: &'static str, p: &Palette) -> Line<'static> {
    Line::styled(text, Style::new().fg(p.faint).add_modifier(Modifier::BOLD))
}

fn hint(text: &'static str, p: &Palette) -> Line<'static> {
    Line::styled(text, Style::new().fg(p.muted))
}
