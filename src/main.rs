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

#[derive(Debug)]
struct Path {
    paths: Vec<std::path::PathBuf>,
}

impl Path {
    fn init() -> Self {
        let path_var = std::env::var("PATH");
        if path_var.is_err() {
            return Self { paths: vec![] };
        }
        let path_var = unsafe { path_var.unwrap_unchecked() };

        Self {
            paths: std::env::split_paths(&path_var)
                .filter(|p| p.is_dir())
                .collect(),
        }
    }

    fn get_command(&self, command: &str) -> Option<std::path::PathBuf> {
        for path in &self.paths {
            let full_path = path.join(command);
            if full_path.is_file() {
                return Some(full_path);
            }
        }
        None
    }
}

fn main() {
    let path = Path::init();

    loop {
        print!("$ ");
        std::io::stdout().flush().unwrap();

        let mut command = String::new();
        std::io::stdin()
            .read_line(&mut command)
            .expect("Failed to read command");

        command = command.trim().to_string();

        let (command, args) = command.split_once(' ').unwrap_or((&command, ""));

        if let Some(builtin) = Builtin::from_str(command) {
            match builtin {
                Builtin::Echo => println!("{args}"),
                Builtin::Exit => break,
                Builtin::Type => {
                    if Builtin::from_str(args).is_some() {
                        println!("{} is a shell builtin", args);
                    } else if let Some(command) = path.get_command(args) {
                        println!("{} is {}", args, command.to_str().unwrap_or(""));
                    } else {
                        println!("{args}: not found");
                    }
                }
            };
            continue;
        }

        if let Some(command) = path.get_command(command) {
            println!("Executing: {:?}", command);
            continue;
        }

        println!("{}: command not found", command);
    }
}
