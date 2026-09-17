use runtime::{ComputeRuntime, CpuRuntime, GpuRuntime};

async fn test_vector_add<R: ComputeRuntime>() {
    let runtime = R::new()
        .expect("failed to initialize runtime");

    let result = runtime
        .execute(
            "vector_add",
            serde_json::json!({
                "a": [1.0, 2.0, 3.0, 4.0],
                "b": [10.0, 20.0, 30.0, 40.0],
            }),
        )
        .await
        .expect("vector_add failed");

    assert_eq!(
        result,
        serde_json::json!({
            "values": [11.0, 22.0, 33.0, 44.0]
        })
    );
}

#[tokio::test]
async fn cpu_vector_add_works() {
    test_vector_add::<CpuRuntime>().await;
}

#[tokio::test]
async fn gpu_vector_add_works() {
    test_vector_add::<GpuRuntime>().await;
}