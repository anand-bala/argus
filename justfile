build *ARGS:
  cargo build {{ARGS}}

test *ARGS:
  cargo test {{ARGS}}

pytest *ARGS:
  cd pyargus && pytest {{ARGS}}

check:
  pre-commit run -a
  cd pyargus && stubtest argus

test-coverage $CARGO_INCREMENTAL="0" $RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort" $RUSTDOCFLAGS="-Cpanic=abort" $LLVM_PROFILE_FILE="argus-%p-%m.profraw":
  fd -e gcda -e profraw --no-ignore -x rm
  cargo +nightly build
  cargo +nightly test

html-cov: test-coverage
  grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/

doc:
  cargo doc --no-deps
  fd -e md . doc/ -x rustdoc {}
  fd -e html . doc/ -x mv {} target/doc/argus/

serve-doc: doc
  python3 -m http.server -d target/doc/
