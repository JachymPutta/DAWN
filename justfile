default: build lint test

build:
  echo Building nix dap adapter..
  cargo build --workspace --examples --bins --tests

test:
  echo Testing nix dap adapter...
  cargo test --workspace -- --nocapture --test-threads=1

clean:
  echo Cleaning...
  cargo clean

lint:
  echo Linting…
  cargo clippy --workspace --examples --bins --tests

fmt:
  cargo fmt

fix:
  cargo fix --allow-dirty --allow-staged

doc:
  cargo doc --workspace --open
