mod core;

use crate::core::file_counter::FileCounter;
use crate::core::kconfig_counter::KconfigCounter;
use anyhow::Result;
use clap::{Arg, Parser};
use log::{error, info, warn};
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::PathBuf;
// use crate::core::log::set_logger;

fn parse_bool(s: &str) -> Result<bool, String> {
    match s.to_lowercase().as_str() {
        "true" | "t" | "yes" | "y" | "1" => Ok(true),
        "false" | "f" | "no" | "n" | "0" => Ok(false),
        _ => Err(format!("invalid value for a boolean: {}", s)),
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    /// 指定需要解析的模块架构
    #[arg(long, short = 'a', value_delimiter = ',', default_value = "riscv")]
    arch: Vec<PathBuf>,

    /// 是否需要解析代码
    #[arg(long, short = 'c')]
    code: bool,

    /// 是否需要解析Kconfig
    #[arg(long, short = 'k')]
    kconfig: bool,

    /// 是否需要解析对应代码，该选项必须依赖于`kconfig`的设定
    #[arg(long, short = 'r')]
    kconfig_code: bool,

    /// 指定需要解析的内核位置
    #[arg(long, short = 'p', default_value = "/opt/linux-6.9.5")]
    kernel_path: PathBuf,

    /// 是否需要解析全部Kconfig
    #[arg(long, short = 'f')]
    full: bool,
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

    if args.code {
        for arg in &args.arch {
            info!("fetch arch: {:?}", arg);
            let mut arch_dir = args.kernel_path.clone();
            arch_dir.push("arch");
            arch_dir.push(arg);
            warn!("fetch {:?} arch directory path -> {:?}", arg, arch_dir);

            let mut fc = FileCounter::new(
                arg.clone().to_string_lossy().into_owned(),
                version.clone(),
                arch_dir,
            );
            fc.search();
            fc.print();
        }
    }

    if args.kconfig && !args.kconfig_code {
        for arg in &args.arch {
            info!("fetch arch: {:?}", arg);
            let mut arch_path = args.kernel_path.clone();
            arch_path.push("arch");
            arch_path.push(arg);
            arch_path.push("Kconfig");
            warn!("fetch {:?} arch Kconfig path -> {:?}", arg, arch_path);

            let mut kc = KconfigCounter::new(
                arg.clone().to_string_lossy().into_owned(),
                version.clone(),
                arch_path,
            );
            if args.full {
                kc.set_check_all();
            }
            kc.parse_kconfig();
            kc.print();
        }
    }

    if args.kconfig_code {
        if !args.kconfig {
            error!("Error: --kconfig_code (-r) requires --kconfig (-k) to be set");
            std::process::exit(1);
        }
        for arg in &args.arch {
            info!("fetch arch: {:?}", arg);
            let mut arch_path = args.kernel_path.clone();
            arch_path.push("arch");
            arch_path.push(arg);
            arch_path.push("Kconfig");
            warn!("fetch {:?} arch Kconfig path -> {:?}", arg, arch_path);

            let mut kc = KconfigCounter::new(
                arg.clone().to_string_lossy().into_owned(),
                version.clone(),
                arch_path,
            );
            if args.full {
                kc.set_check_all();
            }
            kc.parse_kconfig();
            kc.analyze_code();
            kc.print();
        }
    }

    Ok(())
}
