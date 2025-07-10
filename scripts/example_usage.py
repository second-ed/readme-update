"""This is how you'd use the tool."""

import readme_update
from pathlib import Path

path = Path(__file__)

readme_update.py_main(
    str(path.parent),
    str(path.parents[1] / "README.md")
)

