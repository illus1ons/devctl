mod commands;
mod output;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "devctl", about = "로컬 개발 환경 진단 및 실행 CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 개발 환경 진단
    Doctor,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Doctor => commands::doctor::execute(),
    }
}
