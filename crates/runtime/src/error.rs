use std::fmt;

#[derive(Debug)]
pub enum ComputeRuntimeError {
    InvalidPayload(String),
    UnsupportedWorkload(String),
    ExecutionFailed(String),
}

impl fmt::Display for ComputeRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPayload(message) => {
                write!(f, "invalid payload: {message}")
            }

            Self::UnsupportedWorkload(workload) => {
                write!(f, "unsupported workload: {workload}")
            }

            Self::ExecutionFailed(message) => {
                write!(f, "execution failed: {message}")
            }
        }
    }
}

impl std::error::Error for ComputeRuntimeError {}