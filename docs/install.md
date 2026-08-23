# Install

The `clincalc` CLI is a single self-contained binary with no runtime dependencies, network access, or configuration. A Python package is also available from PyPI. Pick whichever route suits you.

## Python package

For CPython 3.9 or later:

```bash
python -m pip install clincalc
```

Verify it with a real BMI calculation:

```bash
python -c "import clincalc; result = clincalc.calculate('bmi', {'weight_kg': 70, 'height_cm': 175}); print(result['result'])"
```

This prints `22.9`. See [Python package](python.md) for discovery, input schemas, result fields, and pandas batch calculation.

## Release installers

The docs site hosts short installer proxies that fetch cargo-dist's latest release installer:

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

## With Cargo

Install the published crate from crates.io:

```bash
cargo install clincalc
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
centor        Centor / McIsaac Score (Strep Pharyngitis)
alvarado      Alvarado Score (Appendicitis)
...
```

If you see the catalogue, you are ready. Head to the [Walkthrough](walkthrough.md).

## Requirements

- Python package: CPython 3.9 or later; no Rust toolchain is needed when a wheel is available for your platform.
- CLI via release installer: no runtime dependencies.
- CLI via Cargo: a Rust toolchain with edition 2024 support, installable via [rustup](https://rustup.rs).
