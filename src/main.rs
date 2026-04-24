use std::fs;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;

use digest::Digest;
use gumdrop::Options;

const BUFSIZE: usize = 512;

#[derive(Debug, Options)]
struct Opts {
    #[options(help = "how many threads to run", short = "j", long = "jobs")]
    jobs: Option<usize>,

    #[options(free)]
    root: Option<String>,
}

#[derive(Debug)]
struct Error {
    msg: String,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error {
            msg: format!("IO error: {}", err),
        }
    }
}

#[derive(Debug)]
struct Line {
    size: u64,
    md5: [u8; 16],
    sha256: [u8; 32],
    path: PathBuf,
}

#[derive(Debug)]
enum Output {
    Warning(String),
    Error(Error),
    Entry(Line),
}

fn process_file(root: &Path, path: &Path) -> Result<Line, Error> {
    let size = fs::metadata(path)?.size();
    let mut md5_hash = md5::Md5::new();
    let mut sha256_hash = sha2::Sha256::new();
    {
        let mut file = fs::File::open(path)?;
        let mut buf = [0; BUFSIZE];
        loop {
            let amt = file.read(&mut buf)?;
            if amt == 0 {
                break;
            }

            md5_hash.update(&buf[..amt]);
            sha256_hash.update(&buf[..amt]);
        }
    }
    let md5 = md5_hash.finalize().into();
    let sha256 = sha256_hash.finalize().into();
    Ok(Line {
        size,
        md5,
        sha256,
        path: root.join(path),
    })
}

fn main() {
    let opts = Opts::parse_args_default_or_exit();

    let root = &fs::canonicalize(opts.root.unwrap_or(".".to_owned())).unwrap();
    let jobs = opts.jobs.unwrap_or(32);

    let walker = ignore::WalkBuilder::new(root)
        .threads(jobs)
        .build_parallel();
    let (send, recv) = channel();
    walker.run(|| {
        let s = send.clone();
        Box::new(move |result| {
            let Ok(entry) = result else {
                //???
                return ignore::WalkState::Continue;
            };

            let Some(file_type) = entry.file_type() else {
                s.send(Output::Warning(format!(
                    "Unable to get type for {:?}",
                    entry.path()
                )))
                .unwrap();
                return ignore::WalkState::Continue;
            };

            if file_type.is_file() {
                match process_file(root, entry.path()) {
                    Ok(line) => s.send(Output::Entry(line)).unwrap(),
                    Err(err) => s.send(Output::Error(err)).unwrap(),
                }
            }
            ignore::WalkState::Continue
        })
    });
    drop(send);

    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    for output in recv {
        match output {
            Output::Entry(l) => println!(
                "{},{},{},{}",
                l.size,
                hexhex::hex(l.md5),
                hexhex::hex(l.sha256),
                l.path.display()
            ),
            Output::Warning(l) => warnings.push(l),
            Output::Error(e) => errors.push(e),
        }
    }

    if !errors.is_empty() {
        println!("\x1b[91mErrors:\x1b[39m");
        for err in errors {
            println!("  \x1b[91m{}\x1b[39m", err.msg);
        }
    }

    if !warnings.is_empty() {
        println!("\x1b[93mWarnings:\x1b[39m",);
        for w in warnings {
            println!("  \x1b[94m{}\x1b[39m", w);
        }
    }
}
