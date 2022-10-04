use std::{io, path::PathBuf, process::Command};

use crate::{
    config::Config,
    files::{Language, SourceFile},
    util::process_output,
};

pub fn compile_c(file: &SourceFile, config: &Config) -> io::Result<PathBuf> {
    let out_file = file.out_path.with_extension("o");
    println!("Compiling {} to {}", file.name, out_file.display());
    // Compile C file
    let child = Command::new(&config.compilers.cc)
        .arg("-c")
        .arg(&file.path)
        .arg("-o")
        .arg(&out_file)
        .args(&config.flags.cflags)
        .args(
            config
                .includes
                .include_dirs
                .iter()
                .map(|dir| format!("{}{}", &config.includes.include_prefix, dir)),
        )
        .spawn()
        .expect("Failed to compile C file");

    let output = child
        .wait_with_output()
        .expect("Failed to get C compiler output");

    process_output(output, &file.name, "compile")?;
    Ok(out_file)
}

pub fn compile_cxx(file: &SourceFile, config: &Config) -> io::Result<PathBuf> {
    let out_file = file.out_path.with_extension("o");
    println!("Compiling {} to {}", file.name, out_file.display());
    // Compile C++ file
    let child = Command::new(&config.compilers.cxx)
        .arg("-c")
        .arg(&file.path)
        .arg("-o")
        .arg(&out_file)
        .args(&config.flags.cxxflags)
        .args(
            config
                .includes
                .include_dirs
                .iter()
                .map(|dir| format!("{}{}", &config.includes.include_prefix, dir)),
        )
        .spawn()
        .expect("Failed to compile C++ file");

    let output = child
        .wait_with_output()
        .expect("Failed to get C compiler output");

    process_output(output, &file.name, "compile")?;
    Ok(out_file)
}

pub fn compile_src_files(src_files: &Vec<SourceFile>, config: &Config) -> io::Result<Vec<PathBuf>> {
    let mut out_files = Vec::new();

    for file in src_files {
        match file.lang {
            Language::C => {
                let out_file = compile_c(file, &config)?;
                out_files.push(out_file);
            }
            Language::CXX => {
                let out_file = compile_cxx(file, &config)?;
                out_files.push(out_file);
            }
        };
    }
    if out_files.len() == 0 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "No object files found in build directory",
        ));
    }
    Ok(out_files)
}

pub fn link_object_files(
    obj_files: &Vec<PathBuf>,
    build_dir: &PathBuf,
    config: &Config,
) -> io::Result<PathBuf> {
    // Link object files
    let out_name = match &config.build.executable {
        Some(name) => name.to_owned(),
        None => "full_project_out".to_owned(),
    };
    let out_file = build_dir.join(&out_name).with_extension("o");
    let child = Command::new(&config.compilers.linker)
        .arg("-relocatable")
        .args(obj_files)
        .arg("-o")
        .arg(&out_file)
        .args(&config.flags.ldflags)
        .spawn()
        .expect("Failed to link object files");

    let output = child.wait_with_output()?;
    process_output(output, &out_name, "link")?;
    Ok(out_file)
}

pub fn create_executable(
    executable_name: &str,
    obj_file: &PathBuf,
    build_dir: &PathBuf,
    config: &Config,
) -> io::Result<()> {
    // Compile object file
    let executable_path = build_dir.join(executable_name);
    println!("Creating executable {}", executable_path.display());
    let child = Command::new(&config.compilers.cc)
        .arg(&obj_file)
        .arg("-o")
        .arg(&executable_path)
        .args(&config.flags.cflags)
        .args(
            config
                .includes
                .include_dirs
                .iter()
                .map(|dir| format!("{}{}", &config.includes.include_prefix, dir)),
        )
        .spawn()
        .expect("Failed to compile object file");

    let output = child.wait_with_output()?;
    process_output(
        output,
        &obj_file.display().to_string(),
        "create executable from",
    )?;
    Ok(())
}
