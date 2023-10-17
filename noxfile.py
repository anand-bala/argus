import os
import shutil
import sys
from pathlib import Path

import nox
import nox.registry

CURRENT_DIR = Path(__file__).parent.resolve()
TARGET_DIR = CURRENT_DIR / "target"
COVERAGE_DIR = TARGET_DIR / "debug/coverage"

if shutil.which("mamba"):
    nox.options.default_venv_backend = "mamba"
else:
    nox.options.default_venv_backend = "conda"

nox.options.reuse_existing_virtualenvs = True

ENV = dict(
    CARGO_TARGET_DIR=str(TARGET_DIR),
)

DEFAULT_PYTHON = f"{sys.version_info.major}.{sys.version_info.minor}"
if os.environ.get("CI"):
    PYTHONS = [f"3.{i}" for i in range(8, 12 + 1)]
    ENV["RUST_BACKTRACE"] = "1"
else:
    PYTHONS = [DEFAULT_PYTHON]


@nox.session(python=False)
def clean(session: nox.Session):
    session.run(
        "git",
        "clean",
        "-e",
        "!.envrc",
        "-e",
        "!.nox/**",
        "-e",
        "!.nox",
        "-dfX",
        external=True,
    )


@nox.session
def dev(session: nox.Session):
    session.conda_install("pre-commit")
    session.run("pre-commit", "install")


@nox.session(python=DEFAULT_PYTHON)
def docs(session: nox.Session):
    session.conda_install(
        "sphinx",
        "furo",
        "sphinx-copybutton",
        "myst-parser",
    )
    session.install("sphinx-autoapi", "sphinx-multiversion")
    with session.chdir(CURRENT_DIR / "pyargus"):
        session.install("-e", ".")
    session.run("sphinx-multiversion", "docs", "_site", "-b", "html")
    session.run("cp", "./docs/_templates/index-redirect.html", "_site/index.html")


@nox.session(tags=["style", "fix", "rust"], python=False)
def rustfmt(session: nox.Session):
    if len(session.posargs) > 0:
        session.run("cargo", "+nightly", "fmt", *session.posargs, external=True)
    else:
        session.run("cargo", "+nightly", "fmt", "--all", external=True)


@nox.session(tags=["lint", "fix", "rust"], python=False)
def cargo_check(session: nox.Session):
    session.run("cargo", "+nightly", "fmt", "--all", "--check", external=True)
    session.run("cargo", "+nightly", "check", "--workspace", external=True)
    session.run("cargo", "+nightly", "clippy", "--workspace", external=True)


@nox.session(tags=["style", "fix", "python"], python=DEFAULT_PYTHON)
def black(session: nox.Session):
    session.conda_install("black")
    session.run("black", str(__file__))
    with session.chdir(CURRENT_DIR / "pyargus"):
        session.run("black", ".")


@nox.session(tags=["style", "fix", "python"], python=DEFAULT_PYTHON)
def isort(session: nox.Session):
    session.conda_install("isort")
    with session.chdir(CURRENT_DIR / "pyargus"):
        session.run("isort", ".")


@nox.session(tags=["lint", "python"], python=DEFAULT_PYTHON)
def flake8(session: nox.Session):
    session.conda_install(
        "flake8",
        "Flake8-pyproject",
        "flake8-bugbear",
        "flake8-pyi",
    )
    with session.chdir(CURRENT_DIR / "pyargus"):
        session.run("flake8")


@nox.session(tags=["lint", "fix", "python"], python=DEFAULT_PYTHON)
def ruff(session: nox.Session):
    session.conda_install("ruff")
    with session.chdir(CURRENT_DIR / "pyargus"):
        session.run("ruff", "--fix", "--exit-non-zero-on-fix", ".")


@nox.session(tags=["lint", "python"], python=PYTHONS)
def mypy(session: nox.Session):
    session.conda_install("mypy", "typing-extensions", "pytest", "hypothesis", "lark")
    session.env.update(ENV)

    with session.chdir(CURRENT_DIR / "pyargus"):
        session.install("-e", ".")
        session.run("mypy", ".")
        # session.run(
        #     "stubtest",
        #     "argus",
        #     "--allowlist",
        #     str(CURRENT_DIR / "pyargus/stubtest_allow.txt"),
        # )


@nox.session(python=PYTHONS)
def tests(session: nox.Session):
    session.conda_install("pytest", "hypothesis", "lark", "maturin")
    session.env.update(ENV)
    try:
        session.run(
            "cargo",
            "test",
            "--release",
            "--workspace",
            "--exclude",
            "pyargus",
            external=True,
        )
    except Exception:
        ...
    try:
        session.run(
            "maturin",
            "develop",
            "--release",
            "-m",
            "./pyargus/Cargo.toml",
            "-E",
            "test",
            silent=True,
        )
        with session.chdir(CURRENT_DIR / "pyargus"):
            session.run("pytest", ".", "--hypothesis-explain")
    except Exception:
        ...


@nox.session(python=DEFAULT_PYTHON)
def coverage(session: nox.Session):
    session.conda_install("pytest", "coverage", "hypothesis", "lark", "maturin", "lcov")
    session.run("cargo", "install", "grcov", external=True, silent=True)

    session.env.update(ENV)
    session.env.update(
        dict(
            RUSTC_BOOTSTRAP="1",
            CARGO_INCREMENTAL="0",
            RUSTFLAGS=" ".join(
                [
                    "-Zprofile",
                    "-Ccodegen-units=1",
                    "-Copt-level=0",
                    "-Clink-dead-code",
                    "-Coverflow-checks=off",
                    "-Zpanic_abort_tests",
                    "-Cpanic=unwind",
                ]
            ),
            RUSTDOCFLAGS="-Cpanic=abort",
            LLVM_PROFILE_FILE="argus-%p-%m.profraw",
        )
    )
    session.run("cargo", "+nightly", "build", external=True, silent=True)
    session.run(
        "maturin",
        "develop",
        "-m",
        "./pyargus/Cargo.toml",
        "-E",
        "test",
        silent=True,
    )
    try:
        COVERAGE_DIR.mkdir(exist_ok=True)
        session.run(
            "cargo",
            "+nightly",
            "test",
            "--workspace",
            "--exclude",
            "pyargus",
            external=True,
            silent=True,
        )
    except Exception:
        ...

    try:
        session.run(
            "coverage",
            "run",
            "--source",
            "pyargus/argus,pyargus/src",
            "-m",
            "pytest",
            silent=True,
        )
    except Exception:
        ...

    session.run(
        "grcov",
        ".",
        "-s",
        f"{CURRENT_DIR}",
        "--binary-path",
        f"{TARGET_DIR}/debug",
        "--filter",
        "covered",
        "-t",
        "lcov",
        "--branch",
        "--ignore-not-existing",
        "--ignore",
        f"{Path.home()}/.cargo/**",
        "-o",
        "rust.lcov",
        external=True,
    )
    session.run("coverage", "lcov", "-o", "python.lcov")

    session.run(
        "genhtml",
        "--show-details",
        "--highlight",
        "--ignore-errors",
        "source",
        "--legend",
        "-o",
        "htmlcov/",
        *map(str, CURRENT_DIR.glob("*.lcov")),
    )


skip = {"dev", "clean", "coverage"}
nox.options.sessions = [key for key in nox.registry.get() if key not in skip]
