mod error;
mod cpu;
mod gpu;
mod runtime;

pub use cpu::CpuRuntime;
pub use gpu::GpuRuntime;
pub use error::ComputeRuntimeError;
pub use runtime::ComputeRuntime;