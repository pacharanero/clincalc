// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

/**
 * Typed wrappers around the Tauri `invoke` channel.
 *
 * The shapes here match the Rust types in `src-tauri/src/lib.rs` exactly -
 * if you change the Rust side, change these in the same commit. Both sides
 * ultimately defer to `clincalc`, so the canonical contract is the Rust
 * struct, not the TypeScript interface.
 */

import { invoke } from "@tauri-apps/api/core";

/** One catalogue entry (mirrors `CalcSummary` in lib.rs). */
export interface CalcSummary {
  name: string;
  title: string;
  description: string;
  tags: string[];
  /** True for confirmed proprietary or licence-locked entries. */
  proprietary: boolean;
  /** True when invoking the entry returns an explanation instead of a score. */
  unavailable: boolean;
}

/** A computed result (mirrors `clincalc::CalculationResponse`). */
export interface CalculationResponse {
  calculator: string;
  /** Number for most scores; a short string for categorical results. */
  result: number | string;
  interpretation: string;
  /** Every intermediate value the score depended on, snake_case keys. */
  working: Record<string, unknown>;
  reference: string;
}

export async function listCalculators(): Promise<CalcSummary[]> {
  try {
    return await invoke<CalcSummary[]>("list_calculators");
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error("[calc] list_calculators failed:", err);
    throw err;
  }
}

export async function calculate(
  name: string,
  input: Record<string, unknown>,
): Promise<CalculationResponse> {
  try {
    return await invoke<CalculationResponse>("calculate", { name, input });
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error(
      `[calc] calculate(${name}) failed; input was:`,
      input,
      "error:",
      err,
    );
    throw err;
  }
}
