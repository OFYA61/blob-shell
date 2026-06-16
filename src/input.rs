use std::io;
use std::io::Write;

use crossterm::ExecutableCommand;
use crossterm::cursor::MoveLeft;
use crossterm::cursor::MoveToColumn;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use tokio::sync::MutexGuard;

use crate::autocomplete::Candidate;
use crate::builtin;
use crate::env::Env;

#[inline]
fn ring_bell() -> Result<(), io::Error> {
    print!("\x07");
    io::stdout().flush()?;
    Ok(())
}

enum AutoCompleteAction {
    None,
    FetchCandidates,
    DislpayCandidates,
}

struct Candidates {
    list: Vec<Candidate>,
}

impl Candidates {
    fn init() -> Self {
        Candidates { list: vec![] }
    }

    fn append(&mut self, other_list: &mut Vec<Candidate>) {
        self.list.append(other_list);
    }

    fn sort(&mut self) {
        self.list.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    }

    fn dedup(&mut self) {
        self.list.dedup_by(|a, b| {
            let a_str = a.as_str();
            let b_str = b.as_str();

            if a_str != b_str {
                return false;
            }

            !matches!(a, Candidate::Directory(_)) && !matches!(b, Candidate::Directory(_))
        });
    }

    fn clear(&mut self) {
        self.list.clear();
    }
}

pub fn get_input(env: MutexGuard<'_, Env>) -> Result<String, io::Error> {
    enable_raw_mode().expect("Failed to enable raw mode");
    io::stdout().execute(MoveToColumn(0))?;
    print!("$ ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    let mut next_auto_complete_action = AutoCompleteAction::FetchCandidates;
    let mut auto_complete_candidates = Candidates::init();

    loop {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            if code != KeyCode::Tab {
                next_auto_complete_action = AutoCompleteAction::FetchCandidates;
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
                    if let Some(i) = input_split.last().map(|i| *i)
                        && let Some(program) = input_split.first().map(|i| *i)
                        && (!i.is_empty() || input_split.len() > 1)
                    {
                        match next_auto_complete_action {
                            AutoCompleteAction::None => {}
                            AutoCompleteAction::FetchCandidates => {
                                let mut chars_to_skip_on_auto_complete = i.len();

                                auto_complete_candidates.clear();
                                if input_split.len() == 1 {
                                    auto_complete_candidates
                                        .append(&mut builtin::get_auto_complete_candidates(i));
                                    auto_complete_candidates
                                        .append(&mut env.get_auto_complete_program_candidates(i));
                                } else {
                                    let mut ran_completer = false;
                                    if let Some(completer) = env.get_completer(program) {
                                        let completer_args = vec![
                                            program,
                                            i,
                                            input_split.iter().nth_back(1).unwrap(),
                                        ];
                                        if let Ok(completer_output) =
                                            completer.run(completer_args, &input)
                                        {
                                            auto_complete_candidates.append(
                                                &mut completer_output
                                                    .split("\n")
                                                    .filter(|s| !s.is_empty())
                                                    .map(|s| Candidate::ProgramArg(s.to_owned()))
                                                    .collect(),
                                            );
                                            ran_completer = true;
                                        }
                                    }

                                    if !ran_completer {
                                        let dir;
                                        let prefix;
                                        if i.contains("/") {
                                            (dir, prefix) = i.rsplit_once("/").unwrap();
                                        } else if i.len() == 0 {
                                            dir = ".";
                                            prefix = "";
                                        } else {
                                            dir = ".";
                                            prefix = i;
                                        }
                                        auto_complete_candidates.append(
                                            &mut env.get_auto_complete_dir_candidates(dir, prefix),
                                        );
                                        chars_to_skip_on_auto_complete = prefix.len();
                                    }
                                }
                                auto_complete_candidates.sort();
                                auto_complete_candidates.dedup();

                                if auto_complete_candidates.list.is_empty() {
                                    ring_bell()?;
                                    next_auto_complete_action = AutoCompleteAction::FetchCandidates;
                                    continue;
                                }

                                if auto_complete_candidates.list.len() == 1 {
                                    let candidate = auto_complete_candidates.list.first().unwrap();
                                    candidate
                                        .chars()
                                        .skip(chars_to_skip_on_auto_complete)
                                        .for_each(|c| {
                                            input.push(c);
                                            print!("{}", c)
                                        });
                                    input.push(candidate.get_trailing_char());
                                    print!("{}", candidate.get_trailing_char());
                                    io::stdout().flush()?;
                                    next_auto_complete_action = if candidate.is_directory() {
                                        AutoCompleteAction::FetchCandidates
                                    } else {
                                        AutoCompleteAction::None
                                    };
                                    continue;
                                }

                                let mut auto_complete_lcp =
                                    auto_complete_candidates.list.get(0).unwrap().as_str();
                                auto_complete_candidates.list.iter().for_each(|candidate| {
                                    for (index, char) in auto_complete_lcp.chars().enumerate() {
                                        if let Some(candidate_char) = candidate.chars().nth(index)
                                            && candidate_char != char
                                        {
                                            auto_complete_lcp = &auto_complete_lcp[..index];
                                            break;
                                        }
                                    }
                                });
                                if i == auto_complete_lcp {
                                    ring_bell()?;
                                    next_auto_complete_action =
                                        AutoCompleteAction::DislpayCandidates;
                                    continue;
                                }

                                let lcp_to_add =
                                    auto_complete_lcp.chars().skip(i.len()).collect::<String>();

                                input.push_str(&lcp_to_add);
                                print!("{}", lcp_to_add);
                                io::stdout().flush()?;
                                next_auto_complete_action = AutoCompleteAction::DislpayCandidates;
                            }
                            AutoCompleteAction::DislpayCandidates => {
                                println!();
                                io::stdout().execute(MoveToColumn(0))?;

                                auto_complete_candidates.list.iter().for_each(|candidate| {
                                    print!(
                                        "{}{}",
                                        candidate.as_str(),
                                        candidate.get_trailing_char()
                                    );
                                    if candidate.is_directory() {
                                        print!(" ");
                                    }
                                });
                                println!();
                                io::stdout().execute(MoveToColumn(0))?;

                                print!("$ {}", input);
                                io::stdout().flush()?;
                            }
                        };
                    }
                }
                _ => {}
            }
        }
    }

    io::stdout().execute(MoveToColumn(0))?;

    disable_raw_mode()?;

    Ok(input)
}
