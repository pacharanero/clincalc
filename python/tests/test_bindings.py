# SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Tests for the clincalc Python bindings.

Requires the extension to be built and installed (``maturin develop``).
Run with ``pytest`` from the ``python/`` directory.
"""

from importlib.metadata import distribution
from pathlib import Path

import clincalc
import pytest


class TestCalculate:
    def test_calculate_returns_response_dict(self):
        result = clincalc.calculate("feverpain", {
            "fever": True,
            "purulence": True,
            "attend_rapidly": True,
            "inflamed_tonsils": True,
            "absence_of_cough": True,
        })
        assert isinstance(result, dict)
        assert result["calculator"] == "feverpain"
        assert result["result"] == 5
        assert isinstance(result["interpretation"], str)
        assert isinstance(result["working"], dict)
        assert isinstance(result["reference"], str)

    def test_calculate_bmi(self):
        result = clincalc.calculate("bmi", {"weight_kg": 70, "height_cm": 175})
        assert result["calculator"] == "bmi"
        assert round(result["result"], 1) == 22.9

    def test_calculate_unknown_calculator_raises_value_error(self):
        with pytest.raises(ValueError, match="unknown calculator: nope"):
            clincalc.calculate("nope", {})

    def test_calculate_invalid_input_raises_value_error(self):
        with pytest.raises(ValueError, match="invalid input"):
            clincalc.calculate("feverpain", {"fever": "not-a-boolean"})

    def test_asrs_is_unavailable_without_reproduction_permission(self):
        result = clincalc.calculate("asrs", {})

        assert result["result"] == "unavailable: proprietary"
        assert result["working"]["status"] == "unavailable-proprietary"
        assert "require permission" in result["interpretation"]


class TestListCalculators:
    def test_returns_nonempty_list(self):
        calcs = clincalc.list_calculators()
        assert isinstance(calcs, list)
        assert len(calcs) == 78

    def test_each_entry_has_required_keys(self):
        calcs = clincalc.list_calculators()
        for c in calcs:
            assert "name" in c
            assert "title" in c
            assert "description" in c
            assert "license" in c
            assert "license_source" in c
            assert "tags" in c

    def test_feverpain_is_in_list(self):
        names = [c["name"] for c in clincalc.list_calculators()]
        assert "feverpain" in names


class TestGetSchema:
    def test_returns_schema_dict(self):
        schema = clincalc.get_schema("feverpain")
        assert isinstance(schema, dict)
        assert schema["title"] == "FeverPainInput"
        assert "fever" in schema["properties"]
        assert "fever" in schema["required"]

    def test_unknown_calculator_raises_value_error(self):
        with pytest.raises(ValueError, match="unknown calculator: nope"):
            clincalc.get_schema("nope")


class TestGetTemplate:
    def test_returns_template_dict(self):
        template = clincalc.get_template("feverpain")
        assert isinstance(template, dict)
        assert "fever" in template
        assert isinstance(template["fever"], str)

    def test_unknown_calculator_raises_value_error(self):
        with pytest.raises(ValueError, match="unknown calculator: nope"):
            clincalc.get_template("nope")


class TestRoundTrip:
    def test_template_then_calculate_round_trips(self):
        """Every calculator's template should be valid JSON that calculate accepts
        (after filling in plausible values). We test with feverpain as a known case."""
        template = clincalc.get_template("feverpain")
        template.update({
            "fever": True,
            "purulence": True,
            "attend_rapidly": True,
            "inflamed_tonsils": True,
            "absence_of_cough": True,
        })
        result = clincalc.calculate("feverpain", template)
        assert result["result"] == 5


class TestPandasBatch:
    def test_batch_calculates_each_row(self):
        pd = pytest.importorskip("pandas")
        patients = pd.DataFrame({
            "weight_kg": [70, 90],
            "height_cm": [175, 180],
        })

        results = clincalc.batch("bmi", patients)

        assert results["calculator"].tolist() == ["bmi", "bmi"]
        assert results["result"].round(1).tolist() == [22.9, 27.8]

    def test_batch_maps_different_column_names(self):
        pd = pytest.importorskip("pandas")
        patients = pd.DataFrame({"weight": [70], "height": [175]})

        results = clincalc.batch(
            "bmi",
            patients,
            input_columns={"weight_kg": "weight", "height_cm": "height"},
        )

        assert round(results.loc[0, "result"], 1) == 22.9

    def test_batch_rejects_non_dataframe(self):
        pytest.importorskip("pandas")

        with pytest.raises(TypeError, match="df must be a pandas.DataFrame"):
            clincalc.batch("bmi", [{"weight_kg": 70, "height_cm": 175}])


class TestDistributionMetadata:
    def test_installed_distribution_contains_legal_files(self):
        files = distribution("clincalc").files or []
        repository = Path(__file__).resolve().parents[2]
        license_file = next(
            path
            for path in files
            if str(path).endswith("licenses/LICENSE-AGPL-3.0-or-later.txt")
        )
        notices_file = next(
            path for path in files if str(path).endswith("licenses/medikquantis-notice.md")
        )

        assert license_file.locate().read_bytes() == (repository / "LICENSE").read_bytes()
        assert notices_file.locate().read_bytes() == (
            repository / "third-party-notices.md"
        ).read_bytes()

    def test_python_legal_files_match_repository_copies(self):
        repository = Path(__file__).resolve().parents[2]

        assert (repository / "python/LICENSE-AGPL-3.0-or-later.txt").read_bytes() == (
            repository / "LICENSE"
        ).read_bytes()
        assert (repository / "python/medikquantis-notice.md").read_bytes() == (
            repository / "third-party-notices.md"
        ).read_bytes()
