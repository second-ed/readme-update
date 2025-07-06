use pyo3::prelude::*;

use crate::core::{extract_docinfo, extract_pyfiles, list_files};

#[pyfunction]
fn entry_point(path: String) -> PyResult<()> {
    dbg!(extract_docinfo(extract_pyfiles(list_files(path))));
    Ok(())
}

#[pymodule]
fn rs_py_experiment(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(entry_point, m)?)?;
    Ok(())
}

mod core {
    use regex::Regex;
    use std::{ffi::OsStr, fs, path::PathBuf};
    use walkdir::WalkDir;

    pub fn list_files(path: String) -> Vec<PathBuf> {
        WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension() == Some(OsStr::new("py")))
            .map(|entry| entry.path().to_path_buf())
            .collect()
    }

    #[derive(Debug)]
    pub struct PyFile {
        path: PathBuf,
        code: String,
        docstring: String,
    }

    pub fn extract_pyfiles(paths: Vec<PathBuf>) -> Vec<PyFile> {
        paths
            .into_iter()
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
    #[derive(Debug)]
    pub struct DocInfo {
        path: PathBuf,
        desc: String,
        link: String,
    }

    pub fn extract_docinfo(py_files: Vec<PyFile>) -> Vec<DocInfo> {
        py_files
            .into_iter()
            .map(|py_file| {
                let mut desc = String::new();
                let mut link = String::new();

                for line in py_file.docstring.lines() {
                    let trimmed_line = line.trim_start();

                    if let Some(rest) = trimmed_line.strip_prefix("Description: ") {
                        desc = rest.to_string();
                    } else if let Some(rest) = trimmed_line.strip_prefix("Link: ") {
                        link = rest.to_string();
                    }
                }
                DocInfo {
                    path: py_file.path,
                    desc,
                    link,
                }
            })
            .collect()
    }
}
