use std::{fs::File, io::Write, path::Path, thread, time::Duration};

use anyhow::{anyhow, Result};
use blessings::{CursorStyle, Screen};
use crossterm::{event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers}, style::Color, terminal};

#[derive(Copy, Clone, PartialEq)]
enum Mode {
    Normal,
    Insert,
    Command,
}

impl Mode {
    fn to_str(&self) -> &str {
        match self {
            Mode::Normal => "Normal",
            Mode::Insert => "Insert",
            Mode::Command => "Command",
        }
    }

    fn get_color(&self) -> Color {
        match self {
            Mode::Normal => Color::Blue,
            Mode::Insert => Color::Magenta,
            Mode::Command => Color::Green,
        }
    }
}

pub(crate) struct Editor {
    screen: Screen,
    mode: Mode,
    path: Option<String>,
    width: u16,
    height: u16,
    terminate: bool,
    text: Vec<String>,
    col: usize,
    row: usize,
    changed: bool,
    command: String,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let text = vec![String::new()];
        Editor::create(text, None)
    }

    pub fn new_with_file(path: String) -> Result<Self> {
        let text = Editor::load_file(&path)?;

        Editor::create(text, Some(path.to_string()))
    }

    fn create(text: Vec<String>, path: Option<String>) -> Result<Self> {
        let (width, height) = terminal::size()?;

        let col = text[0].len();

        let screen = Screen::new()?;

        Ok(Self {
            screen,
            mode: Mode::Normal,
            path,
            width,
            height,
            terminate: false,
            text,
            col,
            row: 0,
            changed: false,
            command: String::new(),
        })
    }

    fn load_file<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
        let s = std::fs::read_to_string(path)?.replace("\r", "");
        let text = s.split('\n').map(|line| line.to_string()).collect::<Vec<String>>();
        Ok(text)
    }

    pub fn run(&mut self) -> Result<()> {
        self.screen.begin()?;

        for i in 0..self.text.len().min(self.height as usize - 2) {
            self.screen.print_at(0, i as u16, &self.text[i]);
        }

        self.change_mode(Mode::Normal)?;

        while !self.terminate {
            // handle all input events
            while event::poll(std::time::Duration::ZERO)? {
                match event::read()? {
                    Event::Key { 0: key_event } => {
                        self.handle_key(key_event)?;
                    },
                    Event::Resize { 0: width, 1: height } => {
                        self.width = width;
                        self.height = height;
                        self.col = self.col.min(self.width as usize - 1);
                        self.row = self.row.min(self.height as usize - 1);
                        self.screen.resize(width, height);
                        self.print_debug_message(format!("Resized: {}x{}", width, height))?;
                    },
                    e => {
                        self.print_debug_message(format!("unhandled event: {:?}", e))?;
                    },
                }
            }

            // show rendered screen
            self.screen.show()?;

            thread::sleep(Duration::from_millis(16));
        }

        self.screen.end()?;

        Ok(())
    }

    fn handle_key(&mut self, event: KeyEvent) -> Result<()> {
        if event.kind == KeyEventKind::Press {
            match self.mode {
                Mode::Normal => self.handle_keypress_normal(event)?,
                Mode::Insert => self.handle_keypress_insert(event)?,
                Mode::Command => self.handle_keypress_command(event)?,
            }
        }

        Ok(())
    }

    fn handle_keypress_normal(&mut self, event: KeyEvent) -> Result<()> {
        assert!(self.mode == Mode::Normal);
        assert!(event.kind == KeyEventKind::Press);

        match event.code {
            KeyCode::Up => {
                self.move_up()?;
            },
            KeyCode::Down => {
                self.move_down()?;
            },
            KeyCode::Left => {
                self.move_left()?;
            },
            KeyCode::Right => {
                self.move_right()?;
            },
            KeyCode::Char(':') => {
                self.change_mode(Mode::Command)?;
            },
            KeyCode::Char(c) => {
                if event.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        's' => {
                            if self.path.is_some() {
                                self.save_file()?;
                            }
                        },
                        _ => {},
                    }
                } else {
                    match c {
                        'i' => {
                            self.change_mode(Mode::Insert)?;
                        },
                        _ => {},
                    }
                }
            },
            _ => {},
        }

        Ok(())
    }

    fn handle_keypress_insert(&mut self, event: KeyEvent) -> Result<()> {
        assert!(self.mode == Mode::Insert);
        assert!(event.kind == KeyEventKind::Press);

        match event.code {
            KeyCode::Esc => {
                self.change_mode(Mode::Normal)?;
            },
            KeyCode::Backspace => {
                if self.col > 0 {
                    if self.col == self.text[self.row].len() {
                        self.text[self.row].pop();
                        self.col -= 1;
                        self.move_to_current_position();
                        self.screen.print_char(Screen::EMPTY_CHAR);
                        self.move_to_current_position();
                    } else {
                        self.col -= 1;
                        self.text[self.row].remove(self.col);
                        self.move_to_current_position();
                        self.screen.clear(blessings::ClearType::UntilNewline);
                        self.screen.print(&self.text[self.row][self.col..]);
                        self.move_to_current_position();
                    }

                    self.changed = true;
                } else {
                    if self.row > 0 {
                        let old_line = self.text.remove(self.row);
                        self.row -= 1;
                        self.col = self.text[self.row].len();

                        // append remainder of removed line to new current line
                        self.move_to_current_position();
                        self.screen.print(&old_line[..old_line.len().min(self.width as usize - self.text[self.row].len())]);
                        self.text[self.row].push_str(&old_line);

                        // move up subsequent lines
                        for row in (self.row as u16 + 1)..(self.height - 2).min(self.text.len() as u16) {
                            self.screen.move_to(0, row);
                            self.screen.clear(blessings::ClearType::CurrentLine);
                            self.screen.print(&self.text[row as usize][..self.text[row as usize].len()]);
                        }
                        // clear previously last line
                        self.screen.move_to(0, self.text.len() as u16);
                        self.screen.clear(blessings::ClearType::CurrentLine);

                        // fix cursor position
                        self.move_to_current_position();

                        self.changed = true;
                    }
                }
            },
            KeyCode::Enter => {
                if self.text.len() < self.height as usize - 2 {
                    if self.col == self.text[self.row].len() {
                        self.text.insert(self.row + 1, String::new());
                        self.row += 1;
                        self.col = 0;

                        // shift subsequent lines down
                        for row in self.row..self.text.len().min(self.height as usize - 2) {
                            self.screen.move_to(0, row as u16);
                            self.screen.clear(blessings::ClearType::CurrentLine);
                            self.screen.print(&self.text[row][..self.text[row].len()]);
                        }

                        self.move_to_current_position();
                    } else {
                        // FIXME: this won't work for multibyte-encodings
                        let new_line = self.text[self.row].split_off(self.col);
                        self.row += 1;
                        self.col = 0;
                        self.text.insert(self.row, new_line);

                        // reprint changed lines and shift down subsequent lines
                        for row in (self.row - 1)..self.text.len().min(self.height as usize - 2) {
                            self.screen.move_to(0, row as u16);
                            self.screen.clear(blessings::ClearType::CurrentLine);
                            self.screen.print(&self.text[row][..self.text[row].len()]);
                        }

                        self.move_to_current_position();
                    }

                    self.changed = true;
                }
            },
            KeyCode::Up => {
                self.move_up()?;
            },
            KeyCode::Down => {
                self.move_down()?;
            },
            KeyCode::Left => {
                self.move_left()?;
            },
            KeyCode::Right => {
                self.move_right()?;
            },
            KeyCode::Char(c) => {
                if event.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'c' => {
                            self.change_mode(Mode::Normal)?;
                        },
                        _ => {},
                    }
                } else {
                    if self.col < self.width as usize {
                        self.screen.print_char(c);

                        if self.col == self.text[self.row].len() {
                            self.text[self.row].push(c);
                            self.col += 1;
                        } else {
                            self.text[self.row].insert(self.col, c);
                            self.col += 1;

                            self.screen.print(&self.text[self.row][self.col..]);

                            self.move_to_current_position();
                        }

                        self.changed = true;
                    }
                }
            },
            _ => {},
        }

        Ok(())
    }

    fn handle_keypress_command(&mut self, event: KeyEvent) -> Result<()> {
        assert!(self.mode == Mode::Command);
        assert!(event.kind == KeyEventKind::Press);

        match event.code {
            KeyCode::Esc => {
                self.command.clear();
                self.clear_message()?;
                self.change_mode(Mode::Normal)?;
            }
            KeyCode::Backspace => {
                if self.command.len() > 0 {
                    self.screen.move_to(self.command.len() as u16, self.height - 1);
                    self.screen.print_char(Screen::EMPTY_CHAR);
                    self.screen.move_to(self.command.len() as u16, self.height - 1);

                    self.command.pop();
                } else {
                    self.screen.move_to(0, self.height - 1);
                    self.screen.print_char(Screen::EMPTY_CHAR);
                    self.change_mode(Mode::Normal)?;
                }
            },
            KeyCode::Enter => {
                self.change_mode(Mode::Normal)?;

                self.execute_command()?;
                self.command.clear();
            },
            KeyCode::Char(c) => {
                if event.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'c' => {
                            self.command.clear();
                            self.clear_message()?;
                            self.change_mode(Mode::Normal)?;
                        },
                        _ => {},
                    }
                } else {
                    self.screen.print_char(c);

                    self.command.push(c);
                }
            },
            _ => {},
        }

        Ok(())
    }

    fn move_up(&mut self) -> Result<()> {
        if self.row > 0 {
            self.row -= 1;
            self.col = self.col.min(self.text[self.row].len());
            self.move_to_current_position();
        }

        Ok(())
    }

    fn move_down(&mut self) -> Result<()> {
        if self.row < self.height as usize - 3 && self.row < self.text.len() - 1 {
            self.row += 1;
            self.col = self.col.min(self.text[self.row].len());
            self.move_to_current_position();
        }

        Ok(())
    }

    fn move_left(&mut self) -> Result<()> {
        if self.col > 0 {
            self.col -= 1;
            self.move_to_current_position();
        }

        Ok(())
    }

    fn move_right(&mut self) -> Result<()> {
        if self.col < self.width as usize - 1 && self.col < self.text[self.row].len() {
            self.col += 1;
            self.move_to_current_position();
        }

        Ok(())
    }

    fn change_mode(&mut self, mode: Mode) -> Result<()> {
        self.mode = mode;
        
        self.print_mode(mode)?;
        match mode {
            Mode::Command => {
                self.screen.move_to(0, self.height - 1);
                self.screen.clear(blessings::ClearType::CurrentLine);
                self.screen.print_char(':');
            },
            Mode::Normal | Mode::Insert => {
                self.move_to_current_position();
            },
        }

        match mode {
            Mode::Normal | Mode::Command => {
                self.screen.set_cursor_style(CursorStyle::SteadyBlock);
            },
            Mode::Insert => {
                self.screen.set_cursor_style(CursorStyle::SteadyBar);
            },
        }

        Ok(())
    }

    fn print_mode(&mut self, mode: Mode) -> Result<()> {
        self.screen.move_to(0, self.height - 2);
        self.screen.clear(blessings::ClearType::CurrentLine);

        // TODO: make text bold
        self.screen.set_colors(Color::Black, mode.get_color());
        self.screen.print(format!(" {} ", mode.to_str()));
        self.screen.clear_colors();

        Ok(())
    }

    fn move_to_current_position(&mut self) {
        self.screen.move_to(self.col as u16, self.row as u16);
    }
    
    fn print_message<S: std::fmt::Display>(&mut self, message: S) -> Result<()> {
        self.screen.save_position();

        self.screen.move_to(0, self.height - 1);
        self.screen.clear(blessings::ClearType::CurrentLine);

        self.screen.print(&message.to_string());

        self.screen.restore_position();

        Ok(())
    }

    fn print_debug_message<S: std::fmt::Display>(&mut self, message: S) -> Result<()> {
        self.screen.save_position();

        self.screen.move_to(0, self.height - 1);
        self.screen.clear(blessings::ClearType::CurrentLine);

        self.screen.set_colors(Color::Black, Color::Magenta);
        self.screen.print(&message.to_string());
        self.screen.clear_colors();

        self.screen.restore_position();

        Ok(())
    }

    fn print_error_message<S: std::fmt::Display>(&mut self, message: S) -> Result<()> {
        self.screen.save_position();

        self.screen.move_to(0, self.height - 1);
        self.screen.clear(blessings::ClearType::CurrentLine);

        self.screen.set_colors(Color::Black, Color::Red);
        self.screen.print(&message.to_string());
        self.screen.clear_colors();

        self.screen.restore_position();

        Ok(())
    }

    fn clear_message(&mut self) -> Result<()> {
        self.screen.save_position();
        self.screen.move_to(0, self.height - 1);
        self.screen.clear(blessings::ClearType::CurrentLine);
        self.screen.restore_position();

        Ok(())
    }

    fn save_file(&mut self) -> Result<()> {
        if let Some(path) = &self.path {
            let mut file = File::create(path)?;
            for i in 0..(self.text.len() - 1) {
                file.write_all(&self.text[i].as_bytes())?;
                file.write_all("\n".as_bytes())?;
            }
            file.write_all(self.text.last().unwrap().as_bytes())?;

            self.print_message(format!("\"{}\" {}L written", path, self.text.len()))?;

            self.changed = false;
            Ok(())
        } else {
            Err(anyhow!("path not set"))
        }
    }

    fn execute_command(&mut self) -> Result<()> {
        if self.command.starts_with("print ") {
            self.print_message(self.command["print ".len()..].to_string())?;
        } else if self.command == "q" {
            if self.changed {
                // not necessary to clear here because the error message is longer than the command
                self.print_error_message("No write since last change")?;
            } else {
                self.terminate = true;
            }
        } else if self.command == "q!" {
            self.terminate = true;
        } else if self.command == "w" {
            if self.path.is_some() {
                self.save_file()?;
            } else {
                // not necessary to clear here because the error message is longer than the command
                self.print_error_message("No opened file, can't write")?;
            }
        } else if self.command == "wq" {
            if self.path.is_some() {
                self.save_file()?;
                self.terminate = true;
            } else {
                // not necessary to clear here because the error message is longer than the command
                self.print_error_message("No opened file, can't write")?;
            }
        } else {
            self.print_error_message(format!("Not an editor command: {}", self.command))?;
        }

        Ok(())
    }
}
