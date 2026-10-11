use wasm_bindgen_test::*;

use runtime::{ComputeRuntime, CpuRuntime};
use protocol::Workload;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn cpu_matrix_multiply_works() {
    let runtime = CpuRuntime::new().expect("initialize CPU runtime");

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
        .expect("matrix multiplication");

    assert_eq!(
        result,
        serde_json::json!({
            "values": [58.0, 64.0, 139.0, 154.0]
        })
    );
}
