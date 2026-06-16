#![allow(dead_code)]

use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::symlink;
use std::path::Path;
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
    pub dir: TestDir,
}

impl TestShell {
    /// Creates a new test shell with a temporary test directory for it, the shell will `cd` into
    /// the test directory on startup.
    pub fn new() -> Self {
        let bin_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(TARGET_SUBDIR)
            .join("blob-shell");
        println!("Running binary {:?}", bin_path);

        let dir = TestDir::new();

        let path_var = env::var("PATH").expect("Failed to get PATH environment variable");
        let mut paths = env::split_paths(&path_var).collect::<Vec<_>>();
        paths.push(dir.path_as_buf());
        let new_path_var = env::join_paths(paths).expect("Failed to join paths");

        let mut command = Command::new(&bin_path.to_str().expect("Failed to get exectuable path"));
        command.env("PATH", new_path_var);

        let mut pty_session = spawn_command(command, Some(2000)).expect("Failed to spawn shell");

        pty_session
            .exp_string("$ ")
            .expect("Failed to get initial '$ '");

        let mut shell = Self { pty_session, dir };

        shell.send(&format!("cd {}\r", shell.dir.path_as_str()));
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

pub struct TestDir {
    pub dir: tempfile::TempDir,
}

impl TestDir {
    fn new() -> Self {
        println!("Creating temprorary directory");
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        println!("Created temproary directory {:?}", dir.path());
        Self { dir }
    }

    pub fn create_file(&self, name: &str, content: &str) -> TestFile {
        TestFile::create(self, name, content)
    }

    pub fn create_executable(&self, name: &str) -> TestExecutable {
        TestExecutable::create(self, name)
    }

    pub fn create_executable_with_content(&self, name: &str, content: &str) -> TestExecutable {
        TestExecutable::create_with_content(self, name, content)
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn path_as_str(&self) -> &str {
        self.path().to_str().unwrap()
    }

    pub fn path_as_buf(&self) -> PathBuf {
        self.path().to_path_buf()
    }
}

pub struct TestFile {
    path: PathBuf,
}

impl TestFile {
    pub fn create(dir: &TestDir, name: &str, content: &str) -> Self {
        let test_file = Self::open(dir, name);

        if let Some(path) = test_file.path.parent() {
            fs::create_dir_all(path).expect("Failed to create path all");
        }

        let mut file = File::create(&test_file.path).expect("Failed to create test file");
        file.write_all(content.as_bytes())
            .expect("Failed to write contents of test file");
        println!("Created test files");

        test_file
    }

    pub fn open(dir: &TestDir, name: &str) -> Self {
        let path = dir.path().join(name);
        println!("Creating test file {:?}", path);
        Self { path }
    }
}

pub struct TestExecutable {
    path: PathBuf,
}

impl TestExecutable {
    pub fn create(dir: &TestDir, name: &str) -> Self {
        let path = dir.path().join(name);
        println!("Creating test executable {:?}", path);

        symlink("/bin/true", &path).expect("Failed to create symlink to '/bin/true'");

        Self { path }
    }

    pub fn create_with_content(dir: &TestDir, name: &str, content: &str) -> Self {
        let path = dir.path().join(name);
        println!(
            "Creating test executable {:?} with content '{}'",
            path, content
        );

        let mut executable = File::create(&path).expect("Failed to create test executable");
        executable
            .write_all(content.as_bytes())
            .expect("Failed to add contents to test executable");
        let mut perms = fs::metadata(&path)
            .expect("Failed to get executable metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).expect("Failed to set executable permissions");

        Self { path }
    }

    pub fn path_as_string(&self) -> String {
        self.path.to_str().unwrap().to_owned()
    }
}
