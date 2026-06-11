mod common;

use self::common::TestFile;
use self::common::TestShell;
use self::common::create_dir;

#[test]
fn test_stdout_redirection_new_file() {
    let dir = create_dir();

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
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
fn test_stdout_redirection_existing_file() {
    let dir = create_dir();

    let _ = TestFile::create(&dir, "output.txt", "some random content");
    let _ = TestFile::create(&dir, "output2.txt", "some random content 2");

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
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
fn test_stdout_redirection_append_new_file() {
    let dir = create_dir();

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
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
fn test_stdout_redirection_append_existing_file() {
    let dir = create_dir();

    let _ = TestFile::create(&dir, "output.txt", "some random content");
    let _ = TestFile::create(&dir, "output2.txt", "some random content 2");

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
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
fn test_stderr_redirection_new_file() {
    let dir = create_dir();

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
    shell.send("cat nonexistent 2> output.txt");
    shell.exp_string("cat nonexistent 2> output.txt");
    shell.send("\r");
    shell.cat_file_contents("output.txt", "cat: nonexistent: No such file or directory");
}

#[test]
fn test_stderr_redirection_existing_file() {
    let dir = create_dir();

    let _ = TestFile::create(&dir, "output.txt", "some random content");

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
    shell.send("cat nonexistent 2> output.txt");
    shell.exp_string("cat nonexistent 2> output.txt");
    shell.send("\r");
    shell.cat_file_contents("output.txt", "cat: nonexistent: No such file or directory");
}

#[test]
fn test_stderr_redirection_append_new_file() {
    let dir = create_dir();

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
    shell.send("cat nonexistent 2>> output.txt");
    shell.exp_string("cat nonexistent 2>> output.txt");
    shell.send("\r");
    shell.cat_file_contents("output.txt", "cat: nonexistent: No such file or directory");
}

#[test]
fn test_stderr_redirection_append_existing_file() {
    let dir = create_dir();

    let _ = TestFile::create(&dir, "output.txt", "some random content");

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
    shell.send("cat nonexistent 2>> output.txt");
    shell.exp_string("cat nonexistent 2>> output.txt");
    shell.send("\r");
    shell.cat_file_contents(
        "output.txt",
        "some random contentcat: nonexistent: No such file or directory",
    );
}
