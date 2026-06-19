use std::fmt::Display;

#[derive(Debug)]
pub struct Job {
    pub id: usize,
    pub pid: i32,
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
