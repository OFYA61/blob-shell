mod common;

use self::common::TestFile;
use self::common::run_shell_with_path;

#[test]
fn test_stdout_redirection_new_file() {
    let dir = common::create_dir();

    run_shell_with_path(
        r#"
        echo hello > output.txt
        echo hello world 1> output2.txt
        exit
        "#,
        dir.path(),
    )
    .success();

    common::assert_file_contents(dir.path().join("output.txt"), "hello\n");
    common::assert_file_contents(dir.path().join("output2.txt"), "hello world\n");
}

#[test]
fn test_stdout_redirection_existing_file() {
    let dir = common::create_dir();

    let file1_path = dir.path().join("output.txt");
    let file2_path = dir.path().join("output2.txt");
    common::create_file(
        &dir,
        TestFile::new(file1_path.clone(), "some random content"),
    );
    common::create_file(
        &dir,
        TestFile::new(file2_path.clone(), "some random content 2"),
    );

    run_shell_with_path(
        r#"
        echo hello > output.txt
        echo hello world 1> output2.txt
        exit
        "#,
        dir.path(),
    )
    .success();

    common::assert_file_contents(file1_path, "hello\n");
    common::assert_file_contents(file2_path, "hello world\n");
}

#[test]
fn test_stdout_redirection_append_new_file() {
    let dir = common::create_dir();

    run_shell_with_path(
        r#"
        echo hello >> output.txt
        echo hello world 1>> output2.txt
        exit
        "#,
        dir.path(),
    )
    .success();

    common::assert_file_contents(dir.path().join("output.txt"), "hello\n");
    common::assert_file_contents(dir.path().join("output2.txt"), "hello world\n");
}

#[test]
fn test_stderr_redirection_new_file() {
    let dir = common::create_dir();

    run_shell_with_path(
        r#"
        cat nonexistent 2> output.txt
        exit
        "#,
        dir.path(),
    )
    .success();

    common::assert_file_contents(
        dir.path().join("output.txt"),
        "cat: nonexistent: No such file or directory\n",
    );
}

#[test]
fn test_stderr_redirection_existing_file() {
    let dir = common::create_dir();

    let file_path = dir.path().join("output.txt");
    common::create_file(
        &dir,
        TestFile::new(file_path.clone(), "some random content"),
    );

    run_shell_with_path(
        r#"
        cat nonexistent 2> output.txt
        exit
        "#,
        dir.path(),
    )
    .success();

    common::assert_file_contents(file_path, "cat: nonexistent: No such file or directory\n");
}

#[test]
fn test_stderr_redirection_append_new_file() {
    let dir = common::create_dir();

    run_shell_with_path(
        r#"
        cat nonexistent 2>> output.txt
        exit
        "#,
        dir.path(),
    )
    .success();

    common::assert_file_contents(
        dir.path().join("output.txt"),
        "cat: nonexistent: No such file or directory\n",
    );
}
