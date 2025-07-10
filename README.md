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

# Some line afterwards
blah blah
