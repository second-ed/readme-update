import argparse
import os
import sys
import readme_update

if __name__ == "__main__":
    parser = argparse.ArgumentParser()

    parser.add_argument(
        "--scripts-root",
        type=os.path.abspath,
        required=True,
        help="Path to the root of the scripts to generate the table for.",
    )
    parser.add_argument(
        "--readme-path",
        type=os.path.abspath,
        default="./README.md",
        help="Path to the readme file.",
    )
    parser.add_argument(
        "--table-fields",
        type=list,
        default=["Description", "Link"],
        help="Fields to dynamically add to the README.md table.",
    )
    parser.add_argument(
        "--link-fields",
        type=list,
        default=["Link"],
        help="Which of the provided table fields should be rendered as links.",
    )
    args = parser.parse_args()
    sys.exit(
        int(
            readme_update.py_main(
                args.scripts_root, args.readme_path, args.table_fields, args.link_fields
            )
        )
    )
