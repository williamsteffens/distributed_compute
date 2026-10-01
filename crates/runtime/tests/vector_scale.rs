use runtime::{ComputeRuntime, CpuRuntime, GpuRuntime};
use protocol::Workload;

async fn test_vector_scale<R: ComputeRuntime>() {
    let runtime = R::new()
        .expect("failed to initialize runtime");

    let result = runtime
        .execute(
            Workload::VectorScale,
            serde_json::json!({
                "values": [1.0, 2.0, 3.0, 4.0],
                "scalar": 10.0,
            }),
        )
        .await
        .expect("vector_scale failed");

    assert_eq!(
        result,
        serde_json::json!({
            "values": [10.0, 20.0, 30.0, 40.0]
        })
    );
}

#[tokio::test]
async fn cpu_vector_scale_works() {
    test_vector_scale::<CpuRuntime>().await;
}

#[tokio::test]
async fn gpu_vector_scale_works() {
    test_vector_scale::<GpuRuntime>().await;
}