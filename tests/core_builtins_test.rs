mod common;

use predicates::prelude::*;

use self::common::run_shell;

#[test]
fn test_prompt_and_invalid_commands() {
    run_shell("invalid_command_1\ninvalid_command_2\n")
        .success()
        .stdout(predicate::str::contains("$ "))
        .stdout(predicate::str::contains(
            "invalid_command_1: command not found",
        ))
        .stdout(predicate::str::contains(
            "invalid_command_2: command not found",
        ));
}

#[test]
fn test_exit_builtin() {
    run_shell("exit\n").success();
}

#[test]
fn test_echo_builtin() {
    run_shell("echo apple banana\necho pear pineapple orange\n")
        .success()
        .stdout(predicate::str::contains("apple banana"))
        .stdout(predicate::str::contains("pear pineapple orange"));
}

#[test]
fn test_type_builtin_basics() {
    run_shell("type echo\ntype exit\ntype type\ntype invalid_command\n")
        .success()
        .stdout(predicate::str::contains("echo is a shell builtin"))
        .stdout(predicate::str::contains("exit is a shell builtin"))
        .stdout(predicate::str::contains("type is a shell builtin"))
        .stdout(predicate::str::contains("invalid_command: not found"));
}
