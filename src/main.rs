use std::fs;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Sender, channel};

use digest::Digest;

mod opts;

const BUFSIZE: usize = 512;

#[derive(Debug)]
struct KhzdError {
    msg: String,
}

impl From<std::io::Error> for KhzdError {
    fn from(err: std::io::Error) -> KhzdError {
        KhzdError {
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
    Error(KhzdError),
    Entry(Line),
}

fn process_file(opts: &opts::Opts, path: &Path) -> Result<Line, KhzdError> {
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
    let path = if opts.absolute {
        opts.root.join(path)
    } else {
        path.into()
    };
    Ok(Line {
        size,
        md5,
        sha256,
        path,
    })
}

fn walk_loop(opts: &opts::Opts, send: Sender<Output>) -> Result<(), KhzdError> {
    let mut walker = ignore::WalkBuilder::new(&opts.root);
    let walker = walker.threads(opts.jobs);
    if let Some(f) = &opts.ignore
        && let Some(err) = walker.add_ignore(f)
    {
        return Err(KhzdError {
            msg: format!("Unable to read ignore file {}: {}", &f, err),
        });
    };
    walker.hidden(!opts.include_hidden);

    let walker = walker.build_parallel();
    walker.run(|| {
        let s = send.clone();
        Box::new(move |result| {
            let entry = match result {
                Ok(e) => e,
                Err(err) => {
                    s.send(Output::Warning(err.to_string())).unwrap();
                    return ignore::WalkState::Continue;
                }
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
                match process_file(opts, entry.path()) {
                    Ok(line) => s.send(Output::Entry(line)).unwrap(),
                    Err(err) => s.send(Output::Error(err)).unwrap(),
                }
            }
            ignore::WalkState::Continue
        })
    });

    Ok(())
}

fn main() {
    let opts = opts::Opts::parse();

    let (send, recv) = channel();
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    if let Err(e) = walk_loop(&opts, send) {
        errors.push(e);
    };

    for output in recv {
        match output {
            Output::Entry(l) => {
                print!("{},", l.size);
                if opts.compute_hashes.md5 {
                    print!("{},", hexhex::hex(l.md5));
                }
                if opts.compute_hashes.sha256 {
                    print!("{},", hexhex::hex(l.sha256));
                }
                println!("{}", l.path.display());
            }
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
