set positional-arguments

code-review: check-format check-ts-format build clippy test check-docs

check:
    cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets --all-features

check-warnings:
    RUSTFLAGS="--deny warnings" cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets --all-features

build:
    cargo build --manifest-path src-tauri/Cargo.toml --all-targets --all-features

run *args='':
    npm run tauri dev

run-release:
  npm run tauri dev -- --release

# To run under a rust debugger, *first* use this command and then start the rust binary
dev:
  npm run dev

test:
    cargo test --manifest-path src-tauri/Cargo.toml

doc:
    cargo doc --manifest-path src-tauri/Cargo.toml

clippy:
  cargo clippy --manifest-path src-tauri/Cargo.toml --workspace -- -D warnings -D clippy::all

benchmark *args='':
  cargo criterion --manifest-path src-tauri/Cargo.toml "$@"

# Reformats code. Requires nightly because several useful options (e.g. imports_granularity) are
# nightly-only
# Manifest path seems to not work?
fmt: fix-ts-format
    cd src-tauri && cargo +nightly fmt

check-format:
    # Manifest path seems to not work?
    cd src-tauri && cargo +nightly fmt -- --check

lint-ts:
  npx eslint src

check-ts-format:
  npx prettier src --check

fix-ts-format:
  npx prettier src --write

check-docs:
    RUSTDOCFLAGS="-D rustdoc::broken-intra-doc-links -D rustdoc::private-intra-doc-links -D rustdoc::bare-urls" cargo doc --manifest-path src-tauri/Cargo.toml --all

outdated:
    # Check for outdated dependencies, consider installing cargo-edit and running 'cargo upgrade' if this fails
    cargo outdated ---manifest-path src-tauri/Cargo.toml -exit-code 1

upgrade:
    cargo upgrade --manifest-path src-tauri/Cargo.toml --workspace

machete:
    cargo machete --manifest-path src-tauri/Cargo.toml --fix

remove-unused-deps: machete

internal-clean:
  rm -rf src-tauri/target/debug
  rm -rf src-tauri/target/release
  rm -rf src-tauri/target/tmp
  rm -rf src-tauri/target/release-with-debug

clean: internal-clean

build-release-with-debug:
    cargo build --manifest-path src-tauri/Cargo.toml --no-default-features --bin client  --profile=release-with-debug

samply: build-release-with-debug
    samply record ./src-tauri/target/release-with-debug/client

samply-benchmark *args='':
    #!/bin/zsh
    cargo criterion --manifest-path src-tauri/Cargo.toml --no-run
    ALL_BENCHMARKS=`echo ./src-tauri/target/release/deps/benchmarks-*`
    echo "Found benchmark binaries" $ALL_BENCHMARKS
    BENCHMARK=`echo ./src-tauri/target/release/deps/benchmarks-*([1])`
    echo "Running" $BENCHMARK
    samply record $BENCHMARK --bench --profile-time 5 "$@"
