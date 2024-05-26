use std::io::{stdout, Write};

use crossterm::{terminal::{self, LeaveAlternateScreen}, QueueableCommand};
use editor::Editor;

mod editor;

pub fn main() {
    let mut editor = Editor::new().unwrap();
    match editor.run() {
        Ok(()) => {
            stdout().queue(LeaveAlternateScreen).unwrap();
            stdout().flush().unwrap();
            terminal::disable_raw_mode().unwrap();
        },
        e => e.unwrap(),
    }
}
