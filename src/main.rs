mod autocomplete;
mod builtin;
mod env;
mod input;
mod parser;

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::process::Stdio;
use std::thread;

use crossterm::terminal::disable_raw_mode;

fn main() {
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
                    let mut stdout_files = redirects
                        .iter()
                        .filter(|r| r.is_stdout())
                        .map(|r| r.open_file())
                        .collect::<Vec<File>>();
                    let mut stderr_files = redirects
                        .iter()
                        .filter(|r| r.is_stderr())
                        .map(|r| r.open_file())
                        .collect::<Vec<File>>();

                    if builtin::try_process(exec, &args, &mut stdout_files, &mut stderr_files) {
                        continue;
                    }

                    if let Some(_) = env::get_command(&exec) {
                        let mut child = std::process::Command::new(&exec);
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

                        let stdout_handle = child.stdout.take().map(|stdout| {
                            thread::spawn(move || {
                                let reader = BufReader::new(stdout);
                                for line_result in reader.lines() {
                                    if line_result.is_err() {
                                        continue;
                                    }
                                    let line = line_result.unwrap();
                                    let bytes = format!("{}\n", line).as_bytes().to_vec();
                                    for file in &mut stdout_files {
                                        file.write_all(&bytes).expect("Failed to write to file");
                                        file.flush().expect("Failed to flush file");
                                    }
                                }
                            })
                        });

                        let stderr_handle = child.stderr.take().map(|stderr| {
                            thread::spawn(move || {
                                let reader = BufReader::new(stderr);
                                for line_result in reader.lines() {
                                    if line_result.is_err() {
                                        continue;
                                    }
                                    let line = line_result.unwrap();
                                    let bytes = format!("{}\n", line).as_bytes().to_vec();
                                    for file in &mut stderr_files {
                                        file.write_all(&bytes).expect("Failed to write to file");
                                        file.flush().expect("Failed to flush file");
                                    }
                                }
                            })
                        });

                        if !is_background {
                            child.wait().expect("Failed to wait on command");
                            if let Some(h) = stdout_handle {
                                h.join().expect("Stdout thread paniced")
                            };
                            if let Some(h) = stderr_handle {
                                h.join().expect("Stderr thread paniced")
                            };
                        } else {
                            println!("[{}] {}", job_counter, child.id());
                            job_counter += 1;
                        }

                        continue;
                    }

                    println!("{}: command not found", exec);
                }
            }
        }
    }
}
