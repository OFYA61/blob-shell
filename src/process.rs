use std::process::Stdio;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncBufReadExt;
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

use crate::builtin::Builtin;
use crate::state::State;

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

    args: Vec<String>,
}

impl Process {
    pub async fn init(
        state: Arc<Mutex<State>>,
        exec: &str,
        args: Vec<String>,
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
            return Err(ProcessError::ProcessNotFound);
        }

        Ok(Self {
            state,
            pid,
            kind,
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

    pub fn get_stdin(&mut self) -> Option<ChildStdin> {
        match &mut self.kind {
            ProcessKind::Builtin(_) => todo!(),
            ProcessKind::Command(child) => child.stdin.take(),
        }
    }

    pub fn get_stdout(&mut self) -> Option<ChildStdout> {
        match &mut self.kind {
            ProcessKind::Builtin(_) => todo!(),
            ProcessKind::Command(child) => child.stdout.take(),
        }
    }

    pub fn get_stderr(&mut self) -> Option<ChildStderr> {
        match &mut self.kind {
            ProcessKind::Builtin(_) => todo!(),
            ProcessKind::Command(child) => child.stderr.take(),
        }
    }

    pub async fn wait(&mut self) {
        match &mut self.kind {
            ProcessKind::Builtin(_) => todo!(),
            ProcessKind::Command(child) => child.wait().await.expect("Failed to wait for child"),
        };
    }
}

#[derive(Debug)]
pub enum ProcessError {
    ProcessNotFound,
}
