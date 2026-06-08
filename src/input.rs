use std::io;
use std::io::Write;

use crossterm::ExecutableCommand;
use crossterm::cursor::MoveLeft;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;

pub fn get_input() -> String {
    crossterm::terminal::enable_raw_mode().expect("Failed to enable raw mode");
    io::stdout()
        .execute(MoveLeft(256))
        .expect("Failed to bring cursor to start of line");
    print!("$ ");
    std::io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();

    loop {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read().expect("Failed to read input")
        {
            match code {
                KeyCode::Char(c) => {
                    if c == 'c' && modifiers.contains(KeyModifiers::CONTROL) {
                        input.clear();
                        println!();
                        break;
                    }
                    input.push(c);
                    print!("{}", c);
                    io::stdout().flush().expect("Failed to flush stdout");
                }
                KeyCode::Backspace => {
                    if input.pop().is_some() {
                        io::stdout()
                            .execute(MoveLeft(1))
                            .expect("Failed to execute stdout");
                        print!(" ");
                        io::stdout()
                            .execute(MoveLeft(1))
                            .expect("Failed to execute stdout");
                    }
                }
                KeyCode::Enter => {
                    println!();
                    break;
                }
                KeyCode::Tab => {
                    // TODO: builtint autocomplete
                }
                _ => {}
            }
        }
    }

    io::stdout()
        .execute(MoveLeft(256))
        .expect("Failed to bring cursor to start of line");

    crossterm::terminal::disable_raw_mode().expect("Failed to disable raw mode");

    input
}
