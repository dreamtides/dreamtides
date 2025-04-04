set positional-arguments

code-review: check-format build clippy test check-docs

check:
    cargo check --manifest-path engine/Cargo.toml --workspace --all-targets --all-features

check-warnings:
    RUSTFLAGS="--deny warnings" cargo check --manifest-path engine/Cargo.toml --workspace --all-targets --all-features

build:
    cargo build --manifest-path engine/Cargo.toml --all-targets --all-features

dev:
    cargo run --manifest-path engine/Cargo.toml --bin "dev_server"

watch:
    cargo watch -C engine -x "run --bin dev_server"

test:
    cargo test --manifest-path engine/Cargo.toml

doc:
    cargo doc --manifest-path engine/Cargo.toml

schema:
    cargo run --manifest-path engine/Cargo.toml --bin "schema_generator" > schema.json
    quicktype --lang cs --src-lang schema -t SchemaTypes --namespace Dreamcaller.Schema --csharp-version 6 --array-type list --features complete --check-required -o client/Assets/Dreamcaller/Schema/Schema.cs schema.json
    rm schema.json

plugin_out := "client/Assets/Plugins"
target_ios := "aarch64-apple-ios"

ios-plugin:
    cargo build --manifest-path engine/Cargo.toml -p plugin --release --target={{target_ios}}
    mkdir -p {{plugin_out}}/iOS/
    cp engine/target/{{target_ios}}/release/libplugin.a {{plugin_out}}/iOS

# install via rustup target add aarch64-linux-android
target_android := "aarch64-linux-android"

# Android NDK path
# e.g. /Users/name/Library/Android/sdk/ndk/24.0.8215888
# e.g. /Applications/Unity/Hub/Editor/2021.3.3f1/PlaybackEngines/AndroidPlayer/NDK
android_ndk := env_var_or_default("ANDROID_NDK", "")

llvm_toolchain := if os() == "macos" {
        "darwin-x86_64"
    } else if os() == "linux" {
        "linux-x86_64"
    } else {
        "OS not supported"
    }

# If you get an error about libgcc not being found, see here:
# https://github.com/rust-lang/rust/pull/85806
# "Find directories containing file libunwind.a and create a text file called
# libgcc.a with the text INPUT(-lunwind)"

# Need to set up target toolchain, see https://github.com/briansmith/ring/issues/897

toolchains := "toolchains/llvm/prebuilt"

# Required for Android build:
android_linker := join(android_ndk, toolchains, llvm_toolchain, "bin", "aarch64-linux-android29-clang")
target_ar := join(android_ndk, toolchains, llvm_toolchain, "bin", "aarch64-linux-android-ar")

android-plugin:
    #!/usr/bin/env bash
    # Note: builds for Android that use native plugins must use IL2CPP
    # This is only arm64, need to do arm7 at some point too
    echo "Using linker:\n {{android_linker}}"
    CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="{{android_linker}}" TARGET_AR="{{target_ar}}" TARGET_CC="{{android_linker}}" cargo build --manifest-path engine/Cargo.toml --release -p plugin --target={{target_android}}
    mkdir -p {{plugin_out}}/Android
    # You see, standalone osx builds *do not* want the lib prefix but android fails *without* it...
    cp engine/target/{{target_android}}/release/libplugin.so {{plugin_out}}/Android/

target_arm := "aarch64-apple-darwin"
target_x86 := "x86_64-apple-darwin"

# Builds mac plugin. This will only work in the editor if the "build profile" is set to macOS
mac-plugin:
    # you may need to run codesign --deep -s - -f my.app before running
    cargo build --manifest-path engine/Cargo.toml -p plugin --release --target={{target_arm}}
    cargo build --manifest-path engine/Cargo.toml -p plugin --release --target={{target_x86}}
    # lib prefix breaks on mac standalone
    lipo -create -output plugin.bundle \
        engine/target/{{target_arm}}/release/libplugin.dylib \
        engine/target/{{target_x86}}/release/libplugin.dylib
    mkdir -p {{plugin_out}}/OSX/
    mv plugin.bundle {{plugin_out}}/OSX/

plugins: ios-plugin android-plugin mac-plugin

clippy:
  cargo clippy --manifest-path engine/Cargo.toml --workspace -- -D warnings -D clippy::all

benchmark *args='':
  cargo criterion --manifest-path engine/Cargo.toml "$@"

parser *args='':
  cargo run --manifest-path engine/Cargo.toml --bin "parser_cli" -- "$@"

insta:
  cd engine && cargo insta review

# Reformats code. Requires nightly because several useful options (e.g. imports_granularity) are
# nightly-only
fmt:
    cd engine && cargo +nightly fmt

check-format:
    cd engine && cargo +nightly fmt -- --check

check-docs:
    RUSTDOCFLAGS="-D rustdoc::broken-intra-doc-links -D rustdoc::private-intra-doc-links -D rustdoc::bare-urls" cargo doc --manifest-path engine/Cargo.toml --all

outdated:
    # Check for outdated dependencies, consider installing cargo-edit and running 'cargo upgrade' if this fails
    cargo outdated ---manifest-path engine/Cargo.toml -exit-code 1

upgrade:
    cargo upgrade --manifest-path engine/Cargo.toml --workspace

machete:
    cargo machete --manifest-path engine/Cargo.toml --fix

remove-unused-deps: machete

internal-clean:
  rm -rf engine/target/debug
  rm -rf engine/target/release
  rm -rf engine/target/tmp
  rm -rf engine/target/release-with-debug

clean: internal-clean

build-release-with-debug:
    cargo build --manifest-path engine/Cargo.toml --no-default-features --bin client  --profile=release-with-debug

samply: build-release-with-debug
    samply record ./src-tauri/target/release-with-debug/client

samply-benchmark *args='':
    #!/bin/zsh
    cargo criterion --manifest-path engine/Cargo.toml --no-run
    ALL_BENCHMARKS=`echo ./src-tauri/target/release/deps/benchmarks-*`
    echo "Found benchmark binaries" $ALL_BENCHMARKS
    BENCHMARK=`echo ./src-tauri/target/release/deps/benchmarks-*([1])`
    echo "Running" $BENCHMARK
    samply record $BENCHMARK --bench --profile-time 5 "$@"

