use async_trait::async_trait;
use protocol::Workload;
use serde_json::Value;

use crate::ComputeRuntimeError;

pub(crate) fn get_payload_value<T: serde::de::DeserializeOwned>(
    payload: &Value,
    key: &str,
) -> Result<T, ComputeRuntimeError> {
    serde_json::from_value(
        payload
            .get(key)
            .cloned()
            .ok_or_else(|| {
                ComputeRuntimeError::InvalidPayload(format!("missing payload.{}", key))
            })?,
    )
    .map_err(|e| ComputeRuntimeError::InvalidPayload(e.to_string()))
}

#[async_trait]
pub trait ComputeRuntime: Send + Sync {
    fn new() -> Result<Self, ComputeRuntimeError>
    where
        Self: Sized;

    async fn execute(
        &self,
        workload: Workload,
        parameters: Value,
    ) -> Result<Value, ComputeRuntimeError> {
        match workload {
            Workload::VectorAdd => self.vector_add(&parameters).await,

            Workload::VectorScale => self.vector_scale(&parameters).await,

            Workload::MatrixMultiply => self.matrix_multiply(&parameters).await,

            _ => Err(ComputeRuntimeError::UnsupportedWorkload(workload)),
        }
    }

    async fn vector_add(&self, parameters: &Value) -> Result<Value, ComputeRuntimeError>;

    async fn vector_scale(&self, parameters: &Value) -> Result<Value, ComputeRuntimeError>;

    async fn matrix_multiply(&self, parameters: &Value) -> Result<Value, ComputeRuntimeError>;
}
