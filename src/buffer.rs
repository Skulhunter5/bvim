use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use crate::editor::{LogLevel, Notification};

#[derive(Debug)]
pub struct Buffer {
    pub lines: Vec<String>,
    pub path: Option<PathBuf>,
    pub changed: bool,
}

impl Buffer {
    pub fn new() -> Self {
        Self::new_with_path(None)
    }

    pub fn new_with_path(path: Option<PathBuf>) -> Self {
        Self {
            lines: vec![String::new()],
            path,
            changed: false,
        }
    }

    pub fn new_from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Ok(Self::new_with_path(Some(path.to_path_buf())));
        }
        if !path.is_file() {
            // TODO: Start the editor with the given directory as working directory once working
            // directory is implemented
            return Ok(Buffer::new());
        }

        // Rust's String.lines() doesn't seem to include a last empty line on a trailing newline,
        // so .split('\n') has to be done by hand
        let lines = std::fs::read_to_string(&path)?
            .split('\n')
            .map(|line| line.to_string().replace("\r", ""))
            .collect::<Vec<String>>();
        let path = Some(path.to_path_buf());

        Ok(Self {
            lines,
            path,
            changed: false,
        })
    }

    pub fn is_saved(&self) -> bool {
        !self.changed
    }

    pub fn save(&mut self) -> std::io::Result<Notification> {
        if let Some(path) = &self.path {
            let mut file = File::create(path)?;
            for i in 0..(self.lines.len() - 1) {
                file.write_all(&self.lines[i].as_bytes())?;
                file.write_all("\n".as_bytes())?;
            }
            file.write_all(self.lines.last().unwrap().as_bytes())?;

            self.changed = false;

            let path = match path.to_str() {
                Some(s) => s.to_owned(),
                None => todo!(),
            };
            return Ok(Notification::new(
                format!("\"{}\" {}L written", path, self.lines.len()),
                LogLevel::Info,
            ));
        } else {
            return Ok(Notification::new(
                "Could not save: No file name".to_owned(),
                LogLevel::Error,
            ));
        }
    }

    pub fn line_length(&self, index: usize) -> usize {
        self.lines[index].chars().count()
    }
}
