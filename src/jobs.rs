use std::collections::BTreeMap;
use std::collections::btree_map;
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

    pub fn iter(&self) -> btree_map::Iter<'_, usize, Job> {
        self.map.iter()
    }

    pub fn iter_mut(&mut self) -> btree_map::IterMut<'_, usize, Job> {
        self.map.iter_mut()
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

    pub fn cleanup_completed_jobs(&mut self) {
        let ids_to_remove = self
            .map
            .iter()
            .filter(|(_, job)| job.status == JobStatus::Done)
            .map(|(id, _)| *id)
            .collect::<Vec<usize>>();
        ids_to_remove.iter().for_each(|id| {
            self.map.remove(id);
        });
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
