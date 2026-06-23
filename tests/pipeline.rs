mod common;

use common::TestShell;

#[test]
fn simple_2_command_pipeline() {
    let mut shell = TestShell::new();
    let file = shell.dir.create_file("file.txt", "Hello World\n");

    shell.send(&format!("cat {} | wc\r", file.path()));
    shell.exp_string("      1       2      12");
}

#[test]
fn simple_3_command_pipeline() {
    let mut shell = TestShell::new();
    let file = shell.dir.create_file("file.txt", "Hello\nWorld\nMeh\n");

    shell.send(&format!("cat {} | head -n 2 | wc\r", file.path()));
    shell.exp_string("      2       2      12");
}

#[test]
fn builtin_to_command_pipeline() {
    let mut shell = TestShell::new();
    shell.send("echo apple-orange | wc\r");
    shell.exp_string("      1       1      13");
}

#[test]
fn command_to_builtin_pipeline() {
    let mut shell = TestShell::new();
    shell.send("ls | type exit\r");
    shell.exp_string("exit is a shell builtin");
}
