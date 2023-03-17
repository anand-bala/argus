build *ARGS:
  cargo build {{ARGS}}

test *ARGS:
  cargo test {{ARGS}}

fmt:
  fd -e rs -x rustfmt +nightly {}

doc:
  cargo doc
  fd -e md . doc/ -x rustdoc {}
  fd -e html . doc/ -x mv {} target/doc/argus/

serve-doc: doc
  python3 -m http.server -d target/doc/
