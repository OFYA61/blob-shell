use std::collections::BTreeMap;
use std::collections::btree_map;
use std::fmt::Display;

#[derive(Debug)]
pub struct Jobs {
    id_counter: usize,
    map: BTreeMap<usize, Job>,
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

    pub fn create_job(&mut self, pid: u32, command: String) -> &Job {
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
}

#[derive(Debug)]
pub struct Job {
    pub id: usize,
    pub pid: u32,
    pub command: String,
    pub status: JobStatus,
}

#[derive(Debug)]
pub enum JobStatus {
    Running,
}

impl Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Adding the padding to `println!` doesn't work for some reason, so I have to do it here
        match self {
            Self::Running => f.write_str(&format!("{:<24}", "Running")),
        }
    }
}
