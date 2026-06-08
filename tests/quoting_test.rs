mod common;

use self::common::run_shell;
use predicates::prelude::*;

#[test]
fn test_backslash_outside_quotes() {
    run_shell(
        r#"
        echo hello\ \ \ \ \ \ example
        echo test\nscript
        exit
        "#,
    )
    .success()
    .stdout(predicate::str::contains(r#"hello      example"#))
    .stdout(predicate::str::contains(r#"testnscript"#));
}

#[test]
fn test_double_quotes() {
    run_shell(
        r#"
        echo "script example"
        echo "example  shell"  "hello""script"
        exit
        "#,
    )
    .success()
    .stdout(predicate::str::contains(r#"script example"#))
    .stdout(predicate::str::contains(r#"example  shell helloscript"#));
}

#[test]
fn test_double_quotes_concatenation_on_left() {
    run_shell(
        r#"
        echo "hello"world
        exit
        "#,
    )
    .success()
    .stdout(predicate::str::contains(r#"helloworld"#));
}

#[test]
fn test_double_quotes_concatenation_on_right() {
    run_shell(
        r#"
        echo hello"world"
        exit
        "#,
    )
    .success()
    .stdout(predicate::str::contains(r#"helloworld"#));
}

#[test]
fn test_double_quotes_concatenation_on_left_and_right() {
    run_shell(
        r#"
        echo "hello""world"
        exit
        "#,
    )
    .success()
    .stdout(predicate::str::contains(r#"helloworld"#));
}

#[test]
fn test_double_quotes_concatenation_on_center() {
    run_shell(
        r#"
        echo hello"world"hello
        exit
        "#,
    )
    .success()
    .stdout(predicate::str::contains(r#"helloworldhello"#));
}

#[test]
fn test_backslash_inside_double_quotes() {
    run_shell(
        r#"
        echo "just'one'\\n'backslash"
        echo "inside\"literal_quote."outside\"
        exit
        "#,
    )
    .success()
    .stdout(predicate::str::contains(r#"just'one'\n'backslash"#))
    .stdout(predicate::str::contains(r#"inside"literal_quote.outside""#));
}

#[test]
fn test_single_quotes() {
    run_shell(
        r#"
        echo 'hello example'
        echo 'world     test' 'shell''hello' example''script
        exit
        "#,
    )
    .success()
    .stdout(predicate::str::contains(r#"hello example"#))
    .stdout(predicate::str::contains(
        r#"world     test shellhello examplescript"#,
    ));
}

#[test]
fn test_single_quotes_concatenation_on_left() {
    run_shell(
        r#"
        echo 'hello'world
        exit
        "#,
    )
    .success()
    .stdout(predicate::str::contains(r#"helloworld"#));
}

#[test]
fn test_single_quotes_concatenation_on_right() {
    run_shell(
        r#"
        echo hello'world'
        exit
        "#,
    )
    .success()
    .stdout(predicate::str::contains(r#"helloworld"#));
}

#[test]
fn test_single_quotes_concatenation_on_left_and_right() {
    run_shell(
        r#"
        echo 'hello''world'
        exit
        "#,
    )
    .success()
    .stdout(predicate::str::contains(r#"helloworld"#));
}

#[test]
fn test_single_quotes_concatenation_on_center() {
    run_shell(
        r#"
        echo hello'world'hello
        exit
        "#,
    )
    .success()
    .stdout(predicate::str::contains(r#"helloworldhello"#));
}

#[test]
fn test_backslash_inside_single_quotes() {
    run_shell(
        r#"
        echo 'multiple\\slashes'
        exit
        "#,
    )
    .success()
    .stdout(predicate::str::contains(r#"multiple\\slashes"#));
}

#[test]
fn test_executable_in_quotes() {
    run_shell(
        r#"
        "echo" hello world
        'echo' hello example
        exit
        "#,
    )
    .success()
    .stdout(predicate::str::contains("hello world"))
    .stdout(predicate::str::contains("hello example"));
}
