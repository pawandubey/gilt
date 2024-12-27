use std::collections::HashMap;
use std::fmt::Write;
use std::{path::Path, process::Output};

use crate::OutputType;

pub(crate) trait Stringify {
    fn stringify(&self) -> String;
}

impl Stringify for Output {
    fn stringify(&self) -> String {
        String::from_utf8_lossy(&self.stdout).to_string()
    }
}

type Err = String;

pub(crate) fn get_renderer<'a>(_output_type: OutputType) -> impl Render<'a> {
    StdinRenderer::new()
}

pub(crate) trait Render <'a> {
    fn log(&mut self, out: impl Stringify, repo: &'a Path) -> Result<(), Err>;

    fn render(&self) -> &str;
}

#[derive(Default)]
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

impl <'a> Render<'a> for StdinRenderer {
    fn log(&mut self, out: impl Stringify, repo: &'a Path) -> Result<(), Err> {
        writeln!(
            self.buffer,
            "({}): {}",
            repo.to_str().unwrap(),
            out.stringify()
        )
        .and_then(|_| Ok(()))
        .map_err(|_| {
            format!(
                "Problem processing output for repository: {}",
                repo.to_str().unwrap()
            )
        })
    }

    fn render(&self) -> &str {
        self.buffer.as_str()
    }
}

#[derive(Default)]
struct JSONRenderer<'a> {
    buffer: HashMap<&'a str, &'a str>,
}

impl <'a> Render<'a> for JSONRenderer<'a> {
    fn log(&mut self, out: impl Stringify, repo: &'a Path) -> Result<(), Err> {
        let key = repo.to_str().unwrap();
        let val = out.stringify();
        self.buffer.insert(key, val.as_str());
        Ok(())
    }

    fn render(&self) -> &str {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use crate::rendering::*;

    impl Stringify for String {
        fn stringify(&self) -> String {
            self.to_string()
        }
    }

    #[test]
    fn test_render_stdin_empty() {
        assert_eq!("", StdinRenderer::new().render())
    }

    #[test]
    fn test_render_stdin_non_empty() {
        let mut stdin: StdinRenderer = Default::default();
        let path = Path::new("/path/to/repo");

        stdin.log("main".to_string(), path).unwrap();
        stdin.log("main".to_string(), path).unwrap();

        let expected = "(/path/to/repo): main\n(/path/to/repo): main\n";

        assert_eq!(expected, stdin.render())
    }
}
