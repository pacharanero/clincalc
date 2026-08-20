# Python package

The Python package is a thin wrapper around the Rust engine. It exposes the same registry as the CLI, REST API, and MCP surfaces, so there is no per-calculator Python implementation to drift out of date.

## Install

`clincalc` supports CPython 3.9 and later. Install the current release from [PyPI](https://pypi.org/project/clincalc/):

```bash
python -m pip install clincalc
```

No Rust toolchain is needed when a wheel is available for your platform.

## Verify the installation

Run this copy-paste smoke test from a terminal:

```bash
python -c "import clincalc; result = clincalc.calculate('bmi', {'weight_kg': 70, 'height_cm': 175}); print(result['result'])"
```

It should print:

```text
22.9
```

## Quick start

```python
import clincalc

# Compute BMI
result = clincalc.calculate("bmi", {"weight_kg": 70, "height_cm": 175})
print(result["result"])
print(result["interpretation"])
```

Output:

```text
22.9
BMI 22.9 kg/m2: healthy weight by standard WHO adult categories. BMI is a screening index and does not directly measure body composition or cardiometabolic risk.
```

Each result is a plain `dict` with the same shape as every other surface:

- `calculator` - machine name
- `result` - numeric score or computed value
- `interpretation` - human-readable category, where applicable
- `working` - intermediate arithmetic and notes
- `reference` - primary-source citation

## Find calculators and their inputs

```python
# List the available machine names and titles
for calculator in clincalc.list_calculators():
    print(calculator["name"], calculator["title"])

# Inspect the exact input contract before calculating
schema = clincalc.get_schema("egfr")      # JSON Schema dict
template = clincalc.get_template("egfr")  # fillable example dict
```

The schema is the source of truth for required fields, accepted values, and units. Pass the calculator's machine name and a matching input dictionary to `clincalc.calculate()`.

## Batch computation with pandas

Install the optional `pandas` extra:

```bash
python -m pip install "clincalc[pandas]"
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

Unknown calculator names and invalid inputs raise `ValueError`. Validation detail comes from the same Rust engine, but each surface may add its own context, so callers should rely on the exception type rather than exact cross-surface wording.

## License

Original clincalc code is AGPL-3.0-or-later, matching the Rust crate. The QRISK3 and QFracture modules retain ClinRisk's LGPL-3.0-or-later licence. Wheels and source distributions include the applicable licence texts and the repository's third-party notices.
