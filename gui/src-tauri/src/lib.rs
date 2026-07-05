// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tauri 2 desktop GUI for the open clinical calculators.
//!
//! All real scoring lives in `clincalc`; this crate is purely the
//! webview-to-Rust bridge plus the bundled-binary build. Every Tauri
//! command is a thin wrapper that hands a JSON value to `clincalc` and
//! returns the same `CalculationResponse` shape every surface produces -
//! so the GUI, the CLI, and any MCP host yield byte-identical output.
//!
//! The frontend (React + Mantine + Vite) lives at `gui/src/` and is built
//! to `gui/dist/` for production bundling.

use clincalc::CalculationResponse;
use serde::Serialize;

/// One row of the catalogue, suitable for sidebar / picker rendering.
///
/// Mirrors what `calc list --format json` returns, kept deliberately small.
#[derive(Debug, Serialize)]
struct CalcSummary {
    name: &'static str,
    title: &'static str,
    description: &'static str,
    tags: &'static [&'static str],
    /// True for the 10 proprietary "unavailable" stubs - the frontend can
    /// render them differently (badged, faded) without having to know which
    /// names are stubs.
    proprietary: bool,
}

#[tauri::command]
fn list_calculators() -> Vec<CalcSummary> {
    clincalc::all()
        .iter()
        .map(|c| {
            let tags = c.tags();
            CalcSummary {
                name: c.name(),
                title: c.title(),
                description: c.description(),
                tags,
                proprietary: tags.contains(&"proprietary"),
            }
        })
        .collect()
}

/// Compute a result from a JSON input. The frontend is responsible for
/// building the input object that matches the calculator's schema; this
/// command simply hands it to `clincalc` and surfaces the typed response
/// (or the typed error message).
#[tauri::command]
fn calculate(name: &str, input: serde_json::Value) -> Result<CalculationResponse, String> {
    let calc = clincalc::get(name).ok_or_else(|| format!("unknown calculator: {name}"))?;
    calc.calculate(&input).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![list_calculators, calculate])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
