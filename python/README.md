# clincalc (Python)

Python bindings for the [clincalc](https://github.com/pacharanero/clincalc) Rust engine - open, auditable clinical calculators.

## Install

```bash
pip install clincalc
```

With pandas support for batch computation:

```bash
pip install clincalc[pandas]
```

## Quick start

```python
import clincalc

# Discover available calculators
print([c["name"] for c in clincalc.list_calculators()])

# Calculate BMI
result = clincalc.calculate("bmi", {"weight_kg": 70, "height_cm": 175})
print(result["result"])           # 22.857142857...
print(result["interpretation"])   # "normal"

# Inspect a calculator's input schema
print(clincalc.get_schema("egfr"))

# Get a fillable input template
print(clincalc.get_template("egfr"))
```

## Pandas batch helper

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

When the DataFrame column names differ from the calculator field names, pass a mapping:

```python
patients = pd.DataFrame({"weight": [70], "height": [175]})
results = clincalc.batch(
    "bmi",
    patients,
    input_columns={"weight_kg": "weight", "height_cm": "height"},
)
```

## License

AGPL-3.0-or-later.
