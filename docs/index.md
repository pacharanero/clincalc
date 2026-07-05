# calc

Open, auditable clinical calculators driven by a single Rust engine.

`calc` is one scoring core behind many surfaces - the `calc` command line, single-file web tools, and (inside host applications) an MCP server for LLMs. A score is computed in exactly one place, so the result is identical wherever it appears. Every calculator cites primary literature, is tested against published vectors, and records the licence it is distributed under.

!!! tip "New here? Start with the Walkthrough"
    The **[Walkthrough](walkthrough.md)** runs four real calculators end to end - with copy-paste commands and ready-made example files, so you never have to invent your own inputs.

## Install

=== "With Cargo (from source)"

    ```bash
    cargo install --git https://github.com/pacharanero/calc clincalc
    ```

    Installs the `calc` binary. Needs a Rust toolchain (edition 2024). Full options on the [Install](install.md) page.

=== "From a clone"

    ```bash
    git clone https://github.com/pacharanero/calc
    cd calc
    cargo build --release      # binary at target/release/calc
    ```

    A clone also gives you the `examples/` input files used throughout the walkthrough.

## One interface for every calculator

There are no per-calculator flags. Every calculator is driven the same way - ask for a template, fill it in, pass it back:

```bash
calc list                       # list calculators
calc <name>                     # print a fillable input TEMPLATE (JSON)
calc <name> --schema            # the full JSON Schema contract
calc <name> --license           # the algorithm's distribution licence
calc <name> --input -           # compute, reading JSON from stdin
calc <name> --input data.json   # ...or from a file
calc <name> --input '{...}'     # ...or inline
```

## Try it right now

With `calc` installed, paste this - a five-criterion sore-throat score:

```console
$ calc feverpain --input '{"fever":true,"purulence":true,"attend_rapidly":true,"inflamed_tonsils":false,"absence_of_cough":false}'
feverpain = 3

A score of 3 is associated with 34–40% isolation of streptococcus. A delayed prescribing strategy is appropriate after discussion with the patient.
```

That clean, paste-able block - result, interpretation, working, and the citation - is the whole point. The [Walkthrough](walkthrough.md) builds it up step by step.

## Where to next

- **[Walkthrough](walkthrough.md)** - four calculators, with copy-paste commands and files to pipe in.
- **[CLI reference](cli-reference.md)** - every mode and flag in one place.
- **[Calculator catalogue](calculators.md)** - what is available today.
- **[How it works](how-it-works.md)** - the one-core-many-surfaces design, and embedding `calc` in a host.

## Why calc exists

Clinicians need good digital tools to provide good care, but the incentives to build them into EHRs are weak and the compliance barriers high. The result is a patchwork of calculators scattered across the web, often behind paywalls or implemented inconsistently. `calc` makes them **open source, free to use, evidence-based, and auditable**.

!!! quote "Soft interoperability"
    Copy-and-paste is derided as a kludge, but it is what clinicians actually use. Every calculator produces a clean, editable text summary as a first-class output - so you can drop a result into any record, letter, or message - while also dispatching structured results when embedded in a host.

Tools that cannot be shipped because they are proprietary or licence-locked (FRAX, MMSE, and a handful of others) are **named, not hidden**: each returns a structured explanation and points to an open alternative. See the [catalogue](calculators.md).
