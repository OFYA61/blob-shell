use std::io;
use std::io::Write;

use crossterm::ExecutableCommand;
use crossterm::cursor::MoveLeft;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;

use crate::builtin;

pub fn get_input() -> Result<String, io::Error> {
    enable_raw_mode().expect("Failed to enable raw mode");
    io::stdout().execute(MoveLeft(256))?;
    print!("$ ");
    std::io::stdout().flush()?;

    let mut input = String::new();

    loop {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
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
                    io::stdout().flush()?;
                }
                KeyCode::Backspace => {
                    if input.pop().is_some() {
                        io::stdout().execute(MoveLeft(1))?;
                        print!(" ");
                        io::stdout().execute(MoveLeft(1))?;
                    }
                }
                KeyCode::Enter => {
                    println!();
                    break;
                }
                KeyCode::Tab => {
                    if let Some(i) = input.split(" ").last()
                        && i.len() != 0
                    {
                        if let Some(auto_complete) = builtin::try_auto_complete(i) {
                            auto_complete.chars().skip(i.len()).for_each(|c| {
                                input.push(c);
                                print!("{}", c);
                            });
                            input.push(' ');
                            print!(" ");
                            io::stdout().flush()?;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    io::stdout().execute(MoveLeft(256))?;

    disable_raw_mode()?;

    Ok(input)
}
