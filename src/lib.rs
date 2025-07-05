use pyo3::prelude::*;

use crate::core::{create_file_map, list_files};

#[pyfunction]
fn entry_point(path: String) -> PyResult<()> {
    dbg!(create_file_map(list_files(path)));
    Ok(())
}

#[pymodule]
fn rs_py_experiment(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(entry_point, m)?)?;
    Ok(())
}

mod core {
    use std::{collections::HashMap, ffi::OsStr, fs, path::PathBuf};
    use walkdir::WalkDir;

    pub fn list_files(path: String) -> Vec<PathBuf> {
        WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension() == Some(OsStr::new("py")))
            .map(|entry| entry.path().to_path_buf())
            .collect()
    }

    pub fn create_file_map(paths: Vec<PathBuf>) -> HashMap<PathBuf, String> {
        paths
            .into_iter()
            .filter_map(|path| {
                fs::read_to_string(&path)
                    .ok()
                    .map(|content| (path, content))
            })
            .collect()
    }
}
