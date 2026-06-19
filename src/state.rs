use std::collections::HashMap;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::autocomplete::Candidate;
use crate::completer::Completer;

#[derive(Debug)]
pub enum ChangeDirError {
    DoesNotExist,
}

#[derive(Debug)]
pub struct State {
    home: String,
    #[allow(dead_code)]
    paths: Vec<PathBuf>,
    programs: HashMap<String, PathBuf>,
    completers: HashMap<String, Completer>,
}

impl State {
    pub fn init() -> Self {
        let home = env::var("HOME").expect("Failed to get home environment variable");
        let path_var = env::var("PATH");
        if path_var.is_err() {
            return Self {
                home,
                paths: vec![],
                programs: HashMap::new(),
                completers: HashMap::new(),
            };
        }
        let path_var = unsafe { path_var.unwrap_unchecked() };
        let paths = env::split_paths(&path_var).filter(|p| p.is_dir()).collect();

        let mut programs = HashMap::new();
        for path in &paths {
            if let Ok(entries) = fs::read_dir(&path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file()
                        && let Ok(metadata) = fs::metadata(&p)
                        && metadata.permissions().mode() & 0o111 != 0
                        && let Some(name) = p.file_name().and_then(|name| name.to_str())
                        && programs.get(name).is_none()
                    {
                        programs.insert(name.to_string(), p);
                    }
                }
            }
        }

        Self {
            home,
            paths,
            programs,
            completers: HashMap::new(),
        }
    }

    pub fn reinit(&mut self) {
        *self = Self::init();
    }

    pub fn get_command(&self, command: &str) -> Option<PathBuf> {
        self.programs.get(command).map(|path| path.clone())
    }

    fn get_current_dir(&self) -> PathBuf {
        env::current_dir().expect("Failed to get current directory")
    }

    pub fn get_current_dir_as_string(&self) -> String {
        self.get_current_dir()
            .to_str()
            .expect("Failed to parse to string")
            .to_owned()
    }

    pub fn change_dir(&mut self, new_dir: &str) -> Result<(), ChangeDirError> {
        let dir: String;
        if new_dir.starts_with("~") {
            dir = new_dir.replace("~", &self.home);
        } else {
            dir = new_dir.to_owned();
        }

        env::set_current_dir(&dir).map_err(|_| ChangeDirError::DoesNotExist)
    }

    pub fn get_auto_complete_program_candidates(&self, prefix: &str) -> Vec<Candidate> {
        self.programs
            .keys()
            .filter(|name| name.starts_with(prefix))
            .cloned()
            .map(|s| Candidate::Program(s))
            .collect()
    }

    pub fn get_auto_complete_dir_candidates(&self, subdir: &str, prefix: &str) -> Vec<Candidate> {
        let mut candidates = Vec::new();
        let dir = self.get_current_dir().join(subdir);

        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(name) = p.file_name()
                    && let Some(name) = name.to_str()
                    && name.starts_with(prefix)
                {
                    if p.is_dir() {
                        candidates.push(Candidate::Directory(name.to_owned()));
                    } else {
                        candidates.push(Candidate::File(name.to_owned()));
                    }
                }
            }
        }

        candidates
    }

    pub fn add_completer(&mut self, program: String, path: PathBuf) {
        self.completers.insert(program, Completer::new(path));
    }

    pub fn get_completer(&self, program: &str) -> Option<Completer> {
        self.completers.get(program).cloned()
    }

    pub fn remove_completer(&mut self, program: &str) {
        let _ = self.completers.remove(program);
    }
}
