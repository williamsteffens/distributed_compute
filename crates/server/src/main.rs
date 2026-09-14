use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use protocol::{Job, JobResult, WorkUnit};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    queue: Arc<Mutex<VecDeque<WorkUnit>>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState {
        queue: Arc::new(Mutex::new(VecDeque::new())),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/jobs", post(create_job))
        .route("/work", get(get_work))
        .route("/results", post(submit_result))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("server listening on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "ok"
}

async fn create_job(
    State(state): State<AppState>,
    Json(job): Json<Job>,
) -> Json<Job> {
    let work_unit = WorkUnit {
        id: Uuid::new_v4(),
        job_id: job.id,
        workload: job.workload.clone(),
        payload: job.parameters.clone(),
    };

    state.queue.lock().unwrap().push_back(work_unit);

    Json(job)
}

async fn get_work(
    State(state): State<AppState>,
) -> Json<Option<WorkUnit>> {
    let work = state.queue.lock().unwrap().pop_front();

    Json(work)
}

async fn submit_result(
    Json(result): Json<JobResult>,
) -> &'static str {
    println!("result received: {result:?}");

    "accepted"
}