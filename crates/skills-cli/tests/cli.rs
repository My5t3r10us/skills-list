use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn bin() -> Command {
    Command::cargo_bin("skills-list").unwrap()
}

fn write_skill(path: &Path, name: &str) {
    fs::create_dir_all(path).unwrap();
    fs::write(
        path.join("SKILL.md"),
        format!("---\nname: {name}\ndescription: Test skill\n---\n# {name}\n"),
    )
    .unwrap();
}

#[test]
fn imports_and_searches_skill() {
    let temp = tempdir().unwrap();
    let data_dir = temp.path().join("data");
    let source = temp.path().join("source");
    write_skill(&source, "Find Me");

    bin()
        .args(["--data-dir", data_dir.to_str().unwrap(), "import"])
        .arg(&source)
        .assert()
        .success()
        .stdout(predicate::str::contains("Imported Find Me"));

    bin()
        .args(["--data-dir", data_dir.to_str().unwrap(), "search", "find"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Find Me"));

    bin()
        .args([
            "--data-dir",
            data_dir.to_str().unwrap(),
            "search",
            "find",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\": \"find-me\""));
}

#[test]
fn creates_group_and_installs_group() {
    let temp = tempdir().unwrap();
    let data_dir = temp.path().join("data");
    let source = temp.path().join("source");
    let project = temp.path().join("project");
    fs::create_dir_all(&project).unwrap();
    write_skill(&source, "Ship Me");

    bin()
        .args(["--data-dir", data_dir.to_str().unwrap(), "import"])
        .arg(&source)
        .assert()
        .success();
    bin()
        .args([
            "--data-dir",
            data_dir.to_str().unwrap(),
            "group",
            "create",
            "Starter",
        ])
        .assert()
        .success();
    bin()
        .args([
            "--data-dir",
            data_dir.to_str().unwrap(),
            "group",
            "add",
            "starter",
            "ship-me",
        ])
        .assert()
        .success();
    bin()
        .args([
            "--data-dir",
            data_dir.to_str().unwrap(),
            "install",
            "group",
            "starter",
            "--project",
            project.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Installed Ship Me"));

    assert!(project
        .join(".agents")
        .join("skills")
        .join("ship-me")
        .join("SKILL.md")
        .exists());
}

#[test]
fn adds_and_executes_command_skill_with_yes() {
    let temp = tempdir().unwrap();
    let data_dir = temp.path().join("data");
    let project = temp.path().join("project");
    fs::create_dir_all(&project).unwrap();

    bin()
        .args([
            "--data-dir",
            data_dir.to_str().unwrap(),
            "add-command",
            "Echo Skill",
            "--description",
            "Runs echo",
            "--command",
            "echo ok",
        ])
        .assert()
        .success();

    bin()
        .args([
            "--data-dir",
            data_dir.to_str().unwrap(),
            "install",
            "skill",
            "echo-skill",
            "--project",
            project.to_str().unwrap(),
            "--yes",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Executed command skill Echo Skill",
        ));
}

#[test]
fn refuses_overwrite_without_flag() {
    let temp = tempdir().unwrap();
    let data_dir = temp.path().join("data");
    let source = temp.path().join("source");
    let project = temp.path().join("project");
    fs::create_dir_all(&project).unwrap();
    write_skill(&source, "Existing");

    bin()
        .args(["--data-dir", data_dir.to_str().unwrap(), "import"])
        .arg(&source)
        .assert()
        .success();
    bin()
        .args([
            "--data-dir",
            data_dir.to_str().unwrap(),
            "install",
            "skill",
            "existing",
            "--project",
            project.to_str().unwrap(),
        ])
        .assert()
        .success();
    bin()
        .args([
            "--data-dir",
            data_dir.to_str().unwrap(),
            "install",
            "skill",
            "existing",
            "--project",
            project.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("target already exists"));
}
