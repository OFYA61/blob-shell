use std::fmt::Display;
use std::process::Stdio;
use std::sync::Arc;

use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;
use tokio::io::DuplexStream;
use tokio::process::Child;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::ast::ExprCommand;
use crate::ast::ExprRedirect;
use crate::builtin::Builtin;
use crate::state::State;

#[derive(Debug)]
pub struct BackgroundProcessInfo {
    pub command: String,
}

#[derive(Debug)]
pub enum ProcessKind {
    Builtin(Builtin),
    External(String),
}

#[derive(Debug)]
pub struct Process {
    kind: ProcessKind,
    args: Vec<String>,
    redirects: Vec<ExprRedirect>,

    stdin_rx: Option<DuplexStream>,
    stdout_tx: Option<DuplexStream>,
    stderr_tx: Option<DuplexStream>,
}

impl Process {
    pub async fn init(
        state: Arc<Mutex<State>>,
        command: &ExprCommand,
    ) -> Result<Self, ProcessError> {
        let exec_str = command
            .exec
            .process(state.clone())
            .await
            .map_err(|_| ProcessError::MissingClosingBracket)?;

        let kind = if let Some(builtin) = Builtin::from_str(&exec_str) {
            ProcessKind::Builtin(builtin)
        } else if state.lock().await.get_command(&exec_str).is_some() {
            ProcessKind::External(exec_str)
        } else {
            return Err(ProcessError::ProcessNotFound(format!(
                "{}: command not found",
                exec_str
            )));
        };

        let mut args = Vec::with_capacity(command.args.len());
        for arg in &command.args {
            let s = arg
                .process(state.clone())
                .await
                .map_err(|_| ProcessError::MissingClosingBracket)?;
            if !s.is_empty() {
                args.push(s);
            }
        }

        Ok(Self {
            kind,
            args,
            redirects: command.redirects.clone(),
            stdin_rx: None,
            stdout_tx: None,
            stderr_tx: None,
        })
    }

    pub async fn run(
        self,
        state: Arc<Mutex<State>>,
        background_process_info: Option<BackgroundProcessInfo>,
    ) -> Result<(), ProcessError> {
        let mut tasks: Vec<JoinHandle<()>> = Vec::new();

        // Setup I/O and file redirection
        let (stdout_rx, stdout_tx) = tokio::io::duplex(8192);
        let (stderr_rx, stderr_tx) = tokio::io::duplex(8192);

        let (stdout_files, stderr_files) =
            open_redirect_files(state.clone(), &self.redirects).await?;

        let stdout: Option<Box<dyn AsyncWrite + Unpin + Send>> = if stdout_files.is_empty() {
            Some(Box::new(tokio::io::stdout()))
        } else {
            None
        };
        let stderr: Option<Box<dyn AsyncWrite + Unpin + Send>> = if stderr_files.is_empty() {
            Some(Box::new(tokio::io::stderr()))
        } else {
            None
        };

        tasks.push(tokio::spawn(async move {
            multiplex_stream(stdout_rx, stdout_files, stdout).await;
        }));
        tasks.push(tokio::spawn(async move {
            multiplex_stream(stderr_rx, stderr_files, stderr).await;
        }));

        // Execute
        match self.kind {
            ProcessKind::Builtin(builtin) => {
                let state_clone = state.clone();
                track_job(
                    state,
                    background_process_info,
                    std::process::id(),
                    tasks,
                    async move {
                        builtin
                            .process(
                                state_clone.lock().await,
                                &self.args,
                                None::<tokio::io::Stdin>,
                                stdout_tx,
                                stderr_tx,
                            )
                            .await;
                    },
                )
                .await;
            }
            ProcessKind::External(exec) => {
                let mut cmd = Command::new(exec);
                cmd.args(&self.args);
                cmd.stdin(Stdio::inherit());
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());

                let mut child = cmd.spawn().expect("Failed to execute process");
                bridge_child_streams(&mut child, &mut tasks, None, stdout_tx, stderr_tx);

                track_job(
                    state,
                    background_process_info,
                    child.id().unwrap_or(0),
                    tasks,
                    async move {
                        let _ = child.wait().await;
                    },
                )
                .await;
            }
        };

        Ok(())
    }
}

#[derive(Debug)]
pub enum ProcessError {
    ProcessNotFound(String),
    MissingClosingBracket,
}

impl Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessError::ProcessNotFound(msg) => f.write_str(msg),
            ProcessError::MissingClosingBracket => f.write_str("Missing closing '}'"),
        }
    }
}

pub async fn run_pipeline(
    state: Arc<Mutex<State>>,
    commands: &Vec<ExprCommand>,
) -> Result<(), ProcessError> {
    let len = commands.len();
    if len == 0 {
        return Ok(());
    }

    let mut processes = Vec::with_capacity(commands.len());
    for command in commands {
        let process = Process::init(state.clone(), command).await?;
        processes.push(process);
    }

    let mut tasks: Vec<JoinHandle<()>> = Vec::new();

    // Wire the pipes and file redirections
    for i in 0..len {
        let (stdout_rx, stdout_tx) = tokio::io::duplex(8192);
        let (stderr_rx, stderr_tx) = tokio::io::duplex(8192);

        let process = &mut processes[i];
        let command = &commands[i];

        process.stdout_tx = Some(stdout_tx);
        process.stderr_tx = Some(stderr_tx);

        let is_last = i == len - 1;

        let (stdout_files, stderr_files) =
            open_redirect_files(state.clone(), &command.redirects).await?;

        let extra_stdout_dest: Option<Box<dyn AsyncWrite + Unpin + Send>> = if !is_last {
            let (next_in_rx, next_in_tx) = tokio::io::duplex(8192);
            processes[i + 1].stdin_rx = Some(next_in_rx);
            Some(Box::new(next_in_tx))
        } else if stdout_files.is_empty() {
            Some(Box::new(tokio::io::stdout()))
        } else {
            None
        };

        let extra_stderr_dest: Option<Box<dyn AsyncWrite + Unpin + Send>> =
            if stderr_files.is_empty() {
                Some(Box::new(tokio::io::stderr()))
            } else {
                None
            };

        tasks.push(tokio::spawn(async move {
            multiplex_stream(stdout_rx, stdout_files, extra_stdout_dest).await;
        }));
        tasks.push(tokio::spawn(async move {
            multiplex_stream(stderr_rx, stderr_files, extra_stderr_dest).await
        }));
    }

    // Concurrent execution layer
    let mut children = Vec::new();
    for process in processes.drain(..) {
        let stdin_rx = process.stdin_rx;
        let stdout_tx = process.stdout_tx.unwrap();
        let stderr_tx = process.stderr_tx.unwrap();
        let args = process.args;

        match process.kind {
            ProcessKind::Builtin(builtin) => {
                let state = state.clone();

                tasks.push(tokio::spawn(async move {
                    let state_guard = state.lock().await;

                    builtin
                        .process(state_guard, &args, stdin_rx, stdout_tx, stderr_tx)
                        .await;
                }));
            }
            ProcessKind::External(exec) => {
                let mut cmd = Command::new(exec);
                cmd.args(&args);
                if stdin_rx.is_some() {
                    cmd.stdin(Stdio::piped());
                }
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());

                let mut child = cmd.spawn().expect("Failed to execute process");
                bridge_child_streams(&mut child, &mut tasks, stdin_rx, stdout_tx, stderr_tx);

                children.push(child);
            }
        };
    }

    // Sync and reap
    for mut child in children {
        let _ = child.wait().await;
    }
    for task in tasks {
        let _ = task.await;
    }

    Ok(())
}

async fn open_redirect_files(
    state: Arc<Mutex<State>>,
    redirects: &Vec<ExprRedirect>,
) -> Result<(Vec<File>, Vec<File>), ProcessError> {
    let mut stdout_files = Vec::new();
    let mut stderr_files = Vec::new();
    for redirect in redirects {
        let file = redirect
            .open_file(state.clone())
            .await
            .map_err(|_| ProcessError::MissingClosingBracket)?;
        if redirect.is_stdout() {
            stdout_files.push(file);
        } else if redirect.is_stderr() {
            stderr_files.push(file);
        } else {
            unreachable!();
        }
    }

    Ok((stdout_files, stderr_files))
}

async fn multiplex_stream<R>(
    mut reader: R,
    mut files: Vec<File>,
    mut extra_dest: Option<Box<dyn AsyncWrite + Unpin + Send>>,
) where
    R: AsyncReadExt + Unpin,
{
    let mut buffer = [0u8; 4096];
    while let Ok(n) = reader.read(&mut buffer).await {
        if n == 0 {
            break;
        }
        let chunk = &buffer[..n];

        if let Some(dest) = extra_dest.as_mut() {
            if dest.write_all(chunk).await.is_err() {
                break;
            } else {
                let _ = dest.flush().await;
            }
        }

        for file in &mut files {
            let _ = file.write_all(chunk).await;
            let _ = file.flush().await;
        }
    }
}

fn bridge_child_streams(
    child: &mut Child,
    tasks: &mut Vec<JoinHandle<()>>,
    stdin_rx: Option<DuplexStream>,
    mut stdout_tx: DuplexStream,
    mut stderr_tx: DuplexStream,
) {
    if let Some(mut child_in) = child.stdin.take() {
        if let Some(mut stdin_rx) = stdin_rx {
            tasks.push(tokio::spawn(async move {
                let _ = tokio::io::copy(&mut stdin_rx, &mut child_in).await;
            }));
        }
    }

    if let Some(mut child_out) = child.stdout.take() {
        tasks.push(tokio::spawn(async move {
            let _ = tokio::io::copy(&mut child_out, &mut stdout_tx).await;
        }));
    }

    if let Some(mut child_err) = child.stderr.take() {
        tasks.push(tokio::spawn(async move {
            let _ = tokio::io::copy(&mut child_err, &mut stderr_tx).await;
        }));
    }
}

async fn track_job<F>(
    state: Arc<Mutex<State>>,
    background_process_info: Option<BackgroundProcessInfo>,
    pid: u32,
    tasks: Vec<JoinHandle<()>>,
    execution: F,
) where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    if let Some(BackgroundProcessInfo { command }) = background_process_info {
        let id = state.lock().await.create_job(pid, command);
        tokio::spawn(async move {
            execution.await;
            for task in tasks {
                let _ = task.await;
                state.lock().await.mark_job_done(id);
            }
        });
    } else {
        execution.await;
        for task in tasks {
            let _ = task.await;
        }
    }
}
