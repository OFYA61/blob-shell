mod ast;
mod autocomplete;
mod builtin;
mod completer;
mod input;
mod jobs;
mod state;

use std::process::Stdio;
use std::sync::Arc;

use crossterm::terminal::disable_raw_mode;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::process::Command;
use tokio::sync::Mutex;

use self::builtin::Builtin;
use self::jobs::Job;
use self::jobs::Jobs;
use self::state::State;

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(State::init()));
    let jobs = Arc::new(Mutex::new(Jobs::init()));

    loop {
        jobs.lock().await.reap_done_jobs(true);

        let command_raw = match input::get_input(state.lock().await) {
            Ok(input) => input,
            Err(err) => {
                disable_raw_mode().expect("Failed to disable raw mode");
                println!();
                panic!("Failed to read input {:?}", err);
            }
        };

        let command_raw = command_raw.trim();
        if command_raw.is_empty() {
            continue;
        }

        let ast = ast::parse(command_raw);
        if ast.is_err() {
            continue;
        }

        // Interpret the AST
        for expr in ast.unwrap() {
            match expr {
                ast::Expr::Command {
                    exec,
                    args,
                    redirects,
                    is_background,
                } => {
                    let exec = exec.process();
                    let args = args.iter().map(|arg| arg.process()).collect::<Vec<&str>>();
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

                    if let Some(builtin) = Builtin::from_str(exec) {
                        builtin
                            .process(
                                state.lock().await,
                                jobs.lock().await,
                                args,
                                stdout_files,
                                stderr_files,
                            )
                            .await;
                        continue;
                    }

                    if let Some(_) = state.lock().await.get_command(&exec) {
                        let mut child = Command::new(&exec);
                        if args.len() > 0 {
                            child.args(args);
                        }

                        if !stdout_files.is_empty() {
                            child.stdout(Stdio::piped());
                        }
                        if !stderr_files.is_empty() {
                            child.stderr(Stdio::piped());
                        }

                        let mut child = child.spawn().expect("Failed to execute process");
                        let pid = child.id().unwrap_or(0) as i32;

                        let child_process_handler =
                            async move |id: Option<usize>, jobs: Option<Arc<Mutex<Jobs>>>| {
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

                                let output = child
                                    .wait_with_output()
                                    .await
                                    .expect("Failed to wait for child with output");

                                if let Some(id) = id
                                    && let Some(jobs) = jobs
                                {
                                    jobs.lock().await.mark_job_done(id);
                                }

                                output
                            };

                        if is_background {
                            let command = command_raw[..command_raw.len() - 1].trim().to_owned();
                            let Job { id, pid, .. } = *jobs.lock().await.create_job(pid, command);
                            println!("[{}] {}", id, pid);
                            tokio::spawn(child_process_handler(Some(id), Some(Arc::clone(&jobs))));
                        } else {
                            let _ = child_process_handler(None, None).await;
                        }

                        continue;
                    }

                    println!("{}: command not found", exec);
                }
            }
        }
    }
}
