# clincalc

Open source, community-auditable clinical calculators for the command line, Python, and MCP, on top of a fast Rust core engine.

A clinical calculation score is computed in the core Rust library, so the result is identical wherever it appears. Every calculator cites primary literature, is tested against published vectors, and records the licence it is distributed under.

!!! tip "New here? Start with the Walkthrough"
    The **[Walkthrough](walkthrough.md)** runs four real calculators end to end - with copy-paste commands and ready-made example files, so you never have to invent your own inputs.

## Install

=== "Linux / macOS"

    ```bash
    curl -LsSf https://pacharanero.github.io/clincalc/install.sh | sh
    ```

    Installs the latest prebuilt CLI release. Full options are on the [Install](install.md) page.

=== "Windows PowerShell"

    ```powershell
    powershell -ExecutionPolicy Bypass -c "irm https://pacharanero.github.io/clincalc/install.ps1 | iex"
    ```

    Release installer proxy. Full options on the [Install](install.md) page.

=== "With Cargo"

    ```bash
    cargo install clincalc
    ```

    Builds and installs the `clincalc` CLI from crates.io. Needs a Rust toolchain (edition 2024).

=== "Python"

    ```bash
    python -m pip install clincalc
    ```

    Installs the Python package for CPython 3.9 or later. See the [Python package](python.md) page for a runnable example and pandas support.

=== "From a clone"

    ```bash
    git clone https://github.com/pacharanero/clincalc
    cd clincalc
    cargo build --release      # binary at target/release/clincalc
    ```

    A clone also gives you the `examples/` input files used throughout the walkthrough.

## One interface for every calculator

There are no per-calculator flags. Every calculator is driven the same way - ask for a template, fill it in, pass it back:

```bash
clincalc list                       # list calculators (alias: ls)
clincalc tags                       # list tags with counts
clincalc calc <name>                # print a fillable input TEMPLATE (JSON)
clincalc calc <name> --schema       # the full JSON Schema contract
clincalc calc <name> --license      # the algorithm's distribution licence
clincalc calc <name> --input -      # compute, reading JSON from stdin
clincalc calc <name> --input data.json   # ...or from a file
clincalc calc <name> --input '{...}'     # ...or inline
```

## Try it right now

With `clincalc` installed, paste this - a five-criterion sore-throat score:

```console
$ clincalc calc feverpain --input '{"fever":true,"purulence":true,"attend_rapidly":true,"inflamed_tonsils":false,"absence_of_cough":false}'
feverpain = 3

A score of 3 is associated with 34–40% isolation of streptococcus. A delayed prescribing strategy is appropriate after discussion with the patient.
```

That clean, paste-able block - result, interpretation, working, and the citation - is the whole point. The [Walkthrough](walkthrough.md) builds it up step by step.

## Where to next

- **[Walkthrough](walkthrough.md)** - four calculators, with copy-paste commands and files to pipe in.
- **[CLI reference](cli-reference.md)** - every mode and flag in one place.
- **[Python package](python.md)** - install from PyPI, run a calculation, and process pandas DataFrames.
- **[Calculator catalogue](calculators.md)** - what is available today.
- **[How it works](how-it-works.md)** - the one-core-many-surfaces design, and embedding `clincalc` in a host.

## Why clincalc exists

Clinicians need good digital tools to provide good care, but the incentives to build them into EHRs are weak and the compliance barriers high. The result is a patchwork of calculators scattered across the web, often behind paywalls or implemented inconsistently. `clincalc` makes them **open source, free to use, evidence-based, and auditable**.

!!! quote "Soft interoperability"
    Copy-and-paste is derided as a kludge, but it is what clinicians actually use. Every calculator produces a clean, editable text summary as a first-class output - so you can drop a result into any record, letter, or message - while also dispatching structured results when embedded in a host.

Tools that cannot be shipped because they are proprietary or licence-locked (FRAX, MMSE, and a handful of others) are **named, not hidden**: each returns a structured explanation and points to an open alternative. See the [catalogue](calculators.md).
