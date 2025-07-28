use crate::core::adapters::FileSystem;
use colored::Colorize;
use rayon::prelude::*;
use regex::Regex;
use std::{
    ffi::OsStr,
    io,
    ops::Deref,
    path::{Path, PathBuf},
    process::ExitCode,
};

pub fn main(file_sys: &mut impl FileSystem, scripts_root: String, readme_path: &Path) -> ExitCode {
    let readme = match ReadMe::read(file_sys, readme_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{} {}", "Failed to read README file: ".red().bold(), e);
            return ExitCode::FAILURE;
        }
    };

    let paths = file_sys.list_files(&scripts_root);
    if paths.is_empty() {
        println!(
            "{} `{}`",
            "No files to analyse at path: ".red().bold(),
            scripts_root.clone().yellow()
        );
        return ExitCode::FAILURE;
    }

    let py_files: Vec<PyFile> = extract_pyfiles(file_sys, paths);
    let scripts_docs = generate_scripts_docs(py_files);
    let modified_readme = update_readme(&readme, scripts_docs);

    if modified_readme != readme {
        if let Err(e) = modified_readme.write(file_sys, readme_path) {
            eprintln!("{} {}", "Failed to write README file: ".red().bold(), e);
            return ExitCode::FAILURE;
        };
        println!("{}", "Modified README.md".yellow().bold());
        return ExitCode::FAILURE;
    }
    println!("{}", "Nothing to modify".green().bold());
    ExitCode::SUCCESS
}

#[derive(Debug, PartialEq, Eq)]
pub enum RetCode {
    NoModification,
    ModifiedReadme,
    FailedParsingFile,
    FailedToWriteReadme,
    InvalidFilename,
    InvalidExtension,
}

#[derive(Debug, Eq, PartialEq)]
struct ReadMe(String);

impl ReadMe {
    pub fn read(file_sys: &mut impl FileSystem, path: &Path) -> Result<Self, io::Error> {
        let allowed_exts = ["md", "rst", "txt"];
        let ext = path
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or("")
            .to_ascii_lowercase();

        let valid_ext = allowed_exts.contains(&ext.as_str());

        let valid_file_name = path
            .file_name()
            .and_then(OsStr::to_str)
            .map(|name| name.to_ascii_uppercase().contains("README"))
            .unwrap_or(false);

        if valid_file_name && valid_ext {
            file_sys.read_to_string(path).map(ReadMe)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "File name does not contain `README` or is not valid extension in {:?}",
                    allowed_exts
                ),
            ))
        }
    }

    pub fn write(&self, file_sys: &mut impl FileSystem, path: &Path) -> Result<(), io::Error> {
        file_sys.write(path, &self.0)
    }
}

impl Deref for ReadMe {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
struct PyFile {
    pub path: PathBuf,
    code: String,
    docstring: String,
}

impl PyFile {
    fn new(path: impl AsRef<Path>, code: &str, docstring: &str) -> Result<Self, RetCode> {
        let path = path.as_ref().to_path_buf();

        if path.extension().and_then(|ext| ext.to_str()) != Some("py") {
            return Err(RetCode::InvalidExtension);
        }

        Ok(Self {
            path,
            code: code.to_string(),
            docstring: docstring.to_string(),
        })
    }
}

fn extract_pyfiles(file_sys: &impl FileSystem, paths: Vec<PathBuf>) -> Vec<PyFile> {
    paths
        .into_iter()
        .filter_map(|path| {
            let code = file_sys.read_to_string(&path).ok()?;
            let docstring = extract_module_docstring(&code);
            PyFile::new(&path, &code, &docstring).ok()
        })
        .collect::<Vec<PyFile>>()
}

fn extract_module_docstring(code: &str) -> String {
    let pattern = Regex::new(r#"(?s)\A[ \t]*(?i:r|u)?"""(.*?)"""#).unwrap();
    pattern
        .captures(code)
        .and_then(|caught| caught.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_default()
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DocInfo {
    path: PathBuf,
    desc: String,
    link: String,
}

impl DocInfo {
    pub fn to_readme(&self) -> String {
        let basename: String = self.path.file_name().unwrap().to_string_lossy().to_string();
        format!("| `{}` | {} | {} |", basename, self.desc, self.link)
    }
}

fn extract_docinfo(py_files: Vec<PyFile>) -> Vec<DocInfo> {
    let mut doc_infos: Vec<DocInfo> = py_files
        .into_par_iter()
        .map(|py_file| {
            let mut desc = String::new();
            let mut link = String::new();

            for line in py_file.docstring.lines() {
                let trimmed_line = line.trim_start();

                if let Some(rest) = trimmed_line.strip_prefix("Description: ") {
                    desc = rest.to_string();
                } else if let Some(rest) = trimmed_line.strip_prefix("Link: ") {
                    link = format!("[Link]({})", rest);
                }
            }
            DocInfo {
                path: py_file.path,
                desc,
                link,
            }
        })
        .collect();
    doc_infos.par_sort_by_key(|s| s.path.clone());
    doc_infos
}

fn create_readme(doc_infos: Vec<DocInfo>) -> String {
    [
        "# Scripts",
        "| Name | Description | Link |",
        "|:---|:---|:---|",
    ]
    .into_iter()
    .map(str::to_string)
    .chain(doc_infos.iter().map(|n| n.to_readme()).collect::<Vec<_>>())
    .chain(std::iter::once("::".to_string()))
    .collect::<Vec<_>>()
    .join("\n")
}

fn generate_scripts_docs(py_files: Vec<PyFile>) -> String {
    create_readme(extract_docinfo(py_files))
}

fn update_readme(readme: &ReadMe, scripts_docs: String) -> ReadMe {
    let pattern = Regex::new(r"(?s)(?m)^# Scripts.*?^::").expect("valid regex");

    let updated = if pattern.is_match(&readme.0) {
        pattern.replace(&readme.0, scripts_docs).into_owned()
    } else {
        format!("{}\n\n{}", readme.0, scripts_docs)
    };
    ReadMe(updated)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_scripts_docs() {
        let py_files = [
            PyFile::new(
                PathBuf::from("some/python/file1.py"),
                "",
                "Description: This is a description\n\nLink: some_link.com/link1",
            ),
            PyFile::new(
                PathBuf::from("some/python/file3.py"),
                "",
                "missing description start\n\nLink: some_other_link.com/link2",
            ),
            PyFile::new(
                PathBuf::from("some/python/file2.py"),
                "",
                "Description: This is another description\n\n",
            ),
        ]
        .into_iter()
        .flatten()
        .collect();

        let expected_readme = [
            "# Scripts",
            "| Name | Description | Link |",
            "|:---|:---|:---|",
            "| `file1.py` | This is a description | [Link](some_link.com/link1) |",
            "| `file2.py` | This is another description |  |",
            "| `file3.py` |  | [Link](some_other_link.com/link2) |",
            "::",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<String>>()
        .join("\n");

        assert_eq!(generate_scripts_docs(py_files), expected_readme);
    }
}
