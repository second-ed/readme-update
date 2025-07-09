use std::process::{self, ExitCode};

use pyo3::prelude::*;

use crate::core::main;

#[pyfunction]
fn py_main(scripts_root: String, readme_path: String) -> PyResult<i8> {
    match main(scripts_root, readme_path) {
        ExitCode::SUCCESS => Ok(0),
        ExitCode::FAILURE => Ok(1),
        _ => Ok(-1),
    }
}

#[pymodule]
fn rs_py_experiment(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_main, m)?)?;
    Ok(())
}

mod core {
    use colored::Colorize;
    use rayon::prelude::*;
    use regex::Regex;
    use std::process::ExitCode;
    use std::{ffi::OsStr, fs, io, ops::Deref, path::PathBuf};
    use walkdir::WalkDir;

    pub fn main(scripts_root: String, readme_path: String) -> ExitCode {
        let paths: Vec<PathBuf> = list_files(scripts_root);
        // may as well exit early if no readme
        let readme = ReadMeString::read(&readme_path).expect("Failed to read README.md");
        let scripts_docs = generate_scripts_docs(paths);
        let modified_readme = update_readme(&readme, scripts_docs);
        if modified_readme != readme {
            modified_readme
                .write(&readme_path)
                .expect("Failed to write modified README.md");
            println!("{}", "Modified README.md".yellow().bold());
            return ExitCode::FAILURE;
        }
        println!("{}", "Nothing to modify".green().bold());
        ExitCode::SUCCESS
    }

    pub fn list_files(path: String) -> Vec<PathBuf> {
        WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension() == Some(OsStr::new("py")))
            .map(|entry| entry.path().to_path_buf())
            .collect()
    }

    #[derive(Debug)]
    struct PyFile {
        path: PathBuf,
        code: String,
        docstring: String,
    }

    fn extract_pyfiles(paths: Vec<PathBuf>) -> Vec<PyFile> {
        paths
            .into_par_iter()
            .filter_map(|path| {
                fs::read_to_string(&path).ok().map(|code| {
                    let docstring = extract_module_docstring(&code);
                    PyFile {
                        path,
                        code,
                        docstring,
                    }
                })
            })
            .collect()
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

    fn generate_scripts_docs(paths: Vec<PathBuf>) -> String {
        create_readme(extract_docinfo(extract_pyfiles(paths)))
    }
    #[derive(Debug, Eq, PartialEq)]
    struct ReadMeString(String);

    impl ReadMeString {
        pub fn read(path: &String) -> Result<Self, io::Error> {
            fs::read_to_string(path).map(ReadMeString)
        }

        pub fn write(&self, path: &String) -> Result<(), io::Error> {
            fs::write(path, &self.0)
        }
    }

    impl Deref for ReadMeString {
        type Target = str;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    fn update_readme(readme: &ReadMeString, scripts_docs: String) -> ReadMeString {
        let pattern = Regex::new(r"(?s)(?m)^# Scripts.*?^::").expect("valid regex");

        let updated = if pattern.is_match(&readme.0) {
            pattern.replace(&readme.0, scripts_docs).into_owned()
        } else {
            format!("{}\n\n{}", readme.0, scripts_docs)
        };
        ReadMeString(updated)
    }
}
