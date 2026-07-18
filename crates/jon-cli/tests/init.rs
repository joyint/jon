// Copyright (c) 2026 Joydev GmbH (joydev.com)
// SPDX-License-Identifier: MIT

//! Targeted e2e coverage for `jon init` (JON-0008-23): the happy path
//! bootstraps an empty directory, and a repo that already has a `.joy`
//! store is pointed at the normal Joy continuation instead.

use assert_cmd::Command;
use predicates::prelude::*;

fn jon() -> Command {
    Command::cargo_bin("jon").unwrap()
}

#[test]
fn init_bootstraps_an_empty_directory() {
    let dir = tempfile::tempdir().unwrap();

    jon()
        .current_dir(dir.path())
        .args([
            "init",
            "--tool",
            "claude",
            "--yes",
            "--no-launch",
            "--name",
            "Acme Notes",
            "--user",
            "founder@example.com",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("PDA session prompt"))
        .stdout(predicate::str::contains("Acme Notes"));

    assert!(dir.path().join(".joy/project.yaml").is_file());
    for doc in ["VISION.md", "ARCHITECTURE.md", "CONTRIBUTING.md"] {
        assert!(dir.path().join(doc).is_file(), "missing {doc}");
    }
    assert!(dir.path().join(".claude/skills/joy/SKILL.md").is_file());
}

#[test]
fn init_on_a_joy_project_points_to_the_normal_continuation() {
    let dir = tempfile::tempdir().unwrap();

    jon()
        .current_dir(dir.path())
        .args([
            "init",
            "--tool",
            "claude",
            "--yes",
            "--no-launch",
            "--user",
            "founder@example.com",
        ])
        .assert()
        .success();

    jon()
        .current_dir(dir.path())
        .args(["init", "--yes", "--no-launch"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already is a Joy project"))
        .stdout(predicate::str::contains("joy ai init"));
}

#[test]
fn init_rejects_an_unknown_tool() {
    let dir = tempfile::tempdir().unwrap();

    jon()
        .current_dir(dir.path())
        .args(["init", "--tool", "cursor", "--yes", "--no-launch"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown tool"));
}
