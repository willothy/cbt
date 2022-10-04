use std::{env, fs, io, path::PathBuf};

use crate::config::Config;

pub struct SourceFile {
    pub path: PathBuf,
    pub name: String,
    pub lang: Language,
}

pub enum Language {
    C,
    CXX,
}

pub fn copy_dir_structure(from: &PathBuf, to: &PathBuf) -> io::Result<()> {
    for entry in fs::read_dir(&from)? {
        let entry = entry?;
        let path = entry.path();
        let filename = path.file_name();
        if let Some(filename) = filename {
            if path.is_dir() {
                let new_dir = to.join(filename);
                fs::create_dir_all(new_dir)?;
                copy_dir_structure(&path, to)?;
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

pub fn setup_build_dir(src_dir: &PathBuf, build_dir: &PathBuf) -> io::Result<()> {
    fs::create_dir_all(&build_dir)?;
    copy_dir_structure(&src_dir, &build_dir)?;
    Ok(())
}

pub fn get_src_files(src_dir: &PathBuf, src_files: &mut Vec<SourceFile>) -> io::Result<()> {
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            get_src_files(&path, src_files)?;
        } else {
            let filename = match path.file_name() {
                Some(filename) => match filename.to_str() {
                    Some(filename) => filename.to_owned(),
                    None => "".to_owned(),
                },
                None => "".to_owned(),
            };

            match path.extension() {
                Some(ext) => match ext.to_str() {
                    Some("c") => src_files.push(SourceFile {
                        path: path.clone(),
                        name: filename,
                        lang: Language::C,
                    }),
                    Some("cpp") => src_files.push(SourceFile {
                        path: path.clone(),
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
    Ok(())
}
