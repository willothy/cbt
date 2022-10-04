use std::{
    env, fs,
    io::{self, ErrorKind},
    path::PathBuf,
};

use crate::config::Config;

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
}

pub fn copy_dir_structure(from: &PathBuf, to: &PathBuf, config: &Config) -> io::Result<()> {
    for entry in fs::read_dir(&from)? {
        let entry = entry?;
        let path = entry.path();
        let filename = path.file_name();
        if let Some(filename) = filename {
            if path.is_dir() {
                if config
                    .exclude
                    .dirs
                    .contains(&filename.to_str().unwrap().to_owned())
                {
                    continue;
                }
                let new_dir = to.join(filename);
                fs::create_dir_all(new_dir)?;
                copy_dir_structure(&path, to, config)?;
            }
        } else {
            continue;
        }
    }
    Ok(())
}

pub fn get_dirs(config: &Config) -> io::Result<(PathBuf, PathBuf)> {
    let current_dir = env::current_dir()?;
    let src_dir = current_dir.join(&config.source.source_dir);
    let build_dir = current_dir.join(&config.build.build_dir);
    Ok((src_dir, build_dir))
}

pub fn setup_build_dir(src_dir: &PathBuf, build_dir: &PathBuf, config: &Config) -> io::Result<()> {
    fs::create_dir_all(&build_dir)?;
    copy_dir_structure(&src_dir, &build_dir, config)?;
    Ok(())
}

pub fn get_src_files(src_dir: &PathBuf, config: &Config) -> io::Result<Vec<SourceFile>> {
    let mut src_files = Vec::new();
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        let filename = path.file_name().unwrap();
        if path.is_dir() {
            if config
                .exclude
                .dirs
                .contains(&filename.to_str().unwrap().to_owned())
            {
                continue;
            }
            src_files.extend(get_src_files(&path, config)?);
        } else {
            let filename = match path.file_name() {
                Some(filename) => match filename.to_str() {
                    Some(filename) => filename.to_owned(),
                    None => "".to_owned(),
                },
                None => "".to_owned(),
            };

            if config.exclude.files.contains(&filename) {
                continue;
            }

            let src_dir_base = PathBuf::from(&config.source.source_dir).canonicalize()?;
            let new_path_components = path.components().skip(src_dir_base.components().count());
            let out_path = new_path_components.clone().fold(
                PathBuf::from(&config.build.build_dir),
                |mut path, comp| {
                    path.push(comp);
                    path
                },
            );
            let new_path = new_path_components
                .clone()
                .fold(
                    PathBuf::from(&config.source.source_dir),
                    |mut path, comp| {
                        path.push(comp);
                        path
                    },
                )
                .canonicalize()?;

            match path.extension() {
                Some(ext) => match ext.to_str() {
                    Some("c") => src_files.push(SourceFile {
                        path: new_path.clone(),
                        out_path: out_path.clone(),
                        name: filename,
                        lang: Language::C,
                    }),
                    Some("cpp") => src_files.push(SourceFile {
                        path: new_path.clone(),
                        out_path: out_path.clone(),
                        name: filename,
                        lang: Language::CXX,
                    }),
                    Some(_) => continue,
                    None => continue,
                },
                None => continue,
            }
        }
    }
    if src_files.len() == 0 {
        return Err(io::Error::new(
            ErrorKind::Other,
            "No source files found in source directory",
        ));
    }
    Ok(src_files)
}
