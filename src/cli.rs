use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(propagate_version = true)]
#[command(name = "cbt")]
#[command(bin_name = "cbt")]
#[command(help_template = "\
{name} {version}

{about}

{usage-heading}
  {usage}

{all-args}
{author-section}
")]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    // Build
    #[command(bin_name = "build")]
    #[command(author, about = "Builds the project")]
    #[command(help_template = "\
{name} {version}

{about}

{usage-heading}
  {usage}

{all-args}
{author-section}
    ")]
    Build {
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    // Clean
    #[command(bin_name = "clean")]
    #[command(author, about = "Clean the build directory")]
    #[command(help_template = "\
{name} {version}

{about}

{usage-heading}
  {usage}

{all-args}
{author-section}
    ")]
    Clean,

    // Gen config
    #[command(bin_name = "gen-config")]
    #[command(author, about = "Generate a default config file")]
    #[command(help_template = "\
{name} {version}

{about}

{usage-heading}
  {usage}

{all-args}
{author-section}
    ")]
    GenConfig {
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    // Gen completion
    #[command(bin_name = "gen-completions")]
    #[command(author, about = "Generate shell completions")]
    #[command(help_template = "\
{name} {version}

{about}

{usage-heading}
  {usage}

{all-args}
{author-section}
    ")]
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
