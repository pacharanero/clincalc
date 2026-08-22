// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! PyO3 bindings for the clincalc engine.
//!
//! Compiled by maturin and installed as `clincalc/_clincalc.so`. The public
//! Python API is re-exported from `clincalc/__init__.py`; import from there,
//! not directly from `_clincalc`.

use pyo3::prelude::*;

use clincalc::{COMPILED_LOCALES, SupportedLocale, lookup_locale};

/// Resolve a caller-supplied BCP 47 tag against the compiled locale bundles.
///
/// Mirrors the CLI's `explicit > CLINCALC_LOCALE > en` resolution
/// (`resolve_cli_locale` in `src/cli.rs`), minus the environment-variable
/// fallback: Python callers pass `locale` explicitly per call.
fn resolve_locale(locale: Option<&str>) -> PyResult<SupportedLocale> {
    let requested = locale.unwrap_or("en");
    lookup_locale(requested, COMPILED_LOCALES).ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("unsupported locale: {requested}"))
    })
}

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
/// locale : str, optional
///     BCP 47 language tag (e.g. ``"es"``, ``"es-MX"``). Defaults to
///     English. Calculators without a reviewed translation for the
///     resolved locale fall back to English; ``result["working"]``
///     always reports the ``content_locale`` actually used.
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
///     If the calculator name is unknown, the input fails validation, or
///     ``locale`` is not one of the compiled locale bundles.
#[pyfunction]
#[pyo3(signature = (name, input, *, locale=None))]
fn calculate(
    py: Python<'_>,
    name: &str,
    input: &Bound<'_, PyAny>,
    locale: Option<&str>,
) -> PyResult<Py<PyAny>> {
    let resolved_locale = resolve_locale(locale)?;

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
        .calculate_for(&input_val, resolved_locale)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{e}")))?;

    let resp_str = serde_json::to_string(&response)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{e}")))?;

    Ok(json_mod.call_method1("loads", (resp_str,))?.unbind())
}

/// List all available calculators.
///
/// Parameters
/// ----------
/// locale : str, optional
///     BCP 47 language tag for ``title``/``description``. Defaults to
///     English; calculators without a reviewed translation fall back to
///     English.
///
/// Returns
/// -------
/// list[dict]
///     Each dict has keys: ``name``, ``title``, ``description``, ``license``,
///     ``license_source``, ``tags``.
///
/// Raises
/// ------
/// ValueError
///     If ``locale`` is not one of the compiled locale bundles.
#[pyfunction]
#[pyo3(signature = (*, locale=None))]
fn list_calculators(py: Python<'_>, locale: Option<&str>) -> PyResult<Py<PyAny>> {
    let resolved_locale = resolve_locale(locale)?;

    let calcs = clincalc::all();
    let items: Vec<serde_json::Value> = calcs
        .iter()
        .map(|c| {
            let lic = c.license();
            serde_json::json!({
                "name": c.name(),
                "title": c.title_for(resolved_locale),
                "description": c.description_for(resolved_locale),
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
/// locale : str, optional
///     BCP 47 language tag for schema prose. Defaults to English;
///     calculators without a reviewed translation fall back to English.
///
/// Returns
/// -------
/// dict
///     JSON Schema object describing the required input fields.
///
/// Raises
/// ------
/// ValueError
///     If the calculator name is unknown or ``locale`` is not one of the
///     compiled locale bundles.
#[pyfunction]
#[pyo3(signature = (name, *, locale=None))]
fn get_schema(py: Python<'_>, name: &str, locale: Option<&str>) -> PyResult<Py<PyAny>> {
    let resolved_locale = resolve_locale(locale)?;
    let calc = clincalc::get(name).ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("unknown calculator: {name}"))
    })?;
    let json_mod = py.import("json")?;
    let schema_str = serde_json::to_string(&calc.input_schema_for(resolved_locale)).unwrap();
    Ok(json_mod.call_method1("loads", (schema_str,))?.unbind())
}

/// Get a fillable input template for a calculator.
///
/// Parameters
/// ----------
/// name : str
///     Calculator machine name.
/// locale : str, optional
///     BCP 47 language tag for placeholder prose. Defaults to English;
///     calculators without a reviewed translation fall back to English.
///
/// Returns
/// -------
/// dict
///     Template dict with placeholder values; fill in and pass to
///     :func:`calculate`.
///
/// Raises
/// ------
/// ValueError
///     If the calculator name is unknown or ``locale`` is not one of the
///     compiled locale bundles.
#[pyfunction]
#[pyo3(signature = (name, *, locale=None))]
fn get_template(py: Python<'_>, name: &str, locale: Option<&str>) -> PyResult<Py<PyAny>> {
    let resolved_locale = resolve_locale(locale)?;
    let calc = clincalc::get(name).ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("unknown calculator: {name}"))
    })?;
    let json_mod = py.import("json")?;
    let template_str = serde_json::to_string(&calc.input_template_for(resolved_locale)).unwrap();
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
