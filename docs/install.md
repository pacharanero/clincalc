# Install

`calc` is a single self-contained binary - no runtime dependencies, no network access, no configuration. Pick whichever route suits you.

## With Cargo (from source)

The route that works today:

```bash
cargo install --git https://github.com/pacharanero/calc clincalc
```

This builds and installs the `calc` binary into `~/.cargo/bin`. You need a Rust toolchain with edition 2024 support ([rustup](https://rustup.rs) is the easy way). The package is named `clincalc`; the installed binary is always `calc`.

## From a clone

Clone if you want the source, the example input files, or to contribute:

```bash
git clone https://github.com/pacharanero/calc
cd calc
cargo build --release          # binary at ./target/release/calc
./target/release/calc list
```

To put it on your `PATH` from the clone:

```bash
cargo install --path .
```

A clone also gives you [`examples/`](https://github.com/pacharanero/calc/tree/main/examples) - ready-made JSON inputs used throughout the [Walkthrough](walkthrough.md).

## Verify it works

```console
$ calc list
feverpain     FeverPAIN Score
asrs          ASRS-v1.1 Adult ADHD Screener
phq9          PHQ-9 Depression Severity
gad7          GAD-7 Anxiety Severity
...
```

If you see the catalogue, you are ready. Head to the [Walkthrough](walkthrough.md).

!!! note "Coming with the distribution pipeline"
    Prebuilt binaries, a Homebrew tap, and `cargo install clincalc` from crates.io (no `--git`) are planned once the release pipeline lands. Until then, the two routes above are the way in.

## Requirements

- A Rust toolchain (edition 2024) - install via [rustup](https://rustup.rs).
- Nothing else at runtime: `calc` reads JSON in and writes JSON or text out.
