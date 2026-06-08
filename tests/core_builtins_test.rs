mod common;

use self::common::TestShell;

#[test]
fn test_exit_builtin() {
    let mut shell = TestShell::new();
    shell.exit();
}

#[test]
fn test_invalid_commands() {
    let mut shell = TestShell::new();
    shell.test_command("invalid_command", "invalid_command: command not found");
    shell.exit();
}

#[test]
fn test_echo_builtin() {
    let mut shell = TestShell::new();
    shell.test_command("echo apple banana", "apple banana");
    shell.exit();
}

#[test]
fn test_type_builtin_basics() {
    let mut shell = TestShell::new();
    shell.test_command("type echo", "echo is a shell builtin");
    shell.test_command("type exit", "exit is a shell builtin");
    shell.test_command("type type", "type is a shell builtin");
    shell.test_command("type invalid_command", "invalid_command: not found");
    shell.exit();
}
