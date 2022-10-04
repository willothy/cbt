use std::{fs, path::PathBuf};

use anyhow::Context;
use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub compilers: Compilers,
    pub flags: Flags,
    pub includes: Includes,
    pub exclude: Exclude,
    pub source: Source,
    pub build: Build,
}

#[derive(Deserialize)]
pub struct Compilers {
    pub cc: String,
    pub cxx: String,
    pub asm: String,
    pub linker: String,
}

#[derive(Deserialize)]
pub struct Flags {
    pub cflags: Vec<String>,
    pub cxxflags: Vec<String>,
    pub asmflags: Vec<String>,
    pub ldflags: Vec<String>,
}

#[derive(Deserialize)]
pub struct Includes {
    pub include_dirs: Vec<String>,
    pub include_prefix: String,
}

#[derive(Deserialize)]
pub struct Exclude {
    pub dirs: Vec<String>,
    pub files: Vec<String>,
}

#[derive(Deserialize)]
pub struct Source {
    pub source_dir: String,
}

#[derive(Deserialize)]
pub struct Build {
    pub build_dir: String,
    pub executable: Option<String>,
    pub build_executable: bool,
}

pub fn load_config(config_file: String) -> anyhow::Result<Config> {
    let config_path = PathBuf::from(config_file);
    let config = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file {}", &config_path.display()))?;
    let config: Config = toml::from_str(&config)
        .with_context(|| format!("Failed to parse config toml file from string"))?;
    Ok(config)
}
