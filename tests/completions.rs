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
