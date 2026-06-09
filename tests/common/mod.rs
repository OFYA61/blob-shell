#![allow(dead_code)]

use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::process::Command;

use rexpect;
use rexpect::session::PtySession;
use rexpect::session::spawn_command;
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

    pub fn new_with_extra_path(dir: &tempfile::TempDir) -> Self {
        let bin_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(TARGET_SUBDIR)
            .join("blob-shell");
        println!("BINARY {:?}", bin_path);

        let path_var = env::var("PATH").expect("Failed to get PATH environment variable");
        let mut paths = env::split_paths(&path_var).collect::<Vec<_>>();
        paths.push(dir.path().to_path_buf());
        let new_path_var = env::join_paths(paths).expect("Failed to join paths");

        let mut command = Command::new(&bin_path.to_str().expect("Failed to get exectuable path"));
        command.env("PATH", new_path_var);

        let mut pty_session = spawn_command(command, Some(2000)).expect("Failed to spawn shell");

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
        self.send(command);
        self.flush();
        self.exp_string(command);

        self.send("\r");
        self.flush();

        self.exp_string(expected_output);
        self.exp_string("$ ");
    }

    pub fn test_autocompletion(&mut self, command: &str, expected_command_regex: &str) {
        assert!(command.contains("\t"));

        self.send(command);
        self.flush();
        self.exp_string(expected_command_regex);
    }

    pub fn test_autocompletion_command(
        &mut self,
        command: &str,
        expected_command: &str,
        expected_output: &str,
    ) {
        assert!(command.contains("\t"));

        self.send(command);
        self.flush();
        self.exp_string(expected_command);

        self.send("\r");
        self.flush();

        self.exp_string(expected_output);
        self.exp_string("$ ");
    }

    pub fn test_return(&mut self, expected_output: &str) {
        self.send("\r");
        self.flush();

        self.exp_string(expected_output);
        self.exp_string("$ ");
    }

    pub fn exit(&mut self) {
        self.send("exit\r");
        self.flush();
    }

    fn send(&mut self, command: &str) {
        self.pty_session
            .send(command)
            .expect("Failed to send command");
    }

    fn exp_string(&mut self, expected: &str) {
        if expected.is_empty() {
            return;
        }
        self.pty_session
            .exp_string(expected)
            .expect("Failed to check on expected value");
    }

    fn flush(&mut self) {
        self.pty_session.flush().expect("Failed to flush");
    }
}

pub fn create_dir() -> tempfile::TempDir {
    println!("Creating temprorary directory");
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    println!("Created temproary directory {:?}", dir.path());
    return dir;
}

pub struct TestFile {
    path: PathBuf,
}

impl TestFile {
    pub fn create(dir: &tempfile::TempDir, name: &str, content: &str) -> Self {
        let test_file = Self::open(dir, name);

        let mut file = File::create(&test_file.path).expect("Failed to create test file");
        file.write_all(content.as_bytes())
            .expect("Failed to write contents of test file");
        println!("Created test files");

        test_file
    }

    pub fn open(dir: &tempfile::TempDir, name: &str) -> Self {
        let path = dir.path().join(name);
        println!("Creating test file {:?}", path);
        Self { path }
    }

    pub fn assert_file_contents(&self, expected_content: &str) {
        println!("Asserting file contents for {:?}", self.path);

        let mut file = File::open(&self.path).expect("Failed to open file");
        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect("Failed to read file contents");
        assert_eq!(content, expected_content);
    }
}

pub struct TestExecutable {
    path: PathBuf,
}

impl TestExecutable {
    pub fn create(dir: &tempfile::TempDir, name: &str) -> Self {
        let path = dir.path().join(name);
        println!("Creating test executable {:?}", path);

        symlink("/bin/true", &path).expect("Failed to create symlink to '/bin/true'");

        Self { path }
    }
}
