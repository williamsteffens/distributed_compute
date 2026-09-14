use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub workload: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkUnit {
    pub id: Uuid,
    pub job_id: Uuid,
    pub workload: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobResult {
    pub work_unit_id: Uuid,
    pub success: bool,
    pub result: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkerRegistration {
    pub worker_id: Uuid,
    pub hostname: String,
}