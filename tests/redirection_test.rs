mod common;

use self::common::TestFile;
use self::common::TestShell;
use self::common::create_dir;

#[test]
fn test_stdout_redirection_new_file() {
    let dir = create_dir();

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
    shell.test_command("echo hello > output.txt", "");
    shell.test_command("echo hello world 1> output2.txt", "");
    shell.exit();

    TestFile::open(&dir, "output.txt").assert_file_contents("hello\n");
    TestFile::open(&dir, "output2.txt").assert_file_contents("hello world\n");
}

#[test]
fn test_stdout_redirection_existing_file() {
    let dir = create_dir();

    let test_file1 = TestFile::create(&dir, "output.txt", "some random content");
    let test_file2 = TestFile::create(&dir, "output2.txt", "some random content 2");

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
    shell.test_command("echo hello > output.txt", "");
    shell.test_command("echo hello world 1> output2.txt", "");
    shell.exit();

    test_file1.assert_file_contents("hello\n");
    test_file2.assert_file_contents("hello world\n");
}

#[test]
fn test_stdout_redirection_append_new_file() {
    let dir = create_dir();

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
    shell.test_command("echo hello >> output.txt", "");
    shell.test_command("echo hello world 1>> output2.txt", "");
    shell.exit();

    TestFile::open(&dir, "output.txt").assert_file_contents("hello\n");
    TestFile::open(&dir, "output2.txt").assert_file_contents("hello world\n");
}

#[test]
fn test_stdout_redirection_append_existing_file() {
    let dir = create_dir();

    let test_file1 = TestFile::create(&dir, "output.txt", "some random content");
    let test_file2 = TestFile::create(&dir, "output2.txt", "some random content 2");

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
    shell.test_command("echo hello >> output.txt", "");
    shell.test_command("echo hello world 1>> output2.txt", "");
    shell.exit();

    test_file1.assert_file_contents("some random contenthello\n");
    test_file2.assert_file_contents("some random content 2hello world\n");
}

#[test]
fn test_stderr_redirection_new_file() {
    let dir = create_dir();

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
    shell.test_command("cat nonexistent 2> output.txt", "");
    shell.exit();

    TestFile::open(&dir, "output.txt")
        .assert_file_contents("cat: nonexistent: No such file or directory\n");
}

#[test]
fn test_stderr_redirection_existing_file() {
    let dir = create_dir();

    let test_file = TestFile::create(&dir, "output.txt", "some random content");

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
    shell.test_command("cat nonexistent 2> output.txt", "");
    shell.exit();

    test_file.assert_file_contents("cat: nonexistent: No such file or directory\n");
}

#[test]
fn test_stderr_redirection_append_new_file() {
    let dir = create_dir();

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
    shell.test_command("cat nonexistent 2>> output.txt", "");
    shell.exit();

    TestFile::open(&dir, "output.txt")
        .assert_file_contents("cat: nonexistent: No such file or directory\n");
}

#[test]
fn test_stderr_redirection_append_existing_file() {
    let dir = create_dir();

    let test_file = TestFile::create(&dir, "output.txt", "some random content");

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
    shell.test_command("cat nonexistent 2>> output.txt", "");
    shell.exit();

    test_file
        .assert_file_contents("some random contentcat: nonexistent: No such file or directory\n");
}
