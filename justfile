set positional-arguments

code-review: check-format build workspace-lints clippy style-validator test unity-tests

# Run this before pushing
code-review-rsync: rsync-for-review
    cd ~/dreamtides_tests && just code-review || (osascript -e 'display dialog "Review failed" with icon stop'; just clean-test-dir; exit 1)
    just clean-test-dir

review:
    #!/usr/bin/env bash
    if [ "${REVIEW_PERF:-1}" = "0" ]; then
        just review-legacy
    else
        python3 rules_engine/scripts/review_perf_runner.py
    fi

review-legacy: check-snapshots check-format build clippy style-validator test tv-check tv-clippy tv-test

review-verbose: check-snapshots check-format-verbose build-verbose clippy-verbose style-validator-verbose test-verbose tv-check-verbose tv-clippy-verbose tv-test

review-analyze:
    python3 rules_engine/scripts/analyze_review_perf.py

review-analyze-history commits='100' sample='10':
    python3 rules_engine/scripts/analyze_review_perf.py --backfill-commits "{{commits}}" --sample-every "{{sample}}"

check:
    #!/usr/bin/env bash
    output=$(cargo check --manifest-path rules_engine/Cargo.toml --workspace --all-targets --all-features 2>&1)
    if [ $? -eq 0 ]; then
        echo "Check passed"
    else
        echo "$output"
        exit 1
    fi

check-verbose:
    cargo check --manifest-path rules_engine/Cargo.toml --workspace --all-targets --all-features

check-timed:
    cargo check --manifest-path rules_engine/Cargo.toml --timings --workspace --all-targets --all-features

# Analyze this with crox
# https://github.com/rust-lang/measureme/blob/master/crox/README.md
# crox {crate name}-{pid}.mm_profdata
# Chrome > Dev Tools > Performance > Load Profile
check-profiled:
    RUSTFLAGS="-Z self-profile" cargo +nightly check --manifest-path rules_engine/Cargo.toml -j1 --timings --workspace --all-targets --all-features

check-warnings:
    RUSTFLAGS="--deny warnings" cargo check --manifest-path rules_engine/Cargo.toml --workspace --all-targets --all-features

build:
    #!/usr/bin/env bash
    output=$(cargo build --manifest-path rules_engine/Cargo.toml --all-targets --all-features 2>&1)
    if [ $? -eq 0 ]; then
        echo "Build passed"
    else
        echo "$output"
        exit 1
    fi

build-verbose:
    cargo build --manifest-path rules_engine/Cargo.toml --all-targets --all-features

build-release:
    cargo build --manifest-path rules_engine/Cargo.toml --release --all-targets --all-features

build-timed:
    cargo build --manifest-path rules_engine/Cargo.toml --timings --all-targets --all-features

dev:
    cargo run --manifest-path rules_engine/Cargo.toml --bin "dev_server"

release:
    cargo run --manifest-path rules_engine/Cargo.toml --release --bin "dev_server"

watch:
    cargo watch -C rules_engine -x "run --bin dev_server" --ignore dreamtides.json

watch-release:
    cargo watch -C rules_engine -x "run --release --bin dev_server" --ignore dreamtides.json

test: tabula-check
    #!/usr/bin/env bash
    # Set RUST_MIN_STACK for parser_v2 tests which need extra stack space for
    # deep Chumsky parser hierarchies. Limit test parallelism to prevent memory
    # exhaustion in low-memory environments (like Docker).

    # Detect low-memory environment
    if [ -n "${LOW_MEMORY:-}" ] || [ -f /.dockerenv ] || [ "$(uname -s)" = "Linux" ]; then
        TEST_THREADS="--test-threads=1"
    else
        TEST_THREADS=""
    fi

    if [ "${REVIEW_PERF:-0}" = "1" ]; then
        PROFILE_ARGS=()
        if [ -n "$TEST_THREADS" ]; then
            PROFILE_ARGS+=(--test-threads "${TEST_THREADS#--test-threads=}")
        fi
        RUST_MIN_STACK=8388608 python3 rules_engine/scripts/profile_cargo_test.py \
            --manifest-path rules_engine/Cargo.toml \
            --workspace \
            --exclude tv_tests \
            "${PROFILE_ARGS[@]}"
    else
        output=$(RUST_MIN_STACK=8388608 cargo test --manifest-path rules_engine/Cargo.toml --workspace --exclude tv_tests -- $TEST_THREADS 2>&1)
        if [ $? -eq 0 ]; then
            echo "Tests passed"
        else
            echo "$output"
            exit 1
        fi
    fi

test-verbose:
    #!/usr/bin/env bash
    # Detect low-memory environment
    if [ -n "${LOW_MEMORY:-}" ] || [ -f /.dockerenv ] || [ "$(uname -s)" = "Linux" ]; then
        TEST_THREADS="--test-threads=1"
    else
        TEST_THREADS=""
    fi
    RUST_MIN_STACK=8388608 cargo test --manifest-path rules_engine/Cargo.toml --exclude tv_tests -- $TEST_THREADS

battle-test *args='':
    ./rules_engine/scripts/run_cargo_test.sh battle_tests "$@"

parser-test *args='':
    #!/usr/bin/env bash
    # Detect low-memory environment and set test threads
    if [ -n "${LOW_MEMORY:-}" ] || [ -f /.dockerenv ] || [ "$(uname -s)" = "Linux" ]; then
        export CARGO_TEST_THREADS=1
    fi
    export RUST_MIN_STACK=8388608
    export CARGO_TEST_QUIET=1
    ./rules_engine/scripts/run_cargo_test.sh parser_v2_tests "$@"

parser-baselines:
    just parser-test test_full_card_bracket_locale_leak_detector
    just parser-test test_golden_rendered_output

parser-test-insta *args='':
    #!/usr/bin/env bash
    # Detect low-memory environment
    if [ -n "${LOW_MEMORY:-}" ] || [ -f /.dockerenv ] || [ "$(uname -s)" = "Linux" ]; then
        TEST_THREADS="--test-threads=1"
    else
        TEST_THREADS=""
    fi
    cd rules_engine && RUST_MIN_STACK=8388608 cargo insta test --accept -p parser_v2_tests -- $TEST_THREADS "$@"

lat *args='':
    cargo run --manifest-path rules_engine/Cargo.toml --bin lat -- "$@"

lattice-test *args='':
    ./rules_engine/scripts/run_cargo_test.sh lattice_tests "$@"

lattice-bench *args='':
    cargo criterion --manifest-path rules_engine/Cargo.toml -p lattice_benchmarks -- "$@"

# Alias for lattice-bench (documented in lattice design docs)
bench-lattice *args='':
    cargo criterion --manifest-path rules_engine/Cargo.toml -p lattice_benchmarks -- "$@"

doc:
    cargo doc --manifest-path rules_engine/Cargo.toml

workspace-lints:
    #!/usr/bin/env bash
    output=$(cargo workspace-lints rules_engine/Cargo.toml 2>&1)
    if [ $? -eq 0 ]; then
        echo "Lints passed"
    else
        echo "$output"
        exit 1
    fi

workspace-lints-verbose:
    cargo workspace-lints rules_engine/Cargo.toml

schema:
    cargo run --manifest-path rules_engine/Cargo.toml --bin "schema_generator" > schema.json
    quicktype --lang cs --src-lang schema -t SchemaTypes --namespace Dreamtides.Schema --csharp-version 6 --array-type list --features complete --check-required -o client/Assets/Dreamtides/Schema/Schema.cs schema.json
    rm schema.json

fods:
    python3 client/scripts/xlsm_fods_converter.py --input client/Assets/StreamingAssets/Tabula.xlsm --output client/Assets/StreamingAssets/Tabula.fods

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

target_windows := "x86_64-pc-windows-msvc"
target_windows_gnu := "x86_64-pc-windows-gnu"

# You may need to install mingw, e.g. via brew install mingw-w64
# Note that the plugin name cannot conflict with any .asmdef file
windows-plugin-gnu:
    # Note that you cannot use IL2CPP when cross-compiling for windows
    cargo build --manifest-path rules_engine/Cargo.toml --release -p plugin --target {{target_windows_gnu}}
    mkdir -p {{plugin_out}}/Windows/
    cp rules_engine/target/{{target_windows}}/release/plugin.dll {{plugin_out}}/Windows/

windows-plugin:
    cargo build --manifest-path rules_engine/Cargo.toml --release -p plugin --target {{target_windows}}
    mkdir -p {{plugin_out}}/Windows/
    cp rules_engine/target/{{target_windows}}/release/plugin.dll {{plugin_out}}/Windows/

windows-dev-server:
    cargo build --manifest-path rules_engine/Cargo.toml --release --bin dev_server --target {{target_windows}}
    mkdir -p release_builds/
    cp rules_engine/target/{{target_windows}}/release/dev_server.exe release_builds/

plugins: ios-plugin android-plugin mac-plugin windows-plugin

clippy:
  #!/usr/bin/env bash
  output=$(cargo clippy --manifest-path rules_engine/Cargo.toml --workspace -- -D warnings -D clippy::all 2>&1)
  if [ $? -eq 0 ]; then
      echo "Clippy passed"
  else
      echo "$output"
      exit 1
  fi

clippy-verbose:
  cargo clippy --manifest-path rules_engine/Cargo.toml --workspace -- -D warnings -D clippy::all

fix:
  cargo clippy --manifest-path rules_engine/Cargo.toml --fix --allow-dirty -- -D warnings -D clippy::all

benchmark *args='':
  cargo criterion --manifest-path rules_engine/Cargo.toml "$@"

style-validator:
  #!/usr/bin/env bash
  output=$(cargo run --manifest-path rules_engine/Cargo.toml --bin "style_validator" -- --code-order "$@")
  if [ $? -ne 0 ]; then
      echo "$output"
      exit 1
  else
    echo "Style validation passed"
  fi

style-validator-verbose:
  cargo run --manifest-path rules_engine/Cargo.toml --bin "style_validator" -- --code-order

style-validator-fix:
  #!/usr/bin/env bash
  output=$(cargo run --manifest-path rules_engine/Cargo.toml --bin "style_validator" -- --code-order --fix "$@" 2>&1)
  if [ $? -ne 0 ]; then
      echo "$output"
      exit 1
  fi

parser *args='':
  cargo run --manifest-path rules_engine/Cargo.toml --bin "parser_v2" -- "$@"

parser-release *args='':
  cargo run --manifest-path rules_engine/Cargo.toml --release --bin "parser_v2" -- "$@"

parse-abilities:
  cargo run --manifest-path rules_engine/Cargo.toml --bin "parser_v2" -- parse-abilities --directory rules_engine/tabula --output rules_engine/tabula/parsed_abilities.json

verify-parsed-abilities:
  #!/usr/bin/env bash
  output=$(cargo run --manifest-path rules_engine/Cargo.toml --bin "parser_v2" -- verify-abilities --directory rules_engine/tabula --input rules_engine/tabula/parsed_abilities.json 2>&1)
  if [ $? -eq 0 ]; then
      echo "Parsed abilities verification passed"
  else
      echo "$output"
      exit 1
  fi

llmc *args='':
  @cargo run --manifest-path rules_engine/Cargo.toml -p llmc -- "$@"

llmc-watch:
  cargo watch -C rules_engine -w src/llmc -s 'just llmc down 2>/dev/null || true; sleep 1; cargo build -p llmc && just llmc up'

tabula *args='':
  cargo run --manifest-path rules_engine/Cargo.toml -p tabula_cli -- "$@"

tabula-generate:
  cargo run --manifest-path rules_engine/Cargo.toml -p tabula_cli -- generate

tabula-validate:
  #!/usr/bin/env bash
  spreadsheet="client/Assets/StreamingAssets/Tabula.xlsm"
  if [ ! -f "$spreadsheet" ]; then
      echo "Warning: Tabula spreadsheet not found at $spreadsheet, skipping validation"
      exit 0
  fi
  output=$(cargo run --manifest-path rules_engine/Cargo.toml -p tabula_cli -- validate 2>&1)
  if [ $? -ne 0 ]; then
      echo "$output"
      exit 1
  fi
  output=$(cargo run --manifest-path rules_engine/Cargo.toml -p tabula_cli -- validate --strip-images 2>&1)
  if [ $? -eq 0 ]; then
      echo "Tabula validation passed"
  else
      echo "$output"
      exit 1
  fi

tabula-check:
  #!/usr/bin/env bash
  output=$(cargo run --manifest-path rules_engine/Cargo.toml -p tabula_cli -- check 2>&1)
  if [ $? -eq 0 ]; then
      echo "Tabula check passed"
  else
      echo "$output"
      exit 1
  fi

tabula-roundtrip:
  mv client/Assets/StreamingAssets/Tabula.xlsm /tmp/Tabula.backup.xlsm
  cargo run --manifest-path rules_engine/Cargo.toml -p tabula_cli -- \
      build-toml client/Assets/StreamingAssets/TabulaData.xlsm \
      client/Assets/StreamingAssets/Tabula
  cargo run --manifest-path rules_engine/Cargo.toml -p tabula_cli -- \
      build-xls --output-path client/Assets/StreamingAssets/Tabula.xlsm \
      client/Assets/StreamingAssets/Tabula \
      client/Assets/StreamingAssets/TabulaData.xlsm
  cargo run --manifest-path rules_engine/Cargo.toml -p tabula_cli -- \
      rebuild-images --auto client/Assets/StreamingAssets/Tabula.xlsm

tabula-build-from-toml:
  cp client/Assets/StreamingAssets/Tabula.xlsm /tmp/Tabula.backup.xlsm
  cargo run --manifest-path rules_engine/Cargo.toml -p tabula_cli -- \
      build-xls --output-path client/Assets/StreamingAssets/Tabula.xlsm \
      client/Assets/StreamingAssets/Tabula \
      client/Assets/StreamingAssets/Tabula.xlsm
  cargo run --manifest-path rules_engine/Cargo.toml -p tabula_cli -- \
      rebuild-images --auto client/Assets/StreamingAssets/Tabula.xlsm

tabula-roundtrip-revert:
  cp /tmp/Tabula.backup.xlsm client/Assets/StreamingAssets/Tabula.xlsm

tabula-verify-vba:
   ./rules_engine/scripts/verify_vba.py

check-snapshots:
   ./rules_engine/scripts/check_pending_snapshots.py

watch-tabula:
    cargo watch -C rules_engine -x "run --bin tabula_cli server"

tabula-old *args='':
  cargo run --manifest-path rules_engine/Cargo.toml --bin "tabula_cli" -- \
      --key-file ./service_account_key.json \
      --spreadsheet-id "1AqSIZ_kwo3H3vfNsG-ubPV5GobkPVV_XEQEKZpRWqrA" \
      --string-ids rules_engine/src/tabula_ids/src/string_id.rs \
      --test-card-ids rules_engine/src/tabula_ids/src/test_card.rs \
      --card-lists rules_engine/src/tabula_ids/src/card_lists.rs \
      --write-json client/Assets/StreamingAssets/tabula.json "$@"
  cd rules_engine && cargo +nightly fmt

tabula-add-card *args='':
  cargo run --manifest-path rules_engine/Cargo.toml --bin "tabula_add_card" -- \
      --tabula-path client/Assets/StreamingAssets/tabula.json \
      --string-ids rules_engine/src/tabula_ids/src/string_id.rs \
      --test-card-ids rules_engine/src/tabula_ids/src/test_card.rs \
      --card-lists rules_engine/src/tabula_ids/src/card_lists.rs \
      --codegen "$@"

insta:
  cd rules_engine && cargo insta review

# Reformats code. Requires nightly because several useful options (e.g. imports_granularity) are
# nightly-only
fmt: style-validator-fix
    #!/usr/bin/env bash
    output=$(cd rules_engine && cargo +nightly fmt 2>&1)
    if [ $? -eq 0 ]; then
        echo "Formatted"
    else
        echo "$output"
        exit 1
    fi

fmt-verbose:
    cd rules_engine && cargo +nightly fmt

fmt-csharp:
    cd client && dotnet csharpier format .

check-format:
    #!/usr/bin/env bash
    output=$(cd rules_engine && cargo +nightly fmt -- --check 2>&1)
    if [ $? -eq 0 ]; then
        echo "Format OK"
    else
        echo "$output"
        exit 1
    fi

check-format-verbose:
    cd rules_engine && cargo +nightly fmt -- --check

check-docs:
    RUSTDOCFLAGS="-D rustdoc::broken-intra-doc-links -D rustdoc::private-intra-doc-links -D rustdoc::bare-urls" cargo doc --manifest-path rules_engine/Cargo.toml --all

unity-tests:
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
    ./rules_engine/scripts/profile_benchmark_binary.py --manifest-path benchmarks/battle/Cargo.toml --samply --package battle_benchmarks --benchmark $@

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

bench-parser:
    cargo criterion --manifest-path rules_engine/Cargo.toml -p parser_v2_benchmarks

iai:
    ./rules_engine/scripts/benchmark_on_linux.py 'iai_benchmarks'

bench-iai:
    cargo bench --manifest-path rules_engine/Cargo.toml iai_benchmarks

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
    mkdir -p ~/dreamtides_tests/.cargo
    cp -f .cargo/config.toml ~/dreamtides_tests/.cargo/config.toml 2>/dev/null || true
    echo $'\a'

rsync-third-party:
    rsync --delete --stats --copy-links -avqr --exclude='rules_engine/target' ./rules_engine ~/dreamtides_tests/
    echo $'\a'
    rsync --delete --stats --copy-links -avqr --exclude='client/Library' --exclude='client/test_output' ./client ~/dreamtides_tests/
    echo $'\a'
    cp justfile ~/dreamtides_tests/
    echo $'\a'

unity-test-rsync: rsync-for-review
    cd ~/dreamtides_tests && just unity-tests

regenerate-test-battles:
    cargo run --bin core_11_battle --manifest-path rules_engine/Cargo.toml -- --output rules_engine/benchmarks/benchmark_battles/src/core_11_battle.json

logcat:
    adb logcat -s dreamtides -s Unity -s CRASH

most-called-functions:
    ./rules_engine/scripts/most_called_functions.py --auto-build --benchmark ai_core_11/ai_core_11 -p battle_benchmarks --manifest-path benchmarks/battle/Cargo.toml --limit 25 --demangle  --collapse-generics

prune-remote-branches:
    #!/usr/bin/env bash
    set -euo pipefail
    # List remote branches, exclude the HEAD ref and origin/master, strip the origin/ prefix
    branches=$(git branch -r | sed 's/^ *//' | grep '^origin/' | grep -v -- '->' | grep -v '^origin/master$' | sed 's#^origin/##')
    if [ -z "$branches" ]; then
        echo "No remote branches to delete (only master present)."
        exit 0
    fi
    echo "Deleting remote branches on origin:"
    echo "$branches" | sed 's/^/  /'
    for b in $branches; do
    echo "Deleting remote branch: $b"
        git push -d origin "$b" || true
    done

tv-dev:
    #!/usr/bin/env bash
    # Kill stale processes from a previous run. The deeply-nested process tree
    # (just → pnpm → tauri CLI → pnpm → vite) does not reliably propagate
    # SIGINT to all children on Ctrl+C, leaving orphaned Vite and TV processes.
    lsof -ti :1420 | xargs kill 2>/dev/null || true
    pkill -f 'target/debug/tv' 2>/dev/null || true
    cd ./rules_engine/src/tv && pnpm tauri dev

tv-cards:
    #!/usr/bin/env bash
    lsof -ti :1420 | xargs kill 2>/dev/null || true
    pkill -f 'target/debug/tv' 2>/dev/null || true
    cd ./rules_engine/src/tv && pnpm tauri dev -- -- ~/Documents/GoogleDrive/dreamtides/rules_engine/tabula/cards.toml

tv-check:
    #!/usr/bin/env bash
    output=$(cargo check --manifest-path rules_engine/src/tv/src-tauri/Cargo.toml 2>&1)
    if [ $? -eq 0 ]; then
        echo "TV check passed"
    else
        echo "$output"
        exit 1
    fi
    output=$(cd rules_engine/src/tv && npx tsc --noEmit 2>&1)
    if [ $? -eq 0 ]; then
        echo "TV TypeScript check passed"
    else
        echo "$output"
        exit 1
    fi
    output=$(cd rules_engine/src/tv && npx eslint src/ 2>&1)
    if [ $? -eq 0 ]; then
        echo "TV ESLint check passed"
    else
        echo "$output"
        exit 1
    fi

tv-check-verbose:
    cargo check --manifest-path rules_engine/src/tv/src-tauri/Cargo.toml
    cd rules_engine/src/tv && npx tsc --noEmit
    cd rules_engine/src/tv && npx eslint src/

tv-clippy:
    #!/usr/bin/env bash
    output=$(cargo clippy --manifest-path rules_engine/src/tv/src-tauri/Cargo.toml -- -D warnings -D clippy::all 2>&1)
    if [ $? -eq 0 ]; then
        echo "TV clippy passed"
    else
        echo "$output"
        exit 1
    fi

tv-clippy-verbose:
    cargo clippy --manifest-path rules_engine/src/tv/src-tauri/Cargo.toml -- -D warnings -D clippy::all

tv-test *args='':
    ./rules_engine/scripts/run_cargo_test.sh tv_tests "$@"

tv-build:
    cd rules_engine/src/tv && pnpm tauri build
