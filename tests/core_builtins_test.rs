mod common;

use self::common::run_shell;
use predicates::prelude::*;

#[test]
fn test_prompt_and_invalid_commands() {
    run_shell(
        r#"invalid_command_1
invalid_command_2
"#,
    )
    .success()
    .stdout(predicate::str::contains(r#"$ "#))
    .stdout(predicate::str::contains(
        r#"invalid_command_1: command not found"#,
    ))
    .stdout(predicate::str::contains(
        r#"invalid_command_2: command not found"#,
    ));
}

#[test]
fn test_exit_builtin() {
    run_shell(
        r#"exit
"#,
    )
    .success();
}

#[test]
fn test_echo_builtin() {
    run_shell(
        r#"echo apple banana
echo pear pineapple orange
"#,
    )
    .success()
    .stdout(predicate::str::contains(r#"apple banana"#))
    .stdout(predicate::str::contains(r#"pear pineapple orange"#));
}

#[test]
fn test_type_builtin_basics() {
    run_shell(
        r#"type echo
type exit
type type
type invalid_command
"#,
    )
    .success()
    .stdout(predicate::str::contains(r#"echo is a shell builtin"#))
    .stdout(predicate::str::contains(r#"exit is a shell builtin"#))
    .stdout(predicate::str::contains(r#"type is a shell builtin"#))
    .stdout(predicate::str::contains(r#"invalid_command: not found"#));
}
