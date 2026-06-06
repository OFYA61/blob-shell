mod common;

use predicates::prelude::*;
use std::env;

use self::common::run_shell;

#[test]
fn test_pwd_and_type_pwd() {
    let current_dir = env::current_dir().unwrap();
    let current_dir_str = current_dir.to_str().unwrap();

    run_shell("type pwd\npwd\n")
        .success()
        .stdout(predicate::str::contains("pwd is a shell builtin"))
        .stdout(predicate::str::contains(current_dir_str));
}

#[test]
fn test_cd_absolute_and_errors() {
    run_shell("cd /tmp\npwd\ncd /non-existing-directory\n")
        .success()
        .stdout(predicate::str::contains("/tmp"))
        .stdout(predicate::str::contains(
            "cd: /non-existing-directory: No such file or directory",
        ));
}

#[test]
fn test_cd_relative_paths() {
    run_shell("cd /tmp\ncd .\npwd\n")
        .success()
        .stdout(predicate::str::contains("/tmp"));
}

#[test]
fn test_cd_home_directory() {
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());

    run_shell("cd /tmp\ncd ~\npwd\n")
        .success()
        .stdout(predicate::str::contains(home));
}
