use anyhow::{Result, anyhow};
use runtime::GpuRuntime;
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
pub enum RuntimeKind {
    Cpu,
    Gpu,
}

impl RuntimeKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::Gpu => "GPU",
        }
    }
}

pub struct BenchmarkResult {
    pub runtime: RuntimeKind,
    pub elapsed_ms: f64,
    pub valid: bool,
}

fn cpu_vector_add(a: &[f32], b: &[f32]) -> Vec<f32> {
    a.iter().zip(b.iter()).map(|(a, b)| a + b).collect()
}

fn validate(result: &[f32], expected: &[f32]) -> bool {
    if result.len() != expected.len() {
        return false;
    }

    result
        .iter()
        .zip(expected.iter())
        .all(|(a, b)| (a - b).abs() < 1e-5)
}

pub fn benchmark_vector_add(size: usize, runtime: RuntimeKind) -> Result<BenchmarkResult> {
    if size == 0 {
        return Err(anyhow!("size must be greater than zero"));
    }

    let a: Vec<f32> = (0..size).map(|i| i as f32).collect();

    let b: Vec<f32> = (0..size).map(|i| (i as f32) * 2.0).collect();

    let expected: Vec<f32> = a.iter().zip(b.iter()).map(|(a, b)| a + b).collect();

    match runtime {
        RuntimeKind::Cpu => {
            let start = Instant::now();

            let result = cpu_vector_add(&a, &b);

            let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

            Ok(BenchmarkResult {
                runtime,
                elapsed_ms,
                valid: validate(&result, &expected),
            })
        }

        RuntimeKind::Gpu => {
            // GPU initialization is intentionally
            // outside the timed section.
            let gpu = GpuRuntime::new()?;

            let start = Instant::now();

            let result = gpu.vector_add(&a, &b)?;

            let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

            Ok(BenchmarkResult {
                runtime,
                elapsed_ms,
                valid: validate(&result, &expected),
            })
        }
    }
}
