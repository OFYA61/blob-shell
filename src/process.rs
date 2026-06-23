use std::fmt::Display;
use std::process::Stdio;
use std::sync::Arc;

use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::DuplexStream;
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
        let exec_str = command.exec.process();

        let kind = if let Some(builtin) = Builtin::from_str(exec_str) {
            ProcessKind::Builtin(builtin)
        } else if state.lock().await.get_command(exec_str).is_some() {
            ProcessKind::External(exec_str.to_owned())
        } else {
            return Err(ProcessError::ProcessNotFound(format!(
                "{}: command not found",
                exec_str
            )));
        };

        Ok(Self {
            kind,
            args: command
                .args
                .iter()
                .map(|arg| arg.process().to_owned())
                .collect(),
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
    ) {
        let mut tasks: Vec<JoinHandle<()>> = Vec::new();

        // Setup I/O and file redirection
        let (stdout_rx, mut stdout_tx) = tokio::io::duplex(8192);
        let (stderr_rx, mut stderr_tx) = tokio::io::duplex(8192);

        let (stdout_files, stderr_files) = open_redirect_files(&self.redirects).await;

        tasks.push(tokio::spawn(async move {
            let mut reader = stdout_rx;
            let mut files = stdout_files;
            let mut buffer = [0u8; 4096];

            while let Ok(n) = reader.read(&mut buffer).await {
                if n == 0 {
                    break;
                }
                let chunk = &buffer[..n];

                for file in &mut files {
                    let _ = file.write_all(chunk).await;
                    let _ = file.flush().await;
                }

                if files.is_empty() {
                    let mut stdout = tokio::io::stdout();
                    let _ = stdout.write_all(chunk).await;
                    let _ = stdout.flush().await;
                }
            }
        }));

        tasks.push(tokio::spawn(async move {
            let mut reader = stderr_rx;
            let mut files = stderr_files;
            let mut buffer = [0u8; 4096];

            while let Ok(n) = reader.read(&mut buffer).await {
                if n == 0 {
                    break;
                }
                let chunk = &buffer[..n];

                for file in &mut files {
                    let _ = file.write_all(chunk).await;
                    let _ = file.flush().await;
                }
                if files.is_empty() {
                    let mut stderr = tokio::io::stderr();
                    let _ = stderr.write_all(chunk).await;
                    let _ = stderr.flush().await;
                }
            }
        }));

        // Execute
        match self.kind {
            ProcessKind::Builtin(builtin) => {
                if let Some(BackgroundProcessInfo { command }) = background_process_info {
                    let state_clone = state.clone();
                    let id = state_clone
                        .lock()
                        .await
                        .create_job(std::process::id(), command);

                    tokio::spawn(async move {
                        let state = state.lock().await;
                        builtin
                            .process(
                                state,
                                &self.args,
                                None::<tokio::io::Stdin>,
                                stdout_tx,
                                stderr_tx,
                            )
                            .await;
                        for task in tasks {
                            let _ = task.await;
                        }
                        state_clone.lock().await.mark_job_done(id);
                    });
                } else {
                    let state = state.lock().await;
                    builtin
                        .process(
                            state,
                            &self.args,
                            None::<tokio::io::Stdin>,
                            stdout_tx,
                            stderr_tx,
                        )
                        .await;
                    for task in tasks {
                        let _ = task.await;
                    }
                }
            }
            ProcessKind::External(exec) => {
                let mut cmd = Command::new(exec);
                cmd.args(&self.args);
                cmd.stdin(Stdio::inherit());
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());

                let mut child = cmd.spawn().expect("Failed to execute process");
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

                if let Some(BackgroundProcessInfo { command }) = background_process_info {
                    let state_clone = state.clone();
                    let id = state_clone
                        .lock()
                        .await
                        .create_job(child.id().unwrap_or(0), command);

                    tokio::spawn(async move {
                        let _ = child.wait().await;
                        for task in tasks {
                            let _ = task.await;
                        }
                        state_clone.lock().await.mark_job_done(id);
                    });
                } else {
                    let _ = child.wait().await;
                    for task in tasks {
                        let _ = task.await;
                    }
                }
            }
        };
    }
}

#[derive(Debug)]
pub enum ProcessError {
    ProcessNotFound(String),
}

impl Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessError::ProcessNotFound(msg) => f.write_str(msg),
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

        let (stdout_files, stderr_files) = open_redirect_files(&command.redirects).await;

        let next_stdin_tx = if !is_last {
            let (next_in_rx, next_in_tx) = tokio::io::duplex(8192);
            processes[i + 1].stdin_rx = Some(next_in_rx);
            Some(next_in_tx)
        } else {
            None
        };

        // Multiplex stdout into pipelines and files
        tasks.push(tokio::spawn(async move {
            let mut reader = stdout_rx;
            let mut next_pipe = next_stdin_tx;
            let mut files = stdout_files;
            let mut buffer = [0u8; 4096];

            while let Ok(n) = reader.read(&mut buffer).await {
                if n == 0 {
                    break;
                }

                let chunk = &buffer[..n];
                if let Some(pipe) = next_pipe.as_mut() {
                    if pipe.write_all(chunk).await.is_err() {
                        next_pipe = None;
                    }
                }

                for file in &mut files {
                    let _ = file.write_all(chunk).await;
                    let _ = file.flush().await;
                }
                if is_last && files.is_empty() {
                    let mut stdout = tokio::io::stdout();
                    let _ = stdout.write_all(chunk).await;
                    let _ = stdout.flush().await;
                }
            }
        }));

        // Multiplex stderr into files and stderr streams
        tasks.push(tokio::spawn(async move {
            let mut reader = stderr_rx;
            let mut files = stderr_files;
            let mut buffer = [0u8; 4096];

            while let Ok(n) = reader.read(&mut buffer).await {
                if n == 0 {
                    break;
                }
                let chunk = &buffer[..n];

                for file in &mut files {
                    let _ = file.write_all(chunk).await;
                    let _ = file.flush().await;
                }
                if files.is_empty() {
                    let mut stderr = tokio::io::stderr();
                    let _ = stderr.write_all(chunk).await;
                    let _ = stderr.flush().await;
                }
            }
        }));
    }

    // Concurrent execution layer
    let mut children = Vec::new();
    for process in processes.drain(..) {
        let stdin_rx = process.stdin_rx;
        let mut stdout_tx = process.stdout_tx.unwrap();
        let mut stderr_tx = process.stderr_tx.unwrap();
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

async fn open_redirect_files(redirects: &Vec<ExprRedirect>) -> (Vec<File>, Vec<File>) {
    let mut stdout_files = Vec::new();
    let mut stderr_files = Vec::new();
    for redirect in redirects {
        let file = redirect.open_file().await;
        if redirect.is_stdout() {
            stdout_files.push(file);
        } else if redirect.is_stderr() {
            stderr_files.push(file);
        } else {
            unreachable!();
        }
    }

    (stdout_files, stderr_files)
}
