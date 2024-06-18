use blessings::{Screen, WindowBounds};

use crate::{buffer::Buffer, util::Position};

#[derive(Debug)]
pub struct Window {
    buffer: Buffer,
    scroll: usize,
    cursor: Position<usize>,
    bounds: WindowBounds,
}

impl Window {
    pub fn new(buffer: Buffer, bounds: WindowBounds) -> Self {
        let scroll = 0;
        let cursor = Position::new(0, 0);

        Self {
            buffer,
            scroll,
            cursor,
            bounds,
        }
    }

    pub fn get_buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn get_buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    pub fn set_bounds(&mut self, bounds: WindowBounds) {
        self.bounds = bounds;

        // Fix scroll after resize if necessary
        // TODO: enforce a relative relation between cursor and window instead of just clamping it
        if self.cursor.y - self.scroll >= self.bounds.height as usize {
            self.scroll = self.cursor.y - self.bounds.height as usize + 1;
        }
    }

    pub fn render(&self, screen: &mut Screen) {
        screen.begin_window(0, 0, self.bounds.width, self.bounds.height);

        for i in 0..(self.bounds.height as usize).min(self.buffer.lines.len() - self.scroll) {
            screen.print_at(0, i as u16, &self.buffer.lines[self.scroll + i]);
        }

        screen.move_to(self.cursor.x as u16, (self.cursor.y - self.scroll) as u16);

        screen.end_window();
    }

    pub fn move_up(&mut self) {
        if self.cursor.y > 0 {
            self.cursor.y -= 1;
            // Move cursor to the end of the new line if it's shorter than before
            self.cursor.x = self.cursor.x.min(self.buffer.line_length(self.cursor.y));
            // Scroll viewport if necessary
            if self.cursor.y < self.scroll {
                self.scroll -= 1;
            }
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor.y < self.buffer.lines.len() - 1 {
            self.cursor.y += 1;
            // Move cursor to the end of the new line if it's shorter than before
            self.cursor.x = self.cursor.x.min(self.buffer.line_length(self.cursor.y));
            // Scroll viewport if necessary
            if self.cursor.y >= self.scroll + self.bounds.height as usize {
                self.scroll += 1;
            }
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor.x > 0 {
            self.cursor.x -= 1;
        } else if self.cursor.y > 0 {
            self.cursor.y -= 1;
            // Move cursor to the end of the new line
            self.cursor.x = self.buffer.line_length(self.cursor.y);
            // Scroll viewport if necessary
            if self.cursor.y < self.scroll {
                self.scroll -= 1;
            }
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor.x < self.buffer.line_length(self.cursor.y) {
            self.cursor.x += 1;
        } else if self.cursor.y < self.buffer.lines.len() - 1 {
            self.cursor.y += 1;
            // Move cursor to the beginning of the new line
            self.cursor.x = 0;
            // Scroll viewport if necessary
            if self.cursor.y >= self.scroll + self.bounds.height as usize {
                self.scroll += 1;
            }
        }
    }

    pub fn move_to_start_of_line(&mut self) {
        self.cursor.x = 0;
    }

    pub fn move_to_first_char_in_line(&mut self) {
        let mut chars = self.buffer.lines[self.cursor.y].chars().enumerate();
        while let Some((i, c)) = chars.next() {
            if !c.is_whitespace() {
                self.cursor.x = i;
                break;
            }
        }
    }

    pub fn move_to_end_of_line(&mut self) {
        self.cursor.x = self.buffer.line_length(self.cursor.y);
    }

    pub fn insert_char(&mut self, c: char) {
        match c {
            '\n' => {
                let new_line = if self.cursor.x == 0 {
                    std::mem::replace(&mut self.buffer.lines[self.cursor.y], String::new())
                } else {
                    let index = self.buffer.lines[self.cursor.y]
                        .char_indices()
                        .nth(self.cursor.x);
                    match index {
                        Some((index, _)) => self.buffer.lines[self.cursor.y].split_off(index),
                        None => String::new(),
                    }
                };
                self.buffer.lines.insert(self.cursor.y + 1, new_line);
                self.cursor.y += 1;
                self.cursor.x = 0;
            }
            c => {
                let index = self.buffer.lines[self.cursor.y]
                    .char_indices()
                    .nth(self.cursor.x);
                let index = if let Some((index, _)) = index {
                    index
                } else {
                    self.buffer.lines[self.cursor.y].len()
                };
                self.buffer.lines[self.cursor.y].insert(index, c);
                //self.buffer.lines[self.cursor.y].insert(self.cursor.x, c);
                self.cursor.x += 1;
            }
        }
        self.buffer.changed = true;
    }

    pub fn remove_char(&mut self) {
        if self.cursor.x == 0 {
            if self.cursor.y > 0 {
                let line = self.buffer.lines.remove(self.cursor.y);
                // Move the cursor first because we have to append to the line above anyways
                self.cursor.y -= 1;
                self.cursor.x = self.buffer.line_length(self.cursor.y);
                self.buffer.lines[self.cursor.y].push_str(line.as_str());
                self.buffer.changed = true;
            }
        } else {
            // Remove the character IN FRONT of the cursor
            // Therefore move first, then remove
            self.cursor.x -= 1;
            let index = self.buffer.lines[self.cursor.y]
                .char_indices()
                .nth(self.cursor.x)
                .unwrap()
                .0;
            self.buffer.lines[self.cursor.y].remove(index);
            self.buffer.changed = true;
        }
    }

    pub fn delete_char(&mut self) {
        if self.cursor.x == self.buffer.line_length(self.cursor.y) {
            if self.cursor.y < self.buffer.lines.len() - 2 {
                let line = self.buffer.lines.remove(self.cursor.y + 1);
                self.buffer.lines[self.cursor.y].push_str(line.as_str());
                self.buffer.changed = true;
            }
        } else {
            let index = self.buffer.lines[self.cursor.y]
                .char_indices()
                .nth(self.cursor.x + 1)
                .unwrap()
                .0;
            self.buffer.lines[self.cursor.y].remove(index);
            self.buffer.changed = true;
        }
    }
}
