use crate::core::utils::get_filed;
use anyhow::Result;
use log::{error, info, warn};
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum KconfigComponentType {
    Unknown,
    Bool,
    Value,
}

pub struct KconfigStat {
    default_value: Vec<String>,
    select: Vec<String>,
    depend: Vec<String>,
    value_type: KconfigComponentType,
    count: usize,
}

pub struct KconfigCounter {
    arch: String,
    version: String,
    kconfig_path: PathBuf,
    check_all: bool,
    component: HashMap<String, KconfigStat>,
    total_components: usize,
}

impl KconfigCounter {
    pub fn new(arch: String, version: String, kconfig_path: PathBuf) -> Self {
        KconfigCounter {
            arch,
            version,
            kconfig_path,
            check_all: false,
            component: HashMap::new(),
            total_components: 0,
        }
    }

    pub fn set_check_all(&mut self) {
        self.check_all = true;
    }

    pub fn parse_kconfig(&mut self) -> Result<()> {
        self.parse_kconfig_path(&self.kconfig_path.clone())
    }

    pub fn parse_kconfig_path(&mut self, kconfig_path: &PathBuf) -> Result<()> {
        let file = File::open(kconfig_path)?;
        let reader = io::BufReader::new(file);

        let mut component_name = String::new();
        let mut update = false;

        for line in reader.lines() {
            let line = line?;
            let trim_line = line.trim();
            if trim_line.starts_with('#') {
                continue;
            }

            if trim_line.starts_with("source") {
                let mut kernel_path = self.kconfig_path.clone();
                let kernel_version = format!("linux-{}", self.version);

                while let Some(parent) = kernel_path.parent() {
                    if parent.ends_with(&kernel_version) {
                        kernel_path = parent.to_path_buf();
                        break;
                    }
                    kernel_path = parent.to_path_buf();
                }
                let source_path = get_filed(trim_line, "source");
                let source_path = source_path.trim_matches('"');
                let mut kconfig_path = kernel_path;
                kconfig_path.push(source_path);
                kconfig_path.canonicalize().unwrap();

                if self.check_all || kconfig_path.to_str().unwrap_or("").contains("/arch/") {
                    warn!("fetch a new Kconfig -> {:?}", kconfig_path);
                    info!("entering the Kconfig of corresponding architecture -> {}", self.arch);
                    self.parse_kconfig_path(&kconfig_path);
                } else if self.check_all {
                    warn!("fetch a new Kconfig -> {:?}", kconfig_path);
                    self.parse_kconfig_path(&kconfig_path);
                }
            }

            if trim_line.is_empty() {
                update = true;
                continue;
            }

            if update && trim_line.starts_with("config") {
                // println!("{}", trim_line);
                component_name = get_filed(trim_line, "config");
                update = false;
            }

            if trim_line.starts_with("config ") {
                component_name = get_filed(trim_line, "config");
                // info!("fetch the component name -> {}", component_name);

                let entry = self
                    .component
                    .entry(component_name.clone())
                    .or_insert_with(|| {
                        self.total_components += 1;
                        KconfigStat {
                            default_value: Vec::new(),
                            select: Vec::new(),
                            depend: Vec::new(),
                            value_type: KconfigComponentType::Value,
                            count: 0,
                        }
                    });

                entry.count += 1;
            }

            if trim_line.starts_with("depends on") {
                if let Some(stat) = self.component.get_mut(&component_name) {
                    stat.depend.push(get_filed(trim_line, "depends on"));
                }
            }

            if trim_line.starts_with("bool") {
                if let Some(stat) = self.component.get_mut(&component_name) {
                    stat.value_type = KconfigComponentType::Bool;
                }
            }

            if trim_line.starts_with("default") {
                if let Some(stat) = self.component.get_mut(&component_name) {
                    stat.default_value.push(get_filed(trim_line, "default"));
                }
            }

            if trim_line.starts_with("def_bool") {
                if let Some(stat) = self.component.get_mut(&component_name) {
                    stat.default_value.clear();
                    stat.default_value.push(get_filed(trim_line, "def_bool"));
                    stat.value_type = KconfigComponentType::Bool;
                }
            }

            if trim_line.starts_with("select") {
                if let Some(stat) = self.component.get_mut(&component_name) {
                    stat.select.push(get_filed(trim_line, "select"));
                }
            }
        }

        Ok(())
    }

    pub fn print(&self) {
        println!("{:-<90}", "");
        println!("{:^90}", format!("Linux-{} Arch {}",self.version, self.arch.to_uppercase()));
        println!("{:-<90}", "");
        println!("{:^45} {:^45}", "Component", "Component");
        println!("{:-<90}", "");
        let mut iter = self.component.keys();
        while let Some(name1) = iter.next() {
            let unwrap = String::new();
            let name2 = iter.next().unwrap_or(&unwrap);
            println!("{:^45} | {:^45}", name1, name2);
        }
        println!("{:-<90}", "");
        println!("{:^45} {:>20} Components", "SUM:", self.component.len());
        println!("{:-<90}", "");

        let mut input = String::new();
        loop {
            print!("Enter a component name to view its details (or 'q' to quit)>> ");
            io::stdout().flush().unwrap();
            input.clear();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();

            if input.eq_ignore_ascii_case("q") {
                break;
            }

            if let Some(stat) = self.component.get(input) {
                println!("Component: {}", input);
                println!("  Value Type: {:?}", stat.value_type);
                println!("  Depends on: {:#?}", stat.depend);
                println!("  Default value: {:#?}", stat.default_value);
                println!("  Select: {:#?}", stat.select);
            } else {
                error!("Component '{}' not found.", input);
            }
        }
    }
}
