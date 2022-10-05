use std::{path::PathBuf, process::Command};

use crate::{
    config::{Compilers, Stage},
    error,
    files::{get_dirs, get_src_files, setup_build_dir, Language, SourceFile},
    info, message,
    util::process_output,
};
use anyhow::{bail, Context};

pub fn compile(file: &SourceFile, compilers: &Compilers, stage: &Stage) -> anyhow::Result<PathBuf> {
    let (compiler, flags, out_extension) = match file.lang {
        Language::C => (&compilers.cc, &stage.flags.cflags, "o"),
        Language::CXX => (&compilers.cxx, &stage.flags.cxxflags, "o"),
        Language::ASM => (&compilers.asm, &stage.flags.asmflags, "asm.o"),
        //_ => bail!("Unsupported language"),
    };

    let out_file = file.out_path.with_extension(out_extension);
    if is_up_to_date(&out_file, &file.path)? {
        println!(
            "{}: {} is up to date",
            info!("Skipping compile step"),
            out_file.display()
        );
        return Ok(out_file);
    }
    println!(
        "{} {} to {}",
        message!("Compiling"),
        file.name,
        out_file.display()
    );

    // Spawn compiler process
    let compiler_process = Command::new(compiler)
        .arg("-c")
        .arg(&file.path)
        .arg("-o")
        .arg(&out_file)
        .args(flags)
        .spawn()
        .with_context(|| error!("Failed to spawn {} process", compiler))?;

    let output = compiler_process
        .wait_with_output()
        .with_context(|| error!("Failed to get {} output", compiler))?;

    process_output(output, compiler, &file.name, "compile")?;
    Ok(out_file)
}

pub fn compile_src_files(
    src_files: &Vec<SourceFile>,
    compilers: &Compilers,
    stage: &Stage,
) -> anyhow::Result<Vec<PathBuf>> {
    let mut out_files = Vec::new();

    for file in src_files {
        let out_file = compile(file, &compilers, &stage)?;
        out_files.push(out_file);
    }
    Ok(out_files)
}

pub fn link_object_files(
    obj_files: &Vec<PathBuf>,
    build_dir: &PathBuf,
    compilers: &Compilers,
    stage: &Stage,
) -> anyhow::Result<PathBuf> {
    // Link object files
    let out_name = match &stage.build.executable {
        Some(name) => name.to_owned(),
        None => "full_project_out".to_owned(),
    };
    let out_file = build_dir.join(&out_name).with_extension("o");

    let mut up_to_date = false;
    for obj in obj_files {
        if is_up_to_date(&out_file, obj)? {
            up_to_date = true;
        } else {
            up_to_date = false;
            break;
        }
    }
    if up_to_date {
        println!(
            "{}: {} is up to date",
            info!("Skipping link step"),
            out_file.display()
        );
        return Ok(out_file);
    }

    let child = Command::new(&compilers.linker)
        .arg("-relocatable")
        .args(obj_files)
        .arg("-o")
        .arg(&out_file)
        .args(&stage.flags.ldflags)
        .spawn()
        .with_context(|| error!("Failed to link object files"))?;

    let output = child.wait_with_output().with_context(|| {
        error!(
            "Failed to wait for {} process to complete",
            &compilers.linker
        )
    })?;
    process_output(output, &compilers.linker, &out_name, "link")?;
    Ok(out_file)
}

fn is_up_to_date(target: &PathBuf, source: &PathBuf) -> anyhow::Result<bool> {
    if target.exists() && source.exists() {
        Ok(target
            .metadata()
            .with_context(|| error!("Could not read metadata from {}", target.display()))?
            .created()
            .with_context(|| error!("Could not read metadata from {}", target.display()))?
            >= source
                .metadata()
                .with_context(|| error!("Could not read metadata from {}", source.display()))?
                .modified()
                .with_context(|| error!("Could not read metadata from {}", source.display()))?)
    } else {
        Ok(false)
    }
}

pub fn create_executable(
    executable_name: &str,
    obj_file: &PathBuf,
    build_dir: &PathBuf,
    compilers: &Compilers,
    stage: &Stage,
) -> anyhow::Result<()> {
    // Compile object file
    let executable_path = build_dir.join(executable_name);

    if is_up_to_date(&executable_path, obj_file)? {
        println!(
            "{}: {} is up to date",
            info!("Skipping executable step"),
            executable_path.display()
        );
        return Ok(());
    }

    println!(
        "{} {}",
        message!("Creating executable"),
        executable_path.display()
    );
    let child = Command::new(&compilers.cc)
        .arg(&obj_file)
        .arg("-o")
        .arg(&executable_path)
        .args(&stage.flags.cflags)
        .args(
            stage
                .includes
                .include_dirs
                .iter()
                .map(|dir| format!("{}{}", &stage.includes.include_prefix, dir.display())),
        )
        .spawn()
        .with_context(|| "Failed to compile object file")?;

    let output = child.wait_with_output().with_context(|| {
        error!(
            "Failed to wait for {} compiler process to complete compilation of {}",
            &compilers.cc,
            &executable_path.display()
        )
    })?;
    process_output(
        output,
        &compilers.cc,
        &obj_file.display().to_string(),
        "create executable from",
    )?;
    Ok(())
}

pub fn run_stage(compilers: &Compilers, stage: &Stage) -> anyhow::Result<()> {
    println!("{} {}", message!("Running stage"), stage.name);

    let (src_dir, build_dir) = get_dirs(&stage)?;

    setup_build_dir(&src_dir, &build_dir, &stage)?;

    let src_files = get_src_files(&src_dir, &stage)?;

    let out_files = compile_src_files(&src_files, &compilers, &stage)?;

    let obj_file = if out_files.len() > 1 {
        link_object_files(&out_files, &build_dir, &compilers, &stage)?
    } else {
        if let Some(object) = out_files.first() {
            object.to_owned()
        } else {
            bail!(error!("No object files were created"));
        }
    };

    if stage.build.build_executable {
        let executable_name = match &stage.build.executable {
            Some(name) => name,
            None => "a.out",
        };
        create_executable(executable_name, &obj_file, &build_dir, &compilers, &stage)?;
    }
    message!("{} stage {}", message!("Finished"), stage.name);
    Ok(())
}
