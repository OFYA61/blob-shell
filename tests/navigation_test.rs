mod common;

use self::common::TestShell;
use std::env;
use std::fs;

#[test]
fn test_pwd_and_type_pwd() {
    let current_dir = env::current_dir().unwrap();
    let current_dir_str = current_dir.to_str().unwrap();

    let mut shell = TestShell::new();
    shell.test_command("type pwd", "pwd is a shell builtin");
    shell.test_command("pwd", current_dir_str);
}

#[test]
fn test_cd_absolute_and_errors() {
    let mut shell = TestShell::new();
    shell.test_command("cd /tmp", "");
    shell.test_command("pwd", "/tmp");
    shell.test_command(
        "cd /non-existing-directory",
        "cd: /non-existing-directory: No such file or directory",
    );
}

#[test]
fn test_cd_relative_paths() {
    let dir = common::create_dir();
    let dir_path = dir.path().to_str().unwrap();
    let folder = "test-folder";
    fs::create_dir_all(dir.path().join(folder)).expect("Failed to create subfolder in temp dir");

    let mut shell = TestShell::new();
    shell.test_command(&format!("cd {}", dir_path), "");
    shell.test_command("pwd", dir_path);
    shell.test_command(&format!("cd {}", folder), "");
    shell.test_command("pwd", dir.path().join(folder).to_str().unwrap());
}

#[test]
fn test_cd_home_directory() {
    let home = env::var("HOME")
        .unwrap_or_else(|_| common::create_dir().path().to_str().unwrap().to_owned());

    let mut shell = TestShell::new();
    shell.test_command("cd /", "");
    shell.test_command("cd ~", "");
    shell.test_command("pwd", &home);
}
