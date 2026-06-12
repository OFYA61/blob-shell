use std::str::Chars;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Candidate {
    Builtin(String),
    Program(String),
    File(String),
    Directory(String),
}

impl Candidate {
    pub fn as_str(&self) -> &str {
        match self {
            Candidate::Builtin(s)
            | Candidate::Program(s)
            | Candidate::File(s)
            | Candidate::Directory(s) => s,
        }
    }

    pub fn get_trailing_char(&self) -> char {
        match self {
            Candidate::Builtin(_) | Candidate::Program(_) | Candidate::File(_) => ' ',
            Candidate::Directory(_) => '/',
        }
    }

    pub fn chars(&self) -> Chars<'_> {
        self.as_str().chars()
    }

    pub fn is_directory(&self) -> bool {
        match self {
            Candidate::Directory(_) => true,
            _ => false,
        }
    }
}
