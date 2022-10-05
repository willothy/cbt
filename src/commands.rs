use std::{
    env::{self, set_current_dir},
    fs,
    path::PathBuf,
};

use anyhow::Context;
use clap::CommandFactory;
use clap_complete::generate;

use crate::{
    cli::{Cli, Shell},
    compilation::run_stage,
    config::{load_config, Config},
    error, warning,
};

pub fn build(config_path: Option<PathBuf>) -> anyhow::Result<()> {
    let config_path = if let Some(config_path) = config_path {
        config_path
    } else {
        PathBuf::from("cbt.toml")
    };
    let config_path = config_path.canonicalize().with_context(|| {
        error!(
            "Could not canonicalize config directory {}",
            config_path.display()
        )
    })?;
    let config = load_config(&config_path)?;
    let compilers = &config.compilers;
    let config_dir = match config_path.parent() {
        Some(dir) => dir.to_path_buf(),
        None => {
            println!(
                "{}: Could not get parent directory of config file, defaulting to current directory.",
                warning!("Warning")
            );
            env::current_dir().with_context(|| "Could not get current directory")?
        }
    };

    set_current_dir(config_dir).with_context(|| error!("{}", "Could not set current directory"))?;

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

    let config = Config::default(); /* Config {
                                        compilers: Default::default(),
                                        stages: vec![Stage {
                                            name: "default".to_string(),
                                            source: Default::default(),
                                            build: Default::default(),
                                            exclude: Default::default(),
                                            flags: Default::default(),
                                            includes: Default::default(),
                                        }],
                                    }; */
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
