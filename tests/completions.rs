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
