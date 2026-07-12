<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Desktop GUI

`clincalc` ships a native desktop GUI alongside the CLI. This file is the design spec; the living code lives in [`gui/`](../gui/) and the operational details in [`gui/README.md`](../gui/README.md).

## Goals

1. **Native desktop install** on Windows, macOS, and Linux. Single signed installer per platform; no runtime to pre-install (modern Windows ships WebView2 with Edge, Tauri uses it).
2. **Soft interoperability is the headline.** Every calculator's result is presented as an editable, paste-ready text block with a prominent **Copy** button. Clinician edits before copying are preserved. This is the design feature, not an afterthought.
3. **Same engine, same results.** The GUI calls `clincalc` directly via a Tauri command - no parallel scoring implementation. A FeverPAIN score in the GUI is byte-identical to `clincalc calc feverpain --input ...`.
4. **Sibling product family.** Visual language matches the GitEHR desktop app (IBM Plex Sans + Space Grotesk, teal primary, Mantine components, AppShell + sidebar nav), so a clinician moving between the two apps reads them as one suite.
5. **One UI per calculator, hand-crafted.** No schema-driven form generator. The CLI surface (`clincalc calc <name>` template, `--schema`) handles the generic case; clinician-facing UIs need hand-tuned hints, ordering, and units per calculator.

## Non-goals

- Replacing the CLI. Pipelines, automation, and embedding remain the CLI's job. The GUI is *for clinicians at a desk*.
- Storing patient data. The GUI is stateless across sessions; calculations are local and ephemeral. (If a host like GitEHR embeds `clincalc`, *it* records results into its journal - the standalone `clincalc` GUI does not.)
- Cloud sync, accounts, telemetry. Local-first, no network beyond opening links to external references on user click.

## Stack (matches GitEHR)

- **Tauri 2** - Rust backend, system webview frontend, ~10 MB installer.
- **React 19 + Mantine 8 + TypeScript** - component library with strong accessibility defaults and a complete theme system.
- **Vite 7** - build tool / dev server.
- **`@tabler/icons-react`** - icon set used throughout Mantine docs.
- Fonts: **IBM Plex Sans** (body) + **Space Grotesk** (headings), both via Google Fonts.
- Primary colour: **teal** (matches `docs/stylesheets/extra.css`).

## Layout

- AppShell with 56px header + 280px sidebar + main pane.
- Sidebar: filter input + "Featured" (FeverPAIN, CHA2DS2-VASc, QRISK3) + "All calculators" alphabetical.
- Each calculator's main pane uses a two-card layout: form on the left, live result on the right.
- The **paste-ready summary** sits below, in a teal-bordered card to draw the eye - this is the headline feature.
- Recompute on every input change. No "Calculate" button. The cost of a round-trip to Rust for any calculator we'd ship is sub-millisecond.

## Implemented baseline

The GUI currently has hand-crafted UIs for FeverPAIN, CHA2DS2-VASc, and QRISK3. Everything else renders as `Coming soon`; the CLI works for every calculator, and the GUI is opt-in per calculator.

Open GUI and distribution work is tracked by stable ID in [`spec/roadmap.md`](roadmap.md), especially `GUI-*` and `REL-*` items.

## Distribution

Tauri produces `.exe` (NSIS), `.msi`, `.app` / `.dmg`, `.deb` / `.rpm` / `.AppImage` from one codebase. Windows code-signing, updater support, and release cadence are tracked in [`spec/roadmap.md`](roadmap.md).

## Why not auto-generate forms?

A calculator's `input_schema()` is sufficient for the CLI's generic placeholder (one regular shape across 52 calculators is the whole point of the CLI design). But clinical UIs need:

- Question wording that matches how a clinician thinks (not the machine field name).
- Helpful inline hints, units, and ranges per input.
- Ordering by clinical flow, not alphabetical.
- Custom widgets where appropriate (BMI calculator? Date pickers? Visual analogue scales?).
- Sense-checks and warnings the schema can't express ("are you SURE eGFR is < 5?").

A generic form renderer can be *good enough* for an internal admin panel; it is not good enough for a tool a clinician trusts at the bedside. Hand-crafting each form is more work upfront but it is the safe path. The FeverPAIN component (`gui/src/calculators/FeverPain.tsx`) is the reference shape; clone it and adapt.
