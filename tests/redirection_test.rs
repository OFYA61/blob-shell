mod common;

use self::common::TestShell;
use self::common::TestFile;
use self::common::create_dir;
use self::common::create_file;
use self::common::assert_file_contents;

#[test]
fn test_stdout_redirection_new_file() {
    let dir = create_dir();

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
    shell.test_command("echo hello > output.txt", "");
    shell.test_command("echo hello world 1> output2.txt", "");
    shell.exit();

    assert_file_contents(dir.path().join("output.txt"), "hello\n");
    assert_file_contents(dir.path().join("output2.txt"), "hello world\n");
}

#[test]
fn test_stdout_redirection_existing_file() {
    let dir = create_dir();

    let file1_path = dir.path().join("output.txt");
    let file2_path = dir.path().join("output2.txt");
    create_file(
        &dir,
        TestFile::new(file1_path.clone(), "some random content"),
    );
    create_file(
        &dir,
        TestFile::new(file2_path.clone(), "some random content 2"),
    );

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
    shell.test_command("echo hello > output.txt", "");
    shell.test_command("echo hello world 1> output2.txt", "");
    shell.exit();

    assert_file_contents(file1_path, "hello\n");
    assert_file_contents(file2_path, "hello world\n");
}

#[test]
fn test_stdout_redirection_append_new_file() {
    let dir = create_dir();

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
    shell.test_command("echo hello >> output.txt", "");
    shell.test_command("echo hello world 1>> output2.txt", "");
    shell.exit();

    assert_file_contents(dir.path().join("output.txt"), "hello\n");
    assert_file_contents(dir.path().join("output2.txt"), "hello world\n");
}

#[test]
fn test_stdout_redirection_append_existing_file() {
    let dir = create_dir();

    let file1_path = dir.path().join("output.txt");
    let file2_path = dir.path().join("output2.txt");
    create_file(
        &dir,
        TestFile::new(file1_path.clone(), "some random content"),
    );
    create_file(
        &dir,
        TestFile::new(file2_path.clone(), "some random content 2"),
    );

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
    shell.test_command("echo hello >> output.txt", "");
    shell.test_command("echo hello world 1>> output2.txt", "");
    shell.exit();

    assert_file_contents(file1_path, "some random contenthello\n");
    assert_file_contents(file2_path, "some random content 2hello world\n");
}

#[test]
fn test_stderr_redirection_new_file() {
    let dir = create_dir();

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
    shell.test_command("cat nonexistent 2> output.txt", "");
    shell.exit();

    assert_file_contents(
        dir.path().join("output.txt"),
        "cat: nonexistent: No such file or directory\n",
    );
}

#[test]
fn test_stderr_redirection_existing_file() {
    let dir = create_dir();

    let file_path = dir.path().join("output.txt");
    create_file(
        &dir,
        TestFile::new(file_path.clone(), "some random content"),
    );

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
    shell.test_command("cat nonexistent 2> output.txt", "");
    shell.exit();

    assert_file_contents(file_path, "cat: nonexistent: No such file or directory\n");
}

#[test]
fn test_stderr_redirection_append_new_file() {
    let dir = create_dir();

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
    shell.test_command("cat nonexistent 2>> output.txt", "");
    shell.exit();

    assert_file_contents(
        dir.path().join("output.txt"),
        "cat: nonexistent: No such file or directory\n",
    );
}

#[test]
fn test_stderr_redirection_append_existing_file() {
    let dir = create_dir();

    let file_path = dir.path().join("output.txt");
    create_file(
        &dir,
        TestFile::new(file_path.clone(), "some random content"),
    );

    let mut shell = TestShell::new_with_cd(dir.path().to_str().unwrap());
    shell.test_command("cat nonexistent 2>> output.txt", "");
    shell.exit();

    assert_file_contents(
        file_path,
        "some random contentcat: nonexistent: No such file or directory\n",
    );
}
