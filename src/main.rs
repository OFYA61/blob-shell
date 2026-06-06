use std::io::Write;
use std::os::unix::fs::PermissionsExt;

#[derive(Debug)]
enum Builtin {
    Echo,
    Exit,

    Cd,
    Pwd,

    Type,
}

impl Builtin {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "echo" => Some(Builtin::Echo),
            "exit" => Some(Builtin::Exit),

            "cd" => Some(Builtin::Cd),
            "pwd" => Some(Builtin::Pwd),

            "type" => Some(Builtin::Type),
            _ => None,
        }
    }
}

#[derive(Debug)]
enum ChangeDirError {
    DoesNotExist,
}

#[derive(Debug)]
struct Env {
    home: String,
    paths: Vec<std::path::PathBuf>,
}

impl Env {
    fn init() -> Self {
        let home = std::env::var("HOME").expect("Failed to get home environment variable");
        let path_var = std::env::var("PATH");
        if path_var.is_err() {
            return Self {
                home,
                paths: vec![],
            };
        }
        let path_var = unsafe { path_var.unwrap_unchecked() };

        Self {
            home,
            paths: std::env::split_paths(&path_var)
                .filter(|p| p.is_dir())
                .collect(),
        }
    }

    fn get_command(&self, command: &str) -> Option<std::path::PathBuf> {
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

    fn get_current_directory(&self) -> String {
        std::env::current_dir()
            .expect("Failed to get current directory")
            .to_str()
            .expect("Failed to parse to string")
            .to_owned()
    }

    fn change_directory(&mut self, new_dir: &str) -> Result<(), ChangeDirError> {
        let dir: String;
        if new_dir.starts_with("~") {
            dir = new_dir.replace("~", &self.home);
        } else {
            dir = new_dir.to_owned();
        }

        std::env::set_current_dir(&dir).map_err(|_| ChangeDirError::DoesNotExist)
    }
}

macro_rules! expect_no_argument {
    ($command:expr, $args:expr) => {
        if !$args.is_empty() {
            println!("{}: expects no argument", $command);
            continue;
        }
    };
}

macro_rules! expect_single_argument {
    ($command:expr, $args:expr) => {
        if $args.len() != 1 {
            println!("{}: expects exactly one argument", $command);
            continue;
        } else {
            $args[0]
        }
    };
}

fn main() {
    let mut env = Env::init();

    loop {
        print!("$ ");
        std::io::stdout().flush().unwrap();

        let mut command = String::new();
        std::io::stdin()
            .read_line(&mut command)
            .expect("Failed to read command");

        command = command.trim().to_string();

        let (command, args) = command.split_once(' ').unwrap_or((&command, ""));

        let args = args
            .split_whitespace()
            .filter(|arg| !arg.is_empty())
            .collect::<Vec<&str>>();

        if let Some(builtin) = Builtin::from_str(command) {
            match builtin {
                Builtin::Echo => {
                    println!("{}", args.join(" "));
                }
                Builtin::Exit => {
                    expect_no_argument!("exit", args);
                    break;
                }

                Builtin::Cd => {
                    let new_dir = expect_single_argument!("cd", args);
                    match env.change_directory(new_dir) {
                        Err(err) => match err {
                            ChangeDirError::DoesNotExist => {
                                println!("cd: {new_dir}: No such file or directory")
                            }
                        },
                        _ => {}
                    }
                }
                Builtin::Pwd => {
                    expect_no_argument!("pwd", args);
                    println!("{}", env.get_current_directory());
                }

                Builtin::Type => {
                    let cmd = expect_single_argument!("type", args);
                    if Builtin::from_str(cmd).is_some() {
                        println!("{} is a shell builtin", cmd);
                    } else if let Some(command) = env.get_command(cmd) {
                        println!("{} is {}", cmd, command.to_str().unwrap_or(""));
                    } else {
                        println!("{cmd}: not found");
                    }
                }
            };
            continue;
        }

        if let Some(_) = env.get_command(command) {
            let mut child = std::process::Command::new(command);
            if args.len() > 0 {
                child.args(args);
            }
            child
                .spawn()
                .expect("Failed to execute process")
                .wait()
                .expect("Failed to wait on command");

            continue;
        }

        println!("{}: command not found", command);
    }
}
