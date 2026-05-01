use std::fs;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use digest::Digest;

use crate::common::{Error, Line};
use crate::opts::Opts;

const BUFSIZE: usize = 512;

pub fn process_file(opts: &Opts, path: &Path) -> Result<Line, Error> {
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
