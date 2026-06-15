mod common;

use self::common::TestShell;

#[test]
fn stdout_redirection_new_file() {
    let mut shell = TestShell::new();
    shell.send("echo hello > output.txt");
    shell.exp_string("echo hello > output.txt");
    shell.send("\r");
    shell.cat_file_contents("output.txt", "hello");

    shell.send("echo hello world 1> output2.txt");
    shell.exp_string("echo hello world 1> output2.txt");
    shell.send("\r");
    shell.cat_file_contents("output2.txt", "hello world");
}

#[test]
fn stdout_redirection_existing_file() {
    let mut shell = TestShell::new();
    let _ = shell.dir.create_file("output.txt", "some random content");
    let _ = shell
        .dir
        .create_file("output2.txt", "some random content 2");

    shell.send("echo hello > output.txt");
    shell.exp_string("echo hello > output.txt");
    shell.send("\r");
    shell.cat_file_contents("output.txt", "hello");

    shell.send("echo hello world 1> output2.txt");
    shell.exp_string("echo hello world 1> output2.txt");
    shell.send("\r");
    shell.cat_file_contents("output2.txt", "hello world");
}

#[test]
fn stdout_redirection_append_new_file() {
    let mut shell = TestShell::new();
    shell.send("echo hello >> output.txt");
    shell.exp_string("echo hello >> output.txt");
    shell.send("\r");
    shell.cat_file_contents("output.txt", "hello");

    shell.send("echo hello world 1>> output2.txt");
    shell.exp_string("echo hello world 1>> output2.txt");
    shell.send("\r");
    shell.cat_file_contents("output2.txt", "hello world");
}

#[test]
fn stdout_redirection_append_existing_file() {
    let mut shell = TestShell::new();
    let _ = shell.dir.create_file("output.txt", "some random content");
    let _ = shell
        .dir
        .create_file("output2.txt", "some random content 2");

    shell.send("echo hello >> output.txt");
    shell.exp_string("echo hello >> output.txt");
    shell.send("\r");
    shell.cat_file_contents("output.txt", "some random contenthello");

    shell.send("echo hello world 1>> output2.txt");
    shell.exp_string("echo hello world 1>> output2.txt");
    shell.send("\r");
    shell.cat_file_contents("output2.txt", "some random content 2hello world");
}

#[test]
fn stderr_redirection_new_file() {
    let mut shell = TestShell::new();
    shell.send("cat nonexistent 2> output.txt");
    shell.exp_string("cat nonexistent 2> output.txt");
    shell.send("\r");
    shell.cat_file_contents("output.txt", "cat: nonexistent: No such file or directory");
}

#[test]
fn stderr_redirection_existing_file() {
    let mut shell = TestShell::new();
    let _ = shell.dir.create_file("output.txt", "some random content");

    shell.send("cat nonexistent 2> output.txt");
    shell.exp_string("cat nonexistent 2> output.txt");
    shell.send("\r");
    shell.cat_file_contents("output.txt", "cat: nonexistent: No such file or directory");
}

#[test]
fn stderr_redirection_append_new_file() {
    let mut shell = TestShell::new();
    shell.send("cat nonexistent 2>> output.txt");
    shell.exp_string("cat nonexistent 2>> output.txt");
    shell.send("\r");
    shell.cat_file_contents("output.txt", "cat: nonexistent: No such file or directory");
}

#[test]
fn stderr_redirection_append_existing_file() {
    let mut shell = TestShell::new();
    let _ = shell.dir.create_file("output.txt", "some random content");

    shell.send("cat nonexistent 2>> output.txt");
    shell.exp_string("cat nonexistent 2>> output.txt");
    shell.send("\r");
    shell.cat_file_contents(
        "output.txt",
        "some random contentcat: nonexistent: No such file or directory",
    );
}
