use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub workload: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkUnit {
    pub id: Uuid,
    pub job_id: Uuid,
    pub workload: String,
    pub payload: serde_json::Value,

    // Lease information
    pub assigned_worker: Option<Uuid>,
    pub lease_expires_at: Option<u64>,
    pub attempts: u32,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkerInfo {
    pub worker_id: Uuid,
    pub hostname: String,
    pub status: WorkerStatus,
    pub last_heartbeat: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum WorkerStatus {
    Online,
    Offline,
}