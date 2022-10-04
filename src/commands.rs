use std::{fs, path::PathBuf};

use anyhow::Context;
use clap::CommandFactory;
use clap_complete::generate;

use crate::{
    cli::{Cli, Shell},
    compilation::{compile_src_files, create_executable, link_object_files},
    config::{load_config, Config},
    files::{get_dirs, get_src_files, setup_build_dir},
};

pub fn build(config_path: Option<PathBuf>) -> anyhow::Result<()> {
    let config_path = if let Some(config_path) = config_path {
        config_path
    } else {
        PathBuf::from("cbt.toml")
    };

    let config = load_config(&config_path)?;

    let (src_dir, build_dir) = get_dirs(&config)?;

    setup_build_dir(&src_dir, &build_dir, &config)?;

    let src_files = get_src_files(&src_dir, &config)?;

    let out_files = compile_src_files(&src_files, &config)?;

    let obj_file = if out_files.len() > 1 {
        link_object_files(&out_files, &build_dir, &config)?
    } else {
        out_files[0].clone()
    };

    if config.build.build_executable {
        let executable_name = match &config.build.executable {
            Some(name) => name,
            None => "a.out",
        };
        create_executable(executable_name, &obj_file, &build_dir, &config)?;
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
    let serialized =
        toml::to_string_pretty(&config).with_context(|| "Failed to serialize default config")?;

    std::fs::write(&path, serialized).with_context(|| "Failed to write config file")?;

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
