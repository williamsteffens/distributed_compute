use async_trait::async_trait;
use serde_json::Value;

use super::{ComputeRuntime, ComputeRuntimeError};
use crate::runtime::get_payload_value;

pub struct CpuRuntime;

#[async_trait]
impl ComputeRuntime for CpuRuntime {
    fn new() -> Result<Self, ComputeRuntimeError> {
        Ok(Self)
    }

    async fn vector_add(&self, payload: &Value) -> Result<Value, ComputeRuntimeError> {
        let a: Vec<f32> = get_payload_value(&payload, "a")?;
        let b: Vec<f32> = get_payload_value(&payload, "b")?;

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

    async fn vector_scale(&self, payload: &Value) -> Result<Value, ComputeRuntimeError> {
        let values: Vec<f32> = get_payload_value(&payload, "values")?;
        let scalar: f32 = get_payload_value(&payload, "scalar")?;
        let scaled_values = values.into_iter().map(|v| v * scalar).collect::<Vec<_>>();

        Ok(serde_json::json!({
            "values": scaled_values
        }))
    }

    async fn matrix_multiply(&self, payload: &Value) -> Result<Value, ComputeRuntimeError> {
        let a: Vec<f32> = get_payload_value(&payload, "a")?;
        let b: Vec<f32> = get_payload_value(&payload, "b")?;

        let params = payload
            .get("params")
            .ok_or_else(|| ComputeRuntimeError::InvalidPayload("missing payload.params".into()))?;

        let m: usize = get_payload_value(&params, "m")?;
        let k: usize = get_payload_value(&params, "k")?;
        let n: usize = get_payload_value(&params, "n")?;

        if m.checked_mul(k) != Some(a.len()) || k.checked_mul(n) != Some(b.len()) {
            return Err(ComputeRuntimeError::InvalidPayload(
                "matrix dimensions do not match input lengths".into(),
            ));
        }

        let mut values = vec![0.0_f32; m * n];

        for row in 0..m {
            for col in 0..n {
                for i in 0..k {
                    values[row * n + col] += a[row * k + i] * b[i * n + col];
                }
            }
        }

        Ok(serde_json::json!({
            "values": values
        }))
    }
}
