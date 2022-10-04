use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(name = "cbt")]
#[command(bin_name = "cbt")]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(bin_name = "build")]
    Build {
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    #[command(bin_name = "clean")]
    Clean,
    #[command(bin_name = "gen-config")]
    GenConfig {
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    #[command(bin_name = "gen-completions")]
    GenCompletions { shell: Shell },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Shell {
    Bash,
    Fish,
    Zsh,
    PowerShell,
    Elvish,
}

impl Shell {
    pub fn extension(&self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Fish => "fish",
            Shell::Zsh => "zsh",
            Shell::PowerShell => "ps1",
            Shell::Elvish => "elv",
        }
    }
}
