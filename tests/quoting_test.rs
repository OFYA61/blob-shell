mod common;

use self::common::TestShell;

#[test]
fn test_backslash_outside_quotes() {
    let mut shell = TestShell::new();
    shell.test_command(r#"echo hello\ \ \ \ \ \ example"#, "hello      example");
    shell.test_command(r#"echo test\nscript"#, "testnscript");
}

#[test]
fn test_double_quotes() {
    let mut shell = TestShell::new();
    shell.test_command(r#"echo "script example""#, "script example");
    shell.test_command(
        r#"echo "example  shell"  "hello""script""#,
        "example  shell helloscript",
    );
}

#[test]
fn test_double_quotes_concatenation_on_left() {
    let mut shell = TestShell::new();
    shell.test_command(r#"echo "hello"world"#, "helloworld");
}

#[test]
fn test_double_quotes_concatenation_on_right() {
    let mut shell = TestShell::new();
    shell.test_command(r#"echo hello"world""#, "helloworld");
}

#[test]
fn test_double_quotes_concatenation_on_left_and_right() {
    let mut shell = TestShell::new();
    shell.test_command(r#"echo "hello""world""#, "helloworld");
}

#[test]
fn test_double_quotes_concatenation_on_center() {
    let mut shell = TestShell::new();
    shell.test_command(r#"echo hello"world"hello"#, "helloworldhello");
}

#[test]
fn test_backslash_inside_double_quotes() {
    let mut shell = TestShell::new();
    shell.test_command(
        r##"echo "just'one'\\n'backslash""##,
        r"just'one'\n'backslash",
    );
    shell.test_command(
        r##"echo "inside\"literal_quote."outside\""##,
        r##"inside"literal_quote.outside""##,
    );
}

#[test]
fn test_single_quotes() {
    let mut shell = TestShell::new();
    shell.test_command(r#"echo 'hello example'"#, "hello example");
    shell.test_command(
        r#"echo 'world     test' 'shell''hello' example''script"#,
        "world     test shellhello examplescript",
    );
}

#[test]
fn test_single_quotes_concatenation_on_left() {
    let mut shell = TestShell::new();
    shell.test_command(r#"echo 'hello'world"#, "helloworld");
}

#[test]
fn test_single_quotes_concatenation_on_right() {
    let mut shell = TestShell::new();
    shell.test_command(r#"echo hello'world'"#, "helloworld");
}

#[test]
fn test_single_quotes_concatenation_on_left_and_right() {
    let mut shell = TestShell::new();
    shell.test_command(r#"echo 'hello''world'"#, "helloworld");
}

#[test]
fn test_single_quotes_concatenation_on_center() {
    let mut shell = TestShell::new();
    shell.test_command(r#"echo hello'world'hello"#, "helloworldhello");
}

#[test]
fn test_backslash_inside_single_quotes() {
    let mut shell = TestShell::new();
    shell.test_command(r#"echo 'multiple\\slashes'"#, r#"multiple\\slashes"#);
}

#[test]
fn test_executable_in_quotes() {
    let mut shell = TestShell::new();
    shell.test_command(r#""echo" hello world"#, "hello world");
    shell.test_command(r#"'echo' hello example"#, "hello example");
}
