# SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Pandas batch helpers for clincalc.

Install with the ``pandas`` extra:

    pip install clincalc[pandas]
"""
from __future__ import annotations

import json
from typing import Any

from clincalc._clincalc import calculate


def _has_pandas() -> bool:
    try:
        import pandas  # noqa: F401
    except ImportError:
        return False
    return True


def batch(
    name: str,
    df: object,
    *,
    input_columns: dict[str, str] | None = None,
    locale: str | None = None,
) -> object:
    """Apply a calculator to every row of a pandas DataFrame.

    Parameters
    ----------
    name:
        Calculator machine name (e.g. ``"egfr"``).
    df:
        Input ``pandas.DataFrame``.
    input_columns:
        Optional ``{calculator_field: df_column}`` mapping when the DataFrame
        column names differ from the calculator's field names. Use this when
        a DataFrame column is named differently from the Rust input schema.
    locale:
        BCP 47 language tag (e.g. ``"es"``) applied to every row. Defaults to
        English; calculators without a reviewed translation for the resolved
        locale fall back to English, reported per-row in ``working.content_locale``.

    Returns
    -------
    pandas.DataFrame
        One row per input row with columns derived from the calculation
        response: ``calculator``, ``result``, ``interpretation``, ``working``,
        ``reference``.

    Examples
    --------
    >>> import clincalc.pandas as cp
    >>> cp.batch("bmi", df)  # DataFrame with weight_kg / height_cm columns.
    >>> cp.batch("bmi", df, input_columns={"weight_kg": "weight", "height_cm": "height"})
    """
    if not _has_pandas():
        raise ImportError(
            "pandas batch helper requires pandas. Install it with: pip install clincalc[pandas]"
        )

    import pandas as pd

    if not isinstance(df, pd.DataFrame):
        raise TypeError(f"df must be a pandas.DataFrame, got {type(df).__name__}")

    records: list[dict[str, Any]] = df.to_dict(orient="records")
    outputs: list[dict[str, Any]] = []

    def is_present(value: Any) -> bool:
        return not pd.api.types.is_scalar(value) or bool(pd.notna(value))

    mapping = input_columns or {}
    for raw in records:
        # Apply column-name mapping and drop missing values so the Rust
        # engine sees only provided fields.
        input_row: dict[str, Any] = {
            calc_field: raw[df_col]
            for calc_field, df_col in mapping.items()
            if df_col in raw and is_present(raw[df_col])
        }
        if not mapping:
            input_row = {k: v for k, v in raw.items() if is_present(v)}

        response = calculate(name, input_row, locale=locale)
        response["_input_index"] = len(outputs)
        outputs.append(response)

    result_df = pd.json_normalize(outputs, sep=".")
    result_df = result_df.set_index("_input_index")
    result_df.index.name = None
    return result_df
