# Install

`clincalc` is a single self-contained binary - no runtime dependencies, no network access, no configuration. Pick whichever route suits you.

## Release installers

Once the first GitHub Release has been cut, the docs site hosts short installer proxies that fetch cargo-dist's latest signed release installer:

=== "Linux / macOS"

    ```bash
    curl -LsSf https://pacharanero.github.io/clincalc/install.sh | sh
    ```

    This downloads the latest prebuilt binary for your platform and installs it to Cargo's binary directory (`$CARGO_HOME/bin`, or `~/.cargo/bin` by default).

=== "Windows PowerShell"

    ```powershell
    powershell -ExecutionPolicy Bypass -c "irm https://pacharanero.github.io/clincalc/install.ps1 | iex"
    ```

    This downloads the latest Windows binary and installs it to Cargo's binary directory.

The short URLs proxy to cargo-dist's real installer assets on the [latest GitHub Release](https://github.com/pacharanero/clincalc/releases/latest). If you need to pin a version or audit the installer first, fetch the release asset directly from GitHub.

## With Cargo (from source)

The route that works before the first release, and the easiest route for contributors:

```bash
cargo install --git https://github.com/pacharanero/clincalc clincalc
```

This builds and installs the `clincalc` binary into `~/.cargo/bin`. You need a Rust toolchain with edition 2024 support ([rustup](https://rustup.rs) is the easy way). The package is named `clincalc`; the installed binary is also `clincalc`.

## From a clone

Clone if you want the source, the example input files, or to contribute:

```bash
git clone https://github.com/pacharanero/clincalc
cd clincalc
cargo build --release          # binary at ./target/release/clincalc
./target/release/clincalc list
```

To put it on your `PATH` from the clone:

```bash
cargo install --path .
```

A clone also gives you [`examples/`](https://github.com/pacharanero/clincalc/tree/main/examples) - ready-made JSON inputs used throughout the [Walkthrough](walkthrough.md).

## Verify it works

```console
$ clincalc list
feverpain     FeverPAIN Score
asrs          ASRS-v1.1 Adult ADHD Screener
phq9          PHQ-9 Depression Severity
gad7          GAD-7 Anxiety Severity
...
```

If you see the catalogue, you are ready. Head to the [Walkthrough](walkthrough.md).

!!! note "Distribution status"
    The release pipeline and installer proxies are in place, but the first publish is still waiting on repository secrets for crates.io and the Homebrew tap. Until the first release exists, install from source with Cargo.

## Requirements

- A Rust toolchain (edition 2024) - install via [rustup](https://rustup.rs).
- Nothing else at runtime: `clincalc` reads JSON in and writes JSON or text out.
