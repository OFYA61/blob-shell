use std::io;
use std::io::Write;

#[derive(Debug)]
enum Builtin {
    Echo,
    Exit,
    Type,
}

impl Builtin {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "echo" => Some(Builtin::Echo),
            "exit" => Some(Builtin::Exit),
            "type" => Some(Builtin::Type),
            _ => None,
        }
    }
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read command");

        command = command.trim().to_string();

        let (command, args) = command.split_once(' ').unwrap_or((&command, ""));

        let builtin = Builtin::from_str(command);
        if builtin.is_none() {
            println!("{}: command not found", command);
            continue;
        }
        let builtin = unsafe { builtin.unwrap_unchecked() };

        match builtin {
            Builtin::Echo => println!("{args}"),
            Builtin::Exit => break,
            Builtin::Type => {
                if Builtin::from_str(args).is_some() {
                    println!("{} is a shell builtin", args);
                } else {
                    println!("{args}: not found");
                }
            }
        };
    }
}
