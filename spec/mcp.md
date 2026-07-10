<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# MCP server

`clincalc` should expose its calculator registry as an optional Model Context Protocol (MCP) server via `clincalc mcp`. This is not a second calculator implementation. It is a thin, generated adapter over the existing `clincalc` engine, using the same registry, input schemas, calculation responses, references, and licence metadata as every other surface.

## Decision

Add a reference MCP server in this codebase, behind a non-default Cargo feature named `mcp`, and expose it as:

```bash
clincalc mcp
```

This echoes the `sct mcp` surface and keeps the mental model simple: the installed `clincalc` binary is the calculator tool, and `clincalc mcp` starts the LLM-native server surface when compiled with MCP support.

The always-on engine remains unchanged: with `default-features = false`, `clincalc` stays a strict serde-only leaf with no async runtime, no I/O, no protocol dependency, and no host coupling.

## Why MCP belongs here

The CLI is already LLM-friendly: `clincalc list`, `clincalc <name>`, `clincalc <name> --schema`, JSON input, and deterministic JSON output give a capable agent enough structure to use the calculators safely.

MCP adds a different affordance: native tool discovery. Instead of an LLM needing to learn the `clincalc` command protocol and then ask for schemas one calculator at a time, the MCP host can see each calculator as a typed tool whose input contract is the calculator's own JSON Schema.

This is supportable because `clincalc` already has the right abstractions:

- `clincalc::all()` is the complete calculator registry.
- `Calculator::name()`, `title()`, `description()`, `reference()`, `license()`, and `tags()` are tool metadata.
- `Calculator::input_schema()` is the MCP tool `inputSchema`.
- `Calculator::calculate(input)` is the tool body.
- `CalculationResponse` is the structured result.

The MCP server should therefore be mostly glue code. If adding a calculator to `clincalc::all()` does not automatically expose it through MCP, the design has failed.

## Goals

1. **Native LLM discoverability.** Every calculator in the registry is exposed to MCP hosts as a callable, schema-described tool.
2. **One engine, same results.** A result from `clincalc mcp` is produced by the same `Calculator::calculate()` path as `clincalc <name> --input ...`.
3. **Zero per-calculator MCP code.** Tool definitions are generated from the registry. No hand-written wrappers, no duplicated schemas, no per-calculator dispatch tables.
4. **Strict feature isolation.** MCP dependencies are optional and gated behind `mcp`. The `default-features = false` engine remains a serde-only leaf.
5. **Simple command surface.** `clincalc mcp` starts the server. No separate `clincalc-mcp` binary for the first implementation.
6. **Clinical auditability.** Responses include enough structured result, working, reference, and licence context for an LLM host to display or record the calculation provenance.
7. **Host neutrality.** The server is a reference implementation for any MCP client, not a GitEHR-specific integration.

## Non-goals

- Replacing the CLI. The CLI remains the universal human, shell, script, and low-dependency surface.
- Replacing host MCP integrations. GitEHR or other hosts may still expose calculators from `clincalc::all()` inside their own MCP server when they need patient-context-aware behaviour.
- Adding patient storage, timestamps, audit journals, or host-specific context to `clincalc`. A host that records a calculation adds timestamps and patient linkage outside the engine.
- Making calculator selection autonomous. The MCP server exposes tools; the model and host decide when to call them.
- Adding per-calculator prompt engineering. Descriptions should come from existing calculator metadata unless the trait is deliberately extended for every surface.

## Cargo features and dependency boundaries

The feature shape should be:

```toml
[features]
default = ["cli"]
cli = ["dep:anyhow", "dep:clap", "dep:clap_complete"]
mcp = ["cli", "dep:<mcp-sdk>", "dep:<async-runtime-if-required>"]
```

Exact MCP crate and runtime choices must be researched at implementation time and pinned to current stable versions per the repository dependency policy. The important boundary is that all MCP-only dependencies are optional and appear only in the `mcp` feature.

Expected module layout:

```text
src/
├── cli.rs          # existing CLI surface, still behind `cli`
├── main.rs         # `clincalc` binary; dispatches `clincalc mcp` when feature-enabled
├── mcp.rs          # MCP server implementation, behind `mcp`
└── lib.rs          # pure engine registry, no MCP dependency
```

`src/lib.rs` may expose `pub mod mcp` only under `#[cfg(feature = "mcp")]`. The calculator trait, registry, response schema, tags, templates, and calculator implementations must not depend on MCP types.

## Command behaviour

### With MCP enabled

```bash
clincalc mcp
```

Starts the MCP server using the default transport expected by local MCP hosts, probably stdio for the first implementation.

Potential future flags, only if the selected MCP SDK and real host setup require them:

```bash
clincalc mcp --transport stdio
clincalc mcp --log-level info
```

Do not add network transports until there is a concrete use case. Local stdio is enough for Claude Desktop-style configuration and keeps the security model narrow.

### Without MCP enabled

A `clincalc` binary compiled without `mcp` should not silently treat `mcp` as an unknown calculator. It should reserve the word and print a targeted error such as:

```text
MCP support was not compiled into this clincalc binary.
Reinstall with MCP enabled, for example: cargo install clincalc --features mcp
```

This keeps the user experience clear while preserving the non-default dependency boundary.

## Tool model

Expose one MCP tool per calculator.

Tool naming:

```text
clincalc_<calculator-name>
```

Examples:

```text
clincalc_feverpain
clincalc_news2
clincalc_cha2ds2_vasc
clincalc_qrisk3
```

If calculator machine names contain characters that are not legal or ergonomic in MCP tool names, define a single reversible sanitisation function and test it against every registered calculator. The calculator's original `name()` must still appear in the result.

Tool metadata:

- `name`: `clincalc_` plus the sanitised calculator machine name.
- `title`: calculator title, if the SDK supports titles separately.
- `description`: calculator description plus reference summary where useful.
- `inputSchema`: the exact `Calculator::input_schema()` value.

Tool execution:

1. Receive JSON object from the MCP client.
2. Pass it unchanged to `Calculator::calculate(&input)`.
3. Return the `CalculationResponse` as structured JSON.
4. On validation or scoring error, return an MCP tool error containing the calculator error message.

The server should not pre-validate input against JSON Schema unless the selected MCP SDK requires it. The calculator's typed deserialisation remains the source of truth for validation.

## Result content

The primary MCP result should be structured JSON matching `CalculationResponse`:

```json
{
  "calculator": "feverpain",
  "result": 3,
  "interpretation": "...",
  "working": {},
  "reference": "..."
}
```

If the MCP SDK supports multiple content blocks, the server may also return a text block formatted like the CLI `text` output for easy pasting into a note. That should be additive; the structured response remains canonical.

Consider including licence metadata either:

1. in tool annotations/metadata, if supported by the SDK;
2. in an additional field of the structured MCP payload; or
3. through a separate `clincalc_license`/resource mechanism.

Do not change `CalculationResponse` just for MCP unless the same change is valuable to the CLI, GUI, and embedding hosts.

## Catalogue and resources

The first implementation may rely entirely on MCP's normal tool listing: every calculator is already discoverable as a tool.

Optional later additions:

- `clincalc_list` tool: returns the same catalogue shape as `clincalc list --format json`, with optional tag filters.
- `clincalc_tags` tool or resource: returns the tag taxonomy and counts.
- per-calculator resources for reference, licence, and example input.

Do not add these until a real MCP host demonstrates that tool listing alone is not enough.

## Security and clinical-safety notes

- The server runs locally and should not open a network listener in the first implementation.
- The server must not read patient records, environment-specific clinical context, or files beyond what the MCP protocol itself requires.
- The server must not invent missing clinical inputs. If required fields are absent, the calculator returns a validation error and the model should ask the user or host for the missing information.
- Tool descriptions should avoid implying that scores are diagnoses or management mandates. The existing calculator interpretations and references remain the clinical framing.
- Proprietary unavailable stubs should remain visible. They are valid registry entries and should return their structured unavailable response rather than disappearing from MCP.

## Testing plan

Unit/integration tests should cover:

1. `cargo test --no-default-features` keeps passing and does not compile MCP or CLI dependencies.
2. `cargo test` keeps passing with the default CLI feature only.
3. `cargo test --features mcp` exercises MCP module tests.
4. Every registered calculator produces one MCP tool definition.
5. Each MCP tool's input schema equals `Calculator::input_schema()` exactly.
6. A known calculator call through the MCP dispatch path returns the same `CalculationResponse` as direct `Calculator::calculate()`.
7. Proprietary unavailable stubs are exposed and execute successfully with `{}` where appropriate.
8. The reserved `clincalc mcp` error is clear when the binary is compiled without the feature.
9. Tool-name sanitisation is deterministic and collision-free across `clincalc::all()`.

The normal pre-commit validation remains:

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Before merging MCP support, also run:

```bash
cargo test --no-default-features
cargo test --features mcp
cargo clippy --all-targets --features mcp -- -D warnings
```

If the selected MCP SDK requires feature-specific examples or an MCP inspector command, document that in `docs/cli-reference.md` once implemented.

## Documentation updates when implemented

When `clincalc mcp` exists, update:

- `README.md` - short install and MCP-host configuration example.
- `docs/cli-reference.md` - add `clincalc mcp` under modes, with feature-gated install note.
- `docs/how-it-works.md` - describe the CLI/MCP/GUI surfaces from one registry.
- `spec/calculators.md` - replace future-tense MCP wording with implemented behaviour.
- `spec/roadmap.md` - move the reference MCP server item from Future to Completed or In-progress as appropriate.

## Distribution question

Because `mcp` is non-default, there are two possible release positions:

1. **Featureful source install only at first:** users run `cargo install clincalc --features mcp` to get `clincalc mcp`. The cargo-dist installers continue shipping the default lightweight CLI.
2. **Ship MCP in release binaries:** cargo-dist builds `clincalc` with `--features mcp`, so the official installers include `clincalc mcp` while `clincalc = { default-features = false }` remains leaf-pure for embedders.

Start with option 1 unless there is clear demand for installer-shipped MCP. This keeps the default binary small and avoids adding an async/protocol stack to the standard CLI install before the surface has real users.

## House-style alignment

MCP implementation should happen after, or alongside, a lightweight house-style audit of this repository against `~/code/house-style`. The audit should be tracked separately from the MCP implementation so dependency, CI, release, docs, and clinical-safety standards do not get mixed into protocol work.

Likely checks for this repo:

- feature-gated optional dependencies preserve the leaf rule;
- CI covers default, no-default-features, and featureful builds where appropriate;
- docs and specs use current house-style conventions;
- release packaging makes an explicit decision about whether `mcp` is included in cargo-dist binaries;
- any reusable patterns from `clincalc` - especially the leaf-engine rule, registry-driven surfaces, licence metadata enforcement, schema-derived templates, and proprietary-unavailable stubs - are candidates to document back into `~/code/house-style` if not already covered there.

## Implementation sequence

1. Research current Rust MCP SDK options and select one with stdio support, active maintenance, and a dependency profile compatible with an optional feature.
2. Add optional MCP dependencies behind `mcp`; do not alter default or no-default dependency behaviour.
3. Add `src/mcp.rs` behind `#[cfg(feature = "mcp")]`.
4. Add `clincalc mcp` dispatch in `src/main.rs`, with a clear non-feature error when disabled.
5. Generate tool definitions from `clincalc::all()`.
6. Implement tool execution by looking up the calculator and calling `calculate()`.
7. Add tests for registry coverage, schema equality, result equality, stubs, and tool-name collisions.
8. Add docs for installing and configuring `clincalc mcp` with at least one MCP host.
9. Revisit release packaging once the first host configuration is proven.
