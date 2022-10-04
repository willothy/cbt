use std::{fs, path::PathBuf};

use anyhow::Context;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub compilers: Compilers,
    pub flags: Flags,
    pub includes: Includes,
    pub exclude: Exclude,
    pub source: Source,
    pub build: Build,
}

#[derive(Deserialize, Serialize)]
pub struct Compilers {
    pub cc: String,
    pub cxx: String,
    pub asm: String,
    pub linker: String,
}

#[derive(Deserialize, Serialize)]
pub struct Flags {
    pub cflags: Vec<String>,
    pub cxxflags: Vec<String>,
    pub asmflags: Vec<String>,
    pub ldflags: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Includes {
    pub include_dirs: Vec<String>,
    pub include_prefix: String,
}

#[derive(Deserialize, Serialize)]
pub struct Exclude {
    pub dirs: Vec<String>,
    pub files: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Source {
    pub source_dir: String,
}

#[derive(Deserialize, Serialize)]
pub struct Build {
    pub build_dir: String,
    pub executable: Option<String>,
    pub build_executable: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            compilers: Default::default(),
            flags: Default::default(),
            includes: Default::default(),
            exclude: Default::default(),
            source: Default::default(),
            build: Default::default(),
        }
    }
}

impl Default for Compilers {
    fn default() -> Self {
        Self {
            cc: "gcc".to_owned(),
            cxx: "g++".to_owned(),
            asm: "nasm".to_owned(),
            linker: "ld".to_owned(),
        }
    }
}

impl Default for Flags {
    fn default() -> Self {
        Self {
            cflags: vec![],
            cxxflags: vec![],
            asmflags: vec!["-felf64".to_owned()],
            ldflags: vec![],
        }
    }
}

impl Default for Includes {
    fn default() -> Self {
        Self {
            include_dirs: vec!["include".to_owned()],
            include_prefix: "-I".to_owned(),
        }
    }
}

impl Default for Exclude {
    fn default() -> Self {
        Self {
            dirs: vec![],
            files: vec![],
        }
    }
}

impl Default for Source {
    fn default() -> Self {
        Self {
            source_dir: "src".to_owned(),
        }
    }
}

impl Default for Build {
    fn default() -> Self {
        Self {
            build_dir: "build".to_owned(),
            executable: Some("".to_owned()),
            build_executable: true,
        }
    }
}

pub fn load_config(config_path: &PathBuf) -> anyhow::Result<Config> {
    let config = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file {}", &config_path.display()))?;
    let config: Config = toml::from_str(&config)
        .with_context(|| format!("Failed to parse config toml file from string"))?;
    Ok(config)
}
