use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, EditField, Screen};

pub fn render_screen(frame: &mut Frame, app: &App) {
    // Take the frame area and cut it vertically into three blocks.
    // - Top and bottom ones are fixed 3 character size.
    // - Middle one enlarges or shrinks as needed.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // Prepare the title
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let title = Paragraph::new(Text::styled(
        "Create new JSON",
        Style::default().fg(Color::Green),
    ))
    .block(title_block);

    // Prepare the list of JSON pairs
    let list_items = app.pairs.iter().map(|(k, v)| {
        ListItem::new(Line::from(Span::styled(
            format!("{k:<25} : {v}"),
            Style::default().fg(Color::Yellow),
        )))
    });

    let list = List::new(list_items);

    // Prepare sections for the footer
    let mode_footer = {
        use crate::app::EditField;
        use crate::app::Screen::*;
        use ratatui::style::Color::*;

        let (mode_text, mode_fg_color) = match &app.current_screen {
            Home => ("Normal Mode", Green),
            Editing { .. } => ("Editing Mode", Yellow),
            Exiting => ("Exiting", LightRed),
        };
        let (edit_text, edit_fg_color) = match &app.current_screen {
            Editing { field, .. } if *field == EditField::Key => ("Editing JSON key", Green),
            Editing { field, .. } if *field == EditField::Value => {
                ("Editing JSON value", LightGreen)
            }
            _ => ("Not editing anything", DarkGray),
        };

        Paragraph::new(Line::from(vec![
            Span::styled(mode_text, Style::default().fg(mode_fg_color)),
            Span::styled(" | ", Style::default().fg(White)),
            Span::styled(edit_text, Style::default().fg(edit_fg_color)),
        ]))
        .block(Block::default().borders(Borders::ALL))
    };

    let hint_footer = {
        use crate::app::Screen::*;

        let keys_hint = match &app.current_screen {
            Home => "Quit [q] Create pair [e]",
            Editing { .. } => "Cancel [esc] Switch boxes [tab] Complete [enter]",
            Exiting => "Output JSON? [y/n] Force Quit [q]",
        };

        Paragraph::new(Line::from(Span::styled(
            keys_hint,
            Style::default().fg(Color::Red),
        )))
        .block(Block::default().borders(Borders::ALL))
    };

    // Prepare (nested) layout for footer
    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    // Render components in frame
    frame.render_widget(title, chunks[0]);
    frame.render_widget(list, chunks[1]);
    frame.render_widget(mode_footer, footer_chunks[0]);
    frame.render_widget(hint_footer, footer_chunks[1]);

    // Render the popup for editing new kv-pairs if needed
    if let Screen::Editing {
        key_input,
        value_input,
        field,
    } = &app.current_screen
    {
        let popup_block = Block::default()
            .title("Enter a new key-value pair")
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));

        let area = centered_popup(60, 25, frame.area());

        let popup_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let mut key_block = Block::default().title("Key").borders(Borders::ALL);
        let mut value_block = Block::default().title("Value").borders(Borders::ALL);

        let active_style = Style::default().bg(Color::LightYellow).fg(Color::Black);

        // Assign the style based on current editing section
        match field {
            EditField::Key => key_block = key_block.style(active_style),
            EditField::Value => value_block = value_block.style(active_style),
        }

        let key_text = Paragraph::new(key_input.clone()).block(key_block);
        let value_text = Paragraph::new(value_input.clone()).block(value_block);

        // Render the popup
        frame.render_widget(popup_block, area);
        frame.render_widget(key_text, popup_chunks[0]);
        frame.render_widget(value_text, popup_chunks[1]);
    };

    // Render the confirmation popup if needed
    if let Screen::Exiting = &app.current_screen {
        // First clear the entire screen
        frame.render_widget(Clear, frame.area());

        let popup_block = Block::default()
            .title("Y/N")
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));

        let exit_text = Text::styled(
            "Would you like to output the buffer as JSON? (y/n)",
            Style::default().fg(Color::Red),
        );

        // trim: false avoids cutting the text on overflow
        let exit_text = Paragraph::new(exit_text)
            .block(popup_block)
            .wrap(ratatui::widgets::Wrap { trim: false });

        // Render the popup
        let area = centered_popup(60, 25, frame.area());
        frame.render_widget(exit_text, area);
    }
}

fn centered_popup(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the rectangle vertically into three blocks based on the
    // requested percentage space for the popup
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Cut the middle piece (popup body) horizontally into another three blocks.
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle section
}
