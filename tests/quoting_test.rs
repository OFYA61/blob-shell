mod common;

use predicates::prelude::*;

use self::common::run_shell;

#[test]
fn test_backslash_outside_quotes() {
    run_shell("echo hello\\ \\ \\ \\ \\ \\ example\necho test\\nscript\n")
        .success()
        .stdout(predicate::str::contains("hello      example"))
        .stdout(predicate::str::contains("testnscript"));
}

#[test]
fn test_double_quotes() {
    run_shell("echo \"script example\"\necho \"example  shell\"  \"hello\"\"script\"\n")
        .success()
        .stdout(predicate::str::contains("script example"))
        .stdout(predicate::str::contains("example  shell helloscript"));
}

#[test]
fn test_single_quotes() {
    run_shell("echo 'hello example'\necho 'world     test' 'shell''hello' example''script\n")
        .success()
        .stdout(predicate::str::contains("hello example"))
        .stdout(predicate::str::contains(
            "world     test shellhello examplescript",
        ));
}
