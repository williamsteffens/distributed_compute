use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};

use protocol::{Job, JobResult, WorkUnit, WorkerInfo, WorkerRegistration, WorkerStatus};

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    queue: Arc<Mutex<VecDeque<WorkUnit>>>,
    assigned: Arc<Mutex<HashMap<Uuid, WorkUnit>>>,
    workers: Arc<Mutex<HashMap<Uuid, WorkerInfo>>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState {
        queue: Arc::new(Mutex::new(VecDeque::new())),
        assigned: Arc::new(Mutex::new(HashMap::new())),
        workers: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/jobs", post(create_job))
        .route("/results", post(submit_result))
        .route("/workers/register", post(register_worker))
        .route("/workers/{id}/work", get(get_work))
        .route("/workers/{id}/heartbeat", post(heartbeat))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("server listening on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "ok"
}

async fn create_job(State(state): State<AppState>, Json(job): Json<Job>) -> Json<Job> {
    let work_unit = WorkUnit {
        id: Uuid::new_v4(),
        job_id: job.id,
        workload: job.workload.clone(),
        payload: job.parameters.clone(),
        assigned_worker: None,
        lease_expires_at: None,
        attempts: 0,
    };

    state.queue.lock().unwrap().push_back(work_unit);

    Json(job)
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
        println!(
            "assigned work {} to worker {}",
            work.id, worker_id
        );

        state
            .assigned
            .lock()
            .unwrap()
            .insert(work.id, work.clone());

        return Json(Some(work));
    }

    Json(None)
}

async fn submit_result(
    State(state): State<AppState>,
    Json(result): Json<JobResult>,
) -> &'static str {
    let work = state
        .assigned
        .lock()
        .unwrap()
        .remove(&result.work_unit_id);

    match work {
        Some(work) => {
            println!(
                "work {} completed by {:?}",
                work.id,
                work.assigned_worker
            );

            "accepted"
        }

        None => {
            println!(
                "received result for unknown work {}",
                result.work_unit_id
            );

            "unknown work"
        }
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

async fn register_worker(
    State(state): State<AppState>,
    Json(registration): Json<WorkerRegistration>,
) -> Json<WorkerInfo> {
    let now = now();

    let worker = WorkerInfo {
        worker_id: registration.worker_id,
        hostname: registration.hostname,
        status: WorkerStatus::Online,
        last_heartbeat: now,
    };

    state
        .workers
        .lock()
        .unwrap()
        .insert(worker.worker_id, worker.clone());

    println!("worker registered: {}", worker.worker_id);

    Json(worker)
}

async fn heartbeat(
    State(state): State<AppState>,
    Path(worker_id): Path<Uuid>,
) -> &'static str {
    let mut workers = state.workers.lock().unwrap();

    if let Some(worker) = workers.get_mut(&worker_id) {
        worker.last_heartbeat = now();
        worker.status = WorkerStatus::Online;

        "ok"
    } else {
        "unknown worker"
    }
}