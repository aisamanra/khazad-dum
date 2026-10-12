use std::sync::mpsc::{Receiver, Sender, channel};

use crate::common::{Error, Output};
use crate::opts::Opts;
use crate::process::process_file;

pub struct MainLoop {
    recv: Receiver<Output>,
}

impl MainLoop {
    pub fn setup(opts: &Opts) -> Result<MainLoop, Error> {
        let (send, recv) = channel();
        walk_loop(opts, send)?;
        Ok(MainLoop { recv })
    }

    pub fn into_iter(self) -> impl Iterator<Item = Output> {
        self.recv.into_iter()
    }
}

pub fn walk_loop(opts: &Opts, send: Sender<Output>) -> Result<(), Error> {
    let mut walker = ignore::WalkBuilder::new(&opts.root);
    let walker = walker.threads(opts.jobs);
    if let Some(f) = &opts.ignore
        && let Some(err) = walker.add_ignore(f)
    {
        return Err(format!("Unable to read ignore file {}: {}", f, err).into());
    };
    walker.hidden(!opts.include_hidden);

    let walker = walker.build_parallel();
    walker.run(|| {
        let s = send.clone();
        Box::new(move |result| {
            let entry = match result {
                Ok(e) => e,
                Err(err) => {
                    s.send(Output::warn(err.to_string())).unwrap();
                    return ignore::WalkState::Continue;
                }
            };

            let Some(file_type) = entry.file_type() else {
                s.send(Output::warn(format!(
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
