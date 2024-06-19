use std::{thread, time::Duration};

use anyhow::Result;
use blessings::{ClearType, CursorStyle, Screen, WindowBounds};
use crossterm::{
    event::{self, Event, KeyEvent, KeyEventKind, MouseEventKind},
    style::Color,
    terminal,
};

use crate::{
    buffer::Buffer,
    keymap::{Action, KeyMap},
    window::Window,
};

#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Error,
    Debug,
}

#[derive(Debug, Clone)]
pub struct Notification {
    message: String,
    level: LogLevel,
}

impl Notification {
    pub fn new(message: String, level: LogLevel) -> Self {
        Self { message, level }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Mode {
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
    width: u16,
    height: u16,
    keymap: KeyMap,
    window: Window,
    terminate: bool,
    command: String,
    notification: Option<Notification>,
}

impl Editor {
    pub fn new(path: Option<String>) -> Result<Self> {
        let (width, height) = terminal::size()?;

        let screen = Screen::new()?;

        let keymap = KeyMap::default();

        let window_bounds = WindowBounds::new(0, 0, width, height - 2);
        let buffer = if let Some(path) = &path {
            Buffer::new_from_file(path)?
        } else {
            Buffer::new()
        };
        let window = Window::new(buffer, window_bounds);

        Ok(Self {
            screen,
            mode: Mode::Normal,
            width,
            height,
            keymap,
            window,
            terminate: false,
            command: String::new(),
            notification: None,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.screen.begin()?;
        crossterm::QueueableCommand::queue(
            &mut std::io::stdout(),
            crossterm::event::EnableMouseCapture,
        )?;

        while !self.terminate {
            // handle all input events
            while event::poll(std::time::Duration::ZERO)? {
                match event::read()? {
                    Event::Key(event) => {
                        self.handle_key(event)?;
                    }
                    Event::Mouse(event) => {
                        if let MouseEventKind::Down(button) = event.kind {
                            self.window.mouse_down(button, event.row, event.column);
                        }
                    }
                    Event::Resize {
                        0: width,
                        1: height,
                    } => {
                        self.width = width;
                        self.height = height;

                        self.screen.resize(width, height);
                        self.window.set_bounds(WindowBounds::new(
                            0,
                            0,
                            self.width,
                            self.height - 2,
                        ));
                    }
                    e => {
                        self.notify(format!("unhandled event: {:?}", e), LogLevel::Debug);
                    }
                }
            }

            // render tui
            self.render();
            // show rendered screen
            self.screen.show()?;

            thread::sleep(Duration::from_millis(16));
        }

        crossterm::QueueableCommand::queue(
            &mut std::io::stdout(),
            crossterm::event::DisableMouseCapture,
        )?;
        self.screen.end()?;

        Ok(())
    }

    fn render(&mut self) {
        //let start = std::time::Instant::now();

        let mut cursor = (0, 0);

        // TODO: maybe add a toggle to blessings to stop it from copying the screen buffer
        // contents if we're just going to overwrite them anyways
        self.screen.clear(ClearType::All);

        self.window.render(&mut self.screen);
        if self.mode == Mode::Normal || self.mode == Mode::Insert {
            cursor = self.screen.get_cursor();
        }

        self.render_mode(self.mode);

        if self.mode == Mode::Command {
            self.screen.move_to(0, self.height - 1);
            self.screen.print_char(':');
            self.screen.print(&self.command);

            cursor = self.screen.get_cursor();
        }

        if let Some(notification) = &self.notification {
            let (fg, bg) = match notification.level {
                LogLevel::Info => (Color::Reset, Color::Reset),
                LogLevel::Error => (Color::Red, Color::Reset),
                LogLevel::Debug => (Color::Magenta, Color::Reset),
            };
            self.screen.set_colors(fg, bg);
            self.screen
                .print_at(0, self.height - 1, &notification.message);
            self.screen.clear_colors();
        }

        /*let time = start.elapsed();
        self.screen.move_to(
            self.width - "Frame took:           ".len() as u16,
            self.height - 1,
        );
        self.screen.print(format!("Frame took: {:?}", time));*/

        self.screen.move_to(cursor.0, cursor.1);
    }

    fn render_mode(&mut self, mode: Mode) {
        self.screen.move_to(0, self.height - 2);
        self.screen.clear(ClearType::CurrentLine);

        // TODO: make text bold
        self.screen.set_colors(Color::Black, mode.get_color());
        self.screen.print(format!(" {} ", mode.to_str()));
        self.screen.clear_colors();
    }

    fn handle_key(&mut self, event: KeyEvent) -> Result<()> {
        if event.kind == KeyEventKind::Press {
            if let Some(actions) = self.keymap.handle(self.mode, event) {
                for action in actions {
                    self.execute_action(action)?;
                }
            }
        }

        Ok(())
    }

    fn execute_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::ChangeMode(mode) => self.change_mode(mode),
            Action::MoveUp => self.window.move_up(),
            Action::MoveDown => self.window.move_down(),
            Action::MoveLeft => self.window.move_left(),
            Action::MoveRight => self.window.move_right(),
            Action::InsertChar(c) => self.window.insert_char(c),
            Action::RemoveChar => self.window.remove_char(),
            Action::DeleteChar => self.window.delete_char(),
            Action::ExecuteCommand => {
                self.execute_command()?;
                self.change_mode(Mode::Normal);
            }
            Action::InsertCharCommand(c) => self.command.push(c),
            Action::RemoveCharCommand => {
                self.command.pop();
            }
            Action::MoveToStartOfLine => self.window.move_to_start_of_line(),
            Action::MoveToEndOfLine => self.window.move_to_end_of_line(),
            Action::MoveToFirstCharacterInLine => self.window.move_to_first_char_in_line(),
        }
        Ok(())
    }

    fn change_mode(&mut self, mode: Mode) {
        if self.mode == Mode::Command && mode != Mode::Command {
            self.command.clear();
        }
        if mode == Mode::Command {
            self.notification = None;
        }

        self.mode = mode;

        match mode {
            Mode::Normal | Mode::Command => {
                self.screen.set_cursor_style(CursorStyle::SteadyBlock);
            }
            Mode::Insert => {
                self.screen.set_cursor_style(CursorStyle::SteadyBar);
            }
        }
    }

    // FIXME: move this somewhere else
    // The current problem is that something like
    // > self.window.get_buffer_mut().save(&mut editor)
    // isn't possible due to the burrow checker but this function should print information about
    // the saved file or error messages if there's a problem
    pub fn notify<S: std::fmt::Display>(&mut self, message: S, level: LogLevel) {
        self.notification = Some(Notification::new(message.to_string(), level));
    }

    fn execute_command(&mut self) -> Result<()> {
        if self.command.starts_with("print ") {
            self.notify(self.command["print ".len()..].to_string(), LogLevel::Info);
        } else if self.command == "q" {
            if self.window.get_buffer().is_saved() {
                self.terminate = true;
            } else {
                // TODO: Add the information which buffers haven't been saved once multiple buffers
                // are implemented
                self.notify("No write since last change", LogLevel::Error);
            }
        } else if self.command == "q!" {
            self.terminate = true;
        } else if self.command == "w" {
            match self.window.get_buffer_mut().save() {
                Ok(notification) => self.notify(notification.message, notification.level),
                Err(e) => self.notify(
                    format!("Error when trying to save to file: {}", e),
                    LogLevel::Error,
                ),
            }
        } else if self.command == "wq" {
            match self.window.get_buffer_mut().save() {
                Ok(notification) => {
                    self.terminate = true;
                    self.notify(notification.message, notification.level);
                }
                Err(e) => self.notify(
                    format!("Error when trying to save to file: {}", e),
                    LogLevel::Error,
                ),
            }
        } else {
            self.notify(
                format!("Not an editor command: {}", self.command),
                LogLevel::Error,
            );
        }

        Ok(())
    }
}
