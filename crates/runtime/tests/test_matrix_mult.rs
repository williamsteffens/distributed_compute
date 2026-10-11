use runtime::{ComputeRuntime, CpuRuntime, GpuRuntime};
use protocol::Workload;

async fn test_matrix_multiply<R: ComputeRuntime>() {
    let runtime = R::new()
        .expect("failed to initialize runtime");

    let result = runtime
        .execute(
            Workload::MatrixMultiply,
            serde_json::json!({
                "a": [1, 2, 3, 4, 5, 6],
                "b": [7, 8, 9, 10, 11, 12],
                "params": {
                    "m": 2,
                    "k": 3,
                    "n": 2
                }
            }),
        )
        .await
        .expect("matrix_multiply failed");

    assert_eq!(
        result,
        serde_json::json!({
            "values": [58.0, 64.0, 139.0, 154.0]
        })
    );
}

#[tokio::test]
async fn cpu_matrix_multiply_works() {
    test_matrix_multiply::<CpuRuntime>().await;
}

#[tokio::test]
async fn gpu_matrix_multiply_works() {
    test_matrix_multiply::<GpuRuntime>().await;
}