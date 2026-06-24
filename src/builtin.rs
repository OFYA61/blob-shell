use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use clap::Parser;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::sync::MutexGuard;

use crate::autocomplete::Candidate;
use crate::state::ChangeDirError;
use crate::state::State;

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
        m.insert("history", Builtin::History);
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
    History,
    Type,
}

impl Builtin {
    pub fn from_str(s: &str) -> Option<Self> {
        map().get(s).map(|b| b.clone())
    }

    #[inline(always)]
    pub async fn process<R, W, E>(
        &self,
        state: MutexGuard<'_, State>,
        args: &Vec<String>,
        _stdin: Option<R>,
        stdout: W,
        stderr: E,
    ) where
        R: AsyncReadExt + Unpin,
        W: AsyncWriteExt + Unpin,
        E: AsyncWriteExt + Unpin,
    {
        match self {
            Builtin::Echo => process_echo(args, stdout).await,
            Builtin::Exit => process_exit(args, stderr).await,
            Builtin::Cd => process_cd(state, args, stderr).await,
            Builtin::Pwd => process_pwd(state, args, stdout, stderr).await,
            Builtin::Rehash => process_rehash(state, args, stderr).await,
            Builtin::Complete => process_complete(state, args, stdout, stderr).await,
            Builtin::Jobs => process_jobs(state, args, stdout, stderr).await,
            Builtin::History => process_history(state, args, stdout, stderr).await,
            Builtin::Type => process_type(state, args, stdout, stderr).await,
        }
    }
}

#[inline(always)]
async fn process_echo<W: AsyncWriteExt + Unpin>(args: &Vec<String>, mut stdout: W) {
    let _ = stdout
        .write_all(format!("{}\n", args.join(" ")).as_bytes())
        .await;
}

#[inline(always)]
async fn process_exit<E: AsyncWriteExt + Unpin>(args: &Vec<String>, mut stderr: E) {
    if !args.is_empty() {
        let _ = stderr
            .write_all("exit: expects no argument\n".as_bytes())
            .await;
        return;
    }
    // TODO: do not exit if there are running jobs
    std::process::exit(0);
}

#[inline(always)]
async fn process_cd<E: AsyncWriteExt + Unpin>(
    mut state: MutexGuard<'_, State>,
    args: &Vec<String>,
    mut stderr: E,
) {
    if let Some(new_dir) = args.first() {
        match state.change_dir(new_dir) {
            Err(err) => match err {
                ChangeDirError::DoesNotExist => {
                    let _ = stderr
                        .write_all(
                            format!("cd: {}: No such file or directory\n", new_dir).as_bytes(),
                        )
                        .await;
                }
            },
            _ => {}
        }
    } else {
        let _ = stderr
            .write_all("cd: expects exactly one argument\n".as_bytes())
            .await;
    }
}

#[inline(always)]
async fn process_pwd<W: AsyncWriteExt + Unpin, E: AsyncWriteExt + Unpin>(
    state: MutexGuard<'_, State>,
    args: &Vec<String>,
    mut stdout: W,
    mut stderr: E,
) {
    if !args.is_empty() {
        let _ = stderr
            .write_all("pwd: expects no argument\n".as_bytes())
            .await;
        return;
    }
    let _ = stdout
        .write_all(format!("{}\n", state.get_current_dir_as_string()).as_bytes())
        .await;
}

#[inline(always)]
async fn process_rehash<E: AsyncWriteExt + Unpin>(
    mut state: MutexGuard<'_, State>,
    args: &Vec<String>,
    mut stderr: E,
) {
    if !args.is_empty() {
        let _ = stderr
            .write_all("rehash: expects no argument\n".as_bytes())
            .await;
        return;
    }
    state.reinit();
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

#[inline(always)]
async fn process_complete<W: AsyncWriteExt + Unpin, E: AsyncWriteExt + Unpin>(
    mut state: MutexGuard<'_, State>,
    args: &Vec<String>,
    mut stdout: W,
    mut stderr: E,
) {
    match CompleteArgs::try_parse_from(
        std::iter::once("complete").chain(args.into_iter().map(|s| s.as_str())),
    ) {
        Ok(args) => {
            if let Some(new_completion) = args.completion {
                state.add_completer(args.extra.first().unwrap().clone(), new_completion);
            } else if let Some(program) = args.program {
                if let Some(completion) = state.get_completer(&program) {
                    let _ = stdout
                        .write_all(
                            format!("complete -C '{}' {}\n", completion.path.display(), program)
                                .as_bytes(),
                        )
                        .await;
                } else {
                    let _ = stdout
                        .write_all(
                            format!("complete: {}: no completion specification\n", program)
                                .as_bytes(),
                        )
                        .await;
                }
            } else if let Some(remove) = args.remove {
                state.remove_completer(&remove);
            }
        }
        Err(err) => {
            let _ = stderr.write_all(format!("{}\n", err).as_bytes()).await;
        }
    }
}

#[inline(always)]
async fn process_jobs<W: AsyncWriteExt + Unpin, E: AsyncWriteExt + Unpin>(
    mut state: MutexGuard<'_, State>,
    args: &Vec<String>,
    stdout: W,
    mut stderr: E,
) {
    if !args.is_empty() {
        let _ = stderr
            .write_all("jobs: expects no arguments\n".as_bytes())
            .await;
        return;
    }
    state.log_jobs(stdout).await;
}

#[derive(Parser, Debug)]
#[command(group(
    clap::ArgGroup::new("mode")
        .required(false)
        .args(["read", "write", "append"]),
))]
struct HistoryArgs {
    #[arg(short)]
    read: Option<String>,

    #[arg(short)]
    write: Option<String>,

    #[arg(short)]
    append: Option<String>,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 1)]
    extra: Vec<String>,
}

#[inline(always)]
async fn process_history<W: AsyncWriteExt + Unpin, E: AsyncWriteExt + Unpin>(
    mut state: MutexGuard<'_, State>,
    args: &Vec<String>,
    stdout: W,
    mut stderr: E,
) {
    match HistoryArgs::try_parse_from(
        std::iter::once("history").chain(args.into_iter().map(|s| s.as_str())),
    ) {
        Ok(args) => {
            if let Some(read) = args.read {
                if let Err(err) = state.read_history_file(read.as_str()).await {
                    let _ = stderr
                        .write_all(format!("File {} does not exist: {}", read, err).as_bytes())
                        .await;
                }
            } else if let Some(write) = args.write {
                if let Err(err) = state.write_history_file(write.as_str()).await {
                    let _ = stderr
                        .write_all(format!("File {} does not exist: {}", write, err).as_bytes())
                        .await;
                }
            } else if let Some(append) = args.append {
                if let Err(err) = state.append_history_file(append.as_str()).await {
                    let _ = stderr
                        .write_all(format!("File {} does not exist: {}", append, err).as_bytes())
                        .await;
                }
            } else {
                let tail: Option<usize> = if let Some(arg) = args.extra.first() {
                    if let Ok(num) = arg.parse() {
                        Some(num)
                    } else {
                        let _ = stderr
                            .write_all(format!("{} is not a number\n", arg).as_bytes())
                            .await;
                        return;
                    }
                } else {
                    None
                };
                state.print_history(stdout, tail).await;
            }
        }
        Err(err) => {
            let _ = stderr.write_all(format!("{}\n", err).as_bytes()).await;
        }
    };
}

#[inline(always)]
async fn process_type<W: AsyncWriteExt + Unpin, E: AsyncWriteExt + Unpin>(
    state: MutexGuard<'_, State>,
    args: &Vec<String>,
    mut stdout: W,
    mut stderr: E,
) {
    if let Some(cmd) = args.first() {
        if Builtin::from_str(cmd).is_some() {
            let _ = stdout
                .write_all(format!("{} is a shell builtin\n", cmd).as_bytes())
                .await;
        } else if let Some(command) = state.get_command(cmd) {
            let _ = stdout
                .write_all(format!("{} is {}\n", cmd, command.to_str().unwrap_or("")).as_bytes())
                .await;
        } else {
            let _ = stdout
                .write_all(format!("{cmd}: not found\n").as_bytes())
                .await;
        }
    } else {
        let _ = stderr
            .write_all("type: expects exactly one argument\n".as_bytes())
            .await;
    }
}
