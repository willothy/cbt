use std::{path::PathBuf, process::Command};

use crate::{
    bold,
    config::{Compilers, Stage},
    error,
    files::{get_dirs, get_src_files, setup_build_dir, Language, SourceFile},
    info, message,
    util::process_output,
};
use anyhow::{bail, Context};
use run_script::ScriptOptions;

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
            bold!("{}", out_file.display())
        );
        return Ok(out_file);
    }
    println!(
        "{} {} to {}",
        message!("Compiling"),
        file.name,
        out_file.display()
    );

    let includes = match file.lang {
        Language::C | Language::CXX => {
            let mut includes = Vec::new();
            for include in &stage.includes.include_dirs {
                includes.push(format!("-I{}", include.display().to_string().trim()));
            }
            includes
        }
        _ => Vec::new(),
    };

    // Spawn compiler process
    let mut cmd = Command::new(compiler);
    cmd.arg(match file.lang {
        Language::ASM => "",
        _ => "-c",
    })
    .arg(&file.path)
    .arg("-o")
    .arg(&out_file)
    .args(includes)
    .args(flags);
    //println!("{:?}", cmd);
    let compiler_process = cmd
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

    println!("{} {}", message!("Linking objects to"), out_file.display());

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
            bold!("{}", out_file.file_name().unwrap().to_str().unwrap())
        );
        return Ok(out_file);
    }

    let mut cmd = Command::new(&compilers.linker);
    cmd
        //.arg("-relocatable")
        .args(obj_files)
        .arg("-o")
        .arg(&out_file)
        .args(&stage.flags.ldflags);
    //println!("{:?}", cmd);

    let child = cmd
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
    let executable_dir = if let Some(target_dir) = &stage.build.target_dir {
        if target_dir.canonicalize()?.exists() {
            target_dir
        } else {
            build_dir
        }
    } else {
        build_dir
    };
    let executable_path = executable_dir.join(executable_name);

    if is_up_to_date(&executable_path, obj_file)? {
        println!(
            "{}: {} is up to date",
            info!("Skipping executable step"),
            bold!("{}", executable_path.file_name().unwrap().to_str().unwrap())
        );
        return Ok(());
    }

    println!(
        "{} {}",
        message!("Creating executable"),
        executable_path.display()
    );
    let exe_flags = match stage.build.executable_extra_flags {
        Some(ref flags) => {
            let mut temp = stage.flags.cflags.clone();
            temp.extend(flags.clone());
            temp
        }
        None => stage.flags.cflags.clone(),
    };
    let mut includes = Vec::new();
    for include in &stage.includes.include_dirs {
        includes.push(format!("-I{}", include.display().to_string().trim()));
    }
    let mut cmd = Command::new(&compilers.cc);
    cmd.arg(&obj_file)
        .arg("-o")
        .arg(&executable_path)
        .args(&exe_flags);
    //println!("{:?}", cmd);
    let child = cmd
        //.args(includes)
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
        format!("create executable {} from", executable_path.display()).as_str(),
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

    if let Some(post_script) = &stage.post_script {
        let (_exit_code, output, error) =
            run_script::run(post_script, &vec![], &ScriptOptions::new())
                .with_context(|| error!("Failed to run post script for {}", stage.name))?;
        if error.len() > 0 {
            println!("{}", error);
        }
        if output.len() > 0 {
            println!("{}", output);
        }
    }
    message!("{} stage {}", message!("Finished"), stage.name);
    Ok(())
}
