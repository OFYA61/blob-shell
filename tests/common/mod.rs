#![allow(dead_code)]
use std::fs::File;
use std::io::Write;

use assert_cmd::Command;
use tempfile;

pub fn run_shell(input: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("blob-shell")
        .expect("Failed to startup blob-shell")
        .write_stdin(input)
        .assert()
}

pub fn create_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

pub fn create_files(dir: tempfile::TempDir, file_contents: Vec<&str>) -> Vec<File> {
    let mut files = Vec::new();
    for content in file_contents {
        let mut file = tempfile::tempfile_in(&dir).expect("Failed to create temp file");
        write!(file, "{}", content).expect("Failed to write to tempfile");
        files.push(file);
    }
    files
}
