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

    def test_calculate_body_surface_area(self):
        result = clincalc.calculate(
            "body_surface_area", {"height_cm": 180, "weight_kg": 80}
        )
        assert result["calculator"] == "body_surface_area"
        assert result["result"] == 2.0

    def test_calculate_homa_ir(self):
        result = clincalc.calculate(
            "homa_ir",
            {
                "fasting_glucose": 4.5,
                "glucose_unit": "mmol/L",
                "fasting_insulin_miu_l": 5,
            },
        )
        assert result["calculator"] == "homa_ir"
        assert result["result"] == 1.0
        assert "not a universal diagnostic cut-off" in result["interpretation"]

    def test_calculate_apgar(self):
        result = clincalc.calculate(
            "apgar",
            {
                "minute_after_birth": 5,
                "assessment_during_resuscitation": False,
                "gestational_context": "term_or_late_preterm",
                "heart_rate": "at_least_100",
                "respiratory_effort": "good_with_vigorous_cry",
                "muscle_tone": "active_motion",
                "reflex_irritability": "cough_sneeze_or_active_withdrawal",
                "appearance": "completely_pink",
            },
        )
        assert result["calculator"] == "apgar"
        assert result["result"] == 10

    def test_calculate_binet(self):
        result = clincalc.calculate(
            "binet",
            {
                "cll_diagnosis_confirmed": True,
                "haemoglobin_g_dl": 12,
                "platelet_count_10_9_l": 150,
                "head_and_neck_involved": True,
                "axillae_involved": True,
                "groins_involved": True,
                "spleen_involved": False,
                "liver_involved": False,
            },
        )
        assert result["calculator"] == "binet"
        assert result["result"] == "B"

    def test_calculate_four_ts(self):
        result = clincalc.calculate(
            "four_ts",
            {
                "thrombocytopenia": "fall_gt_50_nadir_ge_20",
                "timing": "clear_day_5_to_10_or_rapid_with_prior_exposure_within_30_days",
                "thrombosis_or_sequelae": "none",
                "other_causes": "none_apparent",
            },
        )
        assert result["calculator"] == "four_ts"
        assert result["result"] == 6

    def test_calculate_khorana(self):
        result = clincalc.calculate(
            "khorana",
            {
                "assessment_context": "adult_ambulatory_before_new_chemotherapy_regimen",
                "cancer_site": "lung",
                "platelet_count_10_9_l": 350,
                "haemoglobin_g_dl": 12,
                "uses_erythropoiesis_stimulating_agent": False,
                "leukocyte_count_10_9_l": 11,
                "bmi_kg_m2": 30,
            },
        )
        assert result["calculator"] == "khorana"
        assert result["result"] == 2
        assert result["working"]["original_risk_band"] == "intermediate"
        assert result["working"]["meets_guideline_consideration_threshold"] is True

    def test_calculate_free_water_deficit(self):
        result = clincalc.calculate(
            "free_water_deficit",
            {
                "assessment_context": "adult_with_hypernatraemia",
                "weight_kg": 60,
                "current_sodium_mmol_l": 166,
                "target_sodium_mmol_l": 140,
                "total_body_water_fraction": 0.5,
            },
        )
        assert result["calculator"] == "free_water_deficit"
        assert result["result"] == 5.6
        assert "not a fluid prescription" in result["interpretation"]

    def test_calculate_isth_overt_dic(self):
        result = clincalc.calculate(
            "isth_overt_dic",
            {
                "underlying_etiology": "sepsis_or_severe_infection",
                "platelet_count_10_9_l": 72,
                "d_dimer_multiple_of_uln": 8.2,
                "pt_prolongation_seconds": 4.1,
                "fibrinogen_g_l": 1.4,
            },
        )
        assert result["calculator"] == "isth_overt_dic"
        assert result["result"] == 5
        assert result["working"]["score_version"] == "2025"
        assert result["working"]["band"] == "consistent_with_overt_dic"

    def test_calculate_ciwa_ar(self):
        result = clincalc.calculate(
            "ciwa_ar",
            {
                "assessment_context": "clinically_identified_alcohol_withdrawal_with_reliable_patient_participation",
                "nausea_and_vomiting": 1,
                "tremor": 2,
                "paroxysmal_sweats": 1,
                "anxiety": 3,
                "agitation": 2,
                "tactile_disturbances": 0,
                "auditory_disturbances": 0,
                "visual_disturbances": 0,
                "headache_or_fullness": 1,
                "orientation_and_clouding": 0,
            },
        )
        assert result["calculator"] == "ciwa_ar"
        assert result["result"] == 10
        assert result["working"]["severity_band"] == "moderate"

    def test_calculate_cows(self):
        result = clincalc.calculate(
            "cows",
            {
                "assessment_context": "clinician_assessment_of_current_possible_opioid_withdrawal",
                "resting_pulse_rate_bpm": 101,
                "sweating": 3,
                "restlessness": 0,
                "pupil_size": 0,
                "bone_or_joint_aches": 0,
                "runny_nose_or_tearing": 0,
                "gastrointestinal_upset": 0,
                "tremor": 0,
                "yawning": 0,
                "anxiety_or_irritability": 0,
                "gooseflesh_skin": 0,
            },
        )
        assert result["calculator"] == "cows"
        assert result["result"] == 5
        assert result["working"]["resting_pulse_rate_points"] == 2
        assert result["working"]["severity_band"] == "mild"

    def test_calculate_meld_3(self):
        result = clincalc.calculate(
            "meld_3",
            {
                "registration_age_years": 40,
                "female_for_adult_meld": False,
                "bilirubin": 6.0,
                "bilirubin_unit": "mg/dL",
                "inr": 1.5,
                "creatinine": 1.5,
                "creatinine_unit": "mg/dL",
                "sodium_mmol_l": 131.0,
                "albumin": 3.5,
                "albumin_unit": "g/dL",
                "qualifying_dialysis_in_prior_7_days": False,
            },
        )
        assert result["calculator"] == "meld_3"
        assert result["result"] == 25
        assert result["working"]["rounded_uncapped_policy_score"] == 25

    def test_calculate_psa_density(self):
        result = clincalc.calculate(
            "psa_density",
            {"total_psa_ng_ml": 6.0, "prostate_volume_ml": 40.0},
        )
        assert result["calculator"] == "psa_density"
        assert result["result"] == 0.15
        assert result["working"]["result_unit"] == "ng/mL/cc"
        assert "No cutoff is universal" in result["interpretation"]

    def test_calculate_pasi(self):
        result = clincalc.calculate(
            "pasi",
            {
                "assessment_context": "clinician_assessed_plaque_psoriasis",
                "head_and_neck": {"area_grade": 0, "erythema": 0, "induration": 0, "desquamation": 0},
                "upper_limbs": {"area_grade": 0, "erythema": 0, "induration": 0, "desquamation": 0},
                "trunk": {"area_grade": 3, "erythema": 3, "induration": 3, "desquamation": 3},
                "lower_limbs": {"area_grade": 0, "erythema": 0, "induration": 0, "desquamation": 0},
            },
        )
        assert result["calculator"] == "pasi"
        assert result["result"] == 8.1
        assert result["working"]["variant"] == "standard_pasi"

    def test_calculate_pitt_bacteraemia(self):
        result = clincalc.calculate(
            "pitt_bacteraemia",
            {
                "assessment_context": "hospitalised_patient_with_cre_infection_and_index_culture",
                "maximum_temperature_c": 39.2,
                "acute_hypotension_on_index_culture_day": True,
                "mechanical_ventilation_on_index_culture_day": False,
                "cardiac_arrest_on_index_day_or_prior_48_hours": False,
                "worst_mental_status_on_index_culture_day": "disoriented",
            },
        )
        assert result["calculator"] == "pitt_bacteraemia"
        assert result["result"] == 4
        assert result["working"]["score_at_least_four"] is True

    def test_calculate_nihss(self):
        result = clincalc.calculate(
            "nihss",
            {
                "assessment_context": "clinician_administered_standard_adult_nihss_using_authorized_scale_materials",
                "level_of_consciousness": "alert",
                "loc_questions": "one_correct",
                "loc_commands": "both_correct",
                "best_gaze": "normal",
                "visual_fields": "partial_hemianopia",
                "facial_palsy": "minor_paralysis",
                "motor_arm_left": "drift_without_hitting_support",
                "motor_arm_right": "no_drift_for_ten_seconds",
                "motor_leg_left": "drift_without_hitting_bed",
                "motor_leg_right": "no_drift_for_five_seconds",
                "limb_ataxia": "absent",
                "sensory": "mild_to_moderate_loss",
                "best_language": "mild_to_moderate_aphasia",
                "dysarthria": "mild_to_moderate",
                "extinction_inattention": "one_modality",
            },
        )
        assert result["calculator"] == "nihss"
        assert result["result"] == 9
        assert result["working"]["total_score"] == 9

    def test_calculate_duke_iscvid(self):
        result = clincalc.calculate(
            "duke_iscvid",
            {
                "assessment_context": "clinician_classification_of_suspected_ie_using_corrected_2023_duke_iscvid_criteria",
                "pathologic_evidence": "none",
                "intracardiac_prosthetic_material": [],
                "blood_culture_organism": "staphylococcus_aureus",
                "positive_blood_culture_sets": 2,
                "major_laboratory_evidence": "none",
                "anatomic_imaging_evidence": "none_or_nonqualifying",
                "pet_ct_evidence": "none_or_nonqualifying",
                "surgical_inspection_evidence": "none_or_nonqualifying",
                "predisposition": [],
                "maximum_documented_temperature_c": 38.6,
                "vascular_phenomena": [],
                "immunologic_phenomena": [],
                "other_minor_microbiology": "none",
                "auscultation_evidence": "none_or_nonqualifying",
                "rejection_evidence": [],
            },
        )
        assert result["calculator"] == "duke_iscvid"
        assert result["result"] == "possible"
        assert result["working"]["major_count"] == 1
        assert result["working"]["minor_count"] == 1

    def test_calculate_ardsnet_predicted_body_weight(self):
        result = clincalc.calculate(
            "ardsnet_predicted_body_weight",
            {
                "assessment_context": "adult_lung_protective_ventilation_protocol_using_ardsnet_predicted_body_weight",
                "height_cm": 152.4,
                "formula_branch": "female",
            },
        )
        assert result["calculator"] == "ardsnet_predicted_body_weight"
        assert result["result"] == 45.5
        assert result["working"]["height_inches"] == 60.0
        assert result["working"]["outside_official_reference_table_range"] is False

    def test_calculate_orbit_returns_rights_review_response(self):
        result = clincalc.calculate("orbit", {})

        assert result["result"] == "unavailable: rights-review"
        assert result["working"]["status"] == "unavailable-rights-review"
        assert clincalc.get_schema("orbit")["properties"] == {}

    def test_calculate_nyha_returns_rights_review_response(self):
        result = clincalc.calculate("nyha", {})

        assert result["result"] == "unavailable: rights-review"
        assert result["working"]["status"] == "unavailable-rights-review"
        assert "proprietary" not in result["interpretation"]

    def test_calculate_sad_persons_returns_clinical_safety_response(self):
        result = clincalc.calculate("sad_persons", {})

        assert result["result"] == "unavailable: clinical-safety"
        assert result["working"]["status"] == "unavailable-clinical-safety"
        assert "returns no score" in result["interpretation"]
        entry = next(
            calculator
            for calculator in clincalc.list_calculators()
            if calculator["name"] == "sad_persons"
        )
        assert "unavailable" in entry["tags"]
        assert "proprietary" not in entry["tags"]
        assert clincalc.get_schema("sad_persons")["properties"] == {}

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

    def test_omitted_locale_preserves_existing_response_contract(self):
        result = clincalc.calculate("bmi", {"weight_kg": 70, "height_cm": 175})
        assert "content_locale" not in result["working"]

    def test_explicit_english_reports_content_locale(self):
        result = clincalc.calculate(
            "bmi", {"weight_kg": 70, "height_cm": 175}, locale="en"
        )
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
        assert len(calcs) == 111

    def test_each_entry_has_required_keys(self):
        calcs = clincalc.list_calculators()
        for c in calcs:
            assert "name" in c
            assert "title" in c
            assert "description" in c
            assert "supported_locales" in c
            assert "license" in c
            assert "license_source" in c
            assert "tags" in c

    def test_feverpain_is_in_list(self):
        names = [c["name"] for c in clincalc.list_calculators()]
        assert "feverpain" in names

    def test_locale_reports_fallback_and_supported_locales(self):
        calcs = clincalc.list_calculators(locale="es")
        bmi = next(calc for calc in calcs if calc["name"] == "bmi")
        assert bmi["content_locale"] == "en"
        assert bmi["supported_locales"] == ["en"]

    def test_omitted_locale_preserves_catalogue_provenance_shape(self):
        bmi = next(
            calc for calc in clincalc.list_calculators() if calc["name"] == "bmi"
        )
        assert "content_locale" not in bmi
        assert bmi["supported_locales"] == ["en"]

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
        assert schema == clincalc.get_schema("feverpain")

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
        assert template == clincalc.get_template("feverpain")

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
