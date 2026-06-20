use std::process::Stdio;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::process::Child;
use tokio::process::Command;
use tokio::sync::Mutex;

use crate::builtin::Builtin;
use crate::state::State;

pub enum ProcessRunError {
    ProcessNotFound,
}

pub async fn run_background(
    state: Arc<Mutex<State>>,
    exec: &str,
    args: Vec<String>,
    stdout_files: Vec<File>,
    stderr_files: Vec<File>,
    command: &str,
) -> Result<(), ProcessRunError> {
    if let Some(builtin) = Builtin::from_str(exec) {
        let id = state
            .lock()
            .await
            .create_job(std::process::id(), command.to_owned());
        let process_handler = async move || {
            run_builtin(builtin, state.clone(), args, stdout_files, stderr_files).await;
            state.lock().await.mark_job_done(id);
        };
        tokio::spawn(process_handler());
        return Ok(());
    }

    let command_maybe = state.lock().await.get_command(&exec);
    if let Some(_) = command_maybe {
        let child = spawn_command(
            exec,
            args,
            !stdout_files.is_empty(),
            !stderr_files.is_empty(),
        );
        let id = state
            .lock()
            .await
            .create_job(child.id().unwrap_or(0), command.to_owned());

        let process_handler = async move || {
            run_command(child, stdout_files, stderr_files).await;
            state.lock().await.mark_job_done(id);
        };
        tokio::spawn(process_handler());
        return Ok(());
    }

    Err(ProcessRunError::ProcessNotFound)
}

pub async fn run(
    state: Arc<Mutex<State>>,
    exec: &str,
    args: Vec<String>,
    stdout_files: Vec<File>,
    stderr_files: Vec<File>,
) -> Result<(), ProcessRunError> {
    if let Some(builtin) = Builtin::from_str(exec) {
        run_builtin(builtin, state, args, stdout_files, stderr_files).await;
        return Ok(());
    }

    let command_maybe = state.lock().await.get_command(&exec);
    if let Some(_) = command_maybe {
        let child = spawn_command(
            exec,
            args,
            !stdout_files.is_empty(),
            !stderr_files.is_empty(),
        );
        run_command(child, stdout_files, stderr_files).await;

        return Ok(());
    }

    Err(ProcessRunError::ProcessNotFound)
}

async fn run_builtin(
    builtin: Builtin,
    state: Arc<Mutex<State>>,
    args: Vec<String>,
    stdout_files: Vec<File>,
    stderr_files: Vec<File>,
) {
    builtin
        .process(state.lock().await, args, stdout_files, stderr_files)
        .await;
}

fn spawn_command(exec: &str, args: Vec<String>, pipe_stdout: bool, pipe_stderr: bool) -> Child {
    let mut child = Command::new(&exec);
    if args.len() > 0 {
        child.args(args);
    }

    if pipe_stdout {
        child.stdout(Stdio::piped());
    }
    if pipe_stderr {
        child.stderr(Stdio::piped());
    }

    child.spawn().expect("Failed to execute process")
}

async fn run_command(mut child: Child, mut stdout_files: Vec<File>, mut stderr_files: Vec<File>) {
    if let Some(stdout) = child.stdout.take() {
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
    }

    if let Some(stderr) = child.stderr.take() {
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
    }

    let _ = child
        .wait_with_output()
        .await
        .expect("Failed to wait for child with output");
}
