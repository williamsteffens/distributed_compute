mod api;
mod benchmark;
mod cli;
mod state;

use anyhow::Result;
use benchmark::benchmark_vector_add;
use clap::Parser;
use cli::{Cli, Command};
use state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Start { port } => {
            start_server(port).await?;
        }

        Command::Run {
            workload,
            size,
            runtime,
            local,
        } => {
            run_command(&workload, size, runtime, local)?;
        }

        Command::Benchmark {
            workload,
            size,
            compare,
            runtime,
            json,
        } => {
            benchmark_command(&workload, size, compare, runtime, json)?;
        }
    }

    Ok(())
}

async fn start_server(port: u16) -> Result<()> {
    let state = AppState::new();

    let app = api::router(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("server listening on http://{addr}");

    axum::serve(listener, app).await?;

    Ok(())
}

fn run_command(workload: &str, size: usize, runtime: cli::RuntimeKind, local: bool) -> Result<()> {
    if !local {
        anyhow::bail!("distributed run is not implemented yet; use --local");
    }

    if workload != "vector-add" && workload != "vector_add" {
        anyhow::bail!("unsupported workload: {workload}");
    }

    let runtime_kind = match runtime {
        cli::RuntimeKind::Cpu => benchmark::RuntimeKind::Cpu,
        cli::RuntimeKind::Gpu => benchmark::RuntimeKind::Gpu,
    };

    let result = benchmark_vector_add(size, runtime_kind)?;

    println!("workload: vector_add");
    println!("size: {size}");
    println!("runtime: {}", runtime_kind.name());
    println!("execution: {:.3} ms", result.elapsed_ms);
    println!("validation: {}", if result.valid { "OK" } else { "FAILED" });

    Ok(())
}

fn benchmark_command(
    workload: &str,
    size: usize,
    compare: Vec<cli::RuntimeKind>,
    runtime: Option<cli::RuntimeKind>,
    json: bool,
) -> Result<()> {
    if workload != "vector-add" && workload != "vector_add" {
        anyhow::bail!("unsupported benchmark workload: {workload}");
    }

    let runtimes = if !compare.is_empty() {
        compare
    } else if let Some(runtime) = runtime {
        vec![runtime]
    } else {
        vec![cli::RuntimeKind::Cpu, cli::RuntimeKind::Gpu]
    };

    if json {
        return benchmark_json(size, &runtimes);
    }

    println!("Benchmark: vector_add");
    println!("Size:      {size}");
    println!();

    let mut results = Vec::new();

    for runtime_kind in runtimes {
        let kind = match runtime_kind {
            cli::RuntimeKind::Cpu => benchmark::RuntimeKind::Cpu,
            cli::RuntimeKind::Gpu => benchmark::RuntimeKind::Gpu,
        };

        let result = benchmark_vector_add(size, kind)?;

        println!("{}", runtime_kind.name());
        println!("  execution: {:>8.3} ms", result.elapsed_ms);
        println!(
            "  validation: {}",
            if result.valid { "OK" } else { "FAILED" }
        );
        println!();

        if !result.valid {
            anyhow::bail!("{} benchmark produced invalid results", runtime_kind.name());
        }

        results.push(result);
    }

    let cpu = results
        .iter()
        .find(|result| matches!(result.runtime, benchmark::RuntimeKind::Cpu));

    let gpu = results
        .iter()
        .find(|result| matches!(result.runtime, benchmark::RuntimeKind::Gpu));

    if let (Some(cpu), Some(gpu)) = (cpu, gpu) {
        println!("Speedup");

        println!("  GPU: {:.2}x", cpu.elapsed_ms / gpu.elapsed_ms);
    }

    Ok(())
}

fn benchmark_json(size: usize, runtimes: &[cli::RuntimeKind]) -> Result<()> {
    let mut results = Vec::new();

    for runtime_kind in runtimes {
        let kind = match runtime_kind {
            cli::RuntimeKind::Cpu => benchmark::RuntimeKind::Cpu,
            cli::RuntimeKind::Gpu => benchmark::RuntimeKind::Gpu,
        };

        let result = benchmark_vector_add(size, kind)?;

        results.push(serde_json::json!({
            "runtime": runtime_kind.name(),
            "elapsed_ms": result.elapsed_ms,
            "valid": result.valid,
        }));
    }

    let output = serde_json::json!({
        "workload": "vector_add",
        "size": size,
        "results": results,
    });

    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}
