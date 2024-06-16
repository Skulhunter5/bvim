use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::editor::Mode;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Key {
    mode: Mode,
    key: KeyCode,
}

impl Key {
    pub fn new(mode: Mode, key: KeyCode) -> Self {
        Self { mode, key }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Action {
    ChangeMode(Mode),
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    InsertChar(char),
    RemoveChar,
    DeleteChar,
    ExecuteCommand,
    InsertCharCommand(char),
    RemoveCharCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyMap {
    mappings: HashMap<Key, Action>,
}

impl KeyMap {
    pub fn handle(&self, mode: Mode, event: KeyEvent) -> Option<Action> {
        // Pass through typed characters in Mode::Insert and Mode::Command
        if event.modifiers.is_empty() || event.modifiers.eq(&KeyModifiers::SHIFT) {
            if let KeyCode::Char(c) = event.code {
                if mode == Mode::Insert {
                    return Some(Action::InsertChar(c));
                }
                if mode == Mode::Command {
                    return Some(Action::InsertCharCommand(c));
                }
            }
        }
        // FIXME: Add "<C-c>" to change back to Mode::Normal this way for now
        if mode != Mode::Normal
            && event.modifiers.eq(&KeyModifiers::CONTROL)
            && event.code == KeyCode::Char('c')
        {
            return Some(Action::ChangeMode(Mode::Normal));
        }

        let key = Key::new(mode, event.code);
        let res = self.mappings.get(&key);
        res.copied()
    }
}

impl Default for KeyMap {
    fn default() -> Self {
        let mut mappings = HashMap::new();
        // Mode changes
        mappings.insert(
            Key::new(Mode::Normal, KeyCode::Char('i')),
            Action::ChangeMode(Mode::Insert),
        );
        mappings.insert(
            Key::new(Mode::Normal, KeyCode::Char(':')),
            Action::ChangeMode(Mode::Command),
        );
        mappings.insert(
            Key::new(Mode::Insert, KeyCode::Esc),
            Action::ChangeMode(Mode::Normal),
        );
        mappings.insert(
            Key::new(Mode::Command, KeyCode::Esc),
            Action::ChangeMode(Mode::Normal),
        );
        // Arrow key movement
        mappings.insert(Key::new(Mode::Normal, KeyCode::Up), Action::MoveUp);
        mappings.insert(Key::new(Mode::Normal, KeyCode::Down), Action::MoveDown);
        mappings.insert(Key::new(Mode::Normal, KeyCode::Left), Action::MoveLeft);
        mappings.insert(Key::new(Mode::Normal, KeyCode::Right), Action::MoveRight);
        mappings.insert(Key::new(Mode::Insert, KeyCode::Up), Action::MoveUp);
        mappings.insert(Key::new(Mode::Insert, KeyCode::Down), Action::MoveDown);
        mappings.insert(Key::new(Mode::Insert, KeyCode::Left), Action::MoveLeft);
        mappings.insert(Key::new(Mode::Insert, KeyCode::Right), Action::MoveRight);
        // Vim-style movement
        mappings.insert(Key::new(Mode::Normal, KeyCode::Char('k')), Action::MoveUp);
        mappings.insert(Key::new(Mode::Normal, KeyCode::Char('j')), Action::MoveDown);
        mappings.insert(Key::new(Mode::Normal, KeyCode::Char('h')), Action::MoveLeft);
        mappings.insert(
            Key::new(Mode::Normal, KeyCode::Char('l')),
            Action::MoveRight,
        );
        // Mode::Insert -- KeyCode::Enter, KeyCode::BackSpace
        mappings.insert(
            Key::new(Mode::Insert, KeyCode::Enter),
            Action::InsertChar('\n'),
        );
        mappings.insert(
            Key::new(Mode::Insert, KeyCode::Backspace),
            Action::RemoveChar,
        );
        mappings.insert(Key::new(Mode::Insert, KeyCode::Delete), Action::DeleteChar);
        mappings.insert(Key::new(Mode::Normal, KeyCode::Delete), Action::DeleteChar);
        mappings.insert(
            Key::new(Mode::Normal, KeyCode::Char('x')),
            Action::DeleteChar,
        );
        // Mode::Command
        mappings.insert(
            Key::new(Mode::Command, KeyCode::Enter),
            Action::ExecuteCommand,
        );
        mappings.insert(
            Key::new(Mode::Command, KeyCode::Backspace),
            Action::RemoveCharCommand,
        );

        Self { mappings }
    }
}
