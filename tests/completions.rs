// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

// The `clincalc` binary only exists with the `cli` feature; skip this whole
// integration test under --no-default-features so the leaf build still passes.
#![cfg(feature = "cli")]

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn clincalc_bin() -> PathBuf {
    let from_cargo = PathBuf::from(env!("CARGO_BIN_EXE_clincalc"));
    if from_cargo.exists() {
        return from_cargo;
    }

    let mut path = std::env::current_exe().expect("current test executable path");
    path.pop();
    if path.file_name().is_some_and(|name| name == "deps") {
        path.pop();
    }
    path.push(format!("clincalc{}", std::env::consts::EXE_SUFFIX));
    path
}

#[test]
fn completions_generate_and_install() {
    let bin = clincalc_bin();
    assert!(bin.exists(), "clincalc binary exists at {}", bin.display());
    let out_dir =
        std::env::temp_dir().join(format!("clincalc-completions-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out_dir);

    let output = Command::new(&bin)
        .args(["completions", "bash"])
        .output()
        .expect("run clincalc completions bash");
    assert!(output.status.success());
    let bash = String::from_utf8(output.stdout).expect("bash completions are utf8");
    assert!(bash.contains("complete"));
    assert!(bash.contains("calc"));
    assert!(bash.contains("list"));
    assert!(bash.contains("tags"));
    assert!(bash.contains("completions"));

    let output = Command::new(&bin)
        .args([
            "completions",
            "install",
            "--shell",
            "zsh",
            "--dir",
            out_dir.to_str().expect("temp path"),
        ])
        .output()
        .expect("run clincalc completions install");
    assert!(output.status.success());
    assert!(out_dir.join("_clincalc").exists());
}

#[test]
fn top_level_commands_and_legacy_shorthand_work() {
    let bin = clincalc_bin();

    let output = Command::new(&bin)
        .arg("list")
        .output()
        .expect("run clincalc list");
    assert!(output.status.success());
    let list = String::from_utf8(output.stdout).expect("list is utf8");
    assert!(list.contains("feverpain"));

    let output = Command::new(&bin)
        .args(["ls", "--tag", "cardiology"])
        .output()
        .expect("run clincalc ls");
    assert!(output.status.success());
    let list = String::from_utf8(output.stdout).expect("filtered list is utf8");
    assert!(list.contains("qrisk3"));

    let output = Command::new(&bin)
        .args(["calc", "feverpain"])
        .output()
        .expect("run clincalc calc feverpain");
    assert!(output.status.success());
    let template = String::from_utf8(output.stdout).expect("template is utf8");
    assert!(template.contains("absence_of_cough"));

    let output = Command::new(&bin)
        .arg("feverpain")
        .output()
        .expect("run legacy clincalc feverpain shorthand");
    assert!(output.status.success());
    let template = String::from_utf8(output.stdout).expect("legacy template is utf8");
    assert!(template.contains("absence_of_cough"));

    let output = Command::new(&bin)
        .args(["version", "--format", "json"])
        .output()
        .expect("run clincalc version");
    assert!(output.status.success());
    let version: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("version output is json");
    assert_eq!(version["name"], "clincalc");
}

#[test]
fn aliases_and_fuzzy_unknown_name_help_work() {
    let bin = clincalc_bin();

    let output = Command::new(&bin)
        .arg("list")
        .output()
        .expect("run clincalc list");
    assert!(output.status.success());
    let list = String::from_utf8(output.stdout).expect("list is utf8");
    assert!(list.contains("aliases: bmr"));
    assert!(list.contains("tdee"));

    let output = Command::new(&bin)
        .args(["calc", "tdee"])
        .output()
        .expect("run clincalc calc tdee alias");
    assert!(output.status.success());
    let template = String::from_utf8(output.stdout).expect("template is utf8");
    assert!(template.contains("activity_factor"));

    let output = Command::new(&bin)
        .arg("fevrpain")
        .output()
        .expect("run clincalc typo");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr is utf8");
    assert!(stderr.contains("Did you mean `feverpain`?"));
}

#[test]
fn energy_requirement_alias_headlines_match_mode() {
    let bin = clincalc_bin();

    let bmr_input = r#"{"equation":"mifflin_st_jeor","sex":"male","age":30,"weight_kg":70.0,"height_cm":175.0}"#;
    let output = Command::new(&bin)
        .args(["calc", "bmr", "--input", bmr_input])
        .output()
        .expect("run bmr alias");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("output is utf8");
    assert!(text.starts_with("BMR/RMR = 1649 kcal/day"));

    let tdee_input = r#"{"equation":"mifflin_st_jeor","sex":"male","age":30,"weight_kg":70.0,"height_cm":175.0,"activity_factor":1.55}"#;
    let output = Command::new(&bin)
        .args(["calc", "tdee", "--input", tdee_input])
        .output()
        .expect("run tdee alias");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("output is utf8");
    assert!(text.starts_with("TDEE = 2556 kcal/day"));
    assert!(text.contains("basal_kcal_day: 1649"));
    assert!(text.contains("maintenance_kcal_day: 2556"));
}

#[test]
fn energy_requirement_human_flags_compute_without_json_input() {
    let bin = clincalc_bin();

    let output = Command::new(&bin)
        .args([
            "calc",
            "tdee",
            "--equation",
            "mifflin_st_jeor",
            "--sex",
            "male",
            "--age",
            "30",
            "--weight-kg",
            "70",
            "--height-cm",
            "175",
            "--activity",
            "moderate",
        ])
        .output()
        .expect("run tdee alias with human field flags");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("output is utf8");
    assert!(text.starts_with("TDEE = 2556 kcal/day"));
    assert!(text.contains("activity_preset: moderate"));
}

#[test]
fn energy_goal_flags_derive_target_adjustment() {
    let bin = clincalc_bin();

    let output = Command::new(&bin)
        .args([
            "calc",
            "tdee",
            "--equation",
            "mifflin_st_jeor",
            "--sex",
            "male",
            "--age",
            "30",
            "--weight-kg",
            "70",
            "--height-cm",
            "175",
            "--activity",
            "moderate",
            "--goal",
            "lose",
            "--rate",
            "0.5",
            "--target-weight",
            "65",
        ])
        .output()
        .expect("run tdee with goal flags");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("output is utf8");
    assert!(text.starts_with("Target intake = 2006 kcal/day"));
    assert!(text.contains("energy_goal: lose"));
    assert!(text.contains("weight_change_rate_kg_week: 0.5"));
    assert!(text.contains("estimated_weeks_to_target"));
}

#[test]
fn body_fat_pct_derives_lean_body_mass_for_cunningham() {
    let bin = clincalc_bin();

    let output = Command::new(&bin)
        .args([
            "calc",
            "bmr",
            "--equation",
            "cunningham",
            "--weight-kg",
            "80",
            "--body-fat-pct",
            "25",
        ])
        .output()
        .expect("run cunningham with body fat percentage");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("output is utf8");
    assert!(text.starts_with("BMR/RMR = 1820 kcal/day"));
    assert!(text.contains("body_fat_pct: 25"));
    assert!(text.contains("derived_lean_body_mass_kg"));
}

#[test]
fn from_record_fills_missing_human_fields() {
    let bin = clincalc_bin();
    let record_path =
        std::env::temp_dir().join(format!("clincalc-record-test-{}.json", std::process::id()));
    std::fs::write(&record_path, r#"{"subject":{"age":60,"sex":"female"}}"#)
        .expect("write temp record");

    let output = Command::new(&bin)
        .args([
            "calc",
            "egfr",
            "--from-record",
            record_path.to_str().expect("record path is utf8"),
            "--creatinine",
            "80",
            "--creatinine-unit",
            "umol/L",
        ])
        .output()
        .expect("run egfr with record defaults");
    let _ = std::fs::remove_file(&record_path);

    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("output is utf8");
    assert!(text.starts_with("egfr = "));
    assert!(text.contains("CKD G-stage"));
}

#[test]
fn egfr_interactive_mode_walks_required_schema_fields() {
    let bin = clincalc_bin();

    let mut child = Command::new(&bin)
        .args(["calc", "egfr", "--interactive"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn interactive egfr");
    child
        .stdin
        .as_mut()
        .expect("stdin available")
        .write_all(b"60\nfemale\n80\numol/L\n")
        .expect("write interactive input");

    let output = child.wait_with_output().expect("egfr interactive output");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");
    assert!(stdout.starts_with("egfr = "));
    assert!(stdout.contains("CKD G-stage"));
    let stderr = String::from_utf8(output.stderr).expect("stderr is utf8");
    assert!(stderr.contains("age:"));
    assert!(stderr.contains("sex (male|female):"));
}

#[test]
fn energy_requirement_activity_presets_inject_factor() {
    let bin = clincalc_bin();
    let input = r#"{"equation":"mifflin_st_jeor","sex":"male","age":30,"weight_kg":70.0,"height_cm":175.0}"#;

    let output = Command::new(&bin)
        .args(["calc", "tdee", "--activity", "moderate", "--input", input])
        .output()
        .expect("run tdee alias with activity preset");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("output is utf8");
    assert!(text.starts_with("TDEE = 2556 kcal/day"));
    assert!(text.contains("activity_factor: 1.55"));
    assert!(text.contains("activity_preset: moderate"));

    let output = Command::new(&bin)
        .args(["calc", "feverpain", "--activity", "moderate"])
        .output()
        .expect("run non-energy calculator with activity preset");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr is utf8");
    assert!(stderr.contains("only supported for energy_requirement"));
}

#[test]
fn curb65_is_localised_from_flag_region_and_environment() {
    let bin = clincalc_bin();
    let input = r#"{"confusion":false,"urea_mmol_l":9,"respiratory_rate":32,"systolic_bp":110,"diastolic_bp":70,"age":72}"#;

    let output = Command::new(&bin)
        .args(["--locale", "es", "curb65", "--input", input])
        .env_remove("CLINCALC_LOCALE")
        .output()
        .expect("run Spanish CURB-65 with shorthand");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("Spanish output is utf8");
    assert!(text.contains("Puntuación 3: gravedad alta"));
    assert!(text.contains("Desglose:"));
    assert!(text.contains("Referencia:"));

    let output = Command::new(&bin)
        .args(["calc", "curb65", "--locale", "es-MX", "--input", input])
        .env_remove("CLINCALC_LOCALE")
        .output()
        .expect("run regional Spanish CURB-65");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("regional output is utf8");
    assert!(text.contains("Puntuación 3: gravedad alta"));

    let output = Command::new(&bin)
        .args(["calc", "curb65", "--input", input, "--format", "json"])
        .env("CLINCALC_LOCALE", "ca")
        .output()
        .expect("run Catalan CURB-65 from environment");
    assert!(output.status.success());
    let response: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("Catalan output is json");
    assert!(
        response["interpretation"]
            .as_str()
            .unwrap()
            .contains("Puntuació 3: gravetat alta")
    );
    assert_eq!(response["working"]["content_locale"], "ca");
}

#[test]
fn locale_precedence_schema_and_unsupported_errors_are_clear() {
    let bin = clincalc_bin();

    let output = Command::new(&bin)
        .args(["--locale", "es", "calc", "curb65", "--schema"])
        .env("CLINCALC_LOCALE", "not-a-locale")
        .output()
        .expect("explicit locale overrides environment");
    assert!(output.status.success());
    let schema: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("schema output is json");
    assert!(
        schema["properties"]["confusion"]["description"]
            .as_str()
            .unwrap()
            .contains("Confusión")
    );

    let output = Command::new(&bin)
        .args(["--locale", "ca", "calc", "curb65"])
        .env_remove("CLINCALC_LOCALE")
        .output()
        .expect("run Catalan template");
    assert!(output.status.success());
    let template = String::from_utf8(output.stdout).expect("template output is utf8");
    assert!(template.contains("<booleà> Confusió de nova aparició"));

    let output = Command::new(&bin)
        .args(["--locale", "es", "calc", "bmi"])
        .env_remove("CLINCALC_LOCALE")
        .output()
        .expect("run unsupported calculator locale");
    assert!(!output.status.success());
    let error = String::from_utf8(output.stderr).expect("error is utf8");
    assert!(error.contains("calculator `bmi` is not available in locale `es`"));
    assert!(error.contains("supported locales: en"));

    let output = Command::new(&bin)
        .arg("list")
        .env("CLINCALC_LOCALE", "not_a_locale")
        .output()
        .expect("run invalid environment locale");
    assert!(!output.status.success());
    let error = String::from_utf8(output.stderr).expect("error is utf8");
    assert!(error.contains("unsupported locale `not_a_locale`"));

    let output = Command::new(&bin)
        .args(["--locale", "not_a_locale", "version"])
        .env_remove("CLINCALC_LOCALE")
        .output()
        .expect("run non-localised command with invalid explicit locale");
    assert!(!output.status.success());
    let error = String::from_utf8(output.stderr).expect("error is utf8");
    assert!(error.contains("unsupported locale `not_a_locale`"));
}
