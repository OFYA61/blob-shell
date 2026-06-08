mod common;

use self::common::run_shell;
use predicates::prelude::*;
use std::env;
use std::fs;

#[test]
fn test_pwd_and_type_pwd() {
    let current_dir = env::current_dir().unwrap();
    let current_dir_str = current_dir.to_str().unwrap();

    run_shell(
        r#"
        type pwd
        pwd
        "#,
    )
    .success()
    .stdout(predicate::str::contains(r#"pwd is a shell builtin"#))
    .stdout(predicate::str::contains(current_dir_str));
}

#[test]
fn test_cd_absolute_and_errors() {
    run_shell(
        r#"
        cd /tmp
        pwd
        cd /non-existing-directory
        "#,
    )
    .success()
    .stdout(predicate::str::contains(r#"/tmp"#))
    .stdout(predicate::str::contains(
        r#"cd: /non-existing-directory: No such file or directory"#,
    ));
}

#[test]
fn test_cd_relative_paths() {
    let dir = common::create_dir();
    let dir_path = dir.path().to_str().unwrap();
    let folder = "test-folder";
    fs::create_dir_all(dir.path().join(folder)).expect("Failed to create subfolder in temp dir");

    run_shell(&format!(
        r#"
        cd {}
        pwd
        cd {}
        pwd
        "#,
        dir_path, folder
    ))
    .success()
    .stdout(predicate::str::contains(dir_path))
    .stdout(predicate::str::contains(
        dir.path().join(folder).to_str().unwrap(),
    ));
}

#[test]
fn test_cd_home_directory() {
    let home = env::var("HOME")
        .unwrap_or_else(|_| common::create_dir().path().to_str().unwrap().to_owned());

    run_shell(
        r#"
        cd /
        cd ~
        pwd
        "#,
    )
    .success()
    .stdout(predicate::str::contains(home));
}
