use log::{error, warn};
use std::collections::HashMap;
use std::io::BufRead;
use std::path::PathBuf;
use std::{fs, io};

#[derive(Eq, Hash, PartialEq, Debug)]
enum FileType {
    TypeC,
    TypeH,
    TypeM,
    TypeK,
    TypeRust,
    TypeAsm,
    TypePython,
    TypeOther,
}

impl FileType {
    fn from_extension(extension: &str) -> Self {
        match extension {
            "c" | "cpp" | "cc" => FileType::TypeC,
            "h" | "hpp" => FileType::TypeH,
            "rs" => FileType::TypeRust,
            "S" | "s" | "asm" => FileType::TypeAsm,
            "py" => FileType::TypePython,
            _ => FileType::TypeOther,
        }
    }

    fn from_filename(filename: &str) -> Self {
        match filename {
            "Makfile" => FileType::TypeM,
            "Kconfig" => FileType::TypeK,
            _ => FileType::TypeOther,
        }
    }
}

#[derive(Default)]
struct FileStat {
    files: usize,
    blank: usize,
    comment: usize,
    code: usize,
}

pub struct FileCounter {
    arch: String,
    version: String,
    dir_path: PathBuf,
    file_count: HashMap<FileType, FileStat>,
}

impl FileCounter {
    pub fn new(arch: String, version: String, dir_path: PathBuf) -> Self {
        FileCounter {
            arch,
            version,
            dir_path,
            file_count: HashMap::new(),
        }
    }

    pub fn search(&mut self) {
        let _ = self.search_dir(&self.dir_path.clone());
    }

    pub fn search_dir(&mut self, path: &PathBuf) -> io::Result<()> {
        warn!("start to seach dir -> {:?}", path);
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_dir() {
                            let _ = self.search_dir(&path);
                        } else if let Some(file_name) = path.file_name() {
                            let file_name_str = file_name.to_string_lossy();
                            let file_type = if file_name_str == "Makefile" {
                                FileType::TypeM
                            } else if file_name_str == "Kconfig" {
                                FileType::TypeK
                            } else if let Some(extension) = path.extension() {
                                FileType::from_extension(extension.to_str().unwrap_or(""))
                            } else {
                                FileType::TypeOther
                            };

                            let stats = self.file_count.entry(file_type).or_default();
                            stats.files += 1;

                            let file = fs::File::open(path)?;
                            let reader = io::BufReader::new(file);

                            let mut blank = 0;
                            let mut comment = 0;
                            let mut code = 0;

                            for line in reader.lines() {
                                let line = line?;
                                let trimmed = line.trim();
                                if trimmed.is_empty() {
                                    blank += 1;
                                } else if trimmed.starts_with("//")
                                    || trimmed.starts_with("/*")
                                    || trimmed.starts_with('*')
                                    || trimmed.starts_with('#')
                                    || trimmed.starts_with(';')
                                {
                                    comment += 1;
                                } else {
                                    code += 1;
                                }
                            }

                            // let (blank, comment, code) = self.count_lines(&path).unwrap_or((0, 0, 0));

                            stats.blank += blank;
                            stats.comment += comment;
                            stats.code += code;
                        }
                    }
                    Err(err) => {
                        error!("{:?} dir error", path);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn count_lines(&mut self, path: &PathBuf) -> io::Result<(usize, usize, usize)> {
        let file = fs::File::open(path)?;
        let reader = io::BufReader::new(file);

        let mut blank = 0;
        let mut comment = 0;
        let mut code = 0;

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                blank += 1;
            } else if trimmed.starts_with("//")
                || trimmed.starts_with("/*")
                || trimmed.starts_with('*')
                || trimmed.starts_with('#')
                || trimmed.starts_with(';')
            {
                comment += 1;
            } else {
                code += 1;
            }
        }

        Ok((blank, comment, code))
    }

    pub fn print(&self) {
        println!("{:-<70}", "");
        println!(
            "{:^70}",
            format!("Linux-{} Arch {}", self.version, self.arch.to_uppercase())
        );
        println!("{:-<70}", "");
        println!(
            "{: <30} {: <10} {: <10} {: <10} {: <10}",
            "Language", "files", "blank", "comment", "code"
        );
        println!("{:-<70}", "");

        let mut total_files = 0;
        let mut total_blank = 0;
        let mut total_comment = 0;
        let mut total_code = 0;

        let mut sorted_stats: Vec<_> = self.file_count.iter().collect();
        sorted_stats.sort_by(|a, b| b.1.code.cmp(&a.1.code));

        for (file_type, stats) in sorted_stats {
            let type_str = match file_type {
                FileType::TypeC => "C",
                FileType::TypeH => "C/C++ Header",
                FileType::TypeRust => "Rust",
                FileType::TypeAsm => "Assembly",
                FileType::TypePython => "Python",
                FileType::TypeM => "Makefile",
                FileType::TypeK => "kconfig",
                FileType::TypeOther => "Other",
            };
            println!(
                "{: <30} {: <10} {: <10} {: <10} {: <10}",
                type_str, stats.files, stats.blank, stats.comment, stats.code
            );

            total_files += stats.files;
            total_blank += stats.blank;
            total_comment += stats.comment;
            total_code += stats.code;
        }

        println!("{:-<70}", "");
        println!(
            "{: <30} {: <10} {: <10} {: <10} {: <10}",
            "SUM:", total_files, total_blank, total_comment, total_code
        );
        println!("{:-<70}", "");
    }
}

impl From<(String, String, PathBuf)> for FileCounter {
    fn from(value: (String, String, PathBuf)) -> Self {
        FileCounter {
            arch: value.0,
            version: value.1,
            dir_path: value.2,
            file_count: HashMap::new(),
        }
    }
}
