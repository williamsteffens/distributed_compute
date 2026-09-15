use anyhow::Result;
use async_trait::async_trait;
use runtime::ComputeRuntime;
use serde_json::Value;

#[async_trait]
pub trait Workload: Send + Sync {
    fn name(&self) -> &'static str;

    async fn execute(
        &self,
        runtime: &dyn ComputeRuntime,
        payload: Value,
    ) -> Result<Value>;
}

pub struct VectorAddWorkload;

#[async_trait]
impl Workload for VectorAddWorkload {
    fn name(&self) -> &'static str {
        "vector_add"
    }

    async fn execute(
        &self,
        runtime: &dyn ComputeRuntime,
        payload: Value,
    ) -> Result<Value> {
        runtime
            .execute(self.name(), payload)
            .await
    }
}

pub fn get_workload(name: &str) -> Option<Box<dyn Workload>> {
    match name {
        "vector_add" => Some(Box::new(VectorAddWorkload)),
        _ => None,
    }
}