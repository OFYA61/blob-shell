use assert_cmd::Command;

pub fn run_shell(input: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("blob-shell")
        .unwrap()
        .write_stdin(input)
        .assert()
}
