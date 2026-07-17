# Python package

`pip install clincalc`

The Python package is a thin wrapper around the Rust engine. It exposes the same registry as the CLI, REST API, and MCP surfaces, so there is no per-calculator Python implementation to drift out of date.

## Quick start

```python
import clincalc

# List all calculators
for c in clincalc.list_calculators():
    print(c["name"], c["title"])

# Compute BMI
result = clincalc.calculate("bmi", {"weight_kg": 70, "height_cm": 175})
print(result["result"])            # 22.857142857142858
print(result["interpretation"])    # "normal"
```

Each result is a plain ``dict`` with the same shape as every other surface:

- ``calculator`` - machine name
- ``result`` - numeric score or computed value
- ``interpretation`` - human-readable category, where applicable
- ``working`` - intermediate arithmetic and notes
- ``reference`` - primary-source citation

## Input schema and template

```python
clincalc.get_schema("egfr")     # JSON Schema dict
clincalc.get_template("egfr")     # fillable example dict
```

## Batch computation with pandas

Install the optional ``pandas`` extra:

```bash
pip install clincalc[pandas]
```

```python
import pandas as pd
import clincalc

patients = pd.DataFrame({
    "weight_kg": [70, 90],
    "height_cm": [175, 180],
})

results = clincalc.batch("bmi", patients)
print(results[["result", "interpretation"]])
```

When DataFrame column names differ from the calculator's field names, pass a mapping:

```python
patients = pd.DataFrame({"weight": [70], "height": [175]})
results = clincalc.batch(
    "bmi",
    patients,
    input_columns={"weight_kg": "weight", "height_cm": "height"},
)
```

Rows with missing values are sent only with the fields that are present, so the engine raises the same validation error it would raise for a missing field in the CLI.

## Error handling

Unknown calculator names and invalid inputs raise ``ValueError`` with the same messages the Rust engine produces for the CLI and REST API.

## License

AGPL-3.0-or-later, matching the Rust crate.
