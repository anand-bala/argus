# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = "Argus"
copyright = "2023, Anand Balakrishnan"
author = "Anand Balakrishnan"

version = "v0.1.0"
release = "0.1.0"

extensions = [
    "autoapi.extension",
    "sphinx.ext.doctest",
    "myst_parser",
    "sphinx_copybutton",
    "sphinx.ext.inheritance_diagram",
]

templates_path = ["_templates"]
exclude_patterns = ["_build", "Thumbs.db", ".DS_Store"]

html_theme = "pydata_sphinx_theme"
html_static_path = ["_static"]
html_theme_options = {"use_edit_page_button": True, "github_url": "https://github.com/anand-bala/argus"}
html_context = {
    "github_user": "anand-bala",
    "github_repo": "argus",
    "github_version": "v0.1.0",
    "doc_path": "docs",
}

source_suffix = {
    ".rst": "restructuredtext",
    ".txt": "markdown",
    ".md": "markdown",
}

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
