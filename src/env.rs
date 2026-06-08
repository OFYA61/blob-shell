use std::env;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::MutexGuard;

#[derive(Debug)]
pub enum ChangeDirError {
    DoesNotExist,
}

#[derive(Debug)]
struct Env {
    home: String,
    paths: Vec<PathBuf>,
}

impl Env {
    fn init() -> Self {
        let home = env::var("HOME").expect("Failed to get home environment variable");
        let path_var = env::var("PATH");
        if path_var.is_err() {
            return Self {
                home,
                paths: vec![],
            };
        }
        let path_var = unsafe { path_var.unwrap_unchecked() };

        Self {
            home,
            paths: env::split_paths(&path_var).filter(|p| p.is_dir()).collect(),
        }
    }

    fn get_command(&self, command: &str) -> Option<PathBuf> {
        for path in &self.paths {
            let full_path = path.join(command);
            if full_path.is_file() || full_path.is_symlink() {
                if let Ok(metadata) = std::fs::metadata(&full_path) {
                    // Check executable permissions
                    if metadata.permissions().mode() & 0o111 != 0 {
                        return Some(full_path);
                    }
                }
            }
        }
        None
    }

    fn get_current_dir(&self) -> String {
        env::current_dir()
            .expect("Failed to get current directory")
            .to_str()
            .expect("Failed to parse to string")
            .to_owned()
    }

    fn change_dir(&mut self, new_dir: &str) -> Result<(), ChangeDirError> {
        let dir: String;
        if new_dir.starts_with("~") {
            dir = new_dir.replace("~", &self.home);
        } else {
            dir = new_dir.to_owned();
        }

        env::set_current_dir(&dir).map_err(|_| ChangeDirError::DoesNotExist)
    }
}

fn env() -> MutexGuard<'static, Env> {
    static ENV: LazyLock<Mutex<Env>> = LazyLock::new(|| Mutex::new(Env::init()));
    ENV.lock().unwrap()
}

pub fn get_command(command: &str) -> Option<PathBuf> {
    env().get_command(command)
}

pub fn get_current_dir() -> String {
    env().get_current_dir()
}

pub fn change_dir(new_dir: &str) -> Result<(), ChangeDirError> {
    env().change_dir(new_dir)
}
