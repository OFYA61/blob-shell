mod autocomplete;
mod builtin;
mod env;
mod input;
mod jobs;
mod parser;

use std::process::Stdio;
use std::sync::Arc;

use crossterm::terminal::disable_raw_mode;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::process::Command;
use tokio::sync::Mutex;

use self::jobs::Job;
use self::jobs::Jobs;

#[tokio::main]
async fn main() {
    let jobs = Arc::new(Mutex::new(Jobs::init()));

    loop {
        jobs.lock().await.cleanup_completed_jobs();

        let command_raw = match input::get_input() {
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

        let ast = parser::parse_to_ast(command_raw);
        if ast.is_err() {
            continue;
        }

        // Interpret the AST
        for expr in ast.unwrap() {
            match expr {
                parser::Expr::Command {
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

                    if builtin::process(
                        Arc::clone(&jobs),
                        exec,
                        &args,
                        &mut stdout_files,
                        &mut stderr_files,
                    )
                    .await
                    {
                        continue;
                    }

                    if let Some(_) = env::get_command(&exec) {
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
                                    jobs.lock()
                                        .await
                                        .iter_mut()
                                        .find(|(jid, _)| **jid == id)
                                        .map(|(_, job)| job.mark_done());
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
