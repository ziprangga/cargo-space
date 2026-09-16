use crate::errors::{CargoResult, error};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::path::Path;

#[derive(Debug)]
pub struct GitRepo;

impl GitRepo {
    pub fn init(path: &Path) -> CargoResult<()> {
        if !path.join(".git").exists() {
            fs::create_dir_all(path)?;
            Self::init_int(path)?;
        }

        Ok(())
    }

    pub fn discover(path: &Path) -> Result<git2::Repository, git2::Error> {
        git2::Repository::discover(path)
    }

    pub fn write_gitignore_file(base_path: &Path, list: &IgnoreList) -> CargoResult<()> {
        let path = base_path.join(".gitignore");

        let ignore = match File::open(&path) {
            Err(err) if err.kind() == ErrorKind::NotFound => list.format_new(),
            Err(err) => return Err(err.into()),
            Ok(file) => list.format_existing(BufReader::new(file))?,
        };

        let mut file = OpenOptions::new().create(true).append(true).open(&path)?;

        file.write_all(ignore.as_bytes())?;

        Ok(())
    }

    fn init_int(path: &Path) -> CargoResult<GitRepo> {
        git2::Repository::init(path)?;
        Ok(GitRepo)
    }
}

pub struct IgnoreList {
    ignore: Vec<String>,
}

impl IgnoreList {
    pub fn new() -> IgnoreList {
        IgnoreList { ignore: Vec::new() }
    }

    pub fn push(&mut self, ignore: &str) {
        self.ignore.push(ignore.to_string());
    }

    fn format_new(&self) -> String {
        self.ignore.join("\n") + "\n"
    }

    fn format_existing<T: BufRead>(&self, existing: T) -> CargoResult<String> {
        let mut existing_items = Vec::new();

        for (i, item) in existing.lines().enumerate() {
            match item {
                Ok(s) => existing_items.push(s),
                Err(err) => match err.kind() {
                    ErrorKind::InvalidData => {
                        return Err(error!(
                            "Character at line {} is invalid. Cargo only supports UTF-8.",
                            i
                        ));
                    }
                    _ => return Err(error!(err)),
                },
            }
        }

        let mut out = String::new();

        out.push_str("\n\n# Added by cargo\n");

        if self.ignore.iter().any(|item| existing_items.contains(item)) {
            out.push_str("#\n# already existing elements were commented out\n");
        }

        out.push('\n');

        for item in &self.ignore {
            if existing_items.contains(item) {
                out.push('#');
            }

            out.push_str(item);
            out.push('\n');
        }

        Ok(out)
    }
}

impl Default for IgnoreList {
    fn default() -> Self {
        Self {
            ignore: vec!["/target".to_string()],
        }
    }
}
