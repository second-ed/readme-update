use crate::core::adapters::FileSystem;
use colored::Colorize;
use rayon::prelude::*;
use regex::Regex;
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
};

pub fn main(
    file_sys: &mut impl FileSystem,
    scripts_root: String,
    readme_path: &Path,
    table_fields: &[String],
    link_fields: &[String],
) -> RetCode {
    let table_set: HashSet<_> = table_fields.iter().collect();
    if !link_fields.iter().all(|f| table_set.contains(f)) {
        eprintln!(
            "{} {:?} {} {:?}",
            "Not all link fields are present in table fields. Link fields: "
                .red()
                .bold(),
            link_fields,
            "Table fields: ".red().bold(),
            table_fields
        );
        return RetCode::InvalidLinkFields;
    }

    let readme = match ReadMe::parse(file_sys, readme_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{} {}", "Failed to read README file: ".red().bold(), e);
            return RetCode::FailedParsingFile;
        }
    };

    let paths = file_sys.list_py_files(&scripts_root);
    if paths.is_empty() {
        println!(
            "{} `{}`",
            "No files to analyse at path: ".red().bold(),
            scripts_root.clone().yellow()
        );
        return RetCode::NoPyFiles;
    }

    let py_files: Vec<PyFile> = extract_pyfiles(file_sys, paths);
    let scripts_docs = generate_scripts_docs(py_files, table_fields, link_fields);
    let modified_readme = update_readme(&readme, scripts_docs);

    if modified_readme != readme {
        if let Err(e) = modified_readme.write(file_sys, readme_path) {
            eprintln!("{} {}", "Failed to write README file: ".red().bold(), e);
            return RetCode::FailedToWriteReadme;
        };
        println!("{}", "Modified README.md".yellow().bold());
        return RetCode::ModifiedReadme;
    }
    println!("{}", "Nothing to modify".green().bold());
    RetCode::NoModification
}

#[derive(Debug, PartialEq, Eq)]
pub enum RetCode {
    NoModification,
    ModifiedReadme,
    NoPyFiles,
    FailedParsingFile,
    FailedToWriteReadme,
    InvalidLinkFields,
}

#[derive(Debug, Eq, PartialEq)]
struct ReadMe(String);

impl ReadMe {
    pub fn parse(file_sys: &mut impl FileSystem, path: &Path) -> Result<Self, io::Error> {
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

#[derive(Debug)]
struct PyFile {
    pub path: PathBuf,
    _code: String,
    docstring: String,
}

impl PyFile {
    fn new(path: impl AsRef<Path>, code: &str, docstring: &str) -> Self {
        let path = path.as_ref().to_path_buf();
        Self {
            path,
            _code: code.to_string(),
            docstring: docstring.to_string(),
        }
    }
}

fn extract_pyfiles(file_sys: &impl FileSystem, paths: Vec<PathBuf>) -> Vec<PyFile> {
    paths
        .into_iter()
        .filter_map(|path| {
            file_sys.read_to_string(&path).ok().as_ref().map(|code| {
                let docstring = extract_module_docstring(code);
                PyFile::new(path, code, &docstring)
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

#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub struct TableValue {
    value: String,
    is_link: bool,
}

impl TableValue {
    pub fn new(value: &str, is_link: bool) -> Self {
        Self {
            value: value.to_string(),
            is_link,
        }
    }

    pub fn to_readme_entry(&self) -> String {
        if self.is_link {
            format!("[Link]({})", self.value)
        } else {
            self.value.clone()
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct DocInfo {
    path: PathBuf,
    table_fields: HashMap<String, TableValue>,
}

impl DocInfo {
    pub fn to_readme(&self, table_fields: &[String]) -> String {
        let basename: String = self.path.file_name().unwrap().to_string_lossy().to_string();
        let cols = table_fields
            .iter()
            .map(|k| {
                self.table_fields
                    .get(k)
                    .cloned()
                    .unwrap_or_default()
                    .to_readme_entry()
            })
            .collect::<Vec<_>>()
            .join(" | ");
        format!("| `{}` | {} |", basename, cols)
    }
}

impl Ord for DocInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path.cmp(&other.path)
    }
}

impl PartialOrd for DocInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn extract_docinfo(
    py_files: Vec<PyFile>,
    table_fields: &[String],
    link_fields: &[String],
) -> Vec<DocInfo> {
    let mut doc_infos: Vec<DocInfo> = py_files
        .into_par_iter()
        .map(|py_file| {
            let mut doc_fields = HashMap::new();

            for line in py_file.docstring.lines() {
                let trimmed_line = line.trim();
                for field in table_fields.iter() {
                    let prefix = format!("{field}: ");
                    if let Some(rest) = trimmed_line.strip_prefix(&prefix) {
                        doc_fields.insert(
                            field.clone(),
                            TableValue::new(rest, link_fields.contains(field)),
                        );
                    }
                }
            }
            DocInfo {
                path: py_file.path,
                table_fields: doc_fields,
            }
        })
        .collect();
    doc_infos.par_sort();
    doc_infos
}

fn create_readme(doc_infos: Vec<DocInfo>, table_fields: &[String]) -> String {
    let header = format!("| Name | {} |", table_fields.join(" | "));
    let separator = format!("|{}|", vec![":---"; table_fields.len() + 1].join("|"));

    std::iter::once("# Scripts".to_string())
        .chain(std::iter::once(header))
        .chain(std::iter::once(separator))
        .chain(
            doc_infos
                .iter()
                .map(|n| n.to_readme(table_fields))
                .collect::<Vec<_>>(),
        )
        .chain(std::iter::once("::".to_string()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn generate_scripts_docs(
    py_files: Vec<PyFile>,
    table_fields: &[String],
    link_fields: &[String],
) -> String {
    create_readme(
        extract_docinfo(py_files, table_fields, link_fields),
        table_fields,
    )
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
        let py_files = vec![
            PyFile::new(
                "some/python/file1.py",
                "",
                "Description: This is a description\n\nLink: some_link.com/link1",
            ),
            PyFile::new(
                "some/python/file3.py",
                "",
                "missing description start\n\nLink: some_other_link.com/link2",
            ),
            PyFile::new(
                "some/python/file2.py",
                "",
                "Description: This is another description\n\n",
            ),
        ];

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

        assert_eq!(
            generate_scripts_docs(
                py_files,
                &["Description".to_string(), "Link".to_string()],
                &["Link".to_string()]
            ),
            expected_readme
        );
    }
}
