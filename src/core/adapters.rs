use std::{
    collections::HashMap,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

pub trait FileSystem {
    fn list_py_files(&self, path: impl AsRef<Path>) -> Vec<PathBuf>;
    fn read_to_string(&self, path: &Path) -> io::Result<String>;
    fn write(&mut self, path: &Path, contents: &str) -> std::result::Result<(), std::io::Error>;
}

pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn list_py_files(&self, path: impl AsRef<Path>) -> Vec<PathBuf> {
        WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension() == Some(OsStr::new("py")))
            .map(|entry| entry.path().to_path_buf())
            .collect()
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    fn write(&mut self, path: &Path, contents: &str) -> std::result::Result<(), std::io::Error> {
        fs::write(path, contents)
    }
}

pub struct FakeFileSystem {
    pub files: HashMap<PathBuf, String>,
    pub operations: Vec<String>,
}

impl FakeFileSystem {
    pub fn new(files: HashMap<PathBuf, String>) -> Self {
        Self {
            files,
            operations: Vec::new(),
        }
    }
}

impl Default for FakeFileSystem {
    fn default() -> Self {
        Self::new(HashMap::new())
    }
}

impl FileSystem for FakeFileSystem {
    fn list_py_files(&self, _path: impl AsRef<Path>) -> Vec<PathBuf> {
        self.files
            .keys()
            .filter(|p| p.extension() == Some(OsStr::new("py")))
            .map(|p| p.to_path_buf())
            .collect()
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        if let Some(contents) = self.files.get(path) {
            Ok(contents.to_owned())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
        }
    }

    fn write(&mut self, path: &Path, contents: &str) -> std::result::Result<(), std::io::Error> {
        self.operations
            .push(format!("write: `{}`", &path.display()));
        self.files
            .insert(path.to_path_buf(), contents.to_string().clone());
        Ok(())
    }
}
