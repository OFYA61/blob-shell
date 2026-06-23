mod common;

use self::common::TestShell;

#[test]
fn exit_builtin() {
    let mut shell = TestShell::new();
    shell.send("exit\r\r");
    shell.assert_is_terminated();
}

#[test]
fn invalid_commands() {
    let mut shell = TestShell::new();

    shell.send("invalid_command");
    shell.exp_string("invalid_command");
    shell.send("\r");
    shell.exp_string("invalid_command: command not found");
}

#[test]
fn echo_builtin() {
    let mut shell = TestShell::new();

    shell.send("echo apple banana");
    shell.exp_string("echo apple banana");
    shell.send("\r");
    shell.exp_string("apple banana");
}

#[test]
fn type_builtin() {
    let mut shell = TestShell::new();

    shell.send("type echo");
    shell.exp_string("type echo");
    shell.send("\r");
    shell.exp_string("echo is a shell builtin");

    shell.send("type exit");
    shell.exp_string("type exit");
    shell.send("\r");
    shell.exp_string("exit is a shell builtin");

    shell.send("type complete");
    shell.exp_string("type complete");
    shell.send("\r");
    shell.exp_string("complete is a shell builtin");

    shell.send("type type");
    shell.exp_string("type type");
    shell.send("\r");
    shell.exp_string("type is a shell builtin");

    shell.send("type history");
    shell.exp_string("type history");
    shell.send("\r");
    shell.exp_string("history is a shell builtin");

    shell.send("type invalid_command");
    shell.exp_string("type invalid_command");
    shell.send("\r");
    shell.exp_string("invalid_command: not found");
}
