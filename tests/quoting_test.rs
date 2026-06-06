mod common;

use self::common::run_shell;
use predicates::prelude::*;

#[test]
fn test_backslash_outside_quotes() {
    // With raw strings, we type exactly what the shell receives: \ followed by a space
    run_shell(
        r#"echo hello\ \ \ \ \ \ example
echo test\nscript
"#,
    )
    .success()
    .stdout(predicate::str::contains(r#"hello      example"#))
    .stdout(predicate::str::contains(r#"testnscript"#));
}

#[test]
fn test_backslash_inside_single_quotes() {
    // No more quadruple backslashes!
    run_shell(r#"echo 'multiple\\slashes'"#)
        .success()
        .stdout(predicate::str::contains(r#"multiple\\slashes"#));
}

#[test]
fn test_backslash_inside_double_quotes() {
    let stdin_input = r#"echo "just'one'\\n'backslash"
echo "inside\"literal_quote."outside\"
exit
"#;

    run_shell(stdin_input)
        .success()
        .stdout(predicate::str::contains(r#"just'one'\n'backslash"#))
        .stdout(predicate::str::contains(r#"inside"literal_quote.outside""#));
}

#[test]
fn test_double_quotes() {
    run_shell(
        r#"echo "script example"
echo "example  shell"  "hello""script"
"#,
    )
    .success()
    .stdout(predicate::str::contains(r#"script example"#))
    .stdout(predicate::str::contains(r#"example  shell helloscript"#));
}

#[test]
fn test_single_quotes() {
    run_shell(
        r#"echo 'hello example'
echo 'world     test' 'shell''hello' example''script
"#,
    )
    .success()
    .stdout(predicate::str::contains(r#"hello example"#))
    .stdout(predicate::str::contains(
        r#"world     test shellhello examplescript"#,
    ));
}

#[test]
fn test_executable_in_quotes() {
    run_shell(
        r#"
        "echo" hello world
        "#,
    )
    .success()
    .stdout(predicate::str::contains("hello world"));
}
