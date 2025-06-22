use crate::core::cli::app::{App, FocusPane};
use crate::utils::logging::{Logging, LogEntry};
use crate::core::c2::C2;
use crate::core::cli::commands;


pub fn up(app: &mut App) {
    if matches!(app.focus, FocusPane::Output) {
        // Scroll output pane up
        app.output_scroll = app.output_scroll.saturating_sub(1);
        
    } else if matches!(app.focus, FocusPane::Logs){
        // Scroll log pane up
        app.log_scroll = app.log_scroll.saturating_sub(1);
    } else {
        // Command history up
        if app.input_history.is_empty() { }
        let max = app.input_history.len() - 1;
        app.history_index = Some(match app.history_index {
            Some(0) | None => max,
            Some(i) => i.saturating_sub(1),
        });
    
        if let Some(i) = app.history_index {
            app.input = app.input_history[i].clone();
        }
    }
}

pub fn down(app: &mut App) {
    if matches!(app.focus, FocusPane::Output) {
        // Scroll output pane down
        app.output_scroll = app.output_scroll.saturating_add(1);
        
    } else if matches!(app.focus, FocusPane::Logs){
        // Scroll log pane down
        app.log_scroll = app.log_scroll.saturating_add(1);
    } else {
        // Command history down
        if app.input_history.is_empty() { }
    
        app.history_index = match app.history_index {
            None => None,
            Some(i) if i >= app.input_history.len() - 1 => None,
            Some(i) => Some(i + 1),
        };
    
        app.input = match app.history_index {
            Some(i) => app.input_history[i].clone(),
            None => String::new(),
        };
    }
}

