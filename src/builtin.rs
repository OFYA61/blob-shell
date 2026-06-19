use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::fs::File;

use clap::Parser;
use tokio::io::AsyncWriteExt;
use tokio::sync::MutexGuard;

use crate::autocomplete::Candidate;
use crate::jobs::Jobs;
use crate::state::ChangeDirError;
use crate::state::State;

macro_rules! write_file {
    ($files:expr, $($arg:tt)*) => {
        for mut file in $files {
            let _ = file.write_all(&format!($($arg)*).as_bytes()).await;
            let _ = file.write("\n".as_bytes()).await;
            file.flush().await.expect("Failed to flush file");
        }
    }
}

macro_rules! write_stdout {
    ($files:expr, $($arg:tt)*) => {
        if $files.is_empty() {
            println!($($arg)*);
        } else {
            write_file!($files, $($arg)*);
        }
    };
}

macro_rules! write_stderr {
    ($files:expr, $($arg:tt)*) => {
        if $files.is_empty() {
            eprintln!($($arg)*);
        } else {
            write_file!($files, $($arg)*);
        }
    };
}

fn map() -> &'static HashMap<&'static str, Builtin> {
    static MAP: OnceLock<HashMap<&'static str, Builtin>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m: HashMap<&'static str, Builtin> = HashMap::new();
        m.insert("echo", Builtin::Echo);
        m.insert("exit", Builtin::Exit);
        m.insert("cd", Builtin::Cd);
        m.insert("pwd", Builtin::Pwd);
        m.insert("rehash", Builtin::Rehash);
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
pub enum Builtin {
    Echo,
    Exit,
    Cd,
    Pwd,
    Rehash,
    Complete,
    Jobs,
    Type,
}

impl Builtin {
    pub fn from_str(s: &str) -> Option<Self> {
        map().get(s).map(|b| b.clone())
    }

    pub async fn process(
        &self,
        env: MutexGuard<'_, State>,
        jobs: MutexGuard<'_, Jobs>,
        args: Vec<&str>,
        stdout_files: Vec<File>,
        stderr_files: Vec<File>,
    ) {
        match self {
            Builtin::Echo => process_echo(args, stdout_files).await,
            Builtin::Exit => process_exit(args, stderr_files).await,
            Builtin::Cd => process_cd(env, args, stderr_files).await,
            Builtin::Pwd => process_pwd(env, args, stdout_files, stderr_files).await,
            Builtin::Rehash => process_rehash(env, args, stderr_files).await,
            Builtin::Complete => process_complete(env, args, stdout_files, stderr_files).await,
            Builtin::Jobs => process_jobs(jobs, args, stderr_files).await,
            Builtin::Type => process_type(env, args, stdout_files, stderr_files).await,
        }
    }
}

#[inline]
async fn process_echo(args: Vec<&str>, stdout_files: Vec<File>) {
    write_stdout!(stdout_files, "{}", args.join(" "));
}

#[inline]
async fn process_exit(args: Vec<&str>, stderr_files: Vec<File>) {
    if !args.is_empty() {
        write_stderr!(stderr_files, "exit: expects no argument");
        return;
    }
    // TODO: do not exit if there are running jobs
    std::process::exit(0);
}

#[inline]
async fn process_cd(mut env: MutexGuard<'_, State>, args: Vec<&str>, stderr_files: Vec<File>) {
    if let Some(new_dir) = args.first() {
        match env.change_dir(new_dir) {
            Err(err) => match err {
                ChangeDirError::DoesNotExist => {
                    write_stderr!(stderr_files, "cd: {new_dir}: No such file or directory");
                }
            },
            _ => {}
        }
    } else {
        write_stderr!(stderr_files, "cd: expects exactly one argument");
    }
}

#[inline]
async fn process_pwd(
    env: MutexGuard<'_, State>,
    args: Vec<&str>,
    stdout_files: Vec<File>,
    stderr_files: Vec<File>,
) {
    if !args.is_empty() {
        write_stderr!(stderr_files, "pwd: expects no argument");
        return;
    }
    write_stdout!(stdout_files, "{}", env.get_current_dir_as_string());
}

#[inline]
async fn process_rehash(mut env: MutexGuard<'_, State>, args: Vec<&str>, stderr_files: Vec<File>) {
    if !args.is_empty() {
        write_stderr!(stderr_files, "rehash: expects no argument");
        return;
    }
    env.reinit();
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

#[inline]
async fn process_complete(
    mut env: MutexGuard<'_, State>,
    args: Vec<&str>,
    stdout_files: Vec<File>,
    stderr_files: Vec<File>,
) {
    match CompleteArgs::try_parse_from(
        std::iter::once("complete").chain(args.into_iter().map(|arg| arg)),
    ) {
        Ok(args) => {
            if let Some(new_completion) = args.completion {
                env.add_completer(args.extra.first().unwrap().clone(), new_completion);
            } else if let Some(program) = args.program {
                if let Some(completion) = env.get_completer(&program) {
                    write_stdout!(
                        stdout_files,
                        "complete -C '{}' {}",
                        completion.path.display(),
                        program
                    );
                } else {
                    write_stdout!(
                        stdout_files,
                        "complete: {}: no completion specification",
                        program
                    );
                }
            } else if let Some(remove) = args.remove {
                env.remove_completer(&remove);
            }
        }
        Err(err) => {
            write_stderr!(stderr_files, "{}", err);
        }
    }
}

#[inline]
async fn process_jobs(mut jobs: MutexGuard<'_, Jobs>, args: Vec<&str>, stderr_files: Vec<File>) {
    if !args.is_empty() {
        write_stderr!(stderr_files, "jobs: expects no argument");
        return;
    }
    jobs.log_jobs();
    jobs.reap_done_jobs(false);
}

#[inline]
async fn process_type(
    env: MutexGuard<'_, State>,
    args: Vec<&str>,
    stdout_files: Vec<File>,
    stderr_files: Vec<File>,
) {
    if let Some(cmd) = args.first() {
        if Builtin::from_str(cmd).is_some() {
            write_stdout!(stdout_files, "{} is a shell builtin", cmd);
        } else if let Some(command) = env.get_command(cmd) {
            write_stdout!(
                stdout_files,
                "{} is {}",
                cmd,
                command.to_str().unwrap_or("")
            );
        } else {
            write_stdout!(stdout_files, "{cmd}: not found");
        }
    } else {
        write_stderr!(stderr_files, "type: expects exactly one argument");
    }
}
