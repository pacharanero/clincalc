// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! PyO3 bindings for the clincalc engine.
//!
//! Compiled by maturin and installed as `clincalc/_clincalc.so`. The public
//! Python API is re-exported from `clincalc/__init__.py`; import from there,
//! not directly from `_clincalc`.

use pyo3::prelude::*;

/// Compute a clinical calculator result.
///
/// Parameters
/// ----------
/// name : str
///     Calculator machine name (e.g. ``"egfr"``, ``"bmi"``). Call
///     :func:`list_calculators` to see all available names.
/// input : dict
///     Input values matching the calculator's schema. Call
///     :func:`get_template` for a fillable example.
///
/// Returns
/// -------
/// dict
///     Keys: ``calculator``, ``result``, ``interpretation``, ``working``,
///     ``reference``.
///
/// Raises
/// ------
/// ValueError
///     If the calculator name is unknown or the input fails validation.
#[pyfunction]
fn calculate(py: Python<'_>, name: &str, input: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    let json_mod = py.import("json")?;
    let json_str: String = json_mod.call_method1("dumps", (input,))?.extract()?;
    let input_val: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("invalid input: {e}")))?;

    let calc = clincalc::get(name).ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "unknown calculator: {name}; call list_calculators() to see available names"
        ))
    })?;

    let response = calc
        .calculate(&input_val)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;

    let resp_str = serde_json::to_string(&response)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{e}")))?;

    Ok(json_mod.call_method1("loads", (resp_str,))?.unbind())
}

/// List all available calculators.
///
/// Returns
/// -------
/// list[dict]
///     Each dict has keys: ``name``, ``title``, ``description``, ``license``,
///     ``license_source``, ``tags``.
#[pyfunction]
fn list_calculators(py: Python<'_>) -> PyResult<Py<PyAny>> {
    let calcs = clincalc::all();
    let items: Vec<serde_json::Value> = calcs
        .iter()
        .map(|c| {
            let lic = c.license();
            serde_json::json!({
                "name": c.name(),
                "title": c.title(),
                "description": c.description(),
                "license": lic.license,
                "license_source": lic.source_url,
                "tags": c.tags(),
            })
        })
        .collect();
    let json_str = serde_json::to_string(&items).unwrap();
    let json_mod = py.import("json")?;
    Ok(json_mod.call_method1("loads", (json_str,))?.unbind())
}

/// Get the JSON Schema for a calculator's input.
///
/// Parameters
/// ----------
/// name : str
///     Calculator machine name.
///
/// Returns
/// -------
/// dict
///     JSON Schema object describing the required input fields.
#[pyfunction]
fn get_schema(py: Python<'_>, name: &str) -> PyResult<Py<PyAny>> {
    let calc = clincalc::get(name).ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("unknown calculator: {name}"))
    })?;
    let json_mod = py.import("json")?;
    let schema_str = serde_json::to_string(&calc.input_schema()).unwrap();
    Ok(json_mod.call_method1("loads", (schema_str,))?.unbind())
}

/// Get a fillable input template for a calculator.
///
/// Parameters
/// ----------
/// name : str
///     Calculator machine name.
///
/// Returns
/// -------
/// dict
///     Template dict with placeholder values; fill in and pass to
///     :func:`calculate`.
#[pyfunction]
fn get_template(py: Python<'_>, name: &str) -> PyResult<Py<PyAny>> {
    let calc = clincalc::get(name).ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("unknown calculator: {name}"))
    })?;
    let json_mod = py.import("json")?;
    let template_str = serde_json::to_string(&calc.input_template()).unwrap();
    Ok(json_mod.call_method1("loads", (template_str,))?.unbind())
}

#[pymodule]
fn _clincalc(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(calculate, m)?)?;
    m.add_function(wrap_pyfunction!(list_calculators, m)?)?;
    m.add_function(wrap_pyfunction!(get_schema, m)?)?;
    m.add_function(wrap_pyfunction!(get_template, m)?)?;
    Ok(())
}
