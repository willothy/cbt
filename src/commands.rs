use std::{fs, path::PathBuf};

use anyhow::Context;
use clap::CommandFactory;
use clap_complete::generate;

use crate::{
    cli::{Cli, Shell},
    compilation::run_stage,
    config::{load_config, Config},
    error,
};

pub fn build(config_path: Option<PathBuf>) -> anyhow::Result<()> {
    let config_path = if let Some(config_path) = config_path {
        config_path
    } else {
        PathBuf::from("cbt.toml")
    };

    let config = load_config(&config_path)?;
    let compilers = &config.compilers;

    for stage in &config.stages {
        run_stage(compilers, stage)?;
    }

    Ok(())
}

pub fn gen_config(path: Option<PathBuf>) -> anyhow::Result<()> {
    let path = if let Some(path) = path {
        path
    } else {
        PathBuf::from("cbt.toml")
    };

    let config = Config::default();
    let serialized = toml::to_string_pretty(&config)
        .with_context(|| error!("Failed to serialize default config"))?;

    std::fs::write(&path, serialized).with_context(|| error!("Failed to write config file"))?;

    Ok(())
}

pub fn gen_completions(shell: Shell) -> anyhow::Result<()> {
    let mut app = Cli::command();
    let app_name = app.clone().get_name().to_owned();

    let out_path = PathBuf::from(format!("{}.{}", app_name, shell.extension()));
    let mut out_file = fs::File::create(out_path)?;

    match shell {
        Shell::Bash => {
            generate(
                clap_complete::shells::Bash,
                &mut app,
                app_name,
                &mut out_file,
            );
        }
        Shell::Fish => {
            generate(
                clap_complete::shells::Bash,
                &mut app,
                app_name,
                &mut out_file,
            );
        }
        Shell::Zsh => {
            generate(
                clap_complete::shells::Bash,
                &mut app,
                app_name,
                &mut out_file,
            );
        }
        Shell::PowerShell => {
            generate(
                clap_complete::shells::Bash,
                &mut app,
                app_name,
                &mut out_file,
            );
        }
        Shell::Elvish => {
            generate(
                clap_complete::shells::Bash,
                &mut app,
                app_name,
                &mut out_file,
            );
        }
    }

    Ok(())
}
