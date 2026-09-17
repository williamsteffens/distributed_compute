use async_trait::async_trait;
use serde_json::Value;

use super::{ComputeRuntime, ComputeRuntimeError};

pub struct CpuRuntime;

#[async_trait]
impl ComputeRuntime for CpuRuntime {
    fn new() -> Result<Self, ComputeRuntimeError> {
        Ok(Self)
    }

    async fn vector_add(
        &self,
        payload: &Value,
    ) -> Result<Value, ComputeRuntimeError> {
        let a: Vec<f32> = serde_json::from_value(
            payload
                .get("a")
                .cloned()
                .ok_or_else(|| ComputeRuntimeError::InvalidPayload("missing payload.a".into()))?,
        )
        .map_err(|e| ComputeRuntimeError::InvalidPayload(e.to_string()))?;

        let b: Vec<f32> = serde_json::from_value(
            payload
                .get("b")
                .cloned()
                .ok_or_else(|| ComputeRuntimeError::InvalidPayload("missing payload.b".into()))?,
        )
        .map_err(|e| ComputeRuntimeError::InvalidPayload(e.to_string()))?;

        if a.len() != b.len() {
            return Err(ComputeRuntimeError::InvalidPayload(
                "vectors must have equal length".into(),
            ));
        }

        let values = a.into_iter().zip(b).map(|(a, b)| a + b).collect::<Vec<_>>();

        Ok(serde_json::json!({
            "values": values
        }))
    }
}
