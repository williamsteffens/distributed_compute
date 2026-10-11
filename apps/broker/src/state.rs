use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use protocol::{WorkUnit, WorkerInfo, JobStatus};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub queue: Arc<Mutex<VecDeque<WorkUnit>>>,
    pub assigned: Arc<Mutex<HashMap<Uuid, WorkUnit>>>,
    pub workers: Arc<Mutex<HashMap<Uuid, WorkerInfo>>>,
    pub jobs: Arc<Mutex<HashMap<Uuid, JobStatus>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            assigned: Arc::new(Mutex::new(HashMap::new())),
            workers: Arc::new(Mutex::new(HashMap::new())),
            jobs: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
