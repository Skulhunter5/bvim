use std::{
    env::args,
    io::{stdout, Write},
    process::exit,
};

use crossterm::{
    terminal::{self, LeaveAlternateScreen},
    QueueableCommand,
};
use editor::Editor;

mod buffer;
mod editor;
mod keymap;
mod util;
mod window;

const HELP_MESSAGE: &str = "\
USAGE: bvim [OPTIONS] [file]

Options:
  -h, --help  Print this help message \
";

fn main() {
    let mut filepath: Option<String> = None;

    let mut args = args();
    args.next(); // void program path
    loop {
        let arg = match args.next() {
            Some(arg) => arg,
            None => break,
        };

        if arg == "--help" || arg == "-h" {
            println!("{}", HELP_MESSAGE);
            exit(0);
        }

        if filepath.is_some() {
            eprintln!("bvim: error: multiple files supplied");
            exit(-1);
        }
        filepath = Some(arg);
    }

    let mut editor = Editor::new(filepath).unwrap();

    match editor.run() {
        Ok(()) => {
            stdout().queue(LeaveAlternateScreen).unwrap();
            stdout().flush().unwrap();
            terminal::disable_raw_mode().unwrap();
        }
        e => e.unwrap(),
    }
}
