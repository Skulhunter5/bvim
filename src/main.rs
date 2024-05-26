use std::io::{stdout, Write};

use crossterm::{terminal::{self, LeaveAlternateScreen}, QueueableCommand};
use editor::Editor;

mod editor;

pub fn main() {
    let mut editor = Editor::new().unwrap();
    editor.run().map_err(|e| {
        stdout().queue(LeaveAlternateScreen).unwrap();
        stdout().flush().unwrap();
        terminal::disable_raw_mode().unwrap();
        e
    }).unwrap();
}
