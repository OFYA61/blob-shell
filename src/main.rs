mod ast;
mod autocomplete;
mod builtin;
mod completer;
mod input;
mod process;
mod state;

use std::sync::Arc;

use crossterm::terminal::disable_raw_mode;
use tokio::sync::Mutex;

use self::ast::ExprCommand;
use self::ast::ExprKind;
use self::ast::ExprPipedCommands;
use self::state::State;

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(State::init()));

    loop {
        state.lock().await.reap_done_jobs(true);

        let command_raw = match input::get_input(state.clone()).await {
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
            match expr.kind {
                ExprKind::Command(ExprCommand {
                    exec,
                    args,
                    redirects,
                }) => {
                    let exec = exec.process();
                    let args = args
                        .iter()
                        .map(|arg| arg.process().to_owned())
                        .collect::<Vec<String>>();
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

                    let result = if expr.is_background {
                        process::run_background(
                            state.clone(),
                            exec,
                            args,
                            stdout_files,
                            stderr_files,
                            command_raw,
                        )
                        .await
                    } else {
                        process::run(state.clone(), exec, args, stdout_files, stderr_files).await
                    };

                    match result {
                        Ok(()) => {}
                        Err(_) => println!("{}: command not found", exec),
                    };
                }
                ExprKind::PipedCommands(ExprPipedCommands { commands }) => {
                    println!("{:#?}", commands);
                }
            }
        }
    }
}
