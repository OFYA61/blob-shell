#![allow(dead_code)]
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use assert_cmd::Command;
use tempfile;

pub fn run_shell(input: &str) -> assert_cmd::assert::Assert {
    let mut input = input.to_owned();
    input.push_str("\nexit");
    Command::cargo_bin("blob-shell")
        .expect("Failed to startup blob-shell")
        .write_stdin(input)
        .assert()
}

pub fn run_shell_with_path(input: &str, dir: &Path) -> assert_cmd::assert::Assert {
    let mut input = input.to_owned();
    input.push_str("\nexit");
    Command::cargo_bin("blob-shell")
        .expect("Failed to startup blob-shell")
        .current_dir(dir)
        .write_stdin(input)
        .assert()
}

pub fn create_dir() -> tempfile::TempDir {
    println!("Creating temprorary directory");
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    println!("Created temproary directory {:?}", dir.path());
    return dir;
}

pub struct TestFile {
    name: PathBuf,
    content: &'static str,
}

impl TestFile {
    pub fn new(name: PathBuf, content: &'static str) -> Self {
        Self { name, content }
    }
}

pub fn create_file(dir: &tempfile::TempDir, test_file: TestFile) {
    let path = dir.path().join(test_file.name);
    println!("Creating test file {:?}", path);

    let mut file = File::create(path).expect("Failed to create test file");
    file.write_all(test_file.content.as_bytes())
        .expect("Failed to write contents of test file");
    println!("Created test files");
}

pub fn assert_file_contents(file_path: PathBuf, expected_content: &'static str) {
    println!("Asserting file contents for {:?}", file_path);

    let mut file = File::open(file_path).expect("Failed to open file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Failed to read file contents");
    assert_eq!(content, expected_content);
}
