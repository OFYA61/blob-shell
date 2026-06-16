mod common;

use self::common::TestShell;

#[test]
fn builtin_completion() {
    let mut shell = TestShell::new();
    shell.send("ech\t123");
    shell.exp_string("echo 123");
    shell.send("\r");
    shell.exp_string("123");
}

#[test]
fn missing_completion() {
    let mut shell = TestShell::new();
    shell.send("invalid_command\t");
    shell.exp_string("invalid_command\x07");
    shell.send("\r");
    shell.exp_string("invalid_command: command not found");
}

#[test]
fn executable_completion() {
    let mut shell = TestShell::new();
    shell.dir.dir.disable_cleanup(true);
    let _ = shell.dir.create_executable("xyz");
    shell.send("rehash\r");

    shell.send("xy\t");
    shell.exp_string("xyz ");
    shell.send("\r");
}

#[test]
fn multiple_executable_completion() {
    let mut shell = TestShell::new();
    let _ = shell.dir.create_executable("xyz");
    let _ = shell.dir.create_executable("xyz_abc");
    let _ = shell.dir.create_executable("xyz_abc_def");
    shell.send("rehash\r");

    shell.send("xyz\t\t");
    shell.exp_string("xyz\x07");
    shell.exp_string("xyz xyz_abc xyz_abc_def");
    shell.exp_string("$ xyz");
    shell.send("_\t");
    shell.exp_string("abc");
    shell.send("_\t");
    shell.exp_string("def");
}
