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

        command = command.trim().to_string();

        if command == "exit" {
            break;
        } else if command.starts_with("echo") {
            println!("{}", &command[5..]);
        } else {
            println!("{}: command not found", command);
        }
    }
}
