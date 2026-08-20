# clincalc (Python)

Python bindings for the [clincalc](https://github.com/pacharanero/clincalc) Rust engine - open, auditable clinical calculators.

## Install

`clincalc` supports CPython 3.9 and later.

```bash
python -m pip install clincalc
```

With pandas support for batch computation:

```bash
python -m pip install "clincalc[pandas]"
```

## Quick start

```python
import clincalc

# Calculate BMI
result = clincalc.calculate("bmi", {"weight_kg": 70, "height_cm": 175})
assert result["result"] == 22.9
print(result["calculator"], result["result"])
# bmi 22.9

# Discover available calculators
print([c["name"] for c in clincalc.list_calculators()])

# Inspect a calculator's input schema
print(clincalc.get_schema("egfr"))

# Get a fillable input template
print(clincalc.get_template("egfr"))
```

You can verify the installation directly from a terminal:

```bash
python -c "import clincalc; result = clincalc.calculate('bmi', {'weight_kg': 70, 'height_cm': 175}); print(result['result'])"
```

The command prints `22.9`.

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

The clincalc Python package is licensed under AGPL-3.0-or-later. Its wheel and source distributions include the full licence text.

The ASRS-v1.1 six-question scorer accepts coded adult responses covering the past six months from the [authorised form](https://license.tov.med.nyu.edu/product/asrs6Qscreener), reports the classic dichotomous and alternative continuous methods separately, and carries the required attribution to New York University and the President and Fellows of Harvard College in every result. Current source and releases from `0.3.0` onward do not distribute questionnaire text or the separately licensed 18-question checklist; legacy `0.2.2` source artifacts did include the checklist and should not be used.
