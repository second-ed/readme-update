use crate::core::adapters::RealFileSystem;
use crate::core::domain::{main, RetCode};
use pyo3::prelude::*;
use std::path::Path;

#[pyfunction]
fn py_main(scripts_root: String, readme_path: String) -> PyResult<i8> {
    let mut file_sys = RealFileSystem;
    match main(&mut file_sys, scripts_root, Path::new(&readme_path)) {
        RetCode::NoModification => Ok(0),
        RetCode::ModifiedReadme => Ok(1),
        RetCode::NoPyFiles => Ok(2),
        RetCode::FailedParsingFile => Ok(3),
        RetCode::FailedToWriteReadme => Ok(4),
    }
}

#[pymodule]
fn readme_update(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_main, m)?)?;
    Ok(())
}
