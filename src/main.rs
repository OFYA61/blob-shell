mod autocomplete;
mod builtin;
mod env;
mod input;
mod parser;

use std::process::Stdio;

use crossterm::terminal::disable_raw_mode;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::process::Command;

#[tokio::main]
async fn main() {
    let mut job_counter = 1;

    loop {
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

                    if builtin::try_process(exec, &args, &mut stdout_files, &mut stderr_files).await
                    {
                        continue;
                    }

                    if let Some(_) = env::get_command(&exec) {
                        let mut child = Command::new(&exec);
                        if args.len() > 0 {
                            child.args(args);
                        }
                        child.stdout(Stdio::piped());
                        child.stderr(Stdio::piped());

                        let mut child = child.spawn().expect("Failed to execute process");
                        let pid = child.id().unwrap_or(0);

                        let child_process_handler = async move {
                            if let Some(stdout) = child.stdout.take() {
                                let mut reader = BufReader::new(stdout).lines();
                                let has_files = !stdout_files.is_empty();
                                while let Ok(Some(line)) = reader.next_line().await {
                                    if !has_files {
                                        println!("{}", line);
                                        continue;
                                    }
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

                            child
                                .wait_with_output()
                                .await
                                .expect("Failed to wait for child with output")
                        };

                        if is_background {
                            println!("[{}] {}", job_counter, pid);
                            job_counter += 1;
                            tokio::spawn(child_process_handler);
                        } else {
                            let _ = child_process_handler.await;
                        }

                        continue;
                    }

                    println!("{}: command not found", exec);
                }
            }
        }
    }
}
