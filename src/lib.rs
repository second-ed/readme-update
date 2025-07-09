use pyo3::prelude::*;

use crate::core::{create_readme, extract_docinfo, extract_pyfiles, list_files};

#[pyfunction]
fn entry_point(path: String) -> PyResult<()> {
    let infos = create_readme(extract_docinfo(extract_pyfiles(list_files(path))));
    println!("{}", infos);
    Ok(())
}

#[pymodule]
fn rs_py_experiment(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(entry_point, m)?)?;
    Ok(())
}

mod core {
    use rayon::prelude::*;
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
            format!("| {} | {} | {} |", basename, self.desc, self.link)
        }
    }

    pub fn extract_docinfo(py_files: Vec<PyFile>) -> Vec<DocInfo> {
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

    pub fn create_readme(doc_infos: Vec<DocInfo>) -> String {
        let readme = std::iter::once("# Scripts".to_string())
            .chain(doc_infos.iter().map(|n| n.to_readme()).collect::<Vec<_>>())
            .chain(std::iter::once("::".to_string()))
            .collect::<Vec<_>>()
            .join("\n");
        readme
    }
}
