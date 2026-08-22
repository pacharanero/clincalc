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

    def test_asrs_scores_six_coded_responses(self):
        result = clincalc.calculate("asrs", {
            "age_at_least_18": True,
            "responses_cover_past_six_months": True,
            "responses": [2, 2, 2, 3, 0, 0],
        })

        assert result["result"] == 4
        assert result["working"]["result_scoring_method"] == "classic_dichotomous"
        assert result["working"]["classic_dichotomous_screen_result"] == "POSITIVE"
        assert result["working"]["continuous_total_score"] == 9
        assert "not diagnostic" in result["interpretation"]

    def test_asrs_rejects_out_of_population_administration(self):
        with pytest.raises(ValueError, match="aged 18 or older"):
            clincalc.calculate("asrs", {
                "age_at_least_18": False,
                "responses_cover_past_six_months": True,
                "responses": [0, 0, 0, 0, 0, 0],
            })


class TestCalculateLocale:
    """No calculator ships a reviewed non-English bundle yet (ENG-001.4/1.5
    in spec/roadmap.md), so every resolved locale still falls back to
    English prose. These tests cover the resolution and reporting contract
    (ENG-001.6), not translated output, which has nothing to assert yet."""

    def test_default_locale_is_english(self):
        result = clincalc.calculate("bmi", {"weight_kg": 70, "height_cm": 175})
        assert result["working"]["content_locale"] == "en"

    def test_recognised_locale_without_translation_falls_back_to_english(self):
        result = clincalc.calculate(
            "bmi", {"weight_kg": 70, "height_cm": 175}, locale="es"
        )
        assert result["working"]["content_locale"] == "en"

    def test_region_variant_resolves_via_rfc4647_lookup(self):
        # es-MX has no compiled bundle of its own; it resolves to "es" (a
        # compiled bundle) rather than being rejected, then falls back to
        # English pending calculator translations, same as "es" above.
        result = clincalc.calculate(
            "bmi", {"weight_kg": 70, "height_cm": 175}, locale="es-MX"
        )
        assert result["working"]["content_locale"] == "en"

    def test_unsupported_locale_raises_value_error(self):
        with pytest.raises(ValueError, match="unsupported locale"):
            clincalc.calculate(
                "bmi", {"weight_kg": 70, "height_cm": 175}, locale="not-a-locale"
            )

    def test_locale_is_keyword_only(self):
        with pytest.raises(TypeError):
            clincalc.calculate("bmi", {"weight_kg": 70, "height_cm": 175}, "es")


class TestListCalculators:
    def test_returns_nonempty_list(self):
        calcs = clincalc.list_calculators()
        assert isinstance(calcs, list)
        assert len(calcs) == 79

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

    def test_locale_kwarg_is_accepted_and_falls_back_to_english_titles(self):
        # No calculator ships a reviewed Spanish bundle yet, so titles are
        # unchanged; this covers that the kwarg is wired, not translated.
        calcs = clincalc.list_calculators(locale="es")
        assert len(calcs) == 79

    def test_unsupported_locale_raises_value_error(self):
        with pytest.raises(ValueError, match="unsupported locale"):
            clincalc.list_calculators(locale="not-a-locale")


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

    def test_locale_kwarg_is_accepted(self):
        schema = clincalc.get_schema("feverpain", locale="es")
        assert schema["title"] == "FeverPainInput"

    def test_unsupported_locale_raises_value_error(self):
        with pytest.raises(ValueError, match="unsupported locale"):
            clincalc.get_schema("feverpain", locale="not-a-locale")


class TestGetTemplate:
    def test_returns_template_dict(self):
        template = clincalc.get_template("feverpain")
        assert isinstance(template, dict)
        assert "fever" in template
        assert isinstance(template["fever"], str)

    def test_unknown_calculator_raises_value_error(self):
        with pytest.raises(ValueError, match="unknown calculator: nope"):
            clincalc.get_template("nope")

    def test_locale_kwarg_is_accepted(self):
        template = clincalc.get_template("feverpain", locale="es")
        assert "fever" in template

    def test_unsupported_locale_raises_value_error(self):
        with pytest.raises(ValueError, match="unsupported locale"):
            clincalc.get_template("feverpain", locale="not-a-locale")


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
    def test_batch_passes_locale_through_to_each_row(self):
        pd = pytest.importorskip("pandas")
        patients = pd.DataFrame({
            "weight_kg": [70, 90],
            "height_cm": [175, 180],
        })

        results = clincalc.batch("bmi", patients, locale="es")

        # No calculator ships a reviewed Spanish bundle yet (ENG-001.4/1.5),
        # so every row falls back to English - this covers that `locale` is
        # wired through per-row, not translated output.
        assert results["working.content_locale"].tolist() == ["en", "en"]

    def test_batch_rejects_unsupported_locale(self):
        pd = pytest.importorskip("pandas")
        patients = pd.DataFrame({"weight_kg": [70], "height_cm": [175]})

        with pytest.raises(ValueError, match="unsupported locale"):
            clincalc.batch("bmi", patients, locale="not-a-locale")

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

    @pytest.mark.parametrize("mapped", [False, True])
    def test_batch_preserves_list_valued_inputs(self, mapped):
        pd = pytest.importorskip("pandas")
        responses_column = "answers" if mapped else "responses"
        patients = pd.DataFrame({
            "age_at_least_18": [True, True],
            "responses_cover_past_six_months": [True, True],
            responses_column: [[2, 2, 2, 3, 0, 0], [0, 0, 0, 0, 0, 0]],
        })
        input_columns = None
        if mapped:
            input_columns = {
                "age_at_least_18": "age_at_least_18",
                "responses_cover_past_six_months": "responses_cover_past_six_months",
                "responses": "answers",
            }

        results = clincalc.batch("asrs", patients, input_columns=input_columns)

        assert results["result"].tolist() == [4, 0]

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
        lgpl_file = next(
            path
            for path in files
            if str(path).endswith("licenses/LICENSE-LGPL-3.0-or-later.txt")
        )
        notices_file = next(
            path
            for path in files
            if str(path).endswith("licenses/clincalc-third-party-notices.md")
        )

        assert license_file.locate().read_bytes() == (repository / "LICENSE").read_bytes()
        assert lgpl_file.locate().read_bytes() == (
            repository / "LICENSES/LGPL-3.0-or-later.txt"
        ).read_bytes()
        assert notices_file.locate().read_bytes() == (
            repository / "third-party-notices.md"
        ).read_bytes()

    def test_python_legal_files_match_repository_copies(self):
        repository = Path(__file__).resolve().parents[2]

        assert (repository / "python/LICENSE-AGPL-3.0-or-later.txt").read_bytes() == (
            repository / "LICENSE"
        ).read_bytes()
        assert (repository / "python/clincalc-third-party-notices.md").read_bytes() == (
            repository / "third-party-notices.md"
        ).read_bytes()
        assert (repository / "python/LICENSE-LGPL-3.0-or-later.txt").read_bytes() == (
            repository / "LICENSES/LGPL-3.0-or-later.txt"
        ).read_bytes()
