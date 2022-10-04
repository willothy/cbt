use serde_derive::Deserialize;
use std::{
    env, fs,
    io::{self, ErrorKind},
    path::PathBuf,
    process::{Command, ExitCode, Output},
};

#[derive(Deserialize)]
struct Config {
    compilers: Compilers,
    flags: Flags,
    includes: Includes,
    source: Source,
    build: Build,
}

#[derive(Deserialize)]
struct Compilers {
    cc: String,
    cxx: String,
    linker: String,
}

#[derive(Deserialize)]
struct Flags {
    cflags: Vec<String>,
    cxxflags: Vec<String>,
    ldflags: Vec<String>,
}

#[derive(Deserialize)]
struct Includes {
    include_dirs: Vec<String>,
    include_prefix: String,
}

#[derive(Deserialize)]
struct Source {
    source_dir: String,
}

#[derive(Deserialize)]
struct Build {
    build_dir: String,
    executable_name: Option<String>,
}

struct SourceFile {
    path: PathBuf,
    name: String,
    lang: Language,
}

enum Language {
    C,
    CXX,
}

const CONFIG_FILENAME: &str = "test.toml";

fn copy_dir_structure(from: &PathBuf, to: &PathBuf) -> io::Result<()> {
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

fn load_config() -> io::Result<Config> {
    let config_path = PathBuf::from(CONFIG_FILENAME);
    let config = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config)?;
    Ok(config)
}

fn get_dirs(config: &Config) -> io::Result<(PathBuf, PathBuf)> {
    let current_dir = env::current_dir()?;
    let src_dir = current_dir.join(&config.source.source_dir);
    let build_dir = current_dir.join(&config.build.build_dir);
    Ok((src_dir, build_dir))
}

fn setup_build_dir(src_dir: &PathBuf, build_dir: &PathBuf) -> io::Result<()> {
    fs::create_dir_all(&build_dir)?;
    copy_dir_structure(&src_dir, &build_dir)?;
    Ok(())
}

fn get_src_files(src_dir: &PathBuf, src_files: &mut Vec<SourceFile>) -> io::Result<()> {
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
    let config = load_config()?;

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
