use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::sync::OnceLock;

use crate::autocomplete::Candidate;
use crate::env;
use crate::env::ChangeDirError;

fn map() -> &'static HashMap<&'static str, Builtin> {
    static MAP: OnceLock<HashMap<&'static str, Builtin>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m: HashMap<&'static str, Builtin> = HashMap::new();
        m.insert("echo", Builtin::Echo);
        m.insert("exit", Builtin::Exit);
        m.insert("cd", Builtin::Cd);
        m.insert("pwd", Builtin::Pwd);
        m.insert("type", Builtin::Type);
        m
    })
}

pub fn try_auto_complete(prefix: &str) -> Vec<Candidate> {
    map()
        .keys()
        .filter(|key| key.starts_with(prefix))
        .map(|key| Candidate::Builtin(key.to_owned().to_owned()))
        .collect()
}

#[derive(Debug, Clone)]
enum Builtin {
    Echo,
    Exit,

    Cd,
    Pwd,

    Type,
}

impl Builtin {
    fn from_str(s: &str) -> Option<Self> {
        map().get(s).map(|b| b.clone())
    }
}

/// Try proecssing a bultin command. If none are found, returns `false`.
/// If it is a bultin command returns `true`, even if wrong argument types get passed
pub fn try_process(
    exec: &str,
    args: &Vec<&str>,
    stdout_files: &Vec<File>,
    stderr_files: &Vec<File>,
) -> bool {
    macro_rules! write_stdout {
        ($($arg:tt)*) => {
            if stdout_files.is_empty() {
                println!($($arg)*);
            } else {
                stdout_files.iter().for_each(|mut file| {
                    writeln!(&mut file, $($arg)*).expect("Failed to write to file");
                });
            }
        };
    }

    macro_rules! write_stderr{
        ($($arg:tt)*) => {
            if stderr_files.is_empty() {
                eprintln!($($arg)*);
            } else {
                stderr_files.iter().for_each(|mut file| {
                    writeln!(&mut file, $($arg)*).expect("Failed to write to file");
                });
            }
        };
    }

    macro_rules! expect_no_argument {
        () => {
            if !args.is_empty() {
                write_stderr!("{}: expects no argument", exec);
            }
        };
    }

    macro_rules! expect_single_argument {
        () => {
            if args.len() != 1 {
                write_stderr!("{}: expects exactly one argument", exec);
                return true;
            } else {
                args[0]
            }
        };
    }

    let builtin = Builtin::from_str(&exec);
    if builtin.is_none() {
        return false;
    }
    let builtin = builtin.unwrap();
    match builtin {
        Builtin::Echo => {
            write_stdout!("{}", args.join(" "));
        }
        Builtin::Exit => {
            expect_no_argument!();
            std::process::exit(0);
        }

        Builtin::Cd => {
            let new_dir = expect_single_argument!();
            match env::change_dir(new_dir) {
                Err(err) => match err {
                    ChangeDirError::DoesNotExist => {
                        write_stdout!("cd: {new_dir}: No such file or directory");
                    }
                },
                _ => {}
            }
        }
        Builtin::Pwd => {
            expect_no_argument!();
            write_stdout!("{}", env::get_current_dir());
        }

        Builtin::Type => {
            let cmd = expect_single_argument!();
            if Builtin::from_str(cmd).is_some() {
                write_stdout!("{} is a shell builtin", cmd);
            } else if let Some(command) = env::get_command(cmd) {
                write_stdout!("{} is {}", cmd, command.to_str().unwrap_or(""));
            } else {
                write_stdout!("{cmd}: not found");
            }
        }
    };

    return true;
}
