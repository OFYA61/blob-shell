#![allow(dead_code)]

use std::env;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::process::Command;

use retry::delay::Fixed;
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
        shell.send(&format!("cd {}\r", cd_dir));
        shell
    }

    /// Send a sequence of strings as input
    pub fn send(&mut self, input: &str) {
        self.pty_session
            .send(input)
            .expect("Failed to send command");
        self.pty_session.flush().expect("Failed to flush");
    }

    /// Assert that the given string showed up on stdout
    pub fn exp_string(&mut self, expected: &str) {
        if expected.is_empty() {
            return;
        }
        self.pty_session
            .exp_string(expected)
            .expect("Failed to check on expected value");
    }

    /// Use to assert file contents
    pub fn cat_file_contents(&mut self, file_name: &str, expected_content: &str) {
        let cmd = format!("cat {}", file_name);
        self.send(&cmd);
        self.exp_string(&cmd);
        self.send("\r");
        self.exp_string(expected_content);
    }

    pub fn assert_is_terminated(&self) {
        let result = retry::retry(Fixed::from_millis(1000).take(10), || {
            if self.pty_session.process().status().is_none() {
                Ok(())
            } else {
                Err("Program did not exit in time")
            }
        });
        assert!(result.is_ok());
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
