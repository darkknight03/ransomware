use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
    style::{Color, Style, Stylize},
};
use crate::core::cli::app::App;
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

// pub fn old_render_ui(f: &mut Frame, app: &App) {
//     let chunks = Layout::default()
//         .direction(Direction::Horizontal)
//         .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
//         .split(f.area());

//     // Main Pane (Command Output)
//     let output = app.output.join("\n");
//     let main = Paragraph::new(output)
//         .block(Block::default().title("Command Output").borders(Borders::ALL))
//         .wrap(Wrap { trim: true })
//         .scroll((app.output_scroll, 0));

//     // Log Pane (Color-coded)
//     let mut text_lines = vec![];
//     for (level, msg) in &app.logs {
//         text_lines.push(Line::styled(
//             format!("[{:?}] {}", level, msg),
//             Style::default().fg(log_color(level)),
//         ));
//     }

//     let log_pane = Paragraph::new(text_lines)
//         .block(Block::default().title("Logs").borders(Borders::ALL))
//         .wrap(Wrap { trim: true })
//         .scroll((app.log_scroll, 0));

//     f.render_widget(main, chunks[0]);
//     f.render_widget(log_pane, chunks[1]);
// }

// pub fn render_ui_bad(f: &mut Frame, app: &App) {
//     let chunks = Layout::default()
//         .direction(Direction::Horizontal)
//         .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
//         .split(f.area());

//     // Layout inside the left/main pane: output + prompt
//     let main_chunks = Layout::default()
//         .direction(Direction::Vertical)
//         .constraints([
//             Constraint::Min(1),          // history/output
//             Constraint::Length(3),       // input area
//         ])
//         .split(chunks[0]);

//     // Output area
//     let output = app.output.join("\n");
//     let main = Paragraph::new(output)
//         .block(Block::default().title("Command Output").borders(Borders::ALL))
//         .wrap(Wrap { trim: true });
//     f.render_widget(main, main_chunks[0]);

//     // Input prompt
//     let prompt_text = format!("c2[{}]> {}", app.current_agent, app.input);
//     let input_para = Paragraph::new(prompt_text)
//         .block(Block::default().title("Prompt").borders(Borders::ALL))
//         .wrap(Wrap { trim: false });
//     f.render_widget(input_para, main_chunks[1]);

//     // Log pane on the right
//     let mut text_lines = vec![];
//     for (level, msg) in &app.logs {
//         text_lines.push(Line::styled(
//             format!("[{:?}] {}", level, msg),
//             Style::default().fg(log_color(level)),
//         ));
//     }

//     let log_pane = Paragraph::new(text_lines)
//         .block(Block::default().title("Logs").borders(Borders::ALL))
//         .wrap(Wrap { trim: true });
//     f.render_widget(log_pane, chunks[1]);
// }

pub fn render_ui(f: &mut Frame, app: &App) {
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


