mod common;

use self::common::TestShell;
use std::env;
use std::fs;

#[test]
fn test_pwd_and_type_pwd() {
    let current_dir = env::current_dir().unwrap();
    let current_dir_str = current_dir.to_str().unwrap();

    let mut shell = TestShell::new();

    shell.send("type pwd");
    shell.exp_string("type pwd");
    shell.send("\r");
    shell.exp_string("pwd is a shell builtin");

    shell.send("pwd");
    shell.exp_string("pwd");
    shell.send("\r");
    shell.exp_string(current_dir_str);
}

#[test]
fn test_cd_absolute_and_errors() {
    let mut shell = TestShell::new();

    shell.send("cd /tmp");
    shell.exp_string("cd /tmp");
    shell.send("\r");

    shell.send("pwd");
    shell.exp_string("pwd");
    shell.send("\r");
    shell.exp_string("/tmp");

    shell.send("cd /non-existing-directory");
    shell.exp_string("cd /non-existing-directory");
    shell.send("\r");
    shell.exp_string("cd: /non-existing-directory: No such file or directory");
}

#[test]
fn test_cd_relative_paths() {
    let dir = common::create_dir();
    let dir_path = dir.path().to_str().unwrap();
    let folder = "test-folder";
    fs::create_dir_all(dir.path().join(folder)).expect("Failed to create subfolder in temp dir");

    let mut shell = TestShell::new();

    shell.send(&format!("cd {}", dir_path));
    shell.exp_string(&format!("cd {}", dir_path));
    shell.send("\r");

    shell.send("pwd");
    shell.exp_string("pwd");
    shell.send("\r");
    shell.exp_string(dir_path);

    shell.send(&format!("cd {}", folder));
    shell.exp_string(&format!("cd {}", folder));
    shell.send("\r");

    shell.send("pwd");
    shell.exp_string("pwd");
    shell.send("\r");
    shell.exp_string(dir.path().join(folder).to_str().unwrap());
}

#[test]
fn test_cd_home_directory() {
    let home = env::var("HOME")
        .unwrap_or_else(|_| common::create_dir().path().to_str().unwrap().to_owned());

    let mut shell = TestShell::new();

    shell.send("cd /");
    shell.exp_string("cd /");
    shell.send("\r");

    shell.send("cd ~");
    shell.exp_string("cd ~");
    shell.send("\r");

    shell.send("pwd");
    shell.exp_string("pwd");
    shell.send("\r");
    shell.exp_string(&home);
}
