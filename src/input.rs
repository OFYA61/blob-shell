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
use crate::env;

#[inline]
fn ring_bell() -> Result<(), io::Error> {
    print!("\x07");
    io::stdout().flush()?;
    Ok(())
}

enum AutoCompleteStage {
    None,
    FetchedCandidates,
    FilledLongestCommonPrefix,
    CompletedOnlyCandidate,
}

pub fn get_input() -> Result<String, io::Error> {
    enable_raw_mode().expect("Failed to enable raw mode");
    io::stdout().execute(MoveLeft(256))?;
    print!("$ ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    let mut auto_complete_stage = AutoCompleteStage::None;
    let mut auto_complete_candidates: Vec<String> = Vec::new();

    loop {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            if code != KeyCode::Tab {
                auto_complete_stage = AutoCompleteStage::None;
            }

            match code {
                KeyCode::Char(c) => {
                    if c == 'c' && modifiers.contains(KeyModifiers::CONTROL) {
                        input.clear();
                        println!();
                        break;
                    }
                    if c == 'j' && modifiers.contains(KeyModifiers::CONTROL) {
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
                    let input_split = input.split(" ").collect::<Vec<&str>>();
                    if let Some(i) = input_split.last()
                        && !i.is_empty()
                    {
                        match auto_complete_stage {
                            AutoCompleteStage::None
                            | AutoCompleteStage::FilledLongestCommonPrefix => {
                                let mut chars_to_skip_on_auto_complete = i.len();

                                auto_complete_candidates.clear();
                                if input_split.len() == 1 {
                                    auto_complete_candidates
                                        .append(&mut builtin::try_auto_complete(i));
                                    auto_complete_candidates
                                        .append(&mut env::try_auto_complete_program(i));
                                } else {
                                    let dir;
                                    let file_prefix;
                                    if i.contains("/") {
                                        (dir, file_prefix) = i.rsplit_once("/").unwrap();
                                    } else {
                                        dir = ".";
                                        file_prefix = i;
                                    }
                                    auto_complete_candidates
                                        .append(&mut env::try_auto_complete_path(dir, file_prefix));
                                    chars_to_skip_on_auto_complete = file_prefix.len();
                                }
                                auto_complete_candidates.dedup();
                                auto_complete_candidates.sort();

                                if auto_complete_candidates.is_empty() {
                                    ring_bell()?;
                                    auto_complete_stage = AutoCompleteStage::None;
                                    continue;
                                }

                                if auto_complete_candidates.len() == 1 {
                                    auto_complete_candidates
                                        .first()
                                        .unwrap()
                                        .chars()
                                        .skip(chars_to_skip_on_auto_complete)
                                        .for_each(|c| {
                                            input.push(c);
                                            print!("{}", c)
                                        });
                                    input.push(' ');
                                    print!(" ");
                                    io::stdout().flush()?;
                                    auto_complete_stage = AutoCompleteStage::CompletedOnlyCandidate;
                                    continue;
                                }

                                let mut auto_complete_lcp =
                                    auto_complete_candidates.get(0).unwrap().as_str();
                                auto_complete_candidates.iter().for_each(|candidate| {
                                    for (index, char) in auto_complete_lcp.chars().enumerate() {
                                        if let Some(candidate_char) = candidate.chars().nth(index)
                                            && candidate_char != char
                                        {
                                            auto_complete_lcp = &auto_complete_lcp[..index];
                                            break;
                                        }
                                    }
                                });
                                if input == auto_complete_lcp {
                                    ring_bell()?;
                                    auto_complete_stage = AutoCompleteStage::FetchedCandidates;
                                    continue;
                                }

                                let old_input_len = input.len();

                                input.clear();
                                input.push_str(auto_complete_lcp);

                                print!("{}", input.chars().skip(old_input_len).collect::<String>());
                                io::stdout().flush()?;
                                auto_complete_stage = AutoCompleteStage::FilledLongestCommonPrefix;
                            }
                            AutoCompleteStage::FetchedCandidates => {
                                println!();
                                io::stdout().execute(MoveLeft(256))?;

                                auto_complete_candidates.iter().for_each(|candidate| {
                                    print!("{} ", candidate);
                                });
                                println!();
                                io::stdout().execute(MoveLeft(256))?;

                                print!("$ {}", input);
                                io::stdout().flush()?;
                            }
                            AutoCompleteStage::CompletedOnlyCandidate => {}
                        };
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
