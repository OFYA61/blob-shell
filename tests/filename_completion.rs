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
}
