# readme-update 🦀
[![PyPI Downloads](https://static.pepy.tech/badge/readme-update)](https://pepy.tech/projects/readme-update)

## Tired of updating documentation?
This tool updates your `README.md` with a one line description for each of the python scripts in a directory you point it to (and recursively). It adds the text for lines that start with `"Description: "` and `"Link: "`. It ignores any that don't have the description.

You can supply alternative `--table-fields` to create a table with columns for the provided fields. The `--link-fields` argument is used to signal which of the `--table-fields` should be rendered as links like this `[Link](this/is/the/link)`.

The idea is that links should link to higher level documentation (if it exists).

This can be used as a `pre-commit` for python projects with standalone scripts for specific processes.

It will update in place if the `# Scripts` block exists or else it will append it to the end of the `README.md`

`example_usage.py` shows how to call the script from python.

# Here is an example output from the tool:

# Scripts
| Name | Description | Link |
|:---|:---|:---|
| `example1.py` | This is an example file that links to my own github. | [Link](https://github.com/second-ed) |
| `example2.py` | Some other description. |  |
| `example3.py` |  | [Link](https://doc.rust-lang.org/book/) |
::

# Installation
```shell
pip install readme-update
```
Or
```shell
uv add readme-update
```

# Usage
Assuming its is run from this location.
```shell
root/
  scripts/
    example_script.py
  README.md
```

```shell
uv run -m update_readme \
--scripts-root "./scripts" \
--readme-path "./README.md"
```


# Args
| Argument           | Type                  | Required | Default | Description                                          |
| ------------------ | --------------------- | -------- | ------- | ---------------------------------------------------- |
| `--scripts-root`      | `str`                 | ✅       |  | Path to the root of the scripts to scan           |
| `--readme-path`    | `str`                 | ❌       | `'./README.md'` | Path to the README file that will be modified        |
| `--table-fields` | `list`                 | ❌ | `["Description", "Link"]` | Fields to dynamically add to the README.md table. |
| `--link-fields` | `list`                 | ❌ | `["Link"]` | Which of the provided table fields should be rendered as links. |


# Ret codes
| RetCode               | int | description           |
| ----------------------| --- | --------------------- |
| `NoModification`      | 0   | The Repo Map reflects the current state of the repo. |
| `ModifiedReadme`      | 1   | The README was updated. |
| `NoPyFiles`   | 2   | No python files found at the `scripts-root` location. |
| `FailedParsingFile` | 3   | Failed to read README file  |
| `FailedToWriteReadme`     | 4   | The given `README.md` path does not match the expected basename. |
| `InvalidLinkFields`     | 5   | The given `link_fields` are not a subset of the given `table_fields`. |


# Repo map
```
├── .github
│   └── workflows
│       ├── ci.yaml
│       └── publish.yaml
├── python
│   └── update_readme
│       ├── __init__.py
│       └── __main__.py
├── scripts
│   ├── example1.py
│   ├── example2.py
│   └── example3.py
├── src
│   ├── core
│   │   ├── adapters.rs
│   │   ├── domain.rs
│   │   └── mod.rs
│   ├── api.rs
│   └── lib.rs
├── tests
│   └── integration_tests.rs
├── .pre-commit-config.yaml
├── Cargo.lock
├── Cargo.toml
├── README.md
├── pyproject.toml
└── uv.lock
::
```
