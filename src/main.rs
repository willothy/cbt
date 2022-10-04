use std::{io, process::ExitCode};

use self::{
    compilation::{compile_src_files, create_executable, link_object_files},
    config::*,
    files::*,
};

mod compilation;
mod config;
mod files;
mod util;

const CONFIG_FILENAME: &str = "test.toml";

fn run() -> io::Result<()> {
    let config = load_config(CONFIG_FILENAME.to_owned())?;

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

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}
