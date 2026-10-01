use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "server", version, about = "Distributed compute server")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Start {
        #[arg(
            long, 
            default_value_t = 42069u16,
            value_parser = clap::value_parser!(u16).range(1024..=65535),
        )]
        port: u16,
    },

    // Run {
    //     workload: String,

    //     #[arg(long)]
    //     size: usize,

    //     #[arg(
    //         long,
    //         value_enum,
    //         default_value_t = RuntimeKind::Gpu
    //     )]
    //     runtime: RuntimeKind,

    //     #[arg(long)]
    //     local: bool,
    // },

    // Benchmark {
    //     workload: String,

    //     #[arg(long)]
    //     size: usize,

    //     #[arg(long, value_delimiter = ',')]
    //     compare: Vec<RuntimeKind>,

    //     #[arg(long, value_enum)]
    //     runtime: Option<RuntimeKind>,

    //     #[arg(long)]
    //     json: bool,
    // },
}
