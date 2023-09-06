from pathlib import Path

import nox

nox.options.default_venv_backend = "mamba"
nox.options.sessions = ["check"]
nox.options.reuse_existing_virtualenvs = True

CURRENT_DIR = Path(__file__).parent.resolve()
TARGET_DIR = CURRENT_DIR / "target"
COVERAGE_DIR = TARGET_DIR / "debug/coverage"

ENV = dict(
    CARGO_TARGET_DIR=str(TARGET_DIR),
)


@nox.session
def dev(session: nox.Session):
    session.conda_install("pre-commit")
    session.run("pre-commit", "install")


@nox.session
def check(session: nox.Session):
    session.conda_install("pre-commit")
    session.run("pre-commit", "run", "-a")


@nox.session
def tests(session: nox.Session):
    session.conda_install("pytest", "hypothesis")
    session.env.update(ENV)
    session.install("-e", "./pyargus")
    session.run("cargo", "test", external=True)
    session.run("pytest", "pyargus")


@nox.session
def coverage(session: nox.Session):
    session.conda_install("pytest", "coverage", "hypothesis", "maturin", "lcov")
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
        silent=True,
    )
    try:
        COVERAGE_DIR.mkdir(exist_ok=True)
        session.run("cargo", "+nightly", "test", external=True, silent=True)
    except Exception:
        ...
    finally:
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

    try:
        session.run(
            "coverage", "run", "--source", "pyargus/argus", "-m", "pytest", silent=True
        )
    except Exception:
        ...
    finally:
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
