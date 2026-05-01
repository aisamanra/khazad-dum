mod common;
mod opts;
mod process;
mod walker;

use common::Output;

fn main() {
    let opts = opts::Opts::parse();

    let main_loop = match walker::MainLoop::setup(&opts) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("\x1b[91mFatal error: {}\x1b[39m", e);
            return;
        }
    };

    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    for output in main_loop.into_iter() {
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
            println!("  \x1b[91m{}\x1b[39m", err);
        }
    }

    if !warnings.is_empty() {
        println!("\x1b[93mWarnings:\x1b[39m",);
        for w in warnings {
            println!("  \x1b[94m{}\x1b[39m", w);
        }
    }
}
