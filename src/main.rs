use std::io;
use std::io::Write;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read command");

        match command.trim() {
            "exit" => break,
            _ => {}
        };

        println!("{}: command not found", command.trim());
    }
}
