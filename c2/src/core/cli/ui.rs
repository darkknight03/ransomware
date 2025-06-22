use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
    style::{Color, Style},
};

use crate::core::cli::app::{App, FocusPane};
use crate::utils::logging::Logging;


fn log_color(level: &Logging) -> Color {
    match level {
        Logging::INFO => Color::Cyan,
        Logging::DEBUG => Color::Magenta,
        Logging::SUCCESS => Color::Green,
        Logging::ERROR => Color::Red,
        Logging::NETWORK => Color::Yellow,
        Logging::RESULT => Color::Blue,
    }
}

pub fn render_ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(f.area());

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),        // output
            Constraint::Length(3),     // input
        ])
        .split(chunks[0]);

    render_output(f, app, main_chunks[0]);
    render_input(f, app, main_chunks[1]);
    render_logs(f, app, chunks[1]);
}

// ────────────────────────────────
// Output Pane
fn render_output(f: &mut Frame, app: &mut App, area: Rect) {
    let output_len = app.output.len() as u16;
    let visible_height = area.height.saturating_sub(2); // account for borders

    // Always scroll to bottom if new output added
    if output_len > visible_height {
        app.output_scroll = output_len - visible_height;
    } else {
        app.output_scroll = 0;
    }

    let block = Block::default()
        .title("Command Output")
        .borders(Borders::ALL)
        .border_style(if matches!(app.focus, FocusPane::Output) {
            Style::default().fg(Color::LightBlue)
        } else {
            Style::default()
        });

    let para = Paragraph::new(app.output.clone())
        .block(block)
        .wrap(Wrap { trim: true })
        .scroll((app.output_scroll, 0));

    f.render_widget(para, area);
}

// ────────────────────────────────
// Input Prompt
fn render_input(f: &mut Frame, app: &App, area: Rect) {
    let prompt = format!("c2[{}]> {}", app.current_agent, app.input);
    let block = Block::default()
        .title("Prompt")
        .borders(Borders::ALL)
        .border_style(if matches!(app.focus, FocusPane::Input) {
            Style::default().fg(Color::LightBlue)
        } else {
            Style::default()
        });

    let para = Paragraph::new(prompt)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(para, area);
}

// ────────────────────────────────
// Logs Pane
fn render_logs(f: &mut Frame, app: &mut App, area: Rect) {
    let logs_len = app.logs.len() as u16;
    let visible_height = area.height.saturating_sub(2);

    // Always scroll to bottom if new output added
    if logs_len > visible_height {
        app.output_scroll = logs_len - visible_height;
    } else {
        app.output_scroll = 0;
    }

    let lines: Vec<Line> = app
        .logs
        .iter()
        .map(|(level, msg)| {
            Line::styled(
                format!("[{:?}] {}", level, msg),
                Style::default().fg(log_color(level)),
            )
        })
        .collect();

    let block = Block::default()
        .title("Logs")
        .borders(Borders::ALL)
        .border_style(if matches!(app.focus, FocusPane::Logs) {
            Style::default().fg(Color::LightBlue)
        } else {
            Style::default()
        });

    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .scroll((app.log_scroll, 0));

    f.render_widget(para, area);
}

pub fn _render_ui2(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(f.area());

    // Split left pane (main) into output and input areas
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),        // output area
            Constraint::Length(3),    // input box
        ])
        .split(chunks[0]);

    // Output area (left top)
    let output_para = Paragraph::new(app.output.clone())
        .block(Block::default().title("Command Output").borders(Borders::ALL))
        .wrap(Wrap { trim: true })
        .scroll((app.output_scroll, 0));
    f.render_widget(output_para, main_chunks[0]);

    // Input area (left bottom)
    let prompt_text = format!("c2[{}]> {}", app.current_agent, app.input);
    let input_para = Paragraph::new(prompt_text)
        .block(Block::default().title("Prompt").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    f.render_widget(input_para, main_chunks[1]);

    // Logs pane (right)
    let mut text_lines = Vec::new();
    for (level, msg) in &app.logs {
        text_lines.push(Line::styled(
            format!("[{:?}] {}", level, msg),
            Style::default().fg(log_color(level)),
        ));
    }

    let log_pane = Paragraph::new(text_lines)
        .block(Block::default().title("Logs").borders(Borders::ALL))
        .wrap(Wrap { trim: true })
        .scroll((app.log_scroll, 0));
    f.render_widget(log_pane, chunks[1]);
}


