mod builtin;
mod env;
mod parser;

use std::io::Write;

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
                parser::Expr::Command { exec, args } => {
                    let exec = exec.process();
                    let args = args.iter().map(|arg| arg.process()).collect::<Vec<&str>>();

                    if builtin::try_process(exec, &args) {
                        continue;
                    }

                    if let Some(exec) = env::get_command(&exec) {
                        let mut child = std::process::Command::new(&exec);
                        if args.len() > 0 {
                            child.args(args);
                        }
                        child
                            .spawn()
                            .expect("Failed to execute process")
                            .wait()
                            .expect("Failed to wait on command");

                        continue;
                    }

                    println!("{}: command not found", exec);
                }
            }
        }
    }
}
