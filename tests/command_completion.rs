mod common;

use self::common::TestExecutable;
use self::common::TestShell;
use self::common::create_dir;

#[test]
fn test_builtin_completion() {
    let mut shell = TestShell::new();
    shell.send("ech\t123");
    shell.exp_string("echo 123");
    shell.send("\r");
    shell.exp_string("123");
}

#[test]
fn test_missing_completion() {
    let mut shell = TestShell::new();
    shell.send("invalid_command\t");
    shell.exp_string("invalid_command\x07");
    shell.send("\r");
    shell.exp_string("invalid_command: command not found");
}

#[test]
fn test_executable_completion() {
    let dir = create_dir();
    let _ = TestExecutable::create(&dir, "xyz");

    let mut shell = TestShell::new_with_extra_path(&dir);
    shell.send("xy\t");
    shell.exp_string("xyz ");
    shell.send("\r");
}

#[test]
fn test_multiple_executable_completion() {
    let dir = create_dir();
    let _ = TestExecutable::create(&dir, "xyz");
    let _ = TestExecutable::create(&dir, "xyz_abc");
    let _ = TestExecutable::create(&dir, "xyz_abc_def");

    let mut shell = TestShell::new_with_extra_path(&dir);
    shell.send("xyz\t\t");
    shell.exp_string("xyz\x07\n\u{1b}[256Dxyz xyz_abc xyz_abc_def \n\u{1b}[256D$ xyz");
    shell.send("_\t");
    shell.exp_string("abc");
    shell.send("_\t");
    shell.exp_string("def");
}
