use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::editor::Mode;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Modifiers {
    Any,
    Match(KeyModifiers),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Key {
    mode: Mode,
    key: KeyCode,
    modifiers: Modifiers,
}

impl Key {
    fn any(mode: Mode, key: KeyCode) -> Self {
        Self {
            mode,
            key,
            modifiers: Modifiers::Any,
        }
    }

    fn unmodified(mode: Mode, key: KeyCode) -> Self {
        Self {
            mode,
            key,
            modifiers: Modifiers::Match(KeyModifiers::empty()),
        }
    }

    fn modified(mode: Mode, key: KeyCode, modifiers: KeyModifiers) -> Self {
        Self {
            mode,
            key,
            modifiers: Modifiers::Match(modifiers),
        }
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
    MoveToStartOfLine,
    MoveToEndOfLine,
    MoveToFirstCharacterInLine,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyMap {
    mappings: HashMap<Key, Vec<Action>>,
}

impl KeyMap {
    pub fn handle(&self, mode: Mode, event: KeyEvent) -> Option<Vec<Action>> {
        // Pass through typed characters in Mode::Insert and Mode::Command
        if event.modifiers.is_empty() || event.modifiers.eq(&KeyModifiers::SHIFT) {
            if let KeyCode::Char(c) = event.code {
                if mode == Mode::Insert {
                    return Some(vec![Action::InsertChar(c)]);
                }
                if mode == Mode::Command {
                    return Some(vec![Action::InsertCharCommand(c)]);
                }
            }
        }

        // First check for a result with the given modifiers
        let key = Key::modified(mode, event.code, event.modifiers);
        let res = self.mappings.get(&key);
        if let Some(res) = res {
            return Some(res.clone());
        }
        // If no result is found, check for a result with any modifiers
        let key_any = Key::any(mode, event.code);
        let res_any = self.mappings.get(&key_any);
        res_any.cloned()
    }
}

impl Default for KeyMap {
    fn default() -> Self {
        let mut mappings = HashMap::new();
        // Mode changes
        mappings.insert(
            Key::unmodified(Mode::Normal, KeyCode::Char('i')),
            vec![Action::ChangeMode(Mode::Insert)],
        );
        mappings.insert(
            Key::unmodified(Mode::Normal, KeyCode::Char('a')),
            vec![Action::MoveRight, Action::ChangeMode(Mode::Insert)],
        );
        mappings.insert(
            Key::unmodified(Mode::Normal, KeyCode::Char(':')),
            vec![Action::ChangeMode(Mode::Command)],
        );
        mappings.insert(
            Key::unmodified(Mode::Insert, KeyCode::Esc),
            vec![Action::ChangeMode(Mode::Normal)],
        );
        mappings.insert(
            Key::modified(Mode::Insert, KeyCode::Char('c'), KeyModifiers::CONTROL),
            vec![Action::ChangeMode(Mode::Normal)],
        );
        mappings.insert(
            Key::unmodified(Mode::Command, KeyCode::Esc),
            vec![Action::ChangeMode(Mode::Normal)],
        );
        mappings.insert(
            Key::modified(Mode::Command, KeyCode::Char('c'), KeyModifiers::CONTROL),
            vec![Action::ChangeMode(Mode::Normal)],
        );
        // Arrow key movement
        mappings.insert(
            Key::unmodified(Mode::Normal, KeyCode::Up),
            vec![Action::MoveUp],
        );
        mappings.insert(
            Key::unmodified(Mode::Normal, KeyCode::Down),
            vec![Action::MoveDown],
        );
        mappings.insert(
            Key::unmodified(Mode::Normal, KeyCode::Left),
            vec![Action::MoveLeft],
        );
        mappings.insert(
            Key::unmodified(Mode::Normal, KeyCode::Right),
            vec![Action::MoveRight],
        );
        mappings.insert(
            Key::unmodified(Mode::Insert, KeyCode::Up),
            vec![Action::MoveUp],
        );
        mappings.insert(
            Key::unmodified(Mode::Insert, KeyCode::Down),
            vec![Action::MoveDown],
        );
        mappings.insert(
            Key::unmodified(Mode::Insert, KeyCode::Left),
            vec![Action::MoveLeft],
        );
        mappings.insert(
            Key::unmodified(Mode::Insert, KeyCode::Right),
            vec![Action::MoveRight],
        );
        // Homing keys
        mappings.insert(
            Key::unmodified(Mode::Normal, KeyCode::Home),
            vec![Action::MoveToStartOfLine],
        );
        mappings.insert(
            Key::unmodified(Mode::Insert, KeyCode::Home),
            vec![Action::MoveToStartOfLine],
        );
        mappings.insert(
            Key::unmodified(Mode::Normal, KeyCode::End),
            vec![Action::MoveToEndOfLine],
        );
        mappings.insert(
            Key::unmodified(Mode::Insert, KeyCode::End),
            vec![Action::MoveToEndOfLine],
        );
        // Vim-style movement
        mappings.insert(
            Key::unmodified(Mode::Normal, KeyCode::Char('k')),
            vec![Action::MoveUp],
        );
        mappings.insert(
            Key::unmodified(Mode::Normal, KeyCode::Char('j')),
            vec![Action::MoveDown],
        );
        mappings.insert(
            Key::unmodified(Mode::Normal, KeyCode::Char('h')),
            vec![Action::MoveLeft],
        );
        mappings.insert(
            Key::unmodified(Mode::Normal, KeyCode::Char('l')),
            vec![Action::MoveRight],
        );
        // Mode::Insert -- KeyCode::Enter, KeyCode::BackSpace, KeyCode::Delete
        mappings.insert(
            Key::unmodified(Mode::Insert, KeyCode::Enter),
            vec![Action::InsertChar('\n')],
        );
        mappings.insert(
            Key::unmodified(Mode::Insert, KeyCode::Backspace),
            vec![Action::RemoveChar],
        );
        mappings.insert(
            Key::unmodified(Mode::Insert, KeyCode::Delete),
            vec![Action::DeleteChar],
        );
        // Mode::Normal -- KeyCode::Delete, KeyCode::Char('x')
        mappings.insert(
            Key::unmodified(Mode::Normal, KeyCode::Delete),
            vec![Action::DeleteChar],
        );
        mappings.insert(
            Key::unmodified(Mode::Normal, KeyCode::Char('x')),
            vec![Action::DeleteChar],
        );
        // Mode::Command
        mappings.insert(
            Key::unmodified(Mode::Command, KeyCode::Enter),
            vec![Action::ExecuteCommand],
        );
        mappings.insert(
            Key::unmodified(Mode::Command, KeyCode::Backspace),
            vec![Action::RemoveCharCommand],
        );
        // Advanced movements
        mappings.insert(
            Key::unmodified(Mode::Normal, KeyCode::Char('0')),
            vec![Action::MoveToStartOfLine],
        );
        mappings.insert(
            Key::any(Mode::Normal, KeyCode::Char('^')),
            vec![Action::MoveToFirstCharacterInLine],
        );
        mappings.insert(
            Key::any(Mode::Normal, KeyCode::Char('$')),
            vec![Action::MoveToEndOfLine],
        );
        mappings.insert(
            Key::modified(Mode::Normal, KeyCode::Char('A'), KeyModifiers::SHIFT),
            vec![Action::MoveToEndOfLine, Action::ChangeMode(Mode::Insert)],
        );
        mappings.insert(
            Key::modified(Mode::Normal, KeyCode::Char('I'), KeyModifiers::SHIFT),
            vec![
                Action::MoveToFirstCharacterInLine,
                Action::ChangeMode(Mode::Insert),
            ],
        );

        Self { mappings }
    }
}
