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
