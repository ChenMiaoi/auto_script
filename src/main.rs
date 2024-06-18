mod core;

use crate::core::file_counter::FileCounter;
use anyhow::Result;
use clap::{Arg, Parser};
use log::{info, warn};
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::PathBuf;
// use crate::core::log::set_logger;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    /// 指定需要解析的模块架构
    #[arg(long, short, value_delimiter = ',', default_value = "riscv")]
    arch: Vec<PathBuf>,

    /// 指定需要解析的内核位置
    #[arg(long, short, default_value = "/opt/linux-6.9.5")]
    kernel_path: PathBuf,
}

fn fetch_kernel_version(kernel_path: &PathBuf) -> Result<String> {
    let file = File::open(kernel_path)?;
    let reader = io::BufReader::new(file);

    let mut version = None;
    let mut patch_level = None;
    let mut sublevel = None;

    for line in reader.lines() {
        let line = line?;
        if line.trim_start().starts_with('#') {
            continue;
        }
        if line.trim().starts_with("VERSION = ") {
            version = Some(line["VERSION = ".len()..].trim().to_string());
            // info!("fetch kernel version: {:?}", version);
        }
        if line.trim().starts_with("PATCHLEVEL = ") {
            patch_level = Some(line["PATCHLEVEL = ".len()..].trim().to_string());
            // info!("fetch kernel patchlevel: {:?}", patch_level);
        }
        if line.trim().starts_with("SUBLEVEL = ") {
            sublevel = Some(line["SUBLEVEL = ".len()..].trim().to_string());
            // info!("fetch kernel sublevel: {:?}", sublevel);
        }
    }

    if let (Some(v), Some(p), Some(s)) = (version, patch_level, sublevel) {
        Ok(format!("{}.{}.{}", v, p, s))
    } else {
        Err(anyhow::anyhow!("Failed to read version information"))
    }
}

fn main() -> Result<()> {
    // set_logger();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();
    let args = Args::parse();

    info!("fetch linux kernel directory: {:?}", args.kernel_path);

    let mut version_file = args.kernel_path.clone();
    version_file.push("Makefile");

    let version = fetch_kernel_version(&version_file)?;
    info!("fetch linux kernel version: {:?}", version);

    for arg in &args.arch {
        info!("fetch arch: {:?}", arg);
        let mut arch_dir = args.kernel_path.clone();
        arch_dir.push("arch");
        arch_dir.push(arg);
        warn!("fetch {:?} arch directory path -> {:?}", arg, arch_dir);

        let mut fc = FileCounter::new(arg.clone().to_string_lossy().into_owned(), arch_dir);
        fc.search();
        fc.print();
    }

    Ok(())
}
