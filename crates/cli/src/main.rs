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
    /// 서비스 실행 상태 확인
    Status,
    /// 환경변수 관리
    Env {
        #[command(subcommand)]
        command: EnvCommands,
    },
}

#[derive(Subcommand)]
enum EnvCommands {
    /// .env.schema.toml 기준으로 .env 검증
    Check,
    /// 스키마와 .env 차이 비교
    Diff,
    /// 스키마에서 .env.example 생성
    Generate,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Doctor => commands::doctor::execute(),
        Commands::Status => commands::status::execute(),
        Commands::Env { command } => match command {
            EnvCommands::Check => commands::env::check(),
            EnvCommands::Diff => commands::env::diff(),
            EnvCommands::Generate => commands::env::generate(),
        },
    }
}
