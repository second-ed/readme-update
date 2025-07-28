# Tired of updating documentation?
This tool updates your `README.md` with a one line description for each of the python scripts in a directory you point it to (and recursively). It adds the text for lines that start with `"Description: "` and `"Link: "`. It ignores any that don't have the description.

The idea is that links should link to higher level documentation (if it exists).

This can be used as a `pre-commit` for python projects with standalone scripts for specific processes.

It will update in place if the `# Scripts` block exists or else it will append it to the end of the `README.md`

`example_usage.py` shows how to call the script from python.

# Scripts
| Name | Description | Link |
|:---|:---|:---|
| `example1.py` | This is an example file that links to my own github. | [Link](https://github.com/second-ed) |
| `example2.py` | Some other description. |  |
| `example3.py` |  | [Link](https://doc.rust-lang.org/book/) |
| `example_usage.py` |  |  |
::

# To install the package
```shell
pip install readme-update
```

# Usage
Assuming its is run from this location.
```shell
root/
  scripts/
    example_script.py
  README.md
```

# example_script.py
```python
import readme_update
from pathlib import Path

path = Path(__file__)

readme_update.py_main(
    str(path.parent),
    str(path.parents[1] / "README.md")
)
```


# Repo map
```
├── .github
│   └── workflows
│       ├── ci.yaml
│       └── publish.yaml
├── scripts
│   ├── example1.py
│   ├── example2.py
│   ├── example3.py
│   └── example_usage.py
├── src
│   ├── core
│   │   ├── adapters.rs
│   │   ├── domain.rs
│   │   └── mod.rs
│   ├── api.rs
│   └── lib.rs
├── .pre-commit-config.yaml
├── Cargo.lock
├── Cargo.toml
├── README.md
├── pyproject.toml
└── uv.lock
::
```
