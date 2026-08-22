# SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
# SPDX-License-Identifier: AGPL-3.0-or-later

"""clincalc - open, auditable clinical calculators for Python.

The core functions are thin wrappers around the Rust engine (``_clincalc``).
For pandas DataFrame support install the ``pandas`` extra and use
:func:`clincalc.pandas.batch` or the convenience :func:`batch` re-export here.

Examples
--------
>>> import clincalc
>>> clincalc.calculate("bmi", {"weight_kg": 70, "height_cm": 175})
{'calculator': 'bmi', 'result': 22.857..., ...}
>>> calcs = clincalc.list_calculators()
>>> tmpl = clincalc.get_template("egfr")
"""
from __future__ import annotations

from clincalc._clincalc import (
    calculate,
    get_schema,
    get_template,
    list_calculators,
)

__all__ = [
    "calculate",
    "get_schema",
    "get_template",
    "list_calculators",
    "batch",
]


def batch(
    name: str,
    df: object,
    *,
    input_columns: dict[str, str] | None = None,
    locale: str | None = None,
) -> object:
    """Apply a calculator to every row of a pandas DataFrame.

    Convenience re-export of :func:`clincalc.pandas.batch`. Requires the
    ``pandas`` extra (``pip install clincalc[pandas]``).

    Parameters
    ----------
    name:
        Calculator machine name (e.g. ``"egfr"``).
    df:
        Input ``pandas.DataFrame``; column names must match the calculator's
        input fields, or supply a mapping via ``input_columns``.
    input_columns:
        Optional ``{calculator_field: df_column}`` mapping when the DataFrame
        column names differ from the calculator's field names.
    locale:
        BCP 47 language tag (e.g. ``"es"``) applied to every row. Defaults to
        English; see :func:`clincalc.pandas.batch` for fallback behaviour.

    Returns
    -------
    pandas.DataFrame
        One row per input row with columns from the ``CalculationResponse``.
    """
    from clincalc.pandas import batch as _batch

    return _batch(name, df, input_columns=input_columns, locale=locale)
