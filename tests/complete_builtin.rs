mod common;

use common::TestShell;

#[test]
fn completer_script_registration() {
    let mut shell = TestShell::new();
    shell.send("complete -p program\r");
    shell.exp_string("complete: program: no completion specification");

    let cmd = "complete -C /path/to/script program";
    shell.send(&cmd);
    shell.exp_string(&cmd);
    shell.send("\r");

    shell.send("complete -p program\r");
    shell.exp_string("complete -C '/path/to/script' program");

    shell.send("complete -r program\r");
    shell.send("complete -p program\r");
    shell.exp_string("complete: program: no completion specification");
}

#[test]
fn completer_script_autocomplete() {
    let mut shell = TestShell::new();
    let completer_script_contents = r#"#!/bin/sh
echo arg1
echo arg2
echo arg3
"#;
    let completer_script = shell
        .dir
        .create_executable_with_content("program-completer", completer_script_contents);

    let cmd = format!("complete -C {} program", completer_script.path_as_string());
    shell.send(&cmd);
    shell.exp_string(&cmd);
    shell.send("\r");

    shell.send("complete -p program\r");
    shell.exp_string(&format!(
        "complete -C '{}' program",
        completer_script.path_as_string()
    ));

    shell.send("program \t");
    shell.exp_string("program arg");
    shell.send("\t");
    shell.exp_string("arg1 arg2 arg3");
    shell.exp_string("$ program arg");
}
