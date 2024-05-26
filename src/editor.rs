use std::{io::{stdout, Write}, thread, time::Duration};

use anyhow::Result;
use crossterm::{cursor::{position, MoveTo, SetCursorStyle}, event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers}, style::{Attribute, Attributes, Color, ContentStyle, Print, PrintStyledContent, StyledContent}, terminal::{self, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, QueueableCommand};

#[derive(Copy, Clone)]
enum Mode {
    Normal,
    Insert,
}

impl Mode {
    pub fn to_str(&self) -> &str {
        match self {
            Mode::Normal => "Normal",
            Mode::Insert => "Insert",
        }
    }
}

pub(crate) struct Editor {
    mode: Mode,
    width: u16,
    height: u16,
    terminate: bool,
    text: Vec<String>,
    col: usize,
    row: usize,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let (width, height) = terminal::size()?;

        let text = vec![String::new()];

        Ok(Self {
            mode: Mode::Normal,
            width,
            height,
            terminate: false,
            text,
            col: 0,
            row: 0,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut stdout = stdout();

        // Setup terminal
        terminal::enable_raw_mode()?;
        stdout.queue(EnterAlternateScreen)?;
        stdout.queue(MoveTo(0, 0))?;
        stdout.flush()?;

        self.change_mode(Mode::Normal)?;

        while !self.terminate {
            //stdout.queue(Clear(terminal::ClearType::All))?;
            //stdout.flush()?;

            // handle all input events
            while event::poll(std::time::Duration::ZERO)? {
                match event::read()? {
                    Event::Key { 0: key_event } => {
                        self.handle_key(key_event)?;
                    },
                    Event::Resize { 0: width, 1: height } => {
                        self.width = width;
                        self.height = height;
                        println!("Resized: {}x{}", width, height);
                    },
                    e => {
                        println!("unhandled event: {:?}", e);
                    },
                }
            }

            // show rendered screen
            stdout.flush()?;
            thread::sleep(Duration::from_millis(16));
        }

        stdout.queue(LeaveAlternateScreen)?;
        stdout.flush()?;
        disable_raw_mode()?;

        Ok(())
    }

    fn handle_key(&mut self, event: KeyEvent) -> Result<()> {
        let mut stdout = stdout();

        match self.mode {
            Mode::Normal => {
                if event.kind == KeyEventKind::Press {
                    match event.code {
                        KeyCode::Char('q') => {
                            self.terminate = true;
                        },
                        KeyCode::Char('i') => {
                            self.change_mode(Mode::Insert)?;
                        },
                        _ => {},
                    }
                }
            },
            Mode::Insert => {
                if event.kind == KeyEventKind::Press {
                    match event.code {
                        KeyCode::Esc => {
                            self.change_mode(Mode::Normal)?;
                        },
                        KeyCode::Backspace => {
                            if self.col > 0 {
                                self.text[self.row].pop();
                                self.col -= 1;
                                self.move_to_current_position()?;
                                stdout.queue(Print(' '))?;
                                self.move_to_current_position()?;
                            }
                        },
                        KeyCode::Enter => {
                            if self.row < self.height as usize - 3 {
                                self.row += 1;
                                self.col = 0;
                                self.text.push(String::new());
                                self.move_to_current_position()?;
                            }
                        },
                        KeyCode::Up => {
                            self.move_up()?;
                        },
                        KeyCode::Down => {
                            self.move_down()?;
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
                                    stdout.queue(Print(c))?;
                                    self.text[self.row].push(c);
                                    self.col += 1;
                                }
                            }
                        },
                        _ => {},
                    }
                }
            }
        }

        Ok(())
    }

    fn move_up(&mut self) -> Result<()> {
        if self.row > 0 {
            self.row -= 1;
            self.col = self.text[self.row].len();
            self.move_to_current_position()?;
        }

        Ok(())
    }

    fn move_down(&mut self) -> Result<()> {
        if self.row < self.height as usize - 3 && self.row < self.text.len() - 1 {
            self.row += 1;
            self.col = self.text[self.row].len();
            self.move_to_current_position()?;
        }

        Ok(())
    }

    fn change_mode(&mut self, mode: Mode) -> Result<()> {
        self.mode = mode;
        let mut stdout = stdout();
        stdout.queue(MoveTo(0, self.height - 2))?;
        stdout.queue(PrintStyledContent(StyledContent::new(
            ContentStyle {
                foreground_color: Some(Color::Black),
                background_color: Some(Color::Blue),
                underline_color: None,
                attributes: Attributes::from(Attribute::Bold),
            },
            format!(" {} ", mode.to_str()),
        )))?;
        self.move_to_current_position()?;

        match mode {
            Mode::Normal => {
                stdout.queue(SetCursorStyle::SteadyBlock)?;
            },
            Mode::Insert => {
                stdout.queue(SetCursorStyle::SteadyBar)?;
            }
        }

        Ok(())
    }

    fn move_to_current_position(&mut self) -> Result<()> {
        stdout().queue(MoveTo(self.col as u16, self.row as u16))?;
        Ok(())
    }
}
