use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::fs::File;

use clap::Parser;
use tokio::io::AsyncWriteExt;

use crate::autocomplete::Candidate;
use crate::env;
use crate::env::ChangeDirError;
use crate::jobs::Jobs;

fn map() -> &'static HashMap<&'static str, Builtin> {
    static MAP: OnceLock<HashMap<&'static str, Builtin>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m: HashMap<&'static str, Builtin> = HashMap::new();
        m.insert("echo", Builtin::Echo);
        m.insert("exit", Builtin::Exit);
        m.insert("cd", Builtin::Cd);
        m.insert("pwd", Builtin::Pwd);
        m.insert("complete", Builtin::Complete);
        m.insert("jobs", Builtin::Jobs);
        m.insert("type", Builtin::Type);
        m
    })
}

pub fn get_auto_complete_candidates(prefix: &str) -> Vec<Candidate> {
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

    Complete,
    Jobs,
    Type,
}

impl Builtin {
    fn from_str(s: &str) -> Option<Self> {
        map().get(s).map(|b| b.clone())
    }
}

#[derive(Parser, Debug)]
#[command(group(
    clap::ArgGroup::new("mode")
        .required(true)
        .args(["program", "completion", "remove"]),
))]
struct CompleteArgs {
    #[arg(short, long)]
    program: Option<String>,

    #[arg(short = 'C', long, requires = "extra")]
    completion: Option<PathBuf>,

    #[arg(short, long)]
    remove: Option<String>,

    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        num_args = 1,
        requires = "completion"
    )]
    extra: Vec<String>,
}

/// Try proecssing a bultin command. If none are found, returns `false`.
/// If it is a bultin command returns `true`, even if wrong argument types get passed
pub async fn process(
    jobs: &Jobs,
    exec: &str,
    args: &Vec<&str>,
    stdout_files: &mut Vec<File>,
    stderr_files: &mut Vec<File>,
) -> bool {
    macro_rules! write_stdout {
        ($($arg:tt)*) => {
            if stdout_files.is_empty() {
                println!($($arg)*);
            } else {
                for file in stdout_files {
                    let _ = file.write_all(&format!($($arg)*).as_bytes()).await;
                    let _ = file.write("\n".as_bytes()).await;
                    file.flush().await.expect("Failed to flush file");
                }
            }
        };
    }

    macro_rules! write_stderr{
        ($($arg:tt)*) => {
            if stderr_files.is_empty() {
                eprintln!($($arg)*);
            } else {
                for file in stderr_files {
                    let _ = file.write_all(&format!($($arg)*).as_bytes()).await;
                    let _ = file.write("\n".as_bytes()).await;
                    file.flush().await.expect("Failed to flush file");
                }
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

        Builtin::Complete => {
            match CompleteArgs::try_parse_from(
                std::iter::once("complete").chain(args.into_iter().map(|arg| *arg)),
            ) {
                Ok(args) => {
                    if let Some(new_completion) = args.completion {
                        env::add_completer(args.extra.first().unwrap().clone(), new_completion);
                    } else if let Some(program) = args.program {
                        if let Some(completion) = env::get_completer(&program) {
                            write_stdout!(
                                "complete -C '{}' {}",
                                completion.path.display(),
                                program
                            );
                        } else {
                            write_stdout!("complete: {}: no completion specification", program);
                        }
                    } else if let Some(remove) = args.remove {
                        env::remove_completer(&remove);
                    }
                }
                Err(err) => {
                    println!("{}", err);
                }
            }
        }

        Builtin::Jobs => {
            let next_job_id = jobs.id_counter;
            for (id, job) in jobs.iter() {
                let marker = if *id == next_job_id - 1 {
                    '+'
                } else if *id == next_job_id - 2 {
                    '-'
                } else {
                    ' '
                };
                println!("[{}]{}  {} {}", job.id, marker, job.status, job.command,);
            }
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
