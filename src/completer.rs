use std::io;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Completer {
    pub path: PathBuf,
}

impl Completer {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn run(&self, args: Vec<&str>, comp_line: &str) -> io::Result<String> {
        let comp_line = comp_line.trim();
        let output = Command::new(&self.path)
            .args(args)
            .env("COMP_LINE", comp_line)
            .env("COMP_POINT", comp_line.len().to_string())
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
