mod common;

use self::common::TestExecutable;
use self::common::TestShell;
use self::common::create_dir;

#[test]
fn test_builtin_completion() {
    let mut shell = TestShell::new();
    shell.test_autocompletion_command("ech\t123", "echo 123", "123");
}

#[test]
fn test_missing_completion() {
    let mut shell = TestShell::new();
    shell.test_autocompletion_command(
        "invalid_command\t",
        "invalid_command\x07",
        "invalid_command: command not found",
    );
}

#[test]
fn test_executable_completion() {
    let dir = create_dir();
    let _ = TestExecutable::create(&dir, "xyz");

    let mut shell = TestShell::new_with_extra_path(&dir);
    shell.test_autocompletion_command("xy\t", "xyz ", "");
}

#[test]
fn test_multiple_executable_completion() {
    let dir = create_dir();
    let _ = TestExecutable::create(&dir, "xyz");
    let _ = TestExecutable::create(&dir, "xyz_abc");
    let _ = TestExecutable::create(&dir, "xyz_abc_def");

    let mut shell = TestShell::new_with_extra_path(&dir);
    // TODO: find a nicer way to include termianl command byte assertions
    shell.test_autocompletion(
        "xyz\t\t",
        "xyz\x07\n\u{1b}[256Dxyz xyz_abc xyz_abc_def \n\u{1b}[256D$ xyz",
    );
    shell.test_autocompletion("_\t", "abc");
    shell.test_autocompletion("_\t", "def");
}
