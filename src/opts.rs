use gumdrop::Options;
use std::{default, fs, path, process};

#[derive(Debug, Options)]
struct RawOpts {
    #[options(help = "how many threads to run", short = "j", long = "jobs")]
    jobs: Option<usize>,

    #[options(help = "include hidden files", short = "h", long = "include-hidden")]
    include_hidden: bool,

    #[options(help = "make files absolute in output", short = "a", long = "absolute")]
    absolute: bool,

    #[options(help = "custom ignore file", short = "i", long = "ignore")]
    ignore: Option<String>,

    #[options(help = "which hashes to use", short = "c", long = "compute-hashes")]
    compute_hashes: Option<String>,

    #[options(free)]
    root: Option<String>,
}

#[derive(Debug)]
pub struct Opts {
    pub jobs: usize,
    pub include_hidden: bool,
    pub absolute: bool,
    pub ignore: Option<String>,
    pub compute_hashes: Hashes,
    pub root: path::PathBuf,
}

#[derive(Debug)]
pub struct Hashes {
    pub md5: bool,
    pub sha256: bool,
}

impl default::Default for Hashes {
    fn default() -> Hashes {
        Hashes {
            md5: true,
            sha256: true,
        }
    }
}

impl Hashes {
    fn from_opt(opt: &Option<String>) -> Hashes {
        let Some(desc) = opt else {
            return Hashes::default();
        };
        let mut hashes = Hashes {
            md5: false,
            sha256: false,
        };
        for algorithm in desc.split(",") {
            match algorithm {
                "md5" => hashes.md5 = true,
                "sha256" => hashes.sha256 = true,
                _ => {
                    eprintln!("Unknown hash algorithm: {}", algorithm);
                    process::exit(1);
                }
            }
        }
        hashes
    }
}

impl Opts {
    pub fn parse() -> Opts {
        let opts = RawOpts::parse_args_default_or_exit();
        let root = opts.root.as_deref().unwrap_or(".");
        let root = if opts.absolute {
            fs::canonicalize(root).unwrap()
        } else {
            root.into()
        };
        Opts {
            jobs: opts.jobs.unwrap_or(32),
            include_hidden: opts.include_hidden,
            absolute: opts.absolute,
            ignore: opts.ignore,
            compute_hashes: Hashes::from_opt(&opts.compute_hashes),
            root,
        }
    }
}
