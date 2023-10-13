# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

import os

project = "Argus"
copyright = "2023, Anand Balakrishnan"
author = "Anand Balakrishnan"


if os.environ.get("CI") is not None:
    # In CI, use Github Action variables
    version = os.environ["GITHUB_REF_NAME"]
else:
    # running locally, just use "dev"
    version = "dev"
release = version

extensions = [
    "autoapi.extension",
    "sphinx.ext.doctest",
    "myst_parser",
    "sphinx_copybutton",
    "sphinx.ext.inheritance_diagram",
    "sphinx_multiversion",
]

templates_path = ["_templates"]
exclude_patterns = ["_build", "Thumbs.db", ".DS_Store"]

html_theme = "furo"
html_static_path = ["_static"]
html_theme_options = {
    "source_repository": "https://github.com/anand-bala/argus/",
    "source_branch": "dev",
    "source_directory": "docs/",
}
html_sidebars = {
    "**": [
        "sidebar/brand.html",
        "sidebar/search.html",
        "sidebar/scroll-start.html",
        "sidebar/navigation.html",
        "sidebar/ethical-ads.html",
        "sidebar/scroll-end.html",
        "versions.html",
    ]
}

source_suffix = {
    ".rst": "restructuredtext",
    ".txt": "markdown",
    ".md": "markdown",
}


# Whitelist pattern for branches (set to None to ignore all branches)
smv_branch_whitelist = r"^dev$"

autoapi_dirs = ["../pyargus/argus"]
autoapi_root = "api"


def skip_members(app, what, name: str, obj, skip, options):
    # print(f"{what} -> {name}")
    if what == "data" and name.endswith("__doc__"):
        skip = True
    elif name.startswith("argus._argus"):
        skip = True
    return skip


def setup(sphinx):
    sphinx.connect("autoapi-skip-member", skip_members)
