use std::{path::PathBuf, process::Command};

use anyhow::{bail, Context};

use crate::{
    config::Config,
    files::{Language, SourceFile},
    util::process_output,
};

pub fn compile(file: &SourceFile, config: &Config) -> anyhow::Result<PathBuf> {
    let (compiler, flags, out_extension) = match file.lang {
        Language::C => (&config.compilers.cc, &config.flags.cflags, "o"),
        Language::CXX => (&config.compilers.cxx, &config.flags.cxxflags, "o"),
        Language::ASM => (&config.compilers.asm, &config.flags.asmflags, "asm.o"),
        //_ => bail!("Unsupported language"),
    };

    let out_file = file.out_path.with_extension(out_extension);
    println!("Compiling {} to {}", file.name, out_file.display());
    // Spawn compiler process
    let compiler_process = Command::new(compiler)
        .arg(&file.path)
        .arg("-o")
        .arg(&out_file)
        .args(flags)
        .spawn()
        .with_context(|| format!("Failed to spawn {} process", compiler))?;

    let output = compiler_process
        .wait_with_output()
        .with_context(|| format!("Failed to get {} output", compiler))?;

    process_output(output, compiler, &file.name, "compile")?;
    Ok(out_file)
}

pub fn compile_src_files(
    src_files: &Vec<SourceFile>,
    config: &Config,
) -> anyhow::Result<Vec<PathBuf>> {
    let mut out_files = Vec::new();

    for file in src_files {
        let out_file = compile(file, &config)?;
        out_files.push(out_file);
    }
    if out_files.len() == 0 {
        bail!("No object files found in build directory");
    }
    Ok(out_files)
}

pub fn link_object_files(
    obj_files: &Vec<PathBuf>,
    build_dir: &PathBuf,
    config: &Config,
) -> anyhow::Result<PathBuf> {
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

    let output = child.wait_with_output().with_context(|| {
        format!(
            "Failed to wait for {} process to complete",
            &config.compilers.linker
        )
    })?;
    process_output(output, &config.compilers.linker, &out_name, "link")?;
    Ok(out_file)
}

pub fn create_executable(
    executable_name: &str,
    obj_file: &PathBuf,
    build_dir: &PathBuf,
    config: &Config,
) -> anyhow::Result<()> {
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

    let output = child.wait_with_output().with_context(|| {
        format!(
            "Failed to wait for {} compiler process to complete compilation of {}",
            &config.compilers.cc,
            &executable_path.display()
        )
    })?;
    process_output(
        output,
        &config.compilers.cc,
        &obj_file.display().to_string(),
        "create executable from",
    )?;
    Ok(())
}
