use anyhow::Result;
use std::{
    io::{stdout, Write},
    thread,
    time::Duration,
};

use crossterm::{
    cursor::{position, MoveTo}, event::{self, Event, KeyCode, KeyEventKind, KeyModifiers}, style::Print, terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen}, QueueableCommand
};

enum Mode {
    Normal,
    Insert,
}

pub fn main() -> Result<()> {
    let mut stdout = stdout();

    // Setup terminal
    terminal::enable_raw_mode()?;
    stdout.queue(EnterAlternateScreen)?;
    stdout.queue(MoveTo(0, 0))?;
    stdout.flush()?;

    let mut mode = Mode::Normal;

    let (width, height) = terminal::size()?;
    let mut x = 0;
    let mut y = 0;
    'outer: loop {
        //stdout.queue(Clear(terminal::ClearType::All))?;
        //stdout.flush()?;

        // handle all input events
        while event::poll(std::time::Duration::ZERO)? {
            match event::read()? {
                Event::Key { 0: key_event } => {
                    match mode {
                        Mode::Normal => {
                            if key_event.kind == KeyEventKind::Press {
                                match key_event.code {
                                    KeyCode::Char('q') => {
                                        break 'outer;
                                    },
                                    KeyCode::Char('i') => {
                                        mode = Mode::Insert;
                                    }
                                    _ => {}
                                }
                            }
                        },
                        Mode::Insert => {
                            if key_event.kind == KeyEventKind::Press {
                                match key_event.code {
                                    KeyCode::Esc => {
                                        mode = Mode::Normal;
                                    }
                                    KeyCode::Backspace => {
                                        (x, y) = position()?;
                                        stdout.queue(MoveTo(x - 1, y))?;
                                        stdout.queue(Print(' '))?;
                                        stdout.queue(MoveTo(x - 1, y))?;
                                    }
                                    KeyCode::Char(c) => {
                                        if key_event.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                                            mode = Mode::Normal;
                                        } else {
                                            stdout.queue(Print(c))?;
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                },
                Event::Resize {
                    0: width,
                    1: height,
                } => {
                    println!("Resized: {}x{}", width, height);
                },
                e => {
                    println!("unhandled event: {:?}", e);
                }
            }
        }

        // show rendered screen
        stdout.flush()?;
        thread::sleep(Duration::from_millis(16));
    }

    stdout.queue(LeaveAlternateScreen)?;
    stdout.flush()?;
    terminal::disable_raw_mode().unwrap();

    Ok(())
}
