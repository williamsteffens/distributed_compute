use protocol::{JobResult, WorkUnit};
use uuid::Uuid;

const SERVER: &str = "http://localhost:3000";

#[tokio::main]
async fn main() {
    println!("worker starting");

    let client = reqwest::Client::new();

    loop {
        let response = client
            .get(format!("{SERVER}/work"))
            .send()
            .await
            .expect("failed to contact server");

        let work: Option<WorkUnit> =
            response.json().await.expect("invalid work response");

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
                println!("no work available");
                tokio::time::sleep(
                    std::time::Duration::from_secs(1)
                ).await;
            }
        }
    }
}