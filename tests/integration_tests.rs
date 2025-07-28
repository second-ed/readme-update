use readme_update::core::{
    adapters::FakeFileSystem,
    domain::{main, RetCode},
};
use std::{collections::HashMap, path::PathBuf};
use test_case::test_case;

#[test_case(
    "repo/root/scripts", "repo/root/README.md",
    "# Some readme\n\n# Scripts\n| Name | Description | Link |\n|:---|:---|:---|\n| `file1.py` | some desc | [Link](some-link.com) |\n::",
    RetCode::NoModification,
    "# Some readme\n\n# Scripts\n| Name | Description | Link |\n|:---|:---|:---|\n| `file1.py` | some desc | [Link](some-link.com) |\n::"
)]
#[test_case(
    "repo/root/scripts", "repo/root/README.md", "# Some readme",
    RetCode::ModifiedReadme,
    "# Some readme\n\n# Scripts\n| Name | Description | Link |\n|:---|:---|:---|\n| `file1.py` | some desc | [Link](some-link.com) |\n::"
)]
#[test_case(
    "repo/root/scripts",
    "repo/root/INVALID.md",
    "# Some readme",
    RetCode::FailedParsingFile,
    "# Some readme"
)]
fn test_readme_update(
    scripts_root: &str,
    readme_path: &str,
    readme_str: &str,
    expected_ret_code: RetCode,
    expected_readme: &str,
) {
    let scripts_root = scripts_root.to_string();

    let files: HashMap<PathBuf, String> = vec![
        (
            "repo/root/scripts/file1.py",
            "\"\"\"Description: some desc\n\nLink: some-link.com\"\"\"",
        ),
        (readme_path, readme_str),
    ]
    .into_iter()
    .map(|(k, v)| (PathBuf::from(k), v.to_string()))
    .collect::<HashMap<PathBuf, String>>();
    let mut file_sys = FakeFileSystem::new(files);

    assert_eq!(
        main(&mut file_sys, scripts_root, &PathBuf::from(readme_path)),
        expected_ret_code
    );

    let actual_readme = file_sys.files.get(&PathBuf::from(readme_path));
    assert_eq!(actual_readme, Some(&expected_readme.to_string()));
}
