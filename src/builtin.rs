use crate::env;
use crate::env::ChangeDirError;

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
        }
    };
}

macro_rules! expect_single_argument {
    ($command:expr, $args:expr) => {
        if $args.len() != 1 {
            println!("{}: expects exactly one argument", $command);
            return true;
        } else {
            $args[0]
        }
    };
}

/// Try proecssing a bultin command. If none are found, returns `false`.
/// If it is a bultin command returns `true`, even if wrong argument types get passed
pub fn try_process(exec: &str, args: &Vec<&str>) -> bool {
    let builtin = Builtin::from_str(&exec);
    if builtin.is_none() {
        return false;
    }
    let builtin = builtin.unwrap();
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

    return true;
}
