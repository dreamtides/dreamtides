set positional-arguments

code-review: check-format build workspace-lints clippy test check-docs unity-tests

review: check-format build workspace-lints clippy test check-docs

check:
    cargo check --manifest-path rules_engine/Cargo.toml --workspace --all-targets --all-features

check-warnings:
    RUSTFLAGS="--deny warnings" cargo check --manifest-path rules_engine/Cargo.toml --workspace --all-targets --all-features

build:
    cargo build --manifest-path rules_engine/Cargo.toml --all-targets --all-features

dev:
    cargo run --manifest-path rules_engine/Cargo.toml --bin "dev_server"

release:
    cargo run --manifest-path rules_engine/Cargo.toml --release --bin "dev_server"

watch:
    cargo watch -C rules_engine -x "run --bin dev_server" --ignore dreamtides.json

watch-release:
    cargo watch -C rules_engine -x "run --release --bin dev_server" --ignore dreamtides.json

test:
    cargo test --manifest-path rules_engine/Cargo.toml

doc:
    cargo doc --manifest-path rules_engine/Cargo.toml

workspace-lints:
    cargo workspace-lints rules_engine/Cargo.toml

schema:
    cargo run --manifest-path rules_engine/Cargo.toml --bin "schema_generator" > schema.json
    quicktype --lang cs --src-lang schema -t SchemaTypes --namespace Dreamtides.Schema --csharp-version 6 --array-type list --features complete --check-required -o client/Assets/Dreamtides/Schema/Schema.cs schema.json
    rm schema.json

plugin_out := "client/Assets/Plugins"
target_ios := "aarch64-apple-ios"

ios-plugin:
    cargo build --manifest-path rules_engine/Cargo.toml -p plugin --release --target={{target_ios}}
    mkdir -p {{plugin_out}}/iOS/
    cp rules_engine/target/{{target_ios}}/release/libplugin.a {{plugin_out}}/iOS

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
target_ar := join(android_ndk, toolchains, llvm_toolchain, "bin", "llvm-ar")

android-plugin:
    #!/usr/bin/env bash
    # Note: builds for Android that use native plugins must use IL2CPP
    # This is only arm64, need to do arm7 at some point too
    echo "Using linker:\n {{android_linker}}"
    CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="{{android_linker}}" TARGET_AR="{{target_ar}}" TARGET_CC="{{android_linker}}" cargo build --manifest-path rules_engine/Cargo.toml --release -p plugin --target={{target_android}}
    mkdir -p {{plugin_out}}/Android
    # You see, standalone osx builds *do not* want the lib prefix but android fails *without* it...
    cp rules_engine/target/{{target_android}}/release/libplugin.so {{plugin_out}}/Android/

target_arm := "aarch64-apple-darwin"
target_x86 := "x86_64-apple-darwin"

# Builds mac plugin. This will only work in the editor if the "build profile" is set to macOS
mac-plugin:
    # you may need to run codesign --deep -s - -f my.app before running
    cargo build --manifest-path rules_engine/Cargo.toml -p plugin --release --target={{target_arm}}
    cargo build --manifest-path rules_engine/Cargo.toml -p plugin --release --target={{target_x86}}
    # lib prefix breaks on mac standalone
    lipo -create -output plugin.bundle \
        rules_engine/target/{{target_arm}}/release/libplugin.dylib \
        rules_engine/target/{{target_x86}}/release/libplugin.dylib
    mkdir -p {{plugin_out}}/OSX/
    mv plugin.bundle {{plugin_out}}/OSX/

target_windows := "x86_64-pc-windows-gnu"

# You may need to install mingw, e.g. via brew install mingw-w64
# Note that the plugin name cannot conflict with any .asmdef file
windows-plugin:
    # Note that you cannot use IL2CPP when cross-compiling for windows
    cargo build --manifest-path rules_engine/Cargo.toml --release -p plugin --target {{target_windows}}
    mkdir -p {{plugin_out}}/Windows/
    cp rules_engine/target/{{target_windows}}/release/plugin.dll {{plugin_out}}/Windows/

windows-dev-server:
    cargo build --manifest-path rules_engine/Cargo.toml --release --bin dev_server --target {{target_windows}}
    mkdir -p release_builds/
    cp rules_engine/target/{{target_windows}}/release/dev_server.exe release_builds/

plugins: ios-plugin android-plugin mac-plugin windows-plugin

clippy:
  cargo clippy --manifest-path rules_engine/Cargo.toml --workspace -- -D warnings -D clippy::all

fix:
  cargo clippy --manifest-path rules_engine/Cargo.toml --fix --allow-dirty -- -D warnings -D clippy::all

benchmark *args='':
  cargo criterion --manifest-path rules_engine/Cargo.toml "$@"

parser *args='':
  cargo run --manifest-path rules_engine/Cargo.toml --bin "parser_cli" -- "$@"

parser-release *args='':
  cargo run --manifest-path rules_engine/Cargo.toml --release --bin "parser_cli" -- "$@"

tabula *args='':
  cargo run --manifest-path rules_engine/Cargo.toml --bin "tabula_cli" -- --key-file ./service_account_key.json --spreadsheet-id "1AqSIZ_kwo3H3vfNsG-ubPV5GobkPVV_XEQEKZpRWqrA" --generate rules_engine/src/tabula_ids/src/string_id.rs

insta:
  cd rules_engine && cargo insta review

# Reformats code. Requires nightly because several useful options (e.g. imports_granularity) are
# nightly-only
fmt:
    cd rules_engine && cargo +nightly fmt

check-format:
    cd rules_engine && cargo +nightly fmt -- --check

check-docs:
    RUSTDOCFLAGS="-D rustdoc::broken-intra-doc-links -D rustdoc::private-intra-doc-links -D rustdoc::bare-urls" cargo doc --manifest-path rules_engine/Cargo.toml --all

unity-tests: mac-plugin
    ./client/scripts/test.py

outdated:
    # Check for outdated dependencies, consider installing cargo-edit and running 'cargo upgrade' if this fails
    cargo outdated ---manifest-path rules_engine/Cargo.toml -exit-code 1

upgrade:
    cargo upgrade --manifest-path rules_engine/Cargo.toml --workspace

machete:
    cargo machete --manifest-path rules_engine/Cargo.toml --fix

remove-unused-deps: machete

internal-clean:
  rm -rf rules_engine/target/debug
  rm -rf rules_engine/target/release
  rm -rf rules_engine/target/tmp
  rm -rf rules_engine/target/release-with-debug
  rm -rf ~/dreamtides_tests/rules_engine/target

clean-test-dir:
  rm -rf ~/dreamtides_tests/rules_engine/target

clean: internal-clean

build-release-with-debug:
    cargo build --manifest-path rules_engine/Cargo.toml --no-default-features --bin client  --profile=release-with-debug

samply: build-release-with-debug
    samply record ./rules_engine/target/release-with-debug/client

samply-matchup *args='':
    cargo build --manifest-path rules_engine/Cargo.toml --bin run_matchup  --profile=release-with-debug
    samply record ./rules_engine/target/release-with-debug/run_matchup -- "$@"

# Example: just samply-battle-benchmark ai_core_11
samply-battle-benchmark *args='':
    #!/bin/zsh
    rm ./rules_engine/target/release/deps/battle_benchmarks-*
    cargo criterion --manifest-path rules_engine/Cargo.toml --no-run
    ALL_BENCHMARKS=`echo ./rules_engine/target/release/deps/battle_benchmarks-*`
    echo "Found benchmark binaries" $ALL_BENCHMARKS
    BENCHMARK=`echo ./rules_engine/target/release/deps/battle_benchmarks-*([1])`
    echo "Running" $BENCHMARK
    samply record $BENCHMARK --bench --profile-time 10 "$@"

@nim *args='':
    cargo run --manifest-path rules_engine/Cargo.toml --bin run_nim -- $@

matchup *args='':
    cargo run --manifest-path rules_engine/Cargo.toml --release --bin run_matchup -- "$@"

card-images:
    ./client/scripts/card_images.py --input client/Assets/ThirdParty/GameAssets/SourceImages --output client/Assets/ThirdParty/GameAssets/CardImages -r 50

resize-images:
    ./client/scripts/resize.py --input client/Assets/ThirdParty/GameAssets/SourceImages

graphviz:
    dot -Tpng rules_engine/search_graph.dot -o rules_engine/search_graph.png && open rules_engine/search_graph.png

bench-evaluate:
    cargo criterion --manifest-path rules_engine/Cargo.toml -p battle_benchmarks -- ai_evaluate/ai_evaluate

bench-s5:
    cargo criterion --manifest-path rules_engine/Cargo.toml -p battle_benchmarks -- ai_starting_5/ai_starting_5

bench-c11:
    cargo criterion --manifest-path rules_engine/Cargo.toml -p battle_benchmarks -- ai_core_11/ai_core_11

bench-full:
    cargo criterion --manifest-path rules_engine/Cargo.toml -p battle_benchmarks -- ai_full/ai_full

symlinks:
     ./rules_engine/scripts/symlinks.py -d ~/Documents/dttmp rules_engine/target client/Library

test-determinism *args='':
    cargo run --manifest-path rules_engine/Cargo.toml --release --bin test_determinism -- $@

commit *args='':
    git add -A && git commit -a -m "$@"

pull:
    git pull

coverage:
    cargo tarpaulin --manifest-path rules_engine/Cargo.toml --out Html

rsync-for-review:
    rsync --delete --stats --copy-links -avqr --exclude='rules_engine/target' ./rules_engine ~/dreamtides_tests/
    echo $'\a'
    rsync --delete --stats --copy-links -avqr --exclude='client/Library' --exclude='client/test_output' --exclude='client/Assets/ThirdParty' ./client ~/dreamtides_tests/
    echo $'\a'
    cp justfile ~/dreamtides_tests/
    echo $'\a'

rsync-third-party:
    rsync --delete --stats --copy-links -avqr --exclude='rules_engine/target' ./rules_engine ~/dreamtides_tests/
    echo $'\a'
    rsync --delete --stats --copy-links -avqr --exclude='client/Library' --exclude='client/test_output' ./client ~/dreamtides_tests/
    echo $'\a'
    cp justfile ~/dreamtides_tests/
    echo $'\a'

code-review-rsync: rsync-for-review
    cd ~/dreamtides_tests && just code-review || (osascript -e 'display dialog "Review failed" with icon stop'; exit 1)

unity-test-rsync: rsync-for-review
    cd ~/dreamtides_tests && just unity-tests
