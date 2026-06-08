#![allow(dead_code)]

use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

use rexpect;
use rexpect::session::PtySession;
use tempfile;

#[cfg(debug_assertions)]
const TARGET_SUBDIR: &str = "debug";
#[cfg(not(debug_assertions))]
const TARGET_SUBDIR: &str = "release";

pub struct TestShell {
    pty_session: PtySession,
}

impl TestShell {
    pub fn new() -> Self {
        let bin_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(TARGET_SUBDIR)
            .join("blob-shell");
        println!("BINARY {:?}", bin_path);
        let mut pty_session = rexpect::spawn(
            bin_path.to_str().expect("Failed to get executable path"),
            Some(2000),
        )
        .expect("Failed to spawn shell");

        pty_session
            .exp_string("$ ")
            .expect("Failed to get initial '$ '");

        Self { pty_session }
    }

    pub fn new_with_cd(cd_dir: &str) -> Self {
        let mut shell = Self::new();
        shell.test_command(&format!("cd {}", cd_dir), "");

        shell
    }

    pub fn test_command(&mut self, command: &str, expected_output: &str) {
        self.pty_session
            .send(command)
            .expect("Failed to send command");
        self.pty_session.flush().expect("Failed to flush");
        self.pty_session
            .exp_string(command)
            .expect("Failed to check read command string");

        self.pty_session.send("\r").expect("Failed to send enter");
        self.pty_session.flush().expect("Failed to flush");

        if !expected_output.is_empty() {
            self.pty_session
                .exp_string(expected_output)
                .expect("Failed to get expected output");
        }

        self.pty_session
            .exp_string("$ ")
            .expect("Failed to get next '$ ' prompt");
    }

    pub fn exit(&mut self) {
        self.pty_session
            .send_line("exit")
            .expect("Failed to send exit command");
    }
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
