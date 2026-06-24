use std::collections::BTreeMap;
use std::collections::HashMap;
use std::env;
use std::fmt::Display;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use tokio::fs::File;
use tokio::fs::OpenOptions;
use tokio::io;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;

use crate::autocomplete::Candidate;
use crate::completer::Completer;

#[derive(Debug)]
pub enum ChangeDirError {
    DoesNotExist,
}

#[derive(Debug)]
pub struct State {
    home: String,
    #[allow(dead_code)]
    paths: Vec<PathBuf>,
    programs: HashMap<String, PathBuf>,
    completers: HashMap<String, Completer>,

    jobs: BTreeMap<usize, Job>,
    pub history: Vec<String>,
}

impl State {
    pub fn init() -> Self {
        let home = env::var("HOME").expect("Failed to get home environment variable");
        let path_var = env::var("PATH");
        if path_var.is_err() {
            return Self {
                home,
                paths: vec![],
                programs: HashMap::new(),
                completers: HashMap::new(),
                jobs: BTreeMap::new(),
                history: vec![String::from("")],
            };
        }
        let path_var = unsafe { path_var.unwrap_unchecked() };
        let paths = env::split_paths(&path_var).filter(|p| p.is_dir()).collect();

        let mut programs = HashMap::new();
        for path in &paths {
            if let Ok(entries) = fs::read_dir(&path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file()
                        && let Ok(metadata) = fs::metadata(&p)
                        && metadata.permissions().mode() & 0o111 != 0
                        && let Some(name) = p.file_name().and_then(|name| name.to_str())
                        && programs.get(name).is_none()
                    {
                        programs.insert(name.to_string(), p);
                    }
                }
            }
        }

        Self {
            home,
            paths,
            programs,
            completers: HashMap::new(),
            jobs: BTreeMap::new(),
            history: vec![String::from("")],
        }
    }

    pub fn reinit(&mut self) {
        *self = Self::init();
    }

    pub fn get_command(&self, command: &str) -> Option<PathBuf> {
        self.programs.get(command).map(|path| path.clone())
    }

    fn get_current_dir(&self) -> PathBuf {
        env::current_dir().expect("Failed to get current directory")
    }

    pub fn get_current_dir_as_string(&self) -> String {
        self.get_current_dir()
            .to_str()
            .expect("Failed to parse to string")
            .to_owned()
    }

    pub fn change_dir(&mut self, new_dir: &str) -> Result<(), ChangeDirError> {
        let dir: String;
        if new_dir.starts_with("~") {
            dir = new_dir.replace("~", &self.home);
        } else {
            dir = new_dir.to_owned();
        }

        env::set_current_dir(&dir).map_err(|_| ChangeDirError::DoesNotExist)
    }

    pub fn get_auto_complete_program_candidates(&self, prefix: &str) -> Vec<Candidate> {
        self.programs
            .keys()
            .filter(|name| name.starts_with(prefix))
            .cloned()
            .map(|s| Candidate::Program(s))
            .collect()
    }

    pub fn get_auto_complete_dir_candidates(&self, subdir: &str, prefix: &str) -> Vec<Candidate> {
        let mut candidates = Vec::new();
        let dir = self.get_current_dir().join(subdir);

        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(name) = p.file_name()
                    && let Some(name) = name.to_str()
                    && name.starts_with(prefix)
                {
                    if p.is_dir() {
                        candidates.push(Candidate::Directory(name.to_owned()));
                    } else {
                        candidates.push(Candidate::File(name.to_owned()));
                    }
                }
            }
        }

        candidates
    }

    pub fn add_completer(&mut self, program: String, path: PathBuf) {
        self.completers.insert(program, Completer::new(path));
    }

    pub fn get_completer(&self, program: &str) -> Option<Completer> {
        self.completers.get(program).cloned()
    }

    pub fn remove_completer(&mut self, program: &str) {
        let _ = self.completers.remove(program);
    }

    pub fn create_job(&mut self, pid: u32, command: String) -> usize {
        let mut available_id = 1;
        for id in self.jobs.keys() {
            if *id == available_id {
                available_id = *id;
                continue;
            }
            available_id = available_id + 1;
            break;
        }

        if let Some(biggest_id) = self.jobs.keys().last()
            && available_id == *biggest_id
        {
            available_id += 1;
        }

        let job = Job {
            id: available_id,
            pid,
            command,
            status: JobStatus::Running,
        };
        println!("[{}] {}", job.id, job.pid);
        assert!(self.jobs.insert(available_id, job).is_none());

        available_id
    }

    pub async fn log_jobs<W: AsyncWriteExt + Unpin>(&mut self, mut stdout: W) {
        for (_, job) in self.jobs.iter().rev().skip(2).rev() {
            let _ = stdout
                .write_all(format!("[{}]   {} {}\n", job.id, job.status, job.command).as_bytes())
                .await;
        }
        if let Some((_, job)) = self.jobs.iter().rev().nth(1) {
            let _ = stdout
                .write_all(format!("[{}]-  {} {}\n", job.id, job.status, job.command).as_bytes())
                .await;
        }
        if let Some((_, job)) = self.jobs.iter().rev().next() {
            let _ = stdout
                .write_all(format!("[{}]+  {} {}\n", job.id, job.status, job.command).as_bytes())
                .await;
        }
        self.reap_done_jobs(stdout, false).await;
    }

    pub async fn reap_done_jobs<W: AsyncWriteExt + Unpin>(&mut self, mut stdout: W, print: bool) {
        let ids_to_remove = self
            .jobs
            .iter()
            .filter(|(_, job)| job.status == JobStatus::Done)
            .map(|(id, _)| *id)
            .collect::<Vec<usize>>();

        let last_id = self.jobs.keys().last().copied();
        let second_last_id = self.jobs.keys().rev().nth(1).copied();
        for id in ids_to_remove {
            let job = self.jobs.remove(&id).unwrap();
            if print {
                let marker = if let Some(last_id) = last_id
                    && last_id == id
                {
                    '+'
                } else if let Some(second_last_id) = second_last_id
                    && second_last_id == id
                {
                    '-'
                } else {
                    ' '
                };

                let _ = stdout
                    .write_all(
                        format!("[{}]{}  {} {}\n", job.id, marker, job.status, job.command)
                            .as_bytes(),
                    )
                    .await;
            }
        }
        let _ = stdout.flush().await;
    }

    pub fn mark_job_done(&mut self, id: usize) {
        self.jobs
            .iter_mut()
            .find(|(jid, _)| **jid == id)
            .map(|(_, job)| job.mark_done());
    }

    pub fn add_history(&mut self, command: String) {
        self.history.push(command);
    }

    pub async fn print_history<W: AsyncWriteExt + Unpin>(
        &self,
        mut stdout: W,
        tail: Option<usize>,
    ) {
        let skip;
        if let Some(tail) = tail {
            skip = self.history.len() - tail;
        } else {
            skip = 1;
        }

        for i in skip..self.history.len() {
            let _ = stdout
                .write_all(format!("    {}  {}\n", i, self.history[i]).as_bytes())
                .await;
        }
        let _ = stdout.flush().await;
    }

    pub async fn append_history_file(&mut self, file: &str) -> io::Result<()> {
        let file = File::open(file).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await? {
            self.history.push(line);
        }
        Ok(())
    }

    pub async fn write_history_file(&mut self, file: &str) -> io::Result<()> {
        let mut file = File::create(file).await?;
        for history in self.history.iter().skip(1) {
            file.write_all(format!("{}\n", history).as_bytes()).await?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Job {
    pub id: usize,
    pub pid: u32,
    pub command: String,
    pub status: JobStatus,
}

impl Job {
    pub fn mark_done(&mut self) {
        self.status = JobStatus::Done;
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum JobStatus {
    Running,
    Done,
}

impl Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Adding the padding to `println!` doesn't work for some reason, so I have to do it here
        match self {
            Self::Running => f.write_str(&format!("{:<24}", "Running")),
            Self::Done => f.write_str(&format!("{:<24}", "Done")),
        }
    }
}
