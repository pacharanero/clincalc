# MCP server

`clincalc` can expose every clinical calculator as a local Model Context Protocol (MCP) server for LLM hosts. This makes the calculator catalogue discoverable as native tools: the host sees one tool per calculator, each with the calculator's own JSON Schema as its input contract.

The MCP surface is optional. The normal `clincalc` CLI remains the default install, and the pure engine still stays a serde-only leaf when built with `default-features = false`.

## Install with MCP support

MCP support is behind the optional `mcp` Cargo feature:

```bash
cargo install clincalc --features mcp
```

Until the crate is published to crates.io, install from Git:

```bash
cargo install --git https://github.com/pacharanero/clincalc clincalc --features mcp
```

Check that the command is available:

```bash
clincalc mcp
```

`clincalc mcp` is a stdio MCP server, so it is expected to keep running while an MCP host is connected. It is not an interactive CLI command and it does not print the calculator catalogue to your terminal.

If your binary was built without MCP support, `clincalc mcp` prints a targeted reinstall hint instead of treating `mcp` as an unknown calculator.

## Configure an MCP host

Most local MCP hosts take a command and arguments. Use:

```json
{
  "mcpServers": {
    "clincalc": {
      "command": "clincalc",
      "args": ["mcp"]
    }
  }
}
```

If `clincalc` is not on the host's `PATH`, use the absolute path to the installed binary:

```json
{
  "mcpServers": {
    "clincalc": {
      "command": "/home/you/.cargo/bin/clincalc",
      "args": ["mcp"]
    }
  }
}
```

The exact configuration file location depends on the MCP host. The important point is that the host launches `clincalc mcp` over stdio.

## Tool names

Each calculator is exposed as:

```text
clincalc_<name>
```

Examples:

```text
clincalc_feverpain
clincalc_news2
clincalc_cha2ds2vasc
clincalc_qrisk3
clincalc_qfracture
```

Tool definitions are generated from `clincalc::all()`. Adding a calculator to the registry automatically exposes it through MCP, with no MCP-specific wrapper.

## Inputs

Each MCP tool's input schema is the same JSON Schema returned by the CLI:

```bash
clincalc calc feverpain --schema
```

For example, `clincalc_feverpain` expects the same object as:

```bash
clincalc calc feverpain --input examples/feverpain.json --format json
```

```json
--8<-- "examples/feverpain.json"
```

The MCP server does not infer missing clinical facts. If a required field is absent or invalid, the calculator returns a validation error and the model should ask for the missing information.

## Results

MCP calls return the same structured `CalculationResponse` shape as the CLI's JSON output:

```json
{
  "calculator": "feverpain",
  "result": 3,
  "interpretation": "...",
  "working": {
    "score": 3
  },
  "reference": "..."
}
```

That shape is intentionally timestamp-free and patient-free. A host that records the calculation should add patient linkage, timestamps, author identity, and audit metadata outside `clincalc`.

## Why use MCP instead of the CLI?

The CLI is still the universal surface for humans, shell scripts, and simple agents. MCP adds one specific advantage: **native LLM discoverability**.

With the CLI, an agent must learn to run `clincalc list`, inspect `clincalc calc <name> --schema`, construct JSON, and call `clincalc calc <name> --input ...`. That works, and the CLI is designed for it.

With MCP, the host presents the calculators directly as typed tools. The model sees the available calculators and their input schemas without first learning the command-line protocol.

## Safety boundaries

- The server is local stdio only; it does not open a network listener.
- The server does not read patient records or workspace files.
- The server does not store results.
- The server does not add timestamps, patient ids, or audit entries.
- The server does not invent missing clinical inputs.
- Proprietary unavailable stubs are still visible and callable, so licence-locked tools are named rather than silently hidden.

## Developer notes

The implementation lives in `src/mcp.rs` and is compiled only with the `mcp` feature. The feature pulls in `rmcp` and `tokio`; these dependencies are not present in the `default-features = false` engine build.

Useful checks while changing MCP support:

```bash
cargo test --no-default-features
cargo test
cargo test --features mcp
cargo clippy --all-targets --features mcp -- -D warnings
```

The design spec is [`spec/mcp.md`](https://github.com/pacharanero/clincalc/blob/main/spec/mcp.md).
