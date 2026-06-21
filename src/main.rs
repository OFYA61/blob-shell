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

use self::ast::ExprKind;
use self::ast::ExprPipedCommands;
use self::process::Pipeline;
use self::process::Process;
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
                ExprKind::Command(command) => {
                    match Process::init_from_expr_command(
                        state.clone(),
                        &command,
                        false,
                        command
                            .redirects
                            .iter()
                            .any(|redirect| redirect.is_stdout()),
                        command
                            .redirects
                            .iter()
                            .any(|redirect| redirect.is_stderr()),
                    )
                    .await
                    {
                        Ok(mut process) => {
                            let (stdout_files, stderr_files) = process.get_redirect_files().await;
                            if expr.is_background {
                                let state = state.clone();
                                let mut command = String::from(command_raw);
                                command.remove(command.rfind("&").unwrap());
                                let command = command.trim().to_owned();
                                let id = state.lock().await.create_job(process.pid, command);
                                tokio::spawn(async move {
                                    process.run(None, stdout_files, stderr_files).await;
                                    state.lock().await.mark_job_done(id);
                                });
                            } else {
                                process.run(None, stdout_files, stderr_files).await;
                            }
                        }
                        Err(err) => {
                            println!("{}", err);
                            continue;
                        }
                    }
                }
                ExprKind::PipedCommands(ExprPipedCommands { commands }) => {
                    if commands.is_empty() {
                        continue;
                    }

                    match Pipeline::init(state.clone(), &commands).await {
                        Ok(mut pipeline) => {
                            pipeline.run().await;
                        }
                        Err(err) => {
                            println!("{}", err);
                            continue;
                        }
                    }
                }
            }
        }
    }
}
