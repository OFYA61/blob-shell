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
use self::process::BackgroundProcessInfo;
use self::process::Process;
use self::state::State;

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(State::init().await));

    loop {
        state
            .lock()
            .await
            .reap_done_jobs(tokio::io::stdout(), true)
            .await;

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

        // TODO: Remove this if statement after codecrafts is no longer a thing, for some reason on that end `exit`
        // does not show up on the history so I'm adding it manually. Otherwise I get flaky test
        // results
        if !command_raw.contains("exit") {
            state.lock().await.add_history(command_raw.to_string());
        }

        let ast = ast::parse(command_raw);
        if ast.is_err() {
            continue;
        }

        // Interpret the AST
        for expr in ast.unwrap() {
            match expr.kind {
                ExprKind::Command(command) => match Process::init(state.clone(), &command).await {
                    Ok(process) => {
                        process
                            .run(
                                state.clone(),
                                if expr.is_background {
                                    let mut command = String::from(command_raw);
                                    command.remove(command.rfind("&").unwrap());
                                    let command = command.trim().to_owned();
                                    Some(BackgroundProcessInfo { command })
                                } else {
                                    None
                                },
                            )
                            .await;
                    }
                    Err(err) => {
                        println!("{}", err);
                        continue;
                    }
                },
                ExprKind::PipedCommands(ExprPipedCommands { commands }) => {
                    // TODO: Implement background processing for pipes
                    if commands.is_empty() {
                        continue;
                    }

                    match process::run_pipeline(state.clone(), &commands).await {
                        Ok(()) => {}
                        Err(err) => {
                            println!("{}", err);
                            continue;
                        }
                    };
                }
            }
        }
    }
}
