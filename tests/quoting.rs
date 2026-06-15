mod common;

use self::common::TestShell;

#[test]
fn backslash_outside_quotes() {
    let mut shell = TestShell::new();

    shell.send(r#"echo hello\ \ \ \ \ \ example"#);
    shell.exp_string(r#"echo hello\ \ \ \ \ \ example"#);
    shell.send("\r");
    shell.exp_string("hello      example");

    shell.send(r#"echo test\nscript"#);
    shell.exp_string(r#"echo test\nscript"#);
    shell.send("\r");
    shell.exp_string("testnscript");
}

#[test]
fn double_quotes() {
    let mut shell = TestShell::new();

    shell.send(r#"echo "script example""#);
    shell.exp_string(r#"echo "script example""#);
    shell.send("\r");
    shell.exp_string("script example");

    shell.send(r#"echo "example  shell"  "hello""script""#);
    shell.exp_string(r#"echo "example  shell"  "hello""script""#);
    shell.send("\r");
    shell.exp_string("example  shell helloscript");
}

#[test]
fn double_quotes_concatenation_on_left() {
    let mut shell = TestShell::new();

    shell.send(r#"echo "hello"world"#);
    shell.exp_string(r#"echo "hello"world"#);
    shell.send("\r");
    shell.exp_string("helloworld");
}

#[test]
fn double_quotes_concatenation_on_right() {
    let mut shell = TestShell::new();

    shell.send(r#"echo hello"world""#);
    shell.exp_string(r#"echo hello"world""#);
    shell.send("\r");
    shell.exp_string("helloworld");
}

#[test]
fn double_quotes_concatenation_on_left_and_right() {
    let mut shell = TestShell::new();

    shell.send(r#"echo "hello""world""#);
    shell.exp_string(r#"echo "hello""world""#);
    shell.send("\r");
    shell.exp_string("helloworld");
}

#[test]
fn double_quotes_concatenation_on_center() {
    let mut shell = TestShell::new();

    shell.send(r#"echo hello"world"hello"#);
    shell.exp_string(r#"echo hello"world"hello"#);
    shell.send("\r");
    shell.exp_string("helloworldhello");
}

#[test]
fn backslash_inside_double_quotes() {
    let mut shell = TestShell::new();

    shell.send(r##"echo "just'one'\\n'backslash""##);
    shell.exp_string(r##"echo "just'one'\\n'backslash""##);
    shell.send("\r");
    shell.exp_string(r"just'one'\n'backslash");

    shell.send(r##"echo "inside\"literal_quote."outside\""##);
    shell.exp_string(r##"echo "inside\"literal_quote."outside\""##);
    shell.send("\r");
    shell.exp_string(r##"inside"literal_quote.outside""##);
}

#[test]
fn single_quotes() {
    let mut shell = TestShell::new();

    shell.send(r#"echo 'hello example'"#);
    shell.exp_string(r#"echo 'hello example'"#);
    shell.send("\r");
    shell.exp_string("hello example");

    shell.send(r#"echo 'world     test' 'shell''hello' example''script"#);
    shell.exp_string(r#"echo 'world     test' 'shell''hello' example''script"#);
    shell.send("\r");
    shell.exp_string("world     test shellhello examplescript");
}

#[test]
fn single_quotes_concatenation_on_left() {
    let mut shell = TestShell::new();

    shell.send(r#"echo 'hello'world"#);
    shell.exp_string(r#"echo 'hello'world"#);
    shell.send("\r");
    shell.exp_string("helloworld");
}

#[test]
fn single_quotes_concatenation_on_right() {
    let mut shell = TestShell::new();

    shell.send(r#"echo hello'world'"#);
    shell.exp_string(r#"echo hello'world'"#);
    shell.send("\r");
    shell.exp_string("helloworld");
}

#[test]
fn single_quotes_concatenation_on_left_and_right() {
    let mut shell = TestShell::new();

    shell.send(r#"echo 'hello''world'"#);
    shell.exp_string(r#"echo 'hello''world'"#);
    shell.send("\r");
    shell.exp_string("helloworld");
}

#[test]
fn single_quotes_concatenation_on_center() {
    let mut shell = TestShell::new();

    shell.send(r#"echo hello'world'hello"#);
    shell.exp_string(r#"echo hello'world'hello"#);
    shell.send("\r");
    shell.exp_string("helloworldhello");
}

#[test]
fn backslash_inside_single_quotes() {
    let mut shell = TestShell::new();

    shell.send(r#"echo 'multiple\\slashes'"#);
    shell.exp_string(r#"echo 'multiple\\slashes'"#);
    shell.send("\r");
    shell.exp_string(r#"multiple\\slashes"#);
}

#[test]
fn executable_in_quotes() {
    let mut shell = TestShell::new();

    shell.send(r#""echo" hello world"#);
    shell.exp_string(r#""echo" hello world"#);
    shell.send("\r");
    shell.exp_string("hello world");

    shell.send(r#"'echo' hello example"#);
    shell.exp_string(r#"'echo' hello example"#);
    shell.send("\r");
    shell.exp_string("hello example");
}
