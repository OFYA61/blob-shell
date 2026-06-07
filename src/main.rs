mod env;
mod parser;

use std::io::Write;

use self::env::ChangeDirError;

#[derive(Debug)]
enum Builtin {
    Echo,
    Exit,

    Cd,
    Pwd,

    Type,
}

impl Builtin {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "echo" => Some(Builtin::Echo),
            "exit" => Some(Builtin::Exit),

            "cd" => Some(Builtin::Cd),
            "pwd" => Some(Builtin::Pwd),

            "type" => Some(Builtin::Type),
            _ => None,
        }
    }
}

macro_rules! expect_no_argument {
    ($command:expr, $args:expr) => {
        if !$args.is_empty() {
            println!("{}: expects no argument", $command);
            continue;
        }
    };
}

macro_rules! expect_single_argument {
    ($command:expr, $args:expr) => {
        if $args.len() != 1 {
            println!("{}: expects exactly one argument", $command);
            continue;
        } else {
            $args[0]
        }
    };
}

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

                    if let Some(builtin) = Builtin::from_str(&exec) {
                        match builtin {
                            Builtin::Echo => {
                                println!("{}", args.join(" "));
                            }
                            Builtin::Exit => {
                                expect_no_argument!("exit", args);
                                std::process::exit(0);
                            }

                            Builtin::Cd => {
                                let new_dir = expect_single_argument!("cd", args);
                                match env::change_dir(new_dir) {
                                    Err(err) => match err {
                                        ChangeDirError::DoesNotExist => {
                                            println!("cd: {new_dir}: No such file or directory")
                                        }
                                    },
                                    _ => {}
                                }
                            }
                            Builtin::Pwd => {
                                expect_no_argument!("pwd", args);
                                println!("{}", env::get_current_dir());
                            }

                            Builtin::Type => {
                                let cmd = expect_single_argument!("type", args);
                                if Builtin::from_str(cmd).is_some() {
                                    println!("{} is a shell builtin", cmd);
                                } else if let Some(command) = env::get_command(cmd) {
                                    println!("{} is {}", cmd, command.to_str().unwrap_or(""));
                                } else {
                                    println!("{cmd}: not found");
                                }
                            }
                        };
                        continue;
                    }

                    if let Some(_) = env::get_command(&exec) {
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
