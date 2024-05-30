use std::{env::args, io::{stdout, Write}, path::Path, process::exit};

use crossterm::{terminal::{self, LeaveAlternateScreen}, QueueableCommand};
use editor::Editor;

mod editor;

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

    let mut editor = if let Some(path) = filepath {
        let ospath = Path::new(&path);
        if !ospath.exists() {
            eprintln!("bvim: error: file doesn't exist");
            exit(-1);
        }
        if !ospath.is_file() {
            eprintln!("bvim: error: not a file");
            exit(-1);
        }
        Editor::new_with_file(path).unwrap()
    } else {
        Editor::new().unwrap()
    };

    match editor.run() {
        Ok(()) => {
            stdout().queue(LeaveAlternateScreen).unwrap();
            stdout().flush().unwrap();
            terminal::disable_raw_mode().unwrap();
        },
        e => e.unwrap(),
    }
}
