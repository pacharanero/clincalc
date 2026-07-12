// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

// The `clincalc` binary only exists with the `cli` feature; skip this whole
// integration test under --no-default-features so the leaf build still passes.
#![cfg(feature = "cli")]

use std::path::PathBuf;
use std::process::Command;

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
    assert!(stderr.contains("--activity is only supported for energy_requirement"));
}
