use std::{
    io::{self, ErrorKind},
    path::PathBuf,
    process::{Command, ExitCode, Output},
};

use self::{config::*, files::*};

mod config;
mod files;

const CONFIG_FILENAME: &str = "test.toml";

fn process_output(output: Output, filename: &String, action: &str) -> io::Result<()> {
    //println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    //println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            ErrorKind::Other,
            format!("Failed to {} {}", action, filename),
        ))
    }
}

fn compile_c(file: &SourceFile, build_dir: &PathBuf, config: &Config) -> io::Result<PathBuf> {
    let out_file = build_dir.join(&file.name).with_extension("o");
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

fn compile_cxx(file: &SourceFile, build_dir: &PathBuf, config: &Config) -> io::Result<PathBuf> {
    let out_file = build_dir.join(&file.path).with_extension("o");
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

fn compile_src_files(
    src_files: &Vec<SourceFile>,
    build_dir: &PathBuf,
    config: &Config,
) -> io::Result<Vec<PathBuf>> {
    let mut out_files = Vec::new();

    for file in src_files {
        match file.lang {
            Language::C => {
                let out_file = compile_c(file, &build_dir.canonicalize()?, &config)?;
                out_files.push(out_file);
            }
            Language::CXX => {
                let out_file = compile_cxx(file, &build_dir.canonicalize()?, &config)?;
                out_files.push(out_file);
            }
        };
    }
    Ok(out_files)
}

fn link_object_files(
    obj_files: &Vec<PathBuf>,
    build_dir: &PathBuf,
    config: &Config,
) -> io::Result<PathBuf> {
    // Link object files
    let out_name = match &config.build.executable_name {
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

fn create_executable(
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

fn run() -> io::Result<()> {
    let config = load_config(CONFIG_FILENAME.to_owned())?;

    let (src_dir, build_dir) = get_dirs(&config)?;

    setup_build_dir(&src_dir, &build_dir)?;

    let mut src_files = Vec::new();
    get_src_files(&src_dir, &mut src_files)?;

    if src_files.len() == 0 {
        return Err(io::Error::new(
            ErrorKind::Other,
            "No source files found in source directory",
        ));
    }

    let out_files = compile_src_files(&src_files, &build_dir, &config)?;

    if out_files.len() == 0 {
        return Err(io::Error::new(
            ErrorKind::Other,
            "No object files found in build directory",
        ));
    }

    let obj_file = if out_files.len() > 1 {
        link_object_files(&out_files, &build_dir, &config)?
    } else {
        out_files[0].clone()
    };

    let executable_name = match &config.build.executable_name {
        Some(name) => name,
        None => "a.out",
    };
    create_executable(executable_name, &obj_file, &build_dir, &config)?;
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
