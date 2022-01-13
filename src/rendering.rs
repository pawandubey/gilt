use std::fmt::Write;
use std::{path::Path, process::Output};

use crate::OutputType;

type Err = String;

pub fn get_renderer(_output_type: OutputType) -> impl Render {
    StdinRenderer::new()
}

pub trait Render {
    fn log(&mut self, out: Output, repo: &Path) -> Result<(), Err>;

    fn render(&self) -> &str;
}

struct StdinRenderer {
    buffer: String,
}

impl StdinRenderer {
    fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }
}

impl Render for StdinRenderer {
    fn log(&mut self, out: Output, repo: &Path) -> Result<(), Err> {
        writeln!(
            self.buffer,
            "({}): {}",
            repo.to_str().unwrap(),
            String::from_utf8_lossy(&out.stdout)
        )
        .and_then(|_| Ok(()))
        .or_else(|_| {
            Err(format!(
                "Problem processing output for repository: {}",
                repo.to_str().unwrap()
            ))
        })
    }

    fn render(&self) -> &str {
        self.buffer.as_str()
    }
}
