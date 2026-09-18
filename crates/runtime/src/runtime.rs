use async_trait::async_trait;
use serde_json::Value;
use protocol::Workload;

use crate::ComputeRuntimeError;

#[async_trait]
pub trait ComputeRuntime: Send + Sync {
    fn new() -> Result<Self, ComputeRuntimeError>
    where
        Self: Sized;

    async fn execute(
        &self,
        workload: Workload,
        payload: Value,
    ) -> Result<Value, ComputeRuntimeError> {
        match workload {
            Workload::VectorAdd => self.vector_add(&payload).await,
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