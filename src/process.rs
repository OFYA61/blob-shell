use std::fmt::Display;
use std::process::Stdio;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::io::BufWriter;
use tokio::process::Child;
use tokio::process::ChildStderr;
use tokio::process::ChildStdin;
use tokio::process::ChildStdout;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::ast::ExprCommand;
use crate::ast::ExprRedirect;
use crate::builtin::Builtin;
use crate::state::State;

#[derive(Debug)]
pub struct Pipeline {
    processes: Vec<Process>,
}

impl Pipeline {
    pub async fn init(
        state: Arc<Mutex<State>>,
        commands: &Vec<ExprCommand>,
    ) -> Result<Self, ProcessError> {
        let mut processes: Vec<Process> = Vec::with_capacity(commands.len());
        for (i, command) in commands.iter().enumerate() {
            let pipe_stdin = if i == 0 { false } else { true };
            let pipe_stdout = i != commands.len() - 1
                || command
                    .redirects
                    .iter()
                    .any(|redirect| redirect.is_stdout());
            let pipe_stderr = command
                .redirects
                .iter()
                .any(|redirect| redirect.is_stderr());
            let process = Process::init_from_expr_command(
                state.clone(),
                &command,
                pipe_stdin,
                pipe_stdout,
                pipe_stderr,
            )
            .await;
            if let Err(err) = process {
                // TODO: Validate that all commands are valid before spawning the processes
                return Err(err);
            }
            processes.push(process.unwrap());
        }

        Ok(Self { processes })
    }

    pub async fn run(&mut self) {
        let mut handles: Vec<JoinHandle<()>> = Vec::new();
        for i in 0..self.processes.len() {
            let is_last = i == self.processes.len() - 1;

            let process = self.processes.get_mut(i).unwrap();
            let stdout = process.get_stdout();
            let stderr = process.get_stderr();

            let (mut stdout_files, mut stderr_files) = process.get_redirect_files().await;

            let next_stdin = if !is_last {
                self.processes.get_mut(i + 1).unwrap().get_stdin()
            } else {
                None
            };

            if let Some(stdout) = stdout {
                let stdout_handle = tokio::spawn(async move {
                    let mut reader = stdout;
                    let mut next_writer = next_stdin.map(BufWriter::new);

                    let mut buffer = [0u8; 4096];
                    loop {
                        match reader.read(&mut buffer).await {
                            Ok(0) => break,
                            Ok(n) => {
                                let buf = &buffer[..n];
                                if let Some(next_writer) = next_writer.as_mut() {
                                    next_writer
                                        .write_all(buf)
                                        .await
                                        .expect("failed to write to next stdin");
                                    next_writer
                                        .flush()
                                        .await
                                        .expect("Failed to flush next writer");
                                }
                                for file in &mut stdout_files {
                                    file.write_all(buf).await.expect("Failed to write to file");
                                    file.flush().await.expect("Failed to flush file");
                                }
                            }
                            Err(_) => break,
                        }
                    }
                });
                handles.push(stdout_handle);
            }

            if let Some(stderr) = stderr {
                let stderr_handle = tokio::spawn(async move {
                    let mut reader = stderr;
                    let mut buffer = [0u8; 4096];
                    loop {
                        match reader.read(&mut buffer).await {
                            Ok(0) => break,
                            Ok(n) => {
                                let buf = &buffer[..n];
                                for file in &mut stderr_files {
                                    file.write_all(buf).await.expect("Failed to write to file");
                                }
                            }
                            Err(_) => break,
                        }
                    }
                });
                handles.push(stderr_handle);
            }
        }

        for process in &mut self.processes {
            process.wait().await;
        }

        for handle in handles {
            handle.await.expect("Failed to join pipe handle");
        }
    }
}

#[derive(Debug)]
enum ProcessKind {
    Builtin(Builtin),
    Command(Child),
}

#[derive(Debug)]
pub struct Process {
    state: Arc<Mutex<State>>,
    pub pid: u32,
    kind: ProcessKind,
    redirects: Vec<ExprRedirect>,

    args: Vec<String>,
}

impl Process {
    pub async fn init_from_expr_command(
        state: Arc<Mutex<State>>,
        command: &ExprCommand,
        pipe_stdin: bool,
        pipe_stdout: bool,
        pipe_stderr: bool,
    ) -> Result<Self, ProcessError> {
        let ExprCommand { exec, args, .. } = command;

        let exec = exec.process();
        let args = args
            .iter()
            .map(|arg| arg.process().to_owned())
            .collect::<Vec<String>>();
        Process::init(
            state,
            exec,
            args,
            command.redirects.clone(),
            pipe_stdin,
            pipe_stdout,
            pipe_stderr,
        )
        .await
    }

    async fn init(
        state: Arc<Mutex<State>>,
        exec: &str,
        args: Vec<String>,
        redirects: Vec<ExprRedirect>,
        pipe_stdin: bool,
        pipe_stdout: bool,
        pipe_stderr: bool,
    ) -> Result<Self, ProcessError> {
        let pid: u32;
        let kind: ProcessKind;
        if let Some(builtin) = Builtin::from_str(exec) {
            pid = std::process::id();
            kind = ProcessKind::Builtin(builtin);
        } else if let Some(_) = state.lock().await.get_command(exec) {
            let mut child = Command::new(&exec);
            if args.len() > 0 {
                child.args(&args);
            }

            if pipe_stdin {
                child.stdin(Stdio::piped());
            }
            if pipe_stdout {
                child.stdout(Stdio::piped());
            }
            if pipe_stderr {
                child.stderr(Stdio::piped());
            }

            let child = child.spawn().expect("Failed to execute process");

            pid = child.id().unwrap_or(0);
            kind = ProcessKind::Command(child);
        } else {
            return Err(ProcessError::ProcessNotFound(format!(
                "{}: command not found",
                exec
            )));
        }

        Ok(Self {
            state,
            pid,
            kind,
            redirects,
            args,
        })
    }

    pub async fn run(
        &mut self,
        in_stream: Option<ChildStdout>,
        mut stdout_files: Vec<File>,
        mut stderr_files: Vec<File>,
    ) {
        match &mut self.kind {
            ProcessKind::Builtin(builtin) => {
                builtin
                    .process(
                        self.state.lock().await,
                        &self.args,
                        stdout_files,
                        stderr_files,
                    )
                    .await;
            }
            ProcessKind::Command(child) => {
                let stdin = child.stdin.take();
                let stdout = child.stdout.take();
                let stderr = child.stderr.take();

                let stdout_handle: Option<JoinHandle<()>> = if let Some(stdout) = stdout {
                    Some(tokio::spawn(async move {
                        if let Some(in_stream) = in_stream
                            && let Some(stdin) = stdin
                        {
                            println!("Piping output");
                            let mut reader = BufReader::new(in_stream).lines();
                            let mut writer = BufWriter::new(stdin);
                            while let Ok(Some(line)) = reader.next_line().await {
                                let bytes = format!("{}\n", line).as_bytes().to_vec();
                                writer
                                    .write_all(&bytes)
                                    .await
                                    .expect("Failed to write to stdin");
                            }
                        }

                        let mut reader = BufReader::new(stdout).lines();

                        while let Ok(Some(line)) = reader.next_line().await {
                            let bytes = format!("{}\n", line).as_bytes().to_vec();
                            for file in &mut stdout_files {
                                file.write_all(&bytes)
                                    .await
                                    .expect("Failed to write to file");
                                file.flush().await.expect("Failed to flush file");
                            }
                        }
                    }))
                } else {
                    None
                };

                let stderr_handle: Option<JoinHandle<()>> = if let Some(stderr) = stderr {
                    Some(tokio::spawn(async move {
                        let mut reader = BufReader::new(stderr).lines();

                        while let Ok(Some(line)) = reader.next_line().await {
                            let bytes = format!("{}\n", line).as_bytes().to_vec();
                            for file in &mut stderr_files {
                                file.write_all(&bytes)
                                    .await
                                    .expect("Failed to write to file");
                                file.flush().await.expect("Failed to flush file");
                            }
                        }
                    }))
                } else {
                    None
                };

                child
                    .wait()
                    .await
                    .expect("Failed to wait for child with output");
                if let Some(handle) = stdout_handle {
                    handle.await.expect("Failed to join stdout handle");
                }
                if let Some(handle) = stderr_handle {
                    handle.await.expect("Failed to join stderr handle");
                }
            }
        };
    }

    pub async fn get_redirect_files(&self) -> (Vec<File>, Vec<File>) {
        let mut stdout_files = Vec::new();
        let mut stderr_files = Vec::new();
        for redirect in &self.redirects {
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

    fn get_stdin(&mut self) -> Option<ChildStdin> {
        match &mut self.kind {
            ProcessKind::Builtin(_) => todo!(),
            ProcessKind::Command(child) => child.stdin.take(),
        }
    }

    fn get_stdout(&mut self) -> Option<ChildStdout> {
        match &mut self.kind {
            ProcessKind::Builtin(_) => todo!(),
            ProcessKind::Command(child) => child.stdout.take(),
        }
    }

    fn get_stderr(&mut self) -> Option<ChildStderr> {
        match &mut self.kind {
            ProcessKind::Builtin(_) => todo!(),
            ProcessKind::Command(child) => child.stderr.take(),
        }
    }

    async fn wait(&mut self) {
        match &mut self.kind {
            ProcessKind::Builtin(_) => todo!(),
            ProcessKind::Command(child) => child.wait().await.expect("Failed to wait for child"),
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
