use anyhow::Result;
use runtime::GpuRuntime;
use serde_json::json;
use workloads::get_workload;

#[tokio::test]
async fn vector_add_workload_works() -> Result<()> {
    let runtime = GpuRuntime::new()?;

    let workload = get_workload("vector_add")
        .expect("vector_add workload not found");

    let payload = json!({
        "a": [1.0, 2.0, 3.0, 4.0],
        "b": [10.0, 20.0, 30.0, 40.0]
    });

    let result = workload
        .execute(&runtime, payload)
        .await?;

    let values = result["result"]
        .as_array()
        .expect("result should be an array");

    assert_eq!(values.len(), 4);
    assert_eq!(values[0], 11.0);
    assert_eq!(values[1], 22.0);
    assert_eq!(values[2], 33.0);
    assert_eq!(values[3], 44.0);

    Ok(())
}