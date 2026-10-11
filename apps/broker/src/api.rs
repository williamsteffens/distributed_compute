use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};

use protocol::{
    Job, JobRequest, JobResult, JobStatus, WorkUnit, WorkerInfo, WorkerRegistration, WorkerStatus,
};

use uuid::Uuid;

use crate::state::AppState;

// communication layer
// module: HTTP RestAPI, websockets, webtrasnport, grpc, webrtc 
// events

// coordination layer
// module: job scheduling, worker management, work unit assignment, result collection

// execution layer
// module: runtime, compute engine, workload execution, resource management

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        // .route("/status", get(get_status))
        .route("/jobs", post(create_job))
        .route("/jobs/{id}", get(get_job))
        .route("/workers/register", post(register_worker))
        .route("/workers/{id}/heartbeat", post(heartbeat))
        .route("/workers/{id}/work", get(get_work))
        .route("/results", post(submit_result))
        .with_state(state)
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

async fn health() -> &'static str {
    "ok"
}

// async fn get_status(State(state): State<AppState>) -> Json<serde_json::Value> {
//     // let jobs = state.jobs.lock().unwrap();
//     // let workers = state.workers.lock().unwrap();
//     // let queue = state.queue.lock().unwrap();

//     // Json(serde_json::json!({
//     //     "jobs": jobs.len(),
//     //     "workers": workers.len(),
//     //     "queue": queue.len(),
//     // }))
// }

async fn create_job(
    State(state): State<AppState>,
    Json(job_request): Json<JobRequest>
) -> Json<Job> {
    let job_id = Uuid::new_v4();

    let job = Job {
        id: job_id,
        workload: job_request.workload,
        parameters: job_request.parameters.clone(),
    };

    // this should not be the job of the API to create work units
    let work_unit = WorkUnit {
        id: Uuid::new_v4(),
        job_id: job_id,
        workload: job_request.workload,
        parameters: job_request.parameters.clone(),

        assigned_worker: None,
        lease_expires_at: None,
        attempts: 0,
    };

    state.queue.lock().unwrap().push_back(work_unit);

    println!("created job {}", job.id);

    Json(job)
}

async fn get_job(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<JobStatus>, StatusCode> {
    let jobs = state.jobs.lock().unwrap();

    match jobs.get(&job_id) {
        Some(status) => Ok(Json(status.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn register_worker(
    State(state): State<AppState>,
    Json(registration): Json<WorkerRegistration>,
) -> Json<WorkerInfo> {
    let worker = WorkerInfo {
        worker_id: registration.worker_id,
        hostname: registration.hostname,
        status: WorkerStatus::Online,
        last_heartbeat: now(),
    };

    state
        .workers
        .lock()
        .unwrap()
        .insert(worker.worker_id, worker.clone());

    println!("worker registered: {}", worker.worker_id);

    Json(worker)
}

async fn heartbeat(State(state): State<AppState>, Path(worker_id): Path<Uuid>) -> &'static str {
    let mut workers = state.workers.lock().unwrap();

    if let Some(worker) = workers.get_mut(&worker_id) {
        worker.last_heartbeat = now();
        worker.status = WorkerStatus::Online;

        "ok"
    } else {
        "unknown worker"
    }
}

async fn get_work(
    State(state): State<AppState>,
    Path(worker_id): Path<Uuid>,
) -> Json<Option<WorkUnit>> {
    let work = {
        let mut queue = state.queue.lock().unwrap();

        queue.pop_front().map(|mut work| {
            work.assigned_worker = Some(worker_id);

            work.attempts += 1;

            work.lease_expires_at = Some(now() + 30);

            work
        })
    };

    if let Some(work) = work {
        println!("assigned work {} to worker {}", work.job_id, worker_id);

        state.assigned.lock().unwrap().insert(work.job_id, work.clone());

        return Json(Some(work));
    }

    Json(None)
}

async fn submit_result(
    State(state): State<AppState>,
    Json(result): Json<JobResult>,
) -> &'static str {
    let work = state.assigned.lock().unwrap().remove(&result.job_id);

    match work {
        Some(work) => {
            println!("work {} completed by {:?}", work.job_id, work.assigned_worker);

            let status = if result.success {
                JobStatus::Completed(result)
            } else {
                JobStatus::Failed(result)
            };

            state
                .jobs
                .lock()
                .unwrap()
                .insert(work.job_id, status);

            "accepted"
        }

        None => {
            println!("received result for unknown work {}", result.job_id);

            "unknown work"
        }
    }
}
