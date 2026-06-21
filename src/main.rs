mod ast;
mod autocomplete;
mod builtin;
mod completer;
mod input;
mod process;
mod state;

use std::sync::Arc;

use crossterm::terminal::disable_raw_mode;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufWriter;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use self::ast::ExprCommand;
use self::ast::ExprKind;
use self::ast::ExprPipedCommands;
use self::ast::ExprRedirect;
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
                    let (stdout_files, stderr_files) = get_redirect_files(&command.redirects).await;
                    let process = init_process(
                        state.clone(),
                        &command,
                        false,
                        !stdout_files.is_empty(),
                        !stderr_files.is_empty(),
                    )
                    .await;
                    if process.is_none() {
                        continue;
                    }

                    let mut process = process.unwrap();
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
                ExprKind::PipedCommands(ExprPipedCommands { commands }) => {
                    if commands.is_empty() {
                        continue;
                    }

                    let mut pipeline = Pipeline::init(state.clone(), &commands).await;
                    pipeline.run().await;

                    // let mut processes: Vec<Process> = Vec::with_capacity(commands.len());
                    // for (i, command) in commands.iter().enumerate() {
                    //     let pipe_stdin = if i == 0 { false } else { true };
                    //     let pipe_stdout = i != commands.len() - 1
                    //         || command
                    //             .redirects
                    //             .iter()
                    //             .any(|redirect| redirect.is_stdout());
                    //     let pipe_stderr = command
                    //         .redirects
                    //         .iter()
                    //         .any(|redirect| redirect.is_stderr());
                    //     let process = init_process(
                    //         state.clone(),
                    //         command,
                    //         pipe_stdin,
                    //         pipe_stdout,
                    //         pipe_stderr,
                    //     )
                    //     .await
                    //     // TODO: handle missing processes gracefully
                    //     .expect("Failed to init command");
                    //     processes.push(process);
                    // }
                    //
                    // let mut handles: Vec<JoinHandle<()>> = Vec::new();
                    // for i in 0..processes.len() {
                    //     let is_last = i == processes.len() - 1;
                    //
                    //     let command = commands.get(i).unwrap();
                    //     let (mut stdout_files, mut stderr_files) =
                    //         get_redirect_files(&command.redirects).await;
                    //
                    //     let process = processes.get_mut(i).unwrap();
                    //     let stdout = process.get_stdout();
                    //     let stderr = process.get_stderr();
                    //
                    //     let next_stdin = if !is_last {
                    //         processes.get_mut(i + 1).unwrap().get_stdin()
                    //     } else {
                    //         None
                    //     };
                    //
                    //     if let Some(stdout) = stdout {
                    //         let stdout_handle = tokio::spawn(async move {
                    //             let mut reader = stdout;
                    //             let mut next_writer = next_stdin.map(BufWriter::new);
                    //
                    //             let mut buffer = [0u8; 4096];
                    //             loop {
                    //                 match reader.read(&mut buffer).await {
                    //                     Ok(0) => break,
                    //                     Ok(n) => {
                    //                         let buf = &buffer[..n];
                    //                         if let Some(next_writer) = next_writer.as_mut() {
                    //                             next_writer
                    //                                 .write_all(buf)
                    //                                 .await
                    //                                 .expect("failed to write to next stdin");
                    //                             next_writer
                    //                                 .flush()
                    //                                 .await
                    //                                 .expect("Failed to flush next writer");
                    //                         }
                    //                         for file in &mut stdout_files {
                    //                             file.write_all(buf)
                    //                                 .await
                    //                                 .expect("Failed to write to file");
                    //                             file.flush().await.expect("Failed to flush file");
                    //                         }
                    //                     }
                    //                     Err(_) => break,
                    //                 }
                    //             }
                    //         });
                    //         handles.push(stdout_handle);
                    //     }
                    //
                    //     if let Some(stderr) = stderr {
                    //         let stderr_handle = tokio::spawn(async move {
                    //             let mut reader = stderr;
                    //             let mut buffer = [0u8; 4096];
                    //             loop {
                    //                 match reader.read(&mut buffer).await {
                    //                     Ok(0) => break,
                    //                     Ok(n) => {
                    //                         let buf = &buffer[..n];
                    //                         for file in &mut stderr_files {
                    //                             file.write_all(buf)
                    //                                 .await
                    //                                 .expect("Failed to write to file");
                    //                         }
                    //                     }
                    //                     Err(_) => break,
                    //                 }
                    //             }
                    //         });
                    //         handles.push(stderr_handle);
                    //     }
                    // }
                    //
                    // for process in &mut processes {
                    //     process.wait().await;
                    // }
                    //
                    // for handle in handles {
                    //     handle.await.expect("Failed to join pipe handle");
                    // }
                }
            }
        }
    }
}

async fn init_process(
    state: Arc<Mutex<State>>,
    command: &ExprCommand,
    pipe_stdin: bool,
    pipe_stdout: bool,
    pipe_stderr: bool,
) -> Option<Process> {
    let ExprCommand { exec, args, .. } = command;

    let exec = exec.process();
    let args = args
        .iter()
        .map(|arg| arg.process().to_owned())
        .collect::<Vec<String>>();
    let process_result =
        Process::init(state, exec, args, pipe_stdin, pipe_stdout, pipe_stderr).await;

    if let Err(_) = process_result {
        println!("{}: command not found", exec);
        return None;
    }

    Some(process_result.unwrap())
}

async fn get_redirect_files(redirects: &Vec<ExprRedirect>) -> (Vec<File>, Vec<File>) {
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
    (stdout_files, stderr_files)
}
