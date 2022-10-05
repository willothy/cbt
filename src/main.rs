use std::process::ExitCode;

use clap::Parser;

mod cli;
mod commands;
mod compilation;
mod config;
mod files;
mod logging;
mod util;

pub fn run() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    match args.subcommand {
        cli::Commands::Build { config } => commands::build(config),
        cli::Commands::Clean => todo!(),
        cli::Commands::GenConfig { path } => commands::gen_config(path),
        cli::Commands::GenCompletions { shell } => commands::gen_completions(shell),
    }
}

fn main() -> ExitCode {
    let result: anyhow::Result<()> = run();

    match result {
        Ok(_) => {
            println!("");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{:?}", e);
            ExitCode::FAILURE
        }
    }
}
