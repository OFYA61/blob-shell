mod common;

use self::common::TestShell;

#[test]
fn file_completion() {
    let mut shell = TestShell::new();
    let _ = shell.dir.create_file("test.txt", "Hello World!");

    shell.send("cat tes\t");
    shell.exp_string("cat test.txt");
    shell.send("\r");
    shell.exp_string("Hello World!");

    shell.send("cat \t");
    shell.exp_string("cat test.txt");
}

#[test]
fn file_in_subfolder_completion() {
    let mut shell = TestShell::new();
    let _ = shell.dir.create_file("subfolder/test.txt", "Hello World!");

    shell.send("cat subfolder/tes\t");
    shell.exp_string("cat subfolder/test.txt");
    shell.send("\r");
    shell.exp_string("Hello World!");
}

#[test]
fn folder_completion() {
    let mut shell = TestShell::new();
    let _ = shell.dir.create_file("subfolder/test.txt", "Hello World!");

    shell.send("cat subfo\t");
    shell.exp_string("cat subfolder/");
    shell.send("\r");

    shell.send("cat \t");
    shell.exp_string("cat subfolder/");
}

#[test]
fn nested_folder_completion() {
    let mut shell = TestShell::new();
    let _ = shell
        .dir
        .create_file("folder1/folder2/folder3/test.txt", "Hello World!");

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
fn multiple_file_completion_with_lcp() {
    let mut shell = TestShell::new();
    let _ = shell.dir.create_file("test1.txt", "Content 1");
    let _ = shell.dir.create_file("test2.txt", "Content 2");
    let _ = shell.dir.create_file("test3.txt", "Content 3");

    shell.send("cat \t");
    shell.exp_string("cat test");
    shell.send("\t");
    shell.exp_string("test1.txt test2.txt test3.txt \n\u{1b}[1G$ cat test");
}

#[test]
fn multiple_file_completion_without_lcp() {
    let mut shell = TestShell::new();
    let _ = shell.dir.create_file("test.txt", "Content 1");
    let _ = shell.dir.create_file("file.txt", "Content 2");
    let _ = shell.dir.create_file("dir/file.txt", "Content 3");

    shell.send("cat \t");
    shell.exp_string("cat \x07");
    shell.send("\t");
    shell.exp_string("dir/ file.txt test.txt \n\u{1b}[1G$ cat ");
}
