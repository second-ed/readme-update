use crate::core::adapters::RealFileSystem;
use crate::core::domain::main;
use pyo3::prelude::*;
use std::{path::Path, process::ExitCode};

#[pyfunction]
fn py_main(scripts_root: String, readme_path: String) -> PyResult<i8> {
    let mut file_sys = RealFileSystem;
    match main(&mut file_sys, scripts_root, Path::new(&readme_path)) {
        ExitCode::SUCCESS => Ok(0),
        ExitCode::FAILURE => Ok(1),
        _ => Ok(-1),
    }
}

#[pymodule]
fn readme_update(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_main, m)?)?;
    Ok(())
}
