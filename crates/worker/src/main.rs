use protocol::{
    JobResult,
    WorkUnit,
    WorkerRegistration
};
use uuid::Uuid;

const SERVER: &str = "http://localhost:3000";

#[tokio::main]
async fn main() {
    println!("worker starting");

    let client = reqwest::Client::new();

    let worker_id = Uuid::new_v4();

    let registration = WorkerRegistration {
        worker_id,
        hostname: hostname(),
    };

    let heartbeat_client = client.clone();

    tokio::spawn(async move {
        loop {
            let _ = heartbeat_client
                .post(format!("{SERVER}/workers/{worker_id}/heartbeat"))
                .send()
                .await;

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    client
        .post(format!("{SERVER}/workers/register"))
        .json(&registration)
        .send()
        .await
        .expect("failed to register worker");

    println!("worker id: {worker_id}");

    loop {
        let response = client
            .get(format!("{SERVER}/workers/{worker_id}/work"))
            .send()
            .await
            .expect("failed to contact server");

        let work: Option<WorkUnit> = response.json().await.expect("invalid work response");

        match work {
            Some(work) => {
                println!("received work: {}", work.id);

                // Fake computation for now.
                let result = JobResult {
                    work_unit_id: work.id,
                    success: true,
                    result: serde_json::json!({
                        "message": "hello from worker"
                    }),
                };

                client
                    .post(format!("{SERVER}/results"))
                    .json(&result)
                    .send()
                    .await
                    .expect("failed to submit result");
            }

            None => {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
}

fn hostname() -> String {
    std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string())
}
