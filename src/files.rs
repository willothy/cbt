use std::{env, fs, path::PathBuf};

use crate::{config::Stage, error};
use anyhow::bail;

#[derive(Debug)]
pub struct SourceFile {
    pub path: PathBuf,
    pub out_path: PathBuf,
    pub name: String,
    pub lang: Language,
}

#[derive(Debug)]
pub enum Language {
    C,
    CXX,
    ASM,
}

pub fn copy_dir_structure(from: &PathBuf, to: &PathBuf, stage: &Stage) -> anyhow::Result<()> {
    for entry in fs::read_dir(&from)? {
        let entry = entry?;
        let path = entry.path();
        let filename = path.file_name();
        if let Some(filename) = filename {
            if path.is_dir() {
                if stage.exclude.dirs.contains(&path) {
                    continue;
                }
                let new_dir = to.join(filename);
                fs::create_dir_all(new_dir)?;
                copy_dir_structure(&path, to, stage)?;
            }
        }
    }
    Ok(())
}

pub fn get_dirs(stage: &Stage) -> anyhow::Result<(PathBuf, PathBuf)> {
    let current_dir = env::current_dir()?;
    let src_dir = current_dir.join(&stage.source.source_dir);
    let build_dir = current_dir.join(&stage.build.build_dir);
    Ok((src_dir, build_dir))
}

pub fn setup_build_dir(
    src_dir: &PathBuf,
    build_dir: &PathBuf,
    stage: &Stage,
) -> anyhow::Result<()> {
    fs::create_dir_all(&build_dir)?;
    copy_dir_structure(&src_dir, &build_dir, stage)?;
    Ok(())
}

pub fn get_src_files(src_dir: &PathBuf, stage: &Stage) -> anyhow::Result<Vec<SourceFile>> {
    let mut src_files = Vec::new();
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if stage.exclude.dirs.contains(&path) {
                continue;
            }
            src_files.extend(get_src_files(&path, stage)?);
        } else {
            let filename = match path.file_name() {
                Some(filename) => match filename.to_str() {
                    Some(filename) => filename.to_owned(),
                    None => bail!(error!("Could not convert filename to str")),
                },
                None => bail!(error!("Could not read filename")),
            };

            if stage.exclude.files.contains(&path) {
                continue;
            }

            let src_dir_base = PathBuf::from(&stage.source.source_dir).canonicalize()?;
            let new_path_components = path.components().skip(src_dir_base.components().count());
            let out_path = new_path_components.clone().fold(
                PathBuf::from(&stage.build.build_dir),
                |mut path, comp| {
                    path.push(comp);
                    path
                },
            );
            let new_path = new_path_components
                .clone()
                .fold(PathBuf::from(&stage.source.source_dir), |mut path, comp| {
                    path.push(comp);
                    path
                })
                .canonicalize()?;

            if let Some(ext) = path.extension() {
                match ext.to_ascii_lowercase().to_str() {
                    Some("c") => src_files.push(SourceFile {
                        path: new_path,
                        out_path,
                        name: filename,
                        lang: Language::C,
                    }),
                    Some("cpp") => src_files.push(SourceFile {
                        path: new_path,
                        out_path,
                        name: filename,
                        lang: Language::CXX,
                    }),
                    Some("s" | "asm") => src_files.push(SourceFile {
                        path: new_path,
                        out_path,
                        name: filename,
                        lang: Language::ASM,
                    }),
                    _ => continue,
                }
            } else {
                continue;
            }
        }
    }
    if src_files.len() == 0 {
        bail!(error!("No source files found in source directory"));
    }
    Ok(src_files)
}
