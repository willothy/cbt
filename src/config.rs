use std::{fs, path::PathBuf};

use anyhow::Context;
use serde_derive::{Deserialize, Serialize};

use crate::error;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub compilers: Compilers,
    #[serde(rename(deserialize = "stage"))]
    #[serde(rename(serialize = "stages"))]
    pub stages: Vec<Stage>,
}

#[derive(Deserialize, Serialize)]
pub struct Stage {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub flags: Flags,
    pub includes: Includes,
    #[serde(default)]
    pub exclude: Exclude,
    pub source: Source,
    pub build: Build,
    pub post_script: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Compilers {
    #[serde(default)]
    pub cc: String,
    #[serde(default)]
    pub cxx: String,
    #[serde(default)]
    pub asm: String,
    #[serde(default)]
    pub linker: String,
}

#[derive(Deserialize, Serialize)]
pub struct Flags {
    #[serde(default)]
    pub cflags: Vec<String>,
    #[serde(default)]
    pub cxxflags: Vec<String>,
    #[serde(default)]
    pub asmflags: Vec<String>,
    #[serde(default)]
    pub ldflags: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Includes {
    #[serde(default)]
    pub include_dirs: Vec<PathBuf>,
    #[serde(default)]
    pub include_prefix: String,
}

#[derive(Deserialize, Serialize)]
pub struct Exclude {
    pub dirs: Vec<PathBuf>,
    pub files: Vec<PathBuf>,
}

#[derive(Deserialize, Serialize)]
pub struct Source {
    pub source_dir: PathBuf,
}

#[derive(Deserialize, Serialize)]
pub struct Build {
    pub build_dir: PathBuf,
    pub target_dir: Option<PathBuf>,
    pub executable: Option<String>,
    pub executable_extra_flags: Option<Vec<String>>,
    pub build_executable: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            compilers: Default::default(),
            stages: vec![Stage {
                name: "default".to_string(),
                source: Default::default(),
                build: Default::default(),
                exclude: Default::default(),
                flags: Default::default(),
                includes: Default::default(),
                post_script: None,
            }],
        }
    }
}

impl Default for Stage {
    fn default() -> Self {
        Self {
            name: "default".to_owned(),
            flags: Default::default(),
            includes: Default::default(),
            exclude: Default::default(),
            source: Default::default(),
            build: Default::default(),
            post_script: None,
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
            include_dirs: vec![PathBuf::from("include")],
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
            source_dir: PathBuf::from("src"),
        }
    }
}

impl Default for Build {
    fn default() -> Self {
        Self {
            build_dir: PathBuf::from("build"),
            target_dir: None,
            executable: Some("default".to_owned()),
            executable_extra_flags: None,
            build_executable: true,
        }
    }
}

pub fn load_config(config_path: &PathBuf) -> anyhow::Result<Config> {
    let config = fs::read_to_string(&config_path)
        .with_context(|| error!("Failed to read config file {}", &config_path.display()))?;
    let config: Config = toml::from_str(&config)
        .with_context(|| error!("Failed to parse config toml file from string"))?;
    Ok(config)
}
