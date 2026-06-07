mod builtin;
mod env;
mod parser;

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::process::Stdio;
use std::thread;

fn main() {
    loop {
        print!("$ ");
        std::io::stdout().flush().expect("Failed to flush stdout");

        let mut command_raw = String::new();
        match std::io::stdin().read_line(&mut command_raw) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => panic!("Failed to read user intpu"),
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
                } => {
                    let exec = exec.process();
                    let args = args.iter().map(|arg| arg.process()).collect::<Vec<&str>>();
                    let mut stdout_files = redirects
                        .iter()
                        .filter(|r| r.is_stdout())
                        .map(|r| r.open_file())
                        .collect::<Vec<File>>();

                    if builtin::try_process(exec, &args, &mut stdout_files) {
                        continue;
                    }

                    if let Some(exec) = env::get_command(&exec) {
                        let should_pipe_stdout = !stdout_files.is_empty();

                        let mut child = std::process::Command::new(&exec);
                        if args.len() > 0 {
                            child.args(args);
                        }

                        if should_pipe_stdout {
                            child.stdout(Stdio::piped());
                        }

                        let mut child = child.spawn().expect("Failed to execute process");

                        if let Some(stdout) = child.stdout.take() {
                            let handle = thread::spawn(move || {
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
                            });

                            handle.join().expect("Thread paniced");
                        }

                        child.wait().expect("Failed to wait on command");
                        continue;
                    }

                    println!("{}: command not found", exec);
                }
            }
        }
    }
}
