//! Drawing one frame: the sidebar on the left, sessions on the right, key hints at the bottom.

use std::borrow::Cow;

use cloudnode_core::time::iso_date;
use cloudnode_keys::{Details, LocalKey, Scan, ScanError};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph};

use crate::palette::Palette;

/// The sidebar's width in columns: about the window's 260 px sidebar at its 13 px font.
const SIDEBAR_COLS: u16 = 32;
/// Keys listed in the sidebar (two lines each) before "… and N more".
const MAX_KEYS: usize = 5;

pub fn draw(frame: &mut Frame, p: &Palette, keys: &Result<Scan, ScanError>) {
    let [body, status] = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());
    let [sidebar, sessions] = Layout::horizontal([Constraint::Length(SIDEBAR_COLS), Constraint::Min(0)]).areas(body);
    draw_sidebar(frame, sidebar, p, keys);
    draw_sessions(frame, sessions, p);
    draw_status(frame, status, p);
}

/// The host lists, with Key Management at the bottom, as in the window.
fn draw_sidebar(frame: &mut Frame, area: Rect, p: &Palette, keys: &Result<Scan, ScanError>) {
    let title = Span::styled(" CloudNode ", Style::new().fg(p.accent).add_modifier(Modifier::BOLD));
    let block = panel(p).title(Line::from(title)).style(Style::new().bg(p.sidebar_bg));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let key_text = key_lines(keys, p);
    let key_rows = u16::try_from(key_text.len()).unwrap_or(u16::MAX);
    let [lists, keys_area] = Layout::vertical([Constraint::Min(0), Constraint::Length(key_rows)]).areas(inner);
    let lists_text = vec![
        header("FAVORITES", p),
        hint("  Star a host to keep it here.", p),
        Line::default(),
        header("CONNECTED", p),
        hint("  No open sessions.", p),
    ];
    frame.render_widget(Paragraph::new(lists_text), lists);
    frame.render_widget(Paragraph::new(key_text), keys_area);
}

/// Key Management: the keys found in ~/.ssh, two lines each like the window's cards, or why
/// there are none.
fn key_lines(keys: &Result<Scan, ScanError>, p: &Palette) -> Vec<Line<'static>> {
    let mut lines = vec![header("KEY MANAGEMENT", p)];
    match keys {
        Err(e) => lines.push(Line::styled(format!("  {e}"), Style::new().fg(p.bad))),
        Ok(scan) if scan.keys.is_empty() => lines.push(hint(format!("  No keys in {}", cloudnode_keys::tilde(&scan.dir)), p)),
        Ok(scan) => {
            for key in scan.keys.iter().take(MAX_KEYS) {
                lines.push(Line::styled(format!("  {}", key.name), Style::new().fg(p.ink_strong).add_modifier(Modifier::BOLD)));
                lines.push(key_summary(key, p));
            }
            if scan.keys.len() > MAX_KEYS {
                lines.push(hint(format!("  … and {} more", scan.keys.len() - MAX_KEYS), p));
            }
        }
    }
    lines
}

/// The line under a key's name: `RSA 4096, created 2023-11-15`, or what is wrong with it.
fn key_summary(key: &LocalKey, p: &Palette) -> Line<'static> {
    let created = key.created.and_then(iso_date);
    match (&key.details, created) {
        (Details::Public(public), Some(date)) => Line::styled(format!("    {}, created {date}", public.algorithm), Style::new().fg(p.ink)),
        (Details::Public(public), None) => Line::styled(format!("    {}", public.algorithm), Style::new().fg(p.ink)),
        (Details::PrivateOnly, _) => Line::styled("    Private key only, no .pub file", Style::new().fg(p.ink)),
        (Details::Unreadable(reason), _) => Line::styled(format!("    {reason}"), Style::new().fg(p.bad)),
    }
}

/// Where sessions go. Until one is open it says how to start one.
fn draw_sessions(frame: &mut Frame, area: Rect, p: &Palette) {
    let block = panel(p).style(Style::new().bg(p.surface));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [_, middle, _] = Layout::vertical([Constraint::Fill(1), Constraint::Length(3), Constraint::Fill(1)]).areas(inner);
    let text = vec![
        Line::styled("CloudNode", Style::new().fg(p.ink_strong).add_modifier(Modifier::BOLD)),
        Line::styled("──", Style::new().fg(p.accent)),
        Line::styled("Pick a host in the sidebar to open a session.", Style::new().fg(p.ink)),
    ];
    frame.render_widget(Paragraph::new(text).alignment(Alignment::Center), middle);
}

/// The key hints on the bottom row.
fn draw_status(frame: &mut Frame, area: Rect, p: &Palette) {
    let line = Line::from(vec![
        Span::styled(" q ", Style::new().fg(p.on_accent).bg(p.accent).add_modifier(Modifier::BOLD)),
        Span::styled(" quit", Style::new().fg(p.ink)),
    ]);
    frame.render_widget(Paragraph::new(line).style(Style::new().bg(p.content_bg)), area);
}

/// A rounded panel outlined in the line colour.
fn panel(p: &Palette) -> Block<'static> {
    Block::bordered().border_type(BorderType::Rounded).border_style(Style::new().fg(p.line))
}

fn header(text: &'static str, p: &Palette) -> Line<'static> {
    Line::styled(text, Style::new().fg(p.ink).add_modifier(Modifier::BOLD))
}

fn hint(text: impl Into<Cow<'static, str>>, p: &Palette) -> Line<'static> {
    Line::styled(text, Style::new().fg(p.faint))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{Duration, UNIX_EPOCH};

    use cloudnode_core::{KeyAlgorithm, Tokens};
    use cloudnode_keys::PublicDetails;

    use super::*;

    fn scan(count: usize) -> Result<Scan, ScanError> {
        let keys = (0..count)
            .map(|i| LocalKey { name: format!("key{i}"), public_file: None, private_file: None, details: Details::PrivateOnly, created: None })
            .collect();
        Ok(Scan { dir: PathBuf::from("/tmp/ssh"), keys })
    }

    fn text(line: &Line<'_>) -> String {
        line.spans.iter().map(|span| span.content.as_ref()).collect()
    }

    #[test]
    fn lists_every_key_under_the_header() {
        let lines = key_lines(&scan(2), &Palette::new(&Tokens::DARK));
        let texts: Vec<String> = lines.iter().map(text).collect();
        assert_eq!(texts, ["KEY MANAGEMENT", "  key0", "    Private key only, no .pub file", "  key1", "    Private key only, no .pub file"]);
    }

    #[test]
    fn readable_keys_show_type_and_date() {
        let details = PublicDetails { algorithm: KeyAlgorithm::Rsa { bits: 4096 }, fingerprint: "SHA256:x".into(), comment: String::new() };
        let key = LocalKey {
            name: "id_rsa_prod".into(),
            public_file: None,
            private_file: None,
            details: Details::Public(details),
            created: Some(UNIX_EPOCH + Duration::from_secs(19_742 * 86_400)),
        };
        assert_eq!(text(&key_summary(&key, &Palette::new(&Tokens::DARK))), "    RSA 4096, created 2024-01-20");
    }

    #[test]
    fn long_lists_end_with_a_count() {
        let lines = key_lines(&scan(MAX_KEYS + 3), &Palette::new(&Tokens::DARK));
        assert_eq!(lines.len(), 1 + 2 * MAX_KEYS + 1);
        assert_eq!(lines.last().map(text).as_deref(), Some("  … and 3 more"));
    }

    #[test]
    fn no_keys_says_where_it_looked() {
        let lines = key_lines(&scan(0), &Palette::new(&Tokens::DARK));
        assert_eq!(lines.len(), 2);
        assert!(text(&lines[1]).starts_with("  No keys in "));
    }
}
