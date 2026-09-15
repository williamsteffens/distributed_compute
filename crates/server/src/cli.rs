use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "gpu-server", version, about = "Distributed GPU compute server")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Start {
        #[arg(long, default_value = "127.0.0.1:3000")]
        bind: String,
    },

    Run {
        workload: String,

        #[arg(long)]
        size: usize,

        #[arg(
            long,
            value_enum,
            default_value_t = RuntimeKind::Gpu
        )]
        runtime: RuntimeKind,

        #[arg(long)]
        local: bool,
    },

    Benchmark {
        workload: String,

        #[arg(long)]
        size: usize,

        #[arg(long, value_delimiter = ',')]
        compare: Vec<RuntimeKind>,

        #[arg(long, value_enum)]
        runtime: Option<RuntimeKind>,

        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
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
