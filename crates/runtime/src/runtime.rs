use async_trait::async_trait;
use serde_json::Value;

use crate::ComputeRuntimeError;

#[async_trait]
pub trait ComputeRuntime: Send + Sync {
    fn new() -> Result<Self, ComputeRuntimeError>
    where
        Self: Sized;

    async fn execute(
        &self,
        workload: &str,
        payload: Value,
    ) -> Result<Value, ComputeRuntimeError> {
        match workload {
            "vector_add" => self.vector_add(&payload).await,
            _ => Err(ComputeRuntimeError::UnsupportedWorkload(
                workload.to_string(),
            )),
        }
    }

    async fn vector_add(
        &self,
        payload: &Value,
    ) -> Result<Value, ComputeRuntimeError>;
}