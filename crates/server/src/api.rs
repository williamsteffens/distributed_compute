use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    Json, 
    Router,
    extract::{Path, State},
    routing::{get, post},
};

use protocol::{
    Job, JobResult, WorkUnit, WorkerInfo, WorkerRegistration, WorkerStatus, Workload
};

use uuid::Uuid;

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/jobs", post(create_job))
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

async fn create_job(State(state): State<AppState>, Json(job): Json<Job>) -> Json<Job> {
    let work_unit = WorkUnit {
        id: Uuid::new_v4(),
        job_id: job.id,
        workload: Workload::VectorAdd, // TODO: make this dynamic
        payload: job.parameters.clone(),
        assigned_worker: None,
        lease_expires_at: None,
        attempts: 0,
    };

    state.queue.lock().unwrap().push_back(work_unit);

    println!("created job {}", job.id);

    Json(job)
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
        println!("assigned work {} to worker {}", work.id, worker_id);

        state.assigned.lock().unwrap().insert(work.id, work.clone());

        return Json(Some(work));
    }

    Json(None)
}

async fn submit_result(
    State(state): State<AppState>,
    Json(result): Json<JobResult>,
) -> &'static str {
    let work = state.assigned.lock().unwrap().remove(&result.work_unit_id);

    match work {
        Some(work) => {
            println!("work {} completed by {:?}", work.id, work.assigned_worker);

            "accepted"
        }

        None => {
            println!("received result for unknown work {}", result.work_unit_id);

            "unknown work"
        }
    }
}
