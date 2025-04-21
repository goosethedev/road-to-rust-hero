use std::collections::HashMap;

use ratatui::crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Default)]
pub struct App {
    pub current_screen: Screen,
    pub pairs: HashMap<String, String>,
}

#[derive(Debug, Default)]
pub enum Screen {
    #[default]
    Home,
    Editing {
        key_input: String,
        value_input: String,
        field: EditField,
    },
    Exiting,
}

#[derive(Debug, PartialEq, Eq, Default)]
pub enum EditField {
    #[default]
    Key,
    Value,
}

impl App {
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Option<bool> {
        use KeyCode::{Backspace, Char, Enter, Esc, Tab};

        match self.current_screen {
            // At Home
            Screen::Home => match key.code {
                Char('e') => {
                    self.current_screen = Screen::Editing {
                        key_input: "".into(),
                        value_input: "".into(),
                        field: EditField::Key,
                    };
                }
                Char('q') => {
                    self.current_screen = Screen::Exiting;
                }
                _ => {}
            },
            // At Editing form
            Screen::Editing {
                ref mut key_input,
                ref mut value_input,
                ref mut field,
            } => match (&field, key.code) {
                (EditField::Key, Tab) => *field = EditField::Value,
                (EditField::Value, Tab) => *field = EditField::Key,
                (EditField::Key, Backspace) => _ = key_input.pop(),
                (EditField::Value, Backspace) => _ = value_input.pop(),
                (EditField::Key, Char(c)) => key_input.push(c),
                (EditField::Value, Char(c)) => value_input.push(c),
                (EditField::Key, Enter) => *field = EditField::Value,
                (EditField::Value, Enter) => {
                    self.pairs.insert(key_input.clone(), value_input.clone());
                    self.current_screen = Screen::Home;
                }
                (_, Esc) => self.current_screen = Screen::Home,
                _ => {}
            },
            // At Exit dialog
            Screen::Exiting => match key.code {
                Char('y') => return Some(true),
                Char('n' | 'q') => return Some(false),
                Esc => self.current_screen = Screen::Home,
                _ => {}
            },
        };
        None
    }

    pub fn print_json(&self) -> serde_json::Result<()> {
        let output = serde_json::to_string(&self.pairs)?;
        println!("{}", output);
        Ok(())
    }
}
