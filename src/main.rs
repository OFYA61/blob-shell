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
                ExprKind::Command(command) => match Process::init(state.clone(), &command).await {
                    Ok(mut process) => {
                        if expr.is_background {
                            // let state = state.clone();
                            // let mut command = String::from(command_raw);
                            // command.remove(command.rfind("&").unwrap());
                            // let command = command.trim().to_owned();
                            // let id = state.lock().await.create_job(process.pid, command);
                            // tokio::spawn(async move {
                            //     process.run().await;
                            //     state.lock().await.mark_job_done(id);
                            // });
                        } else {
                            process.run(state.clone()).await;
                        }
                    }
                    Err(err) => {
                        println!("{}", err);
                        continue;
                    }
                },
                ExprKind::PipedCommands(ExprPipedCommands { commands }) => {
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
