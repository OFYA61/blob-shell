mod common;

use self::common::TestFile;
use self::common::TestShell;
use self::common::create_dir;

#[test]
fn test_file_completion() {
    let dir = create_dir();

    let _ = TestFile::create(&dir, "test.txt", "Hello World!");

    let mut shell = TestShell::new_with_cd(&dir.path().to_str().unwrap());
    shell.send("cat tes\t");
    shell.exp_string("cat test.txt");
    shell.send("\r");
    shell.exp_string("Hello World!");

    shell.send("cat \t");
    shell.exp_string("cat test.txt");
}

#[test]
fn test_file_in_subfolder_completion() {
    let dir = create_dir();

    let _ = TestFile::create(&dir, "subfolder/test.txt", "Hello World!");

    let mut shell = TestShell::new_with_cd(&dir.path().to_str().unwrap());
    shell.send("cat subfolder/tes\t");
    shell.exp_string("cat subfolder/test.txt");
    shell.send("\r");
    shell.exp_string("Hello World!");
}

#[test]
fn test_folder_completion() {
    let dir = create_dir();

    let _ = TestFile::create(&dir, "subfolder/test.txt", "Hello World!");

    let mut shell = TestShell::new_with_cd(&dir.path().to_str().unwrap());

    shell.send("cat subfo\t");
    shell.exp_string("cat subfolder/");
    shell.send("\r");

    shell.send("cat \t");
    shell.exp_string("cat subfolder/");
}

#[test]
fn test_nested_folder_completion() {
    let dir = create_dir();

    let _ = TestFile::create(&dir, "folder1/folder2/folder3/test.txt", "Hello World!");

    let mut shell = TestShell::new_with_cd(&dir.path().to_str().unwrap());

    shell.send("cat folde\t");
    shell.exp_string("cat folder1/");
    shell.send("\t");
    shell.exp_string("folder2/");
    shell.send("\t");
    shell.exp_string("folder3/");
    shell.send("\t");
    shell.exp_string("test.txt");
}

#[test]
fn test_multiple_file_completion_with_lcp() {
    let dir = create_dir();
    let _ = TestFile::create(&dir, "test1.txt", "Content 1");
    let _ = TestFile::create(&dir, "test2.txt", "Content 2");
    let _ = TestFile::create(&dir, "test3.txt", "Content 3");

    let mut shell = TestShell::new_with_cd(&dir.path().to_str().unwrap());

    shell.send("cat \t");
    shell.exp_string("cat test");
    shell.send("\t");
    shell.exp_string("test1.txt test2.txt test3.txt \n\u{1b}[256D$ cat test");
}

#[test]
fn test_multiple_file_completion_without_lcp() {
    let dir = create_dir();
    let _ = TestFile::create(&dir, "test.txt", "Content 1");
    let _ = TestFile::create(&dir, "file.txt", "Content 2");
    let _ = TestFile::create(&dir, "dir/file.txt", "Content 3");

    let mut shell = TestShell::new_with_cd(&dir.path().to_str().unwrap());

    shell.send("cat \t");
    shell.exp_string("cat \x07");
    shell.send("\t");
    shell.exp_string("dir/ file.txt test.txt \n\u{1b}[256D$ cat ");
}
