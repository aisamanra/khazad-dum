use std::collections::HashMap;
use std::default::Default;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use assert_cmd::Command;
use md5::Digest;
use tempdir::TempDir;

#[derive(Debug)]
struct TestCaseFile {
    path: String,
    size: usize,
    contents: Vec<u8>,
    md5: String,
    sha256: String,
    observed: bool,
}

#[derive(Debug, Default)]
struct TestCase {
    files: HashMap<PathBuf, TestCaseFile>,
}

impl TestCase {
    fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let tmpdir = TempDir::new("example")?;
        for (_, case) in self.files.iter() {
            let mut path = tmpdir.path().to_owned();
            for fragment in case.path.split("/") {
                std::fs::create_dir_all(&path)?;
                path = path.join(fragment);
            }
            let mut f = File::create(path)?;
            f.write(&case.contents)?;
        }

        let mut cmd = Command::cargo_bin("khazad-dum")?;
        let assert = cmd.arg(tmpdir.path()).assert().success();
        let output = str::from_utf8(&assert.get_output().stdout)?;
        for line in output.split("\n") {
            if line == "" {
                break;
            }
            let mut parts = line.split(',');
            let size = parts.next().ok_or("No size in output")?;
            let md5 = parts.next().ok_or("No MD5 in output")?;
            let sha256 = parts.next().ok_or("No SHA256 in output")?;
            let path = PathBuf::from(parts.next().ok_or("No path in output")?);

            let rel = path.strip_prefix(tmpdir.path())?.to_owned();
            let case = self
                .files
                .get_mut(&rel)
                .ok_or_else(|| format!("Unable to find case for {:?}", path))?;
            assert_eq!(case.size, size.parse()?);
            assert_eq!(case.md5, md5);
            assert_eq!(case.sha256, sha256);
            case.observed = true;
        }

        for (_, case) in self.files.iter() {
            if !case.observed {
                panic!("Did not see {:?} in output", case.path);
            }
        }

        Ok(())
    }

    fn add(mut self, name: &str, contents: &[u8]) -> TestCase {
        let md5 = md5::Md5::new_with_prefix(&contents).finalize();
        let sha256 = sha2::Sha256::new_with_prefix(&contents).finalize();
        self.files.insert(
            name.into(),
            TestCaseFile {
                path: name.into(),
                size: contents.len(),
                contents: contents.into(),
                md5: hexhex::hex(md5).to_string(),
                sha256: hexhex::hex(sha256).to_string(),
                observed: false,
            },
        );
        self
    }
}

#[test]
fn simple_example() {
    let case = TestCase::default().add("hello.txt", b"Hello, world!");
    case.run().unwrap_or_else(|err| {
        panic!("Failure: {}", err);
    });
}

#[test]
fn multi_file_example() {
    let case = TestCase::default()
        .add("one.txt", b"This is file one")
        .add("two.txt", b"This is file two")
        .add("three.txt", b"This is file three");
    case.run().unwrap_or_else(|err| {
        panic!("Failure: {}", err);
    });
}

#[test]
fn subdir_example() {
    let case = TestCase::default()
        .add("one.txt", b"This is file one")
        .add("nest/two.txt", b"This is file two")
        .add("much/deeper/nest/three.txt", b"This is file three");
    case.run().unwrap_or_else(|err| {
        panic!("Failure: {}", err);
    });
}
