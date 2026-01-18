---
lattice-id: LBSWQN
name: environment-setup
description: |-
  This document records the exact steps followed to make the Dreamtides rules engine
  repository buildable and ready for day-to-day development.
created-at: 2026-01-18T05:52:12.687012Z
updated-at: 2026-01-18T05:52:12.687029Z
---

# Environment Setup

## 1. Verify the Rust toolchain

1. Checked the installed compiler and Cargo versions to confirm the toolchain was
   available:
   ```bash
   rustc --version
   cargo --version
   ```
2. Ensured the stable components Clippy and rustfmt were installed. This completed
   almost instantly:
   ```bash
   rustup component add clippy rustfmt
   ```
3. The `just fmt` recipe runs `cargo +nightly fmt`, so also installed the nightly
   rustfmt component before trying to format again. The download and install took
   roughly 30 seconds on my machine:
   ```bash
   rustup component add rustfmt --toolchain nightly > /dev/null
   ```

## 2. Install the `just` command runner

`just` is required for all project automation. Attempting to check its version
initially failed, so I installed it with Cargo:

```bash
just --version  # produced "command not found"
cargo install -q just     # first build and install took ~3 minutes
just --version            # verified the install succeeded
```

## 3. Install workspace lint support

Running `just review` shells out to `cargo workspace-lints`. That subcommand was
not present by default, so I installed it separately. The install and build phase
completed in about 90 seconds:

```bash
cargo install -q cargo-workspace-lints
```

## 4. Run formatting and validation commands

With the tooling in place, ran the standard validation commands in the order
requested by the repository instructions. Timings below reflect the waits seen
on a fresh build; once the workspace cache is warm the commands finish much
faster.

1. **Format:** The first formatting attempt triggered a nightly toolchain download
   and then failed until the nightly rustfmt component was installed (see step 1).
   After installing that component, `just fmt` completed in about 1.5 seconds.
   ```bash
   just fmt
   ```
2. **Check:** A full workspace check from a cold cache took roughly 4 minutes
   ```bash
   just check
   ```
3. **Clippy:** The lint pass finished in a little under a minute
   ```bash
   just clippy
   ```
4. **Review:** The first invocation of the review pipeline rebuilt the entire
   workspace, ran `cargo workspace-lints`, and executed the full test suite. That
   initial build took just under 22 minutes and failed until `cargo-workspace-lints`
   was installed. After installing it, re-running
   ```bash
   CARGO_BUILD_JOBS=1 just review
   ```
   completed successfully in about 1 minute 20 seconds thanks to cached build
   artifacts. The command runs `cargo +nightly fmt --check`, `cargo build`,
   `cargo workspace-lints`, `cargo clippy`, and `cargo test`, so expect the first
   clean run to take the longest.

Following these steps left the repository fully formatted, linted, and with all
workspace tests passing.
