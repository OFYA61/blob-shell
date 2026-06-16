use std::collections::BTreeMap;
use std::fmt::Display;

#[derive(Debug)]
pub struct Jobs {
    pub id_counter: usize,
    pub map: BTreeMap<usize, Job>,
}

impl Jobs {
    pub fn init() -> Self {
        Self {
            id_counter: 1,
            map: BTreeMap::new(),
        }
    }

    pub fn create_job(&mut self, pid: i32, command: String) -> &Job {
        let id = self.id_counter;
        let job = Job {
            id,
            pid,
            command,
            status: JobStatus::Running,
        };
        assert!(self.map.insert(id, job).is_none());
        self.id_counter += 1;

        self.map.get(&id).unwrap()
    }

    pub fn log_jobs(&self) {
        for (_, job) in self.map.iter().rev().skip(2).rev() {
            println!("[{}]  {} {}", job.id, job.status, job.command);
        }
        if let Some((_, job)) = self.map.iter().rev().nth(1) {
            println!("[{}]- {} {}", job.id, job.status, job.command);
        }
        if let Some((_, job)) = self.map.iter().rev().next() {
            println!("[{}]+ {} {}", job.id, job.status, job.command);
        }
    }

    pub fn reap_done_jobs(&mut self, print: bool) {
        let ids_to_remove = self
            .map
            .iter()
            .filter(|(_, job)| job.status == JobStatus::Done)
            .map(|(id, job)| {
                if print {
                    let marker = if let Some(last_id) = self.map.keys().last()
                        && last_id == id
                    {
                        '+'
                    } else if let Some(second_last_id) = self.map.keys().rev().nth(1)
                        && second_last_id == id
                    {
                        '-'
                    } else {
                        ' '
                    };

                    println!("[{}]{} {} {}", job.id, marker, job.status, job.command);
                }
                *id
            })
            .collect::<Vec<usize>>();

        ids_to_remove.iter().for_each(|id| {
            self.map.remove(id);
        });
    }

    pub fn mark_job_done(&mut self, id: usize) {
        self.map
            .iter_mut()
            .find(|(jid, _)| **jid == id)
            .map(|(_, job)| job.mark_done());
    }
}

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
